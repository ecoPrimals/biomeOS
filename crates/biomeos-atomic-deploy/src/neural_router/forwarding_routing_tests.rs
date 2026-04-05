// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Integration tests for [`super::forwarding`] routing: `should_use_tarpc`,
//! `forward_request`, `forward_via_tarpc`, and `primal_label_for_endpoint`.

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::NeuralRouter;
use crate::living_graph::{LivingGraph, PrimalProtocolState, ProtocolMode};
use biomeos_core::TransportEndpoint;
use biomeos_test_utils::ready_signal;
use biomeos_types::tarpc_types::ProtocolPreference;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

fn create_router(family_id: &str) -> NeuralRouter {
    NeuralRouter::new(family_id)
}

fn unix_ep(path: &std::path::Path) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: path.to_path_buf(),
    }
}

// --- should_use_tarpc tests ---

#[tokio::test]
async fn test_should_use_tarpc_jsonrpc_only_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);
    let ep = unix_ep(&PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_tarpc_only_returns_true() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::TarpcOnly);
    let ep = unix_ep(&PathBuf::from("/tmp/test-primal.sock"));
    assert!(router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_prefer_jsonrpc_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::PreferJsonRpc);
    let ep = unix_ep(&PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_prefer_tarpc_no_graph_returns_false() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::PreferTarpc);
    let ep = unix_ep(&PathBuf::from("/tmp/test-primal.sock"));
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_tarpc_available() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Tarpc;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_jsonrpc_mode_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_tarpc_mode() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Tarpc;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_hybrid_mode() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let tarpc_sock = temp.path().join("beardog.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_sock);

    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
        .with_tarpc_socket(tarpc_sock)
        .with_capabilities(vec!["security".to_string()]);
    state.current_mode = ProtocolMode::Hybrid;
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

#[tokio::test]
async fn test_should_use_tarpc_auto_with_graph_no_tarpc_socket_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("beardog.sock");
    let graph = Arc::new(LivingGraph::new("test"));
    let state = PrimalProtocolState::new("beardog", json_sock.clone());
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
}

// --- forward_via_tarpc error path tests ---

#[tokio::test]
async fn test_forward_via_tarpc_socket_not_found() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("nonexistent.sock");
    let result = router
        .forward_via_tarpc(&socket_path, "health.check", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("tarpc socket not found") || err.contains("not found"));
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_method_requires_tarpc_server() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.unknown_method",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("discovery") || err.contains("connect") || err.contains("tarpc"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_method_requires_tarpc_server() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "security.unknown_method",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("security") || err.contains("connect") || err.contains("tarpc"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_no_tarpc_mapping() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "custom.unknown", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("no tarpc mapping"));
}

// --- forward_request tests (JSON-RPC path) ---

async fn run_mock_jsonrpc_server(
    socket_path: &std::path::Path,
    response: serde_json::Value,
    ready_tx: Option<oneshot::Sender<()>>,
) -> tokio::task::JoinHandle<()> {
    let path = socket_path.to_path_buf();
    let response_json = serde_json::to_string(&response).expect("serialize");

    tokio::spawn(async move {
        let listener = UnixListener::bind(&path).expect("bind");
        if let Some(tx) = ready_tx {
            let _ = tx.send(());
        }
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await.expect("read");
            let _request = &buf[..n];

            let response_line = format!("{response_json}\n");
            stream
                .write_all(response_line.as_bytes())
                .await
                .expect("write");
            stream.flush().await.expect("flush");
        }
    })
}

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
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed to forward")
            || err.contains("connect")
            || err.contains("No such file"),
        "unexpected error: {err}"
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
    use std::time::Duration;

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
async fn test_forward_via_tarpc_discovery_unknown_method_after_socket_exists() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("disc.sock");
    let tarpc_path = temp.path().join("disc.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.not_a_real_method",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("unknown discovery method") || err.contains("connect"),
        "unexpected: {err}"
    );
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
async fn test_forward_via_tarpc_discovery_register_invalid_body() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("p.sock");
    let tarpc_path = temp.path().join("p.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.register",
            &serde_json::json!({"not": "ServiceRegistration"}),
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("serde") || err.contains("connect") || err.contains("register"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_sign_missing_data() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("sec.sock");
    let tarpc_path = temp.path().join("sec.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "security.sign", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("missing param: data") || err.contains("connect"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_unknown_method() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("sec2.sock");
    let tarpc_path = temp.path().join("sec2.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "security.unknown_xyz", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("unknown security") || err.contains("connect"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_health_metrics_alias() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("hm.sock");
    let tarpc_path = temp.path().join("hm.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "health_metrics", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_via_tarpc_health_version_alias() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("hv.sock");
    let tarpc_path = temp.path().join("hv.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "version", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_discover_all() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("da.sock");
    let tarpc_path = temp.path().join("da.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery_discover_all",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_should_use_tarpc_auto_no_primal_in_graph_returns_false() {
    let temp = TempDir::new().expect("temp dir");
    let json_sock = temp.path().join("other.sock");
    let graph = Arc::new(LivingGraph::new("test"));
    let mut state = PrimalProtocolState::new("beardog", temp.path().join("beardog.sock"))
        .with_tarpc_socket(temp.path().join("beardog.tarpc.sock"));
    state.current_mode = ProtocolMode::Tarpc;
    let _ = std::fs::File::create(temp.path().join("beardog.tarpc.sock"));
    graph.register_primal(state).await;

    let router = create_router("test")
        .with_protocol_preference(ProtocolPreference::Auto)
        .with_living_graph(graph);

    assert!(!router.should_use_tarpc(&unix_ep(&json_sock)).await);
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

#[tokio::test]
async fn test_should_use_tarpc_tcp_endpoint_with_auto_no_graph() {
    let router = create_router("test").with_protocol_preference(ProtocolPreference::Auto);
    let ep = TransportEndpoint::TcpSocket {
        host: Arc::from("192.0.2.100"),
        port: 9001,
    };
    assert!(!router.should_use_tarpc(&ep).await);
}

#[tokio::test]
async fn test_primal_label_for_endpoint_variants() {
    let router = create_router("test");

    let unix = TransportEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/beardog.sock"),
    };
    assert_eq!(
        router.primal_label_for_endpoint(&unix),
        Some("beardog".to_string())
    );

    let tcp = TransportEndpoint::TcpSocket {
        host: Arc::from("192.0.2.100"),
        port: 9001,
    };
    assert_eq!(
        router.primal_label_for_endpoint(&tcp),
        Some("192.0.2.100:9001".to_string())
    );

    let abs = TransportEndpoint::AbstractSocket {
        name: Arc::from("squirrel_abc"),
    };
    assert_eq!(
        router.primal_label_for_endpoint(&abs),
        Some("squirrel_abc".to_string())
    );

    let http = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("songbird.local"),
        port: 8080,
    };
    assert_eq!(
        router.primal_label_for_endpoint(&http),
        Some("songbird.local:8080".to_string())
    );
}
