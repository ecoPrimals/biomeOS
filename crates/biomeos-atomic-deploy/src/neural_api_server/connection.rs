// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Connection handling for Neural API Server
//!
//! Handles incoming connections (Unix socket **and** TCP), reads newline-
//! delimited JSON-RPC requests, and writes responses. After a successful
//! BTSP Phase 3 negotiate, the connection switches to length-prefixed
//! ChaCha20-Poly1305 encrypted framing.

use anyhow::Result;
use biomeos_core::btsp_client::{self, BtspHandshakeError, HandshakeOutcome};
use biomeos_types::jsonrpc::{JsonRpcInput, JsonRpcResponse};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, warn};

use super::NeuralApiServer;
use super::btsp_negotiate::{self, BtspCipher, SessionKeys};

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
            Ok(HandshakeOutcome::Authenticated {
                session_id,
                handshake_key,
            }) => {
                debug!(session_id = %session_id, has_key = handshake_key.is_some(), "BTSP authenticated — proceeding with JSON-RPC");
                super::btsp_negotiate::register_session(
                    &self.btsp_sessions,
                    session_id,
                    handshake_key,
                )
                .await;
            }
            Ok(HandshakeOutcome::DevMode) => {
                // No FAMILY_ID — proceed without handshake.
            }
            Ok(HandshakeOutcome::SecurityProviderUnavailable) => {
                warn!("BTSP: security provider unavailable, proceeding without handshake");
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
            Err(BtspHandshakeError::SecurityProviderNotFound) => {
                if enforce {
                    warn!("BTSP enforced but security provider not found — rejecting connection");
                    return Ok(());
                }
                warn!("BTSP: security provider not found, accepting connection without handshake");
            }
            Err(e) => {
                warn!("BTSP handshake failed: {e}");
                return Ok(());
            }
        }

        self.handle_stream_with_negotiate(reader).await
    }

    /// Handle a TCP client connection.
    pub async fn handle_tcp_connection(&self, stream: tokio::net::TcpStream) -> Result<()> {
        self.handle_stream(BufReader::new(stream)).await
    }

    /// Post-handshake handler that checks for Phase 3 negotiate on the first line.
    ///
    /// If the first message is `btsp.negotiate` and results in an encrypted cipher,
    /// the connection switches to length-prefixed ChaCha20-Poly1305 framing.
    /// Otherwise it falls through to the NDJSON plaintext loop.
    async fn handle_stream_with_negotiate<S>(&self, mut reader: BufReader<S>) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin,
    {
        let mut first_line = String::new();

        let read_result = timeout(
            Duration::from_millis(100),
            reader.read_line(&mut first_line),
        )
        .await;

        match read_result {
            Ok(Ok(n)) if n > 0 => {
                if let Some(keys) = self.try_phase3_negotiate(&first_line, &mut reader).await? {
                    return self.handle_encrypted_stream(reader, keys).await;
                }
                if let Some(response_value) = self.dispatch_line(&first_line).await {
                    let response_str = serde_json::to_string(&response_value)? + "\n";
                    let stream = reader.get_mut();
                    stream.write_all(response_str.as_bytes()).await?;
                    stream.flush().await?;
                }
            }
            Ok(Ok(_) | Err(_)) | Err(_) => {
                return Ok(());
            }
        }

        self.handle_stream(reader).await
    }

    /// Check if a line is a `btsp.negotiate` request. If so, dispatch it and
    /// return the derived `SessionKeys` when the cipher is not null.
    ///
    /// The negotiate response is written back on the plaintext channel before
    /// the connection transitions to encrypted framing.
    async fn try_phase3_negotiate<S>(
        &self,
        line: &str,
        reader: &mut BufReader<S>,
    ) -> Result<Option<SessionKeys>>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin,
    {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let method = parsed.get("method").and_then(|v| v.as_str());
        if method != Some("btsp.negotiate") {
            return Ok(None);
        }

        let id = parsed.get("id").cloned().unwrap_or(Value::Null);
        let params = parsed.get("params").cloned().unwrap_or(Value::Null);

        let negotiate_result = btsp_negotiate::handle_negotiate(&self.btsp_sessions, &params).await;

        let response = match &negotiate_result {
            Ok(result_value) => {
                serde_json::json!({"jsonrpc": "2.0", "result": result_value, "id": id})
            }
            Err(e) => serde_json::to_value(JsonRpcResponse::error(
                id,
                biomeos_types::jsonrpc::JsonRpcError {
                    code: -32602,
                    message: e.to_string(),
                    data: None,
                },
            ))?,
        };

        let response_str = serde_json::to_string(&response)? + "\n";
        let stream = reader.get_mut();
        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;

        let Ok(result_value) = negotiate_result else {
            return Ok(None);
        };

        let cipher_str = result_value
            .get("cipher")
            .and_then(|v| v.as_str())
            .unwrap_or("null");

        if BtspCipher::from_str_loose(cipher_str) != BtspCipher::ChaCha20Poly1305 {
            return Ok(None);
        }

        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let sessions = self.btsp_sessions.read().await;
        let keys = sessions
            .get(session_id)
            .and_then(|s| s.session_keys.clone());

        if let Some(ref k) = keys {
            info!(session_id = %session_id, "Phase 3 negotiate succeeded — switching to encrypted framing");
            debug!(session_id = %session_id, "Derived directional keys: {k:?}");
        }

        Ok(keys)
    }

    /// Encrypted frame loop — replaces NDJSON after successful Phase 3 negotiate.
    ///
    /// Wire format per `BTSP_PROTOCOL_STANDARD.md`:
    ///   Read:  `[4B BE u32 len][payload]` → `decrypt_frame(c2s_key, payload)` → JSON-RPC
    ///   Write: JSON-RPC → `encrypt_frame(s2c_key, plaintext)` → `[4B len][payload]`
    async fn handle_encrypted_stream<S>(
        &self,
        mut reader: BufReader<S>,
        keys: SessionKeys,
    ) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin,
    {
        let mut len_buf = [0u8; 4];

        loop {
            let read_result =
                timeout(Duration::from_secs(30), reader.read_exact(&mut len_buf)).await;

            match read_result {
                Ok(Ok(_)) => {}
                Ok(Err(_)) | Err(_) => break,
            }

            let frame_len = u32::from_be_bytes(len_buf) as usize;
            if frame_len > 16 * 1024 * 1024 {
                warn!(frame_len, "Encrypted frame too large — dropping connection");
                break;
            }

            let mut payload = vec![0u8; frame_len];
            timeout(Duration::from_secs(30), reader.read_exact(&mut payload))
                .await
                .map_err(|_| anyhow::anyhow!("Timeout reading encrypted frame payload"))??;

            let plaintext = match btsp_negotiate::decrypt_frame(&keys.client_to_server, &payload) {
                Ok(pt) => pt,
                Err(e) => {
                    warn!("Decryption failed: {e} — dropping connection");
                    break;
                }
            };

            let line = String::from_utf8_lossy(&plaintext);

            if let Some(response_value) = self.dispatch_line(&line).await {
                let response_bytes = serde_json::to_vec(&response_value)?;

                let frame = btsp_negotiate::encrypt_frame(&keys.server_to_client, &response_bytes)
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

                let stream = reader.get_mut();
                stream.write_all(&frame).await?;
                stream.flush().await?;
            }
        }

        Ok(())
    }

    /// Plaintext NDJSON connection handler.
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
    use crate::neural_api_server::btsp_negotiate;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

    fn create_test_server() -> NeuralApiServer {
        let temp = tempfile::tempdir().expect("temp dir");
        NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"))
    }

    #[tokio::test]
    async fn test_encrypted_frame_roundtrip_via_handle_encrypted_stream() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let keys = btsp_negotiate::derive_session_keys(&[0xAAu8; 32], &[1u8; 12], &[2u8; 12]);
        let client_keys = keys.clone();

        let client_task = tokio::spawn(async move {
            let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
            let frame =
                btsp_negotiate::encrypt_frame(&client_keys.client_to_server, request.as_bytes())
                    .expect("encrypt");
            client_stream.write_all(&frame).await.expect("write frame");
            client_stream.flush().await.expect("flush");

            let mut len_buf = [0u8; 4];
            client_stream
                .read_exact(&mut len_buf)
                .await
                .expect("read len");
            let resp_len = u32::from_be_bytes(len_buf) as usize;
            let mut payload = vec![0u8; resp_len];
            client_stream
                .read_exact(&mut payload)
                .await
                .expect("read payload");
            let plaintext = btsp_negotiate::decrypt_frame(&client_keys.server_to_client, &payload)
                .expect("decrypt");
            String::from_utf8(plaintext).expect("utf8")
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_encrypted_stream(reader, keys)
            .await
            .expect("handle_encrypted_stream");

        let response = client_task.await.expect("client task");
        assert!(response.contains("jsonrpc"), "response: {response}");
    }

    #[tokio::test]
    async fn test_encrypted_stream_wrong_key_drops_connection() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let server_keys =
            btsp_negotiate::derive_session_keys(&[0xAAu8; 32], &[1u8; 12], &[2u8; 12]);
        let wrong_keys = btsp_negotiate::derive_session_keys(&[0xBBu8; 32], &[3u8; 12], &[4u8; 12]);

        let client_task = tokio::spawn(async move {
            let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
            let frame =
                btsp_negotiate::encrypt_frame(&wrong_keys.client_to_server, request.as_bytes())
                    .expect("encrypt");
            client_stream.write_all(&frame).await.expect("write frame");
            client_stream.flush().await.expect("flush");

            let mut buf = [0u8; 4];
            let result = client_stream.read_exact(&mut buf).await;
            result.is_err() || buf == [0u8; 4]
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_encrypted_stream(reader, server_keys)
            .await
            .expect("should gracefully close");

        let dropped = client_task.await.expect("client task");
        assert!(dropped, "connection should be dropped on bad key");
    }

    #[tokio::test]
    async fn test_encrypted_stream_oversized_frame_drops_connection() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let keys = btsp_negotiate::derive_session_keys(&[0xCCu8; 32], &[5u8; 12], &[6u8; 12]);

        let client_task = tokio::spawn(async move {
            let huge_len: u32 = 20_000_000;
            client_stream
                .write_all(&huge_len.to_be_bytes())
                .await
                .expect("write oversized len");
            client_stream.flush().await.expect("flush");
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_encrypted_stream(reader, keys)
            .await
            .expect("should gracefully close on oversized frame");

        client_task.await.expect("client task");
    }

    #[tokio::test]
    async fn test_negotiate_then_encrypted_stream_end_to_end() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let handshake_key = [0xDDu8; 32];
        let session_id = "e2e-test-session";

        btsp_negotiate::register_session(&server.btsp_sessions, session_id, Some(handshake_key))
            .await;

        let client_nonce = [0x11u8; 32];
        let client_nonce_b64 = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(client_nonce)
        };

        let client_task = tokio::spawn(async move {
            let negotiate_req = format!(
                r#"{{"jsonrpc":"2.0","method":"btsp.negotiate","params":{{"session_id":"{}","preferred_cipher":"chacha20-poly1305","client_nonce":"{}","bond_type":"Covalent"}},"id":1}}"#,
                session_id, client_nonce_b64
            );
            client_stream
                .write_all((negotiate_req + "\n").as_bytes())
                .await
                .expect("write negotiate");
            client_stream.flush().await.expect("flush");

            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            let mut negotiate_resp = String::new();
            reader
                .read_line(&mut negotiate_resp)
                .await
                .expect("read negotiate response");

            let resp: serde_json::Value =
                serde_json::from_str(&negotiate_resp).expect("parse negotiate response");
            let result = resp.get("result").expect("has result");
            assert_eq!(result["cipher"], "chacha20-poly1305");

            let server_nonce_b64 = result["server_nonce"].as_str().expect("server_nonce");
            let server_nonce = {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(server_nonce_b64)
                    .expect("decode server nonce")
            };

            let keys =
                btsp_negotiate::derive_session_keys(&handshake_key, &client_nonce, &server_nonce);

            let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":2}"#;
            let frame = btsp_negotiate::encrypt_frame(&keys.client_to_server, request.as_bytes())
                .expect("encrypt");

            let client_stream = reader.into_inner();
            client_stream.write_all(&frame).await.expect("write frame");
            client_stream.flush().await.expect("flush");

            let mut len_buf = [0u8; 4];
            client_stream
                .read_exact(&mut len_buf)
                .await
                .expect("read len");
            let resp_len = u32::from_be_bytes(len_buf) as usize;
            let mut payload = vec![0u8; resp_len];
            client_stream
                .read_exact(&mut payload)
                .await
                .expect("read payload");
            let plaintext =
                btsp_negotiate::decrypt_frame(&keys.server_to_client, &payload).expect("decrypt");
            String::from_utf8(plaintext).expect("utf8")
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_stream_with_negotiate(reader)
            .await
            .expect("handle_stream_with_negotiate");

        let response = client_task.await.expect("client task");
        assert!(
            response.contains("jsonrpc"),
            "encrypted response should be valid JSON-RPC: {response}"
        );
    }

    #[tokio::test]
    async fn test_negotiate_null_cipher_stays_on_ndjson() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        btsp_negotiate::register_session(&server.btsp_sessions, "null-sess", None).await;

        let client_task = tokio::spawn(async move {
            let negotiate_req = r#"{"jsonrpc":"2.0","method":"btsp.negotiate","params":{"session_id":"null-sess","preferred_cipher":"chacha20-poly1305","client_nonce":"AAAA"},"id":1}"#;
            client_stream
                .write_all((negotiate_req.to_string() + "\n").as_bytes())
                .await
                .expect("write negotiate");
            client_stream.flush().await.expect("flush");

            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            let mut negotiate_resp = String::new();
            reader
                .read_line(&mut negotiate_resp)
                .await
                .expect("read negotiate response");

            let resp: serde_json::Value =
                serde_json::from_str(&negotiate_resp).expect("parse negotiate response");
            assert_eq!(
                resp["result"]["cipher"], "null",
                "should fall back to null without handshake key"
            );

            negotiate_resp
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_stream_with_negotiate(reader)
            .await
            .expect("handle_stream_with_negotiate");

        client_task.await.expect("client task");
    }

    #[tokio::test]
    async fn test_non_negotiate_first_line_stays_on_ndjson() {
        let (server_stream, mut client_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let server = create_test_server();

        let client_task = tokio::spawn(async move {
            let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
            client_stream
                .write_all((request.to_string() + "\n").as_bytes())
                .await
                .expect("write");
            client_stream.flush().await.expect("flush");

            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            let mut resp = String::new();
            reader.read_line(&mut resp).await.expect("read response");
            resp
        });

        let reader = tokio::io::BufReader::new(server_stream);
        server
            .handle_stream_with_negotiate(reader)
            .await
            .expect("handle_stream_with_negotiate");

        let response = client_task.await.expect("client task");
        assert!(
            response.contains("jsonrpc"),
            "NDJSON response expected: {response}"
        );
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
