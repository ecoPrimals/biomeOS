// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::Path;

use biomeos_types::JsonRpcRequest;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

use super::common::spawn_mock_neural_api;
use super::super::{poll_execution, send_jsonrpc};

#[tokio::test]
async fn test_send_jsonrpc_connection_failure() {
    let request = JsonRpcRequest::new("health.check", serde_json::json!({}));
    let result = send_jsonrpc(
        Path::new("/tmp/nonexistent-neural-api-send-jsonrpc.sock"),
        &request,
    )
    .await;

    let err = result.expect_err("connection to missing socket should fail");
    assert!(
        err.to_string().contains("Neural API") || err.to_string().contains("connect"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_send_jsonrpc_invalid_json_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("invalid-json.sock");
    let sock = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock).expect("bind");
        let _ = ready_tx.send(());
        if let Ok((stream, _)) = listener.accept().await {
            let (_, mut writer) = stream.into_split();
            let _ = writer.write_all(b"not valid json\n").await;
        }
    });

    ready_rx.await.expect("server ready");

    let request = JsonRpcRequest::new("test.method", serde_json::json!({}));
    let result = send_jsonrpc(&socket_path, &request).await;
    server.abort();

    let err = result.expect_err("invalid JSON should fail");
    assert!(
        err.to_string().contains("Invalid JSON-RPC response"),
        "expected parse error: {err}"
    );
}

#[tokio::test]
async fn test_send_jsonrpc_success() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "result": { "ok": true }
    });

    let (socket, server) = spawn_mock_neural_api(vec![response]).await;
    let request = JsonRpcRequest::new("signal.dispatch", serde_json::json!({"signal": "test"}));

    let result = send_jsonrpc(&socket, &request).await;
    server.abort();

    let value = result.expect("send_jsonrpc should succeed");
    assert_eq!(
        value.get("result").and_then(|v| v.get("ok")),
        Some(&serde_json::json!(true))
    );
}

#[tokio::test]
async fn test_poll_execution_completed() {
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "state": "completed",
            "execution_id": "exec-poll-done",
            "nodes": { "retrieve_content": { "result": { "data": "x" } } }
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![status_response]).await;

    let status = poll_execution(&socket, "exec-poll-done")
        .await
        .expect("poll should return completed status");

    server.abort();
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("completed")
    );
}

#[tokio::test]
async fn test_poll_execution_failed() {
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "state": "failed",
            "error": "node timeout"
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![status_response]).await;

    let result = poll_execution(&socket, "exec-poll-fail").await;
    server.abort();

    let err = result.expect_err("failed state should bail");
    assert!(
        err.to_string().contains("failed"),
        "expected failure message: {err}"
    );
}

#[tokio::test]
async fn test_poll_execution_retries_on_transient_socket_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("retry-poll.sock");
    let sock = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock).expect("bind");
        let _ = ready_tx.send(());

        // First accept: drop connection without responding (client gets EOF / parse issues on retry path)
        if let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }

        // Second accept: return completed status
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": { "state": "completed", "execution_id": "exec-retry" }
                });
                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
            }
        }
    });

    ready_rx.await.expect("server ready");

    // First poll attempt hits dropped stream; poll_execution logs and retries with backoff.
    let status = poll_execution(&socket_path, "exec-retry")
        .await
        .expect("poll should eventually succeed after retry");

    server.abort();
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("completed")
    );
}
