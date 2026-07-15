// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::transport::call_unix_socket_rpc;
use biomeos_test_utils::ready_signal;

#[tokio::test]
async fn test_call_unix_socket_rpc_non_json_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_bad_json.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, b"NOT VALID JSON {{{\n").await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "ping", serde_json::json!({}))
            .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid JSON-RPC") || msg.contains("JSON") || msg.contains("invalid"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_result_deserialize_mismatch() {
    #[derive(Debug, serde::Deserialize)]
    struct NeedsField {
        // Field is never read: this test only exercises the Err deserialize path.
        #[expect(dead_code, reason = "serde expected shape; only Err path is taken")]
        required_only: String,
    }

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_shape.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"wrong": "shape"},
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

    let result = call_unix_socket_rpc::<NeedsField>(&socket_path, "m", serde_json::json!({})).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("deserialize")
            || msg.contains("Failed to deserialize")
            || msg.contains("missing"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_call_unix_socket_rpc_server_closes_without_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("nucleus_eof.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "ping", serde_json::json!({}))
            .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_unix_socket_rpc_read_error_empty_after_headers() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("early_eof.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
            )
            .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let result =
        call_unix_socket_rpc::<serde_json::Value>(&socket_path, "m", serde_json::json!({})).await;
    assert!(result.is_err());
}
