// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{create_router, run_mock_jsonrpc_server, unix_ep};
use crate::living_graph::{LivingGraph, PrimalProtocolState};
use biomeos_test_utils::ready_signal;
use biomeos_types::tarpc_types::ProtocolPreference;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::UnixListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn test_forward_request_jsonrpc_success() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("test-primal.sock");
    let rpc_response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"healthy": true, "uptime_secs": 42},
        "id": 1
    });

    let (ready_tx, ready_rx) = oneshot::channel();
    let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
    ready_rx.await.expect("server ready");

    let router = create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);

    let result = router
        .forward_request(
            &unix_ep(&socket_path),
            "health.check",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["healthy"], true);
    assert_eq!(value["uptime_secs"], 42);
}

#[tokio::test]
async fn test_forward_request_jsonrpc_socket_not_found() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("nonexistent.sock");

    let router = create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);

    let result = router
        .forward_request(
            &unix_ep(&socket_path),
            "health.check",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Failed to forward")
            || msg.contains("connect")
            || msg.contains("No such file"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn test_forward_request_tarpc_fallback_to_jsonrpc() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("test-primal.sock");
    let rpc_response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"ok": true},
        "id": 1
    });

    let (ready_tx, ready_rx) = oneshot::channel();
    let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
    ready_rx.await.expect("server ready");

    let router = create_router("test").with_protocol_preference(ProtocolPreference::PreferTarpc);

    let result = router
        .forward_request(
            &unix_ep(&socket_path),
            "some.method",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap()["ok"], true);
}

#[tokio::test]
async fn test_forward_request_jsonrpc_times_out_when_server_hangs() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("hanging.sock");
    let path = socket_path.clone();

    let (mut ready_tx, ready_rx) = ready_signal();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path).expect("bind");
        ready_tx.signal();
        if let Ok((_, _)) = listener.accept().await {
            std::future::pending::<()>().await;
        }
    });

    ready_rx.wait().await.expect("mock server ready");

    let mut router =
        create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);
    router.request_timeout = Duration::from_millis(200);

    let result = router
        .forward_request(
            &unix_ep(&socket_path),
            "health.check",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_err(), "expected timeout error, got {result:?}");
}

#[tokio::test]
async fn test_forward_request_with_living_graph_records_success_path() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("record.sock");
    let rpc_response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"recorded": true},
        "id": 1
    });

    let (ready_tx, ready_rx) = oneshot::channel();
    let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
    ready_rx.await.expect("server ready");

    let graph = Arc::new(LivingGraph::new("test-family"));
    let state = PrimalProtocolState::new("record", socket_path.clone());
    graph.register_primal(state).await;

    let router = create_router("test-family")
        .with_protocol_preference(ProtocolPreference::JsonRpcOnly)
        .with_living_graph(graph);

    let result = router
        .forward_request(&unix_ep(&socket_path), "any.method", &serde_json::json!({}))
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap()["recorded"], true);
}

#[tokio::test]
async fn test_forward_request_tarpc_only_fails_without_server() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("solo.sock");

    let router = create_router("test").with_protocol_preference(ProtocolPreference::TarpcOnly);

    let result = router
        .forward_request(
            &unix_ep(&socket_path),
            "health.check",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_request_jsonrpc_error_response_from_server() {
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("err.sock");
    let rpc_response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -1, "message": "method not found"},
        "id": null
    });

    let (ready_tx, ready_rx) = oneshot::channel();
    let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
    ready_rx.await.expect("server ready");

    let router = create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);

    let result = router
        .forward_request(&unix_ep(&socket_path), "bad.method", &serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
