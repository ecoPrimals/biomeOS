// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unix socket JSON-RPC transport (connect, framed write, read with timeout).

use std::path::Path;
use std::time::Duration;

use biomeos_core::{TransportEndpoint, send_jsonrpc_request};
use biomeos_types::JsonRpcRequest;
use tokio::time::timeout;
use tracing::debug;

use crate::{Error, Result};

const RPC_TIMEOUT_SECS: u64 = 30;

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
    let endpoint = TransportEndpoint::UnixSocket {
        path: socket_path.to_path_buf(),
    };

    debug!(
        socket = %socket_path.display(),
        method = %method,
        "Calling Unix socket JSON-RPC"
    );

    let request = JsonRpcRequest::new(method, params);

    let response = timeout(
        Duration::from_secs(RPC_TIMEOUT_SECS),
        send_jsonrpc_request(&endpoint, request),
    )
    .await
    .map_err(|_| Error::timeout("Socket read", RPC_TIMEOUT_SECS))?
    .map_err(|err| map_send_error(socket_path, &err))?;

    if let Some(error) = response.error {
        return Err(Error::jsonrpc_failed(
            method,
            format!("Code {}: {}", error.code, error.message),
        ));
    }

    let result = response.result.ok_or_else(|| {
        Error::invalid_response(
            socket_path.display().to_string(),
            "Missing 'result' field in JSON-RPC response",
        )
    })?;

    serde_json::from_value(result).map_err(|e| {
        Error::invalid_response(
            socket_path.display().to_string(),
            format!("Failed to deserialize result: {e}"),
        )
    })
}

fn map_send_error(path: &Path, err: &anyhow::Error) -> Error {
    let msg = err.to_string();

    if msg.contains("Failed to connect") {
        if let Some(io_err) = io_error_from_anyhow(err) {
            return Error::socket_connection_failed(path, io_err);
        }
        return Error::socket_connection_failed(path, std::io::Error::other(msg));
    }

    if msg.contains("Failed to read") {
        return Error::discovery_failed(format!("Read error: {err}"), None);
    }

    if msg.contains("Failed to parse") {
        return Error::invalid_response(
            path.display().to_string(),
            format!("Invalid JSON-RPC response: {err}"),
        );
    }

    Error::discovery_failed(format!("JSON-RPC request failed: {err}"), None)
}

fn io_error_from_anyhow(err: &anyhow::Error) -> Option<std::io::Error> {
    if let Some(e) = err.downcast_ref::<std::io::Error>() {
        return Some(std::io::Error::new(e.kind(), e.to_string()));
    }
    for cause in err.chain() {
        if let Some(e) = cause.downcast_ref::<std::io::Error>() {
            return Some(std::io::Error::new(e.kind(), e.to_string()));
        }
    }
    None
}
