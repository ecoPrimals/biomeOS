// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::atomic_client::AtomicClient;
use serde_json::Value;
use std::time::Duration;

#[tokio::test]
async fn test_atomic_client_call_connection_refused() {
    let client = AtomicClient::unix("/nonexistent/socket/path/12345.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("No such file")
            || err.contains("Connection refused"),
        "Expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_atomic_client_try_call_connection_refused() {
    use biomeos_types::IpcError;

    let client = AtomicClient::unix("/nonexistent/socket/path/67890.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.try_call("ping", Value::Null).await;
    assert!(result.is_err());
    let ipc_err = result.unwrap_err();
    assert!(
        matches!(ipc_err, IpcError::ConnectionFailed { .. }) || ipc_err.is_timeout(),
        "Expected ConnectionFailed or Timeout, got: {ipc_err:?}"
    );
}

#[tokio::test]
async fn test_atomic_client_tcp_connection_refused() {
    let client = AtomicClient::tcp("127.0.0.1", 59999).with_timeout(Duration::from_millis(100));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_client_call_timeout() {
    let client =
        AtomicClient::unix("/nonexistent/socket.sock").with_timeout(Duration::from_millis(1));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_stream_connection_refused() {
    let client = AtomicClient::unix("/nonexistent/socket/stream_test.sock")
        .with_timeout(Duration::from_millis(100));

    let mut rx = client
        .call_stream("stream_method", Value::Null)
        .expect("call_stream returns receiver");

    let item = rx.recv().await;
    assert!(item.is_some());
    let item = item.unwrap();
    assert!(
        matches!(item, biomeos_graph::StreamItem::Error { .. })
            || matches!(item, biomeos_graph::StreamItem::End)
    );
}
