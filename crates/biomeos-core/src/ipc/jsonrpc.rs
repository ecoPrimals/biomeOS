// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC over [`TransportStream`] — the unified request/response primitive.

use anyhow::{Context, Result};
use biomeos_types::constants::ribocipher;
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::trace;

use super::TransportStream;
use crate::TransportEndpoint;

/// Connect to `endpoint` and send one JSON-RPC request, returning the response.
///
/// This is the **canonical** way to make a JSON-RPC call in biomeOS. It replaces
/// scattered `send_jsonrpc_uds`, `json_rpc_call`, `call_unix_socket_rpc`, and
/// the various `send_request` implementations that previously required `#[cfg]` gating.
pub async fn send_jsonrpc_request(
    endpoint: &TransportEndpoint,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = super::connect_transport(endpoint)
        .await
        .with_context(|| format!("Failed to connect to {endpoint}"))?;

    send_jsonrpc_over_stream(stream, request).await
}

/// Connect to `endpoint` with riboCipher transport framing (`[0xEC, 0x01]` prefix),
/// then send one JSON-RPC request and return the response.
///
/// Use this for connections to Neural API sockets (and any endpoint that enforces
/// riboCipher signal detection per Wave 113 policy). The 2-byte prefix is written
/// once at connection start before any JSON-RPC payload.
pub async fn send_ribocipher_jsonrpc_request(
    endpoint: &TransportEndpoint,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = super::connect_transport(endpoint)
        .await
        .with_context(|| format!("Failed to connect to {endpoint}"))?;

    send_ribocipher_jsonrpc_over_stream(stream, request).await
}

/// Write the riboCipher clear-tier signal (`[0xEC, 0x01]`) then send JSON-RPC.
///
/// Combines transport framing with request dispatch in a single call. The
/// riboCipher signal is a connection-level prefix that tells the server what
/// transport tier to expect (clear = standard JSON-RPC, no obfuscation).
pub async fn send_ribocipher_jsonrpc_over_stream(
    stream: TransportStream,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let request_str =
        serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

    trace!("Sending riboCipher+JSON-RPC: {}", request_str);

    let (reader, mut writer) = tokio::io::split(stream);

    writer
        .write_all(&[ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1])
        .await?;
    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("Failed to read JSON-RPC response")?;

    trace!("Received JSON-RPC: {}", line.trim());

    let response: JsonRpcResponse =
        serde_json::from_str(line.trim()).context("Failed to parse JSON-RPC response")?;

    Ok(response)
}

/// Write the riboCipher transport signal prefix to a raw writer.
///
/// Exported for callers that manage their own connection lifecycle (BTSP clients,
/// CLI tools) and need to prepend the signal before custom framing.
pub async fn write_ribocipher_signal<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
) -> Result<()> {
    writer
        .write_all(&[ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1])
        .await
        .context("Failed to write riboCipher signal")?;
    Ok(())
}

/// Send one JSON-RPC request over an already-connected stream and read one response.
///
/// Public so callers that need custom connection logic (BTSP, retries) can still
/// use the shared framing.
pub async fn send_jsonrpc_over_stream(
    stream: TransportStream,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let request_str =
        serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

    trace!("Sending JSON-RPC: {}", request_str);

    let (reader, mut writer) = tokio::io::split(stream);

    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("Failed to read JSON-RPC response")?;

    trace!("Received JSON-RPC: {}", line.trim());

    let response: JsonRpcResponse =
        serde_json::from_str(line.trim()).context("Failed to parse JSON-RPC response")?;

    Ok(response)
}
