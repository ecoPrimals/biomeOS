// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Connection handling for Neural API Server
//!
//! Handles incoming Unix socket connections, reads requests, and writes responses.

use anyhow::Result;
use biomeos_types::jsonrpc::{JsonRpcInput, JsonRpcResponse};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::debug;

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Handle a client connection.
    ///
    /// Reads JSON-RPC requests line-by-line.  Supports both single request
    /// objects and JSON-RPC 2.0 Section 6 batch arrays.  Batch elements are
    /// processed concurrently via `futures::future::join_all`.
    pub async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();

            let read_result =
                timeout(Duration::from_millis(100), reader.read_line(&mut line)).await;

            match read_result {
                Ok(Ok(n)) if n > 0 => {
                    if let Some(response_value) = self.dispatch_line(&line).await {
                        let response_str = serde_json::to_string(&response_value)? + "\n";
                        let stream = reader.get_mut();
                        stream.write_all(response_str.as_bytes()).await?;
                        stream.flush().await?;
                    }
                    // No response for notifications (id == None)
                }
                Ok(Ok(_) | Err(_)) | Err(_) => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Dispatch a single input line, handling both single and batch JSON-RPC.
    ///
    /// Returns `None` for JSON-RPC notifications (requests with no `id`),
    /// per JSON-RPC 2.0 Section 4.1: the server MUST NOT reply to a notification.
    async fn dispatch_line(&self, line: &str) -> Option<Value> {
        match JsonRpcInput::parse(line) {
            Ok(JsonRpcInput::Single(req)) => {
                // JSON-RPC 2.0: notifications have no `id` — no response
                if req.id.is_none() {
                    debug!("Received JSON-RPC notification: {}", req.method);
                    let raw = serde_json::to_string(&req).unwrap_or_default();
                    let _ = self.handle_request_json(&raw).await;
                    return None;
                }
                let raw = serde_json::to_string(&req).unwrap_or_default();
                Some(self.handle_request_json(&raw).await)
            }
            Ok(JsonRpcInput::Batch(requests)) => {
                debug!("Processing JSON-RPC batch of {} requests", requests.len());
                let futures: Vec<_> = requests
                    .into_iter()
                    .filter(|req| {
                        // Skip notifications in batch (no response expected)
                        if req.id.is_none() {
                            debug!("Skipping batch notification: {}", req.method);
                            return false;
                        }
                        true
                    })
                    .map(|req| {
                        let raw = serde_json::to_string(&req).unwrap_or_default();
                        async move { self.handle_request_json(&raw).await }
                    })
                    .collect();

                let results = futures::future::join_all(futures).await;
                if results.is_empty() {
                    None
                } else {
                    Some(serde_json::to_value(results).unwrap_or(Value::Array(vec![])))
                }
            }
            Err(err) => Some(
                serde_json::to_value(JsonRpcResponse::error(Value::Null, err))
                    .unwrap_or(Value::Null),
            ),
        }
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
    async fn test_handle_connection_invalid_json_returns_parse_error() {
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
        assert!(
            buf.contains("Parse error") || buf.contains("-32700"),
            "invalid JSON should return parse error, got: {buf}"
        );
    }

    #[tokio::test]
    async fn test_handle_connection_batch_request() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let batch = r#"[{"jsonrpc":"2.0","method":"topology.get","id":1},{"jsonrpc":"2.0","method":"topology.primals","id":2}]"#;
        client_stream
            .write_all((batch.to_string() + "\n").as_bytes())
            .await
            .expect("write batch");
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
        conn_result.expect("handle batch");
        let parsed: serde_json::Value = serde_json::from_str(&buf).expect("response is valid json");
        assert!(parsed.is_array(), "batch response must be an array");
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_handle_connection_empty_batch_returns_invalid_request() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        client_stream
            .write_all(b"[]\n")
            .await
            .expect("write empty batch");
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
        conn_result.expect("handle empty batch");
        assert!(
            buf.contains("Invalid Request") || buf.contains("-32600"),
            "empty batch should return invalid request, got: {buf}"
        );
    }
}
