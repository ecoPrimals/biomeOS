// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Connection handling for Neural API Server
//!
//! Handles incoming connections (Unix socket **and** TCP), reads newline-
//! delimited JSON-RPC requests, and writes responses. After a successful
//! BTSP Phase 3 negotiate, the connection switches to length-prefixed
//! ChaCha20-Poly1305 encrypted framing.

use anyhow::{Context, Result};
use biomeos_core::btsp_client::{self, BtspHandshakeError, HandshakeOutcome};
use biomeos_core::ipc::TransportStream;
use biomeos_types::jsonrpc::{JsonRpcError, JsonRpcInput, JsonRpcResponse};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use super::NeuralApiServer;
use super::btsp_negotiate::{self, BtspCipher, SessionKeys};
use super::protocol_negotiation::{self, NegotiatedProtocol, NegotiationOutcome};

impl NeuralApiServer {
    /// Handle a transport client connection (development mode, no BTSP).
    ///
    /// Supports G65 protocol negotiation: if the client sends a `PROTOCOLS:`
    /// header as the first line, the server negotiates and dispatches to the
    /// appropriate protocol handler. Otherwise falls back to JSON-RPC.
    pub async fn handle_connection(&self, stream: TransportStream) -> Result<()> {
        let mut reader = BufReader::new(stream);

        match protocol_negotiation::negotiate_server(&mut reader).await? {
            NegotiationOutcome::Negotiated(NegotiatedProtocol::Tarpc) => {
                self.handle_tarpc_stream(reader).await
            }
            NegotiationOutcome::Negotiated(NegotiatedProtocol::JsonRpc) => {
                self.handle_stream(reader).await
            }
            NegotiationOutcome::NotNegotiation(first_line) => {
                self.handle_stream_with_first_line(reader, &first_line)
                    .await
            }
            NegotiationOutcome::Closed => Ok(()),
        }
    }

    /// Peek for riboCipher transport signal and consume if present.
    ///
    /// Returns the detected `RiboCipherTier` if a valid signal was found,
    /// or `None` if no signal was present (plain JSON-RPC connection).
    async fn consume_ribocipher_signal<S>(
        reader: &mut BufReader<S>,
    ) -> Option<biomeos_types::constants::ribocipher::RiboCipherTier>
    where
        S: tokio::io::AsyncRead + Unpin,
    {
        use biomeos_types::constants::ribocipher;

        match reader.fill_buf().await {
            Ok(buf)
                if buf.len() >= ribocipher::SIGNAL_LEN && ribocipher::is_signal_byte(buf[0]) =>
            {
                let tier_byte = buf[0];
                let version = buf[1];
                reader.consume(ribocipher::SIGNAL_LEN);
                let tier = ribocipher::RiboCipherTier::from_signal(tier_byte)?;
                debug!("riboCipher signal: tier={tier:?} version={version}");
                Some(tier)
            }
            _ => {
                debug!(
                    "Connection without riboCipher signal — \
                     protocol negotiation fallback (Wave 113)"
                );
                None
            }
        }
    }

    /// Validate a mito-tag for Tier 2 connections.
    ///
    /// Reads the 32-byte mito-tag from the stream and calls `crypto.decode_mito_tag`
    /// via the capability dispatch path. Returns `true` if the tag is valid.
    ///
    /// If bearDog hasn't registered `decode_mito_tag` yet, this falls through to
    /// accept the connection (graceful degradation — Tier 2 validation is defense-in-depth).
    async fn validate_mito_tag<S>(&self, reader: &mut BufReader<S>) -> bool
    where
        S: tokio::io::AsyncRead + Unpin,
    {
        use biomeos_types::constants::ribocipher;

        let mut tag_buf = [0u8; ribocipher::MITO_TAG_LEN];
        match tokio::io::AsyncReadExt::read_exact(reader, &mut tag_buf).await {
            Ok(_) => {
                let tag_hex = hex::encode(tag_buf);
                debug!(tag = %tag_hex, "Mito-tag received, requesting decode");

                let params = serde_json::json!({
                    "capability": "crypto.decode_mito_tag",
                    "args": { "tag": tag_hex }
                });

                match self.capability_handler.call(&Some(params)).await {
                    Ok(outcome) => {
                        let valid = outcome
                            .result
                            .as_object()
                            .and_then(|o| o.get("valid"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if valid {
                            debug!("Mito-tag validated successfully");
                        } else {
                            warn!("Mito-tag validation failed — tag rejected by bearDog");
                        }
                        valid
                    }
                    Err(e) => {
                        debug!(
                            "decode_mito_tag capability unavailable ({e}) — \
                             graceful degradation, accepting connection"
                        );
                        true
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read mito-tag from Tier 2 connection: {e}");
                false
            }
        }
    }

    /// Handle a transport connection with BTSP Phase 2 handshake.
    ///
    /// Attempts a server-side BTSP handshake. If the client sends a raw
    /// JSON-RPC request instead of a `ClientHello`, the behaviour depends
    /// on `enforce`: when true the connection is dropped; when false the
    /// raw line is dispatched and the connection continues as cleartext.
    pub async fn handle_connection_with_btsp(
        &self,
        stream: TransportStream,
        enforce: bool,
    ) -> Result<()> {
        let mut reader = BufReader::new(stream);

        // riboCipher transport signal detection (Wave 113+).
        // Returns the tier or None for plain connections.
        let tier = Self::consume_ribocipher_signal(&mut reader).await;

        match tier {
            None => {
                if enforce {
                    return Ok(());
                }
                debug!("Accepting plain connection (BTSP not enforced) — trying G65 negotiation");
                return self.handle_with_negotiation(reader).await;
            }
            Some(ribocipher_tier) => {
                // Tier 2 (Mito): validate mito-tag before BTSP proceeds
                if ribocipher_tier.requires_mito_validation() {
                    if !self.validate_mito_tag(&mut reader).await {
                        warn!("Tier 2 mito-tag validation failed — dropping connection");
                        return Ok(());
                    }
                }
            }
        }

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
    ///
    /// TCP connections without riboCipher are accepted when `btsp_optional` is set,
    /// enabling plain JSON-RPC callers (cellMembrane, cross-gate tools).
    /// Tier 2 (Mito) connections have mito-tag validated before proceeding.
    pub async fn handle_tcp_connection(&self, stream: tokio::net::TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        match Self::consume_ribocipher_signal(&mut reader).await {
            None => {
                if !self.btsp_optional {
                    return Ok(());
                }
                debug!("Accepting plain JSON-RPC TCP connection (BTSP not enforced)");
            }
            Some(tier) => {
                if tier.requires_mito_validation() && !self.validate_mito_tag(&mut reader).await {
                    warn!("TCP Tier 2 mito-tag validation failed — dropping connection");
                    return Ok(());
                }
            }
        }
        self.handle_stream(reader).await
    }

    /// G65 protocol negotiation on a plain (non-BTSP) connection.
    ///
    /// Peeks at the first line: if it's a `PROTOCOLS:` header, negotiates and
    /// dispatches to the selected protocol. Otherwise treats it as JSON-RPC.
    async fn handle_with_negotiation<S>(&self, mut reader: BufReader<S>) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        match protocol_negotiation::negotiate_server(&mut reader).await? {
            NegotiationOutcome::Negotiated(NegotiatedProtocol::Tarpc) => {
                self.handle_tarpc_stream(reader).await
            }
            NegotiationOutcome::Negotiated(NegotiatedProtocol::JsonRpc) => {
                self.handle_stream(reader).await
            }
            NegotiationOutcome::NotNegotiation(first_line) => {
                self.handle_stream_with_first_line(reader, &first_line)
                    .await
            }
            NegotiationOutcome::Closed => Ok(()),
        }
    }

    /// Dispatch a pre-read first line then continue as JSON-RPC stream.
    async fn handle_stream_with_first_line<S>(
        &self,
        mut reader: BufReader<S>,
        first_line: &str,
    ) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin,
    {
        if let Some(response_value) = self.dispatch_line(first_line).await {
            let response_str = serde_json::to_string(&response_value)? + "\n";
            let stream = reader.get_mut();
            stream.write_all(response_str.as_bytes()).await?;
            stream.flush().await?;
        }
        self.handle_stream(reader).await
    }

    /// Handle a tarpc connection on an already-negotiated stream.
    ///
    /// After G65 negotiation selects tarpc, the stream is wrapped in a
    /// length-delimited Bincode transport and the `HealthRpc` service is
    /// served on it. This enables single-socket tarpc without the `.tarpc.sock`
    /// sidecar — the Phase 3 (G65) destination.
    async fn handle_tarpc_stream<S>(&self, reader: BufReader<S>) -> Result<()>
    where
        S: tokio::io::AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        use biomeos_primal_sdk::tarpc_transport::DefaultHealthService;
        use biomeos_types::tarpc_types::HealthRpc;
        use futures_util::StreamExt;
        use tarpc::server::{BaseChannel, Channel};
        use tokio_serde::formats::Bincode;

        let stream = reader.into_inner();
        let transport = tarpc::serde_transport::new(
            tokio_util::codec::length_delimited::LengthDelimitedCodec::builder().new_framed(stream),
            Bincode::default(),
        );

        let service = DefaultHealthService::new("biomeos");
        let channel = BaseChannel::with_defaults(transport);
        let mut requests = Box::pin(channel.execute(service.serve()));
        while let Some(fut) = requests.next().await {
            fut.await;
        }

        info!("G65 tarpc connection closed");
        Ok(())
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
            biomeos_types::constants::timeouts::POLL_INTERVAL_FAST,
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
            let read_result = timeout(
                biomeos_types::constants::timeouts::DEFAULT_REQUEST_TIMEOUT,
                reader.read_exact(&mut len_buf),
            )
            .await;

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
            timeout(
                biomeos_types::constants::timeouts::DEFAULT_REQUEST_TIMEOUT,
                reader.read_exact(&mut payload),
            )
            .await
            .context("Timeout reading encrypted frame payload")??;

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
                    .context("Encryption failed")?;

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

            let read_result = timeout(
                biomeos_types::constants::timeouts::POLL_INTERVAL_FAST,
                reader.read_line(&mut line),
            )
            .await;

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
                if req.id.is_none() {
                    debug!("Received JSON-RPC notification: {}", req.method);
                    match serde_json::to_string(&req) {
                        Ok(raw) => {
                            let _ = self.handle_request_json(&raw).await;
                        }
                        Err(e) => tracing::warn!("Failed to serialize notification: {e}"),
                    }
                    return None;
                }
                match serde_json::to_string(&req) {
                    Ok(raw) => Some(self.handle_request_json(&raw).await),
                    Err(e) => {
                        tracing::warn!("Failed to serialize request: {e}");
                        let resp = JsonRpcResponse::error(
                            req.id.unwrap_or(Value::Null),
                            JsonRpcError::internal_error(Some(format!("serialization error: {e}"))),
                        );
                        serde_json::to_value(&resp).ok()
                    }
                }
            }
            Ok(JsonRpcInput::Batch(requests)) => {
                debug!("Processing JSON-RPC batch of {} requests", requests.len());
                let futures: Vec<_> = requests
                    .into_iter()
                    .filter(|req| {
                        if req.id.is_none() {
                            debug!("Skipping batch notification: {}", req.method);
                            return false;
                        }
                        true
                    })
                    .filter_map(|req| match serde_json::to_string(&req) {
                        Ok(raw) => Some(async move { self.handle_request_json(&raw).await }),
                        Err(e) => {
                            tracing::warn!("Failed to serialize batch request: {e}");
                            None
                        }
                    })
                    .collect();

                let results = futures_util::future::join_all(futures).await;
                if results.is_empty() {
                    None
                } else {
                    match serde_json::to_value(&results) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            tracing::warn!("Failed to serialize batch results: {e}");
                            Some(Value::Array(vec![]))
                        }
                    }
                }
            }
            Err(err) => {
                let resp = JsonRpcResponse::error(Value::Null, err);
                match serde_json::to_value(&resp) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        tracing::warn!("Failed to serialize error response: {e}");
                        Some(Value::Null)
                    }
                }
            }
        }
    }
}

#[cfg(all(test, unix))]
#[path = "connection_tests.rs"]
mod tests;
