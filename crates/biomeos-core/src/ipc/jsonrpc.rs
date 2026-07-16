// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC over [`TransportStream`] — the unified request/response primitive.

use anyhow::{Context, Result};
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
