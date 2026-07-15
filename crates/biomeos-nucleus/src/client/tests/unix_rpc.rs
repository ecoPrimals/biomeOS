// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::transport::call_unix_socket_rpc;
use biomeos_test_utils::ready_signal;

#[tokio::test]
async fn test_call_unix_socket_rpc_connection_refused() {
    let result = call_unix_socket_rpc::<serde_json::Value>(
        "/nonexistent/socket/path/12345.sock",
        "ping",
        serde_json::json!({}),
    )
    .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("socket")
            || err.to_string().contains("connect")
            || err.to_string().contains("Connection"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_success() {
    #[derive(serde::Deserialize)]
    struct TestResult {
        success: bool,
        primal: String,
    }

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_test.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"success": true, "primal": "beardog"},
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<TestResult>(&socket_path, "test_method", serde_json::json!({}))
            .await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.success);
    assert_eq!(value.primal, "beardog");
}

#[tokio::test]
async fn test_call_unix_socket_rpc_jsonrpc_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_error.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": "Method not found"},
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result = call_unix_socket_rpc::<serde_json::Value>(
        &socket_path,
        "nonexistent",
        serde_json::json!({}),
    )
    .await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Method not found") || err_msg.contains("-32601"),
        "{}",
        err_msg
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_missing_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_missing.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": null,
                "id": 1
            });
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "test", serde_json::json!({}))
            .await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Missing") || err_msg.contains("result"),
        "{}",
        err_msg
    );
}
