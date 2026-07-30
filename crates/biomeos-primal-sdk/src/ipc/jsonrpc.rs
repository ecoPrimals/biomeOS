// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC over [`TransportStream`] — the unified request/response primitive.

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::trace;

use super::TransportEndpoint;
use super::TransportStream;

/// Connect to `endpoint` and send one JSON-RPC request, returning the response.
///
/// When `FAMILY_ID` is set and the endpoint is a family-scoped socket, a BTSP
/// handshake is performed before sending the request. This ensures compatibility
/// with primals running in strict mode (`BIOMEOS_BTSP_ENFORCE=1`).
pub async fn send_jsonrpc_request(
    endpoint: &TransportEndpoint,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = super::connect_transport(endpoint)
        .await
        .with_context(|| format!("Failed to connect to {endpoint}"))?;

    let TransportEndpoint::UnixSocket { path } = endpoint;
    if super::btsp_handshake::should_btsp(path) {
        match super::btsp_handshake::perform_handshake(stream).await {
            Ok(mut reader) => {
                return send_jsonrpc_over_reader(&mut reader, request).await;
            }
            Err(e) => {
                super::btsp_handshake::warn_btsp_skipped(path);
                trace!("BTSP handshake failed, falling back to plaintext: {e}");
                let fallback_stream = super::connect_transport(endpoint)
                    .await
                    .with_context(|| format!("Failed to reconnect to {endpoint}"))?;
                return send_jsonrpc_over_stream(fallback_stream, request).await;
            }
        }
    }

    send_jsonrpc_over_stream(stream, request).await
}

/// Send one JSON-RPC request over an already-connected stream and read one response.
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

/// Send JSON-RPC over a `BufReader` (post-BTSP handshake stream).
async fn send_jsonrpc_over_reader(
    reader: &mut BufReader<TransportStream>,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let request_str =
        serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

    trace!("Sending JSON-RPC (post-BTSP): {}", request_str);

    reader.get_mut().write_all(request_str.as_bytes()).await?;
    reader.get_mut().write_all(b"\n").await?;
    reader.get_mut().flush().await?;

    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("Failed to read JSON-RPC response")?;

    trace!("Received JSON-RPC (post-BTSP): {}", line.trim());

    let response: JsonRpcResponse =
        serde_json::from_str(line.trim()).context("Failed to parse JSON-RPC response")?;

    Ok(response)
}
