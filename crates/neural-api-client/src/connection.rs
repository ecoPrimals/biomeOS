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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    async fn run_mock_server(socket_path: PathBuf, response: serde_json::Value) {
        let listener = UnixListener::bind(&socket_path).expect("bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let _req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
            let line = format!("{}\n", response);
            stream.write_all(line.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        }
    }

    #[tokio::test]
    async fn test_json_rpc_call_success() {
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("neural.sock");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"ok": true},
            "id": 1
        });
        let server = tokio::spawn(run_mock_server(socket_path.clone(), resp));
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path,
            "test.method",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("ok").and_then(|v| v.as_bool()), Some(true));
        server.abort();
    }

    #[tokio::test]
    async fn test_json_rpc_call_error_response() {
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("neural.sock");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid request"},
            "id": 1
        });
        let server = tokio::spawn(run_mock_server(socket_path.clone(), resp));
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path,
            "test.method",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid request"));
        server.abort();
    }

    #[tokio::test]
    async fn test_json_rpc_call_missing_result() {
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("neural.sock");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        let server = tokio::spawn(run_mock_server(socket_path.clone(), resp));
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path,
            "test.method",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("result"));
        server.abort();
    }

    #[tokio::test]
    async fn test_json_rpc_call_nonexistent_socket() {
        let socket_path = PathBuf::from("/nonexistent/path/neural.sock");
        let result = json_rpc_call(
            &socket_path,
            "test.method",
            &serde_json::json!({}),
            Duration::from_millis(100),
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
    }
}
