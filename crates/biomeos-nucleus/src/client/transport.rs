// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unix socket JSON-RPC transport (connect, framed write, read with timeout).

use std::path::Path;

use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::debug;

use crate::{Error, Result};

/// Call a Unix socket JSON-RPC method
///
/// **Deep Debt Principle**: Shared utility, no duplication across layers
///
/// # Errors
///
/// Returns error if:
/// - Unix socket connection fails
/// - JSON-RPC request fails  
/// - Response deserialization fails
pub async fn call_unix_socket_rpc<T: serde::de::DeserializeOwned>(
    socket_path: impl AsRef<Path>,
    method: &str,
    params: serde_json::Value,
) -> Result<T> {
    let socket_path = socket_path.as_ref();

    debug!(
        socket = %socket_path.display(),
        method = %method,
        "Calling Unix socket JSON-RPC"
    );

    // Connect to socket
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| Error::socket_connection_failed(socket_path, e))?;

    // Split stream for concurrent read/write
    let (read_half, mut write_half) = stream.into_split();

    // Create request with unique ID for concurrent request correlation
    let request = JsonRpcRequest::new(method, params);

    // Serialize and send request
    let request_json = serde_json::to_string(&request)?;
    debug!(request = %request_json, "Sending JSON-RPC request");

    write_half.write_all(request_json.as_bytes()).await?;
    write_half.write_all(b"\n").await?; // Newline delimiter
    write_half.flush().await?;

    // Read response with timeout to prevent hangs
    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    // 30 second timeout for socket reads (prevents indefinite hangs)
    timeout(
        Duration::from_secs(30),
        reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| Error::timeout("Socket read", 30))?
    .map_err(|e| Error::discovery_failed(format!("Read error: {e}"), None))?;

    debug!(response = %response_line, "Received JSON-RPC response");

    // Parse response
    let response: JsonRpcResponse = serde_json::from_str(&response_line).map_err(|e| {
        Error::invalid_response(
            socket_path.display().to_string(),
            format!("Invalid JSON-RPC response: {e}"),
        )
    })?;

    // Check for error
    if let Some(error) = response.error {
        return Err(Error::jsonrpc_failed(
            method,
            format!("Code {}: {}", error.code, error.message),
        ));
    }

    // Extract result
    let result = response.result.ok_or_else(|| {
        Error::invalid_response(
            socket_path.display().to_string(),
            "Missing 'result' field in JSON-RPC response",
        )
    })?;

    // Deserialize result
    serde_json::from_value(result).map_err(|e| {
        Error::invalid_response(
            socket_path.display().to_string(),
            format!("Failed to deserialize result: {e}"),
        )
    })
}
