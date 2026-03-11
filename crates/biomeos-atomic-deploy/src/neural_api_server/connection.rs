//! Connection handling for Neural API Server
//!
//! Handles incoming Unix socket connections, reads requests, and writes responses.

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::error;

use super::rpc::internal_error_response;
use super::NeuralApiServer;

impl NeuralApiServer {
    /// Handle a client connection
    ///
    /// Reads JSON-RPC requests line-by-line and processes them.
    /// Uses timeouts to detect when clients close their write side.
    pub async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();

            // Try to read next request with timeout (client may have shut down write side)
            let read_result =
                timeout(Duration::from_millis(100), reader.read_line(&mut line)).await;

            match read_result {
                Ok(Ok(n)) if n > 0 => {
                    // Request received, handle it
                    let response = match self.handle_request(&line).await {
                        Ok(response) => response,
                        Err(e) => {
                            error!("Request error: {}", e);
                            internal_error_response(&e, None)
                        }
                    };

                    // Write response
                    let response_str = serde_json::to_string(&response)? + "\n";
                    let stream = reader.get_mut();
                    stream.write_all(response_str.as_bytes()).await?;
                    stream.flush().await?;

                    // After sending response, check if we can read more (short timeout)
                    // If client shut down write side, this will timeout quickly
                    continue;
                }
                Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {
                    // EOF, error, or timeout - client is done
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::neural_api_server::NeuralApiServer;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    fn create_test_server() -> NeuralApiServer {
        let temp = tempfile::tempdir().expect("temp dir");
        NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"))
    }

    #[tokio::test]
    async fn test_handle_connection_unknown_method_returns_error_response() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let request = r#"{"jsonrpc":"2.0","method":"unknown.method","id":1}"#;
        client_stream
            .write_all((request.to_string() + "\n").as_bytes())
            .await
            .expect("write request");
        client_stream.flush().await.expect("flush");

        let mut buf = String::new();
        let (read_result, conn_result) = tokio::join!(
            async {
                let mut reader = tokio::io::BufReader::new(&mut client_stream);
                reader.read_line(&mut buf).await
            },
            server.handle_connection(server_stream)
        );

        let _ = read_result.expect("read response");
        conn_result.expect("handle_connection should succeed");
        assert!(buf.contains("jsonrpc"));
        assert!(buf.contains("error"));
        assert!(buf.contains("unknown.method"));
    }

    #[tokio::test]
    async fn test_handle_connection_processes_request_and_returns_response() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let request = r#"{"jsonrpc":"2.0","method":"nonexistent","id":42}"#;
        client_stream
            .write_all((request.to_string() + "\n").as_bytes())
            .await
            .expect("write request");
        client_stream.flush().await.expect("flush");

        let mut buf = String::new();
        let (read_result, conn_result) = tokio::join!(
            async {
                let mut reader = tokio::io::BufReader::new(&mut client_stream);
                reader.read_line(&mut buf).await
            },
            server.handle_connection(server_stream)
        );

        let _ = read_result.expect("read response");
        conn_result.expect("handle_connection");
        assert!(buf.contains("jsonrpc"));
        assert!(buf.contains("error") || buf.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_connection_invalid_json_returns_internal_error() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        client_stream
            .write_all(b"{invalid\n")
            .await
            .expect("write invalid json");
        client_stream.flush().await.expect("flush");

        let mut buf = String::new();
        let (read_result, conn_result) = tokio::join!(
            async {
                let mut reader = tokio::io::BufReader::new(&mut client_stream);
                reader.read_line(&mut buf).await
            },
            server.handle_connection(server_stream)
        );

        let _ = read_result.expect("read");
        conn_result.expect("connection handler");
        assert!(buf.contains("Internal error") || buf.contains("-32603"));
    }
}
