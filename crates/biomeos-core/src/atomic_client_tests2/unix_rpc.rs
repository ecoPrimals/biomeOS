// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::atomic_client::AtomicClient;
use biomeos_test_utils::ready_signal;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_try_call_missing_result() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("missing_result.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            if n > 0 {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": null,
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.try_call("test", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, IpcError::MissingResult { .. }));
}

#[tokio::test]
async fn test_try_call_jsonrpc_error() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("jsonrpc_error.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": "Method not found"},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.try_call("nonexistent", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, IpcError::JsonRpcError { code: -32601, .. }));
}

#[tokio::test]
async fn test_call_success_with_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("success.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"status": "ok", "value": 42},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.call("test", Value::Null).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["value"], 42);
}

/// Server accepts the connection but never sends a line — `try_call` must return `IpcError::Timeout`.
#[tokio::test]
async fn test_try_call_timeout_while_reading_response() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("hang.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 256];
            let _ = stream.read(&mut buf).await;
            std::future::pending::<()>().await;
        }
    });

    ready_rx.wait().await.expect("ready");

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_millis(150));
    let result = client.try_call("test", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, IpcError::Timeout { .. }),
        "expected timeout, got {err:?}"
    );
}

#[tokio::test]
async fn test_try_call_jsonrpc_error_includes_code() {
    use biomeos_types::IpcError;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("code.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": "app err"},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let err = client
        .try_call("x", json!({}))
        .await
        .expect_err("jsonrpc err");
    match err {
        IpcError::JsonRpcError { code, .. } => assert_eq!(code, -32000),
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn test_try_call_jsonrpc_response_invalid_json_serialization() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("bad_json.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let _ = stream.write_all(b"not json at all\n").await;
        }
    });

    ready_rx.wait().await.expect("ready");

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let err = client
        .try_call("x", json!({}))
        .await
        .expect_err("invalid JSON line");
    assert!(
        matches!(err, IpcError::Serialization(_)),
        "expected Serialization, got {err:?}"
    );
}
