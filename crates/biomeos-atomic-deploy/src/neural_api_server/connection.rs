// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Connection handling for Neural API Server
//!
//! Handles incoming connections (Unix socket **and** TCP), reads newline-
//! delimited JSON-RPC requests, and writes responses.

use anyhow::Result;
use biomeos_core::btsp_client::{self, BtspHandshakeError, HandshakeOutcome};
use biomeos_types::jsonrpc::{JsonRpcInput, JsonRpcResponse};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, warn};

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Handle a Unix socket client connection (development mode, no BTSP).
    pub async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        self.handle_stream(BufReader::new(stream)).await
    }

    /// Handle a Unix socket connection with BTSP Phase 2 handshake.
    ///
    /// Attempts a server-side BTSP handshake. If the client sends a raw
    /// JSON-RPC request instead of a `ClientHello`, the behaviour depends
    /// on `enforce`: when true the connection is dropped; when false the
    /// raw line is dispatched and the connection continues as cleartext.
    pub async fn handle_connection_with_btsp(
        &self,
        stream: UnixStream,
        enforce: bool,
    ) -> Result<()> {
        let mut reader = BufReader::new(stream);

        match btsp_client::server_handshake(&mut reader).await {
            Ok(HandshakeOutcome::Authenticated { session_id }) => {
                debug!(session_id = %session_id, "BTSP authenticated — proceeding with JSON-RPC");
            }
            Ok(HandshakeOutcome::DevMode) => {
                // No FAMILY_ID — proceed without handshake.
            }
            Ok(HandshakeOutcome::BearDogUnavailable) => {
                warn!("BTSP: BearDog unavailable, proceeding without handshake");
            }
            Err(BtspHandshakeError::RawJsonRpc(first_line)) => {
                if enforce {
                    warn!(
                        "BTSP enforced: rejecting unauthenticated connection (client sent raw JSON-RPC)"
                    );
                    let id = serde_json::from_str::<Value>(&first_line)
                        .ok()
                        .and_then(|v| v.get("id").cloned())
                        .unwrap_or(Value::Null);
                    let err_resp = JsonRpcResponse::error(
                        id,
                        biomeos_types::jsonrpc::JsonRpcError {
                            code: -32000,
                            message: "BTSP handshake required. Start biomeOS with \
                                      --btsp-optional or set BIOMEOS_BTSP_ENFORCE=0 \
                                      to allow unauthenticated connections."
                                .to_string(),
                            data: None,
                        },
                    );
                    let err_str = serde_json::to_string(&err_resp)? + "\n";
                    let stream = reader.get_mut();
                    stream.write_all(err_str.as_bytes()).await?;
                    stream.flush().await?;
                    return Ok(());
                }
                warn!("BTSP: client sent raw JSON-RPC without handshake (warn-only mode)");
                if let Some(response_value) = self.dispatch_line(&first_line).await {
                    let response_str = serde_json::to_string(&response_value)? + "\n";
                    let stream = reader.get_mut();
                    stream.write_all(response_str.as_bytes()).await?;
                    stream.flush().await?;
                }
            }
            Err(BtspHandshakeError::BearDogNotFound) => {
                if enforce {
                    warn!("BTSP enforced but BearDog not found — rejecting connection");
                    return Ok(());
                }
                warn!("BTSP: BearDog not found, accepting connection without handshake");
            }
            Err(e) => {
                warn!("BTSP handshake failed: {e}");
                return Ok(());
            }
        }

        self.handle_stream(reader).await
    }

    /// Handle a TCP client connection.
    pub async fn handle_tcp_connection(&self, stream: tokio::net::TcpStream) -> Result<()> {
        self.handle_stream(BufReader::new(stream)).await
    }

    /// Transport-agnostic connection handler.
    ///
    /// Reads JSON-RPC requests line-by-line. Supports both single request
    /// objects and JSON-RPC 2.0 Section 6 batch arrays. Batch elements are
    /// processed concurrently via `futures::future::join_all`.
    async fn handle_stream<S>(&self, mut reader: BufReader<S>) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin,
    {
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

        // Single-word method (no dot) → MethodNotFound; domain.operation methods
        // route through the semantic capability fallback instead.
        let request = r#"{"jsonrpc":"2.0","method":"nonexistent","id":1}"#;
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
        assert!(buf.contains("nonexistent"));
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
