// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! JSON-RPC connection management for Neural API

use anyhow::{Context, Result};
use biomeos_types::JsonRpcRequest;
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

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

    let request = JsonRpcRequest::new(method, params.clone());

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
        let code = error
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|c| i32::try_from(c).ok())
            .unwrap_or(-1);
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

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
            "Expected connection error, got: {msg}"
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_rpc_error_response() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("rpc_error.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let error_response =
                r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
            stream
                .write_all(error_response.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test.method",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Method not found") || msg.contains("-32601"),
            "Expected RPC error, got: {msg}"
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_missing_result_field() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("no_result.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let no_result = r#"{"jsonrpc":"2.0","id":1}"#;
            stream.write_all(no_result.as_bytes()).await.expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("missing 'result'") || err.to_string().contains("result"),
            "Expected missing result error: {}",
            err
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_rpc_error_with_minimal_error_object() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("minimal_error.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let minimal_error = r#"{"jsonrpc":"2.0","error":{},"id":1}"#;
            stream
                .write_all(minimal_error.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_invalid_json_response() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("invalid_json.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let invalid_json = b"{not valid json";
            stream.write_all(invalid_json).await.expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("JSON"),
            "Expected JSON parse error: {}",
            err
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_rpc_error_code_not_number() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("code_not_num.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let err_code_str =
                r#"{"jsonrpc":"2.0","error":{"code":"not-a-number","message":"Test"},"id":1}"#;
            stream
                .write_all(err_code_str.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_rpc_error_message_not_string() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("msg_not_str.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let err_msg_num = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":123},"id":1}"#;
            stream
                .write_all(err_msg_num.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Unknown") || err.to_string().contains("-32600"),
            "Expected RPC error: {}",
            err
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_json_rpc_call_success_with_result() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("success.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind");

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let success = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
            stream.write_all(success.as_bytes()).await.expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = json_rpc_call(
            &socket_path_clone,
            "test",
            &serde_json::json!({}),
            Duration::from_secs(2),
            Duration::from_secs(2),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("ok"), Some(&serde_json::json!(true)));
    }
}
