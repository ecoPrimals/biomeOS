// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! JSON-RPC connection management for Neural API

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

use crate::error::NeuralApiError;

/// Connect to Neural API and execute JSON-RPC call
pub async fn json_rpc_call(
    socket_path: &PathBuf,
    method: &str,
    params: &Value,
    request_timeout: Duration,
    connection_timeout: Duration,
) -> Result<Value> {
    let mut stream = timeout(connection_timeout, UnixStream::connect(socket_path))
        .await
        .context("Connection timeout")?
        .with_context(|| {
            format!(
                "Failed to connect to Neural API at {}",
                socket_path.display()
            )
        })?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let request_bytes = serde_json::to_vec(&request).context("Failed to serialize request")?;

    stream
        .write_all(&request_bytes)
        .await
        .context("Failed to write request")?;
    stream
        .write_all(b"\n")
        .await
        .context("Failed to write newline")?;
    stream.flush().await.context("Failed to flush stream")?;

    let mut response_bytes = Vec::new();
    timeout(request_timeout, stream.read_to_end(&mut response_bytes))
        .await
        .context("Request timeout")?
        .context("Failed to read response")?;

    let response: Value =
        serde_json::from_slice(&response_bytes).context("Failed to parse JSON-RPC response")?;

    if let Some(error) = response.get("error") {
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error")
            .to_string();

        return Err(NeuralApiError::RpcError { code, message }.into());
    }

    response
        .get("result")
        .ok_or_else(|| anyhow::anyhow!("Response missing 'result' field"))
        .cloned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[tokio::test]
    async fn test_json_rpc_call_connection_timeout_on_nonexistent_socket() {
        let path = std::path::PathBuf::from("/nonexistent/socket/that/does/not/exist.sock");
        let result = json_rpc_call(
            &path,
            "test.method",
            &serde_json::json!({}),
            Duration::from_secs(1),
            Duration::from_millis(10),
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Failed to connect")
                || msg.contains("timeout")
                || msg.contains("Connection"),
            "Expected connection error, got: {}",
            msg
        );
    }
}
