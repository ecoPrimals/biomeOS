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
