// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for `handlers/capability/call/dispatch/mod.rs`.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use biomeos_core::TransportEndpoint;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::handlers::capability_tests::make_handler;
use crate::handlers::graph::GraphHandler;
use crate::neural_router::NeuralRouter;

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

const MINIMAL_SIGNAL_GRAPH_TOML: &str = r#"
[graph]
id = "test_minimal"
version = "1.0.0"
description = "Minimal graph for capability.call dispatch tests"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "signal intercept test"
"#;

fn write_signal_graph(graphs_dir: &Path, tier: &str, signal: &str, toml: &str) {
    let signals_dir = graphs_dir.join("signals");
    std::fs::create_dir_all(&signals_dir).expect("signals dir");
    std::fs::write(signals_dir.join(format!("{tier}_{signal}.toml")), toml).expect("write graph");
}

fn make_graph_handler(graphs_dir: &Path) -> GraphHandler {
    GraphHandler::new(
        graphs_dir,
        "dispatch-test-family",
        Arc::new(RwLock::new(HashMap::new())),
        Arc::new(NeuralRouter::new("dispatch-test-family")),
        Arc::new(RwLock::new(CapabilityTranslationRegistry::new())),
    )
}

fn handler_with_signal_dispatch(
    tier: &str,
    signal: &str,
) -> (crate::handlers::CapabilityHandler, TempDir) {
    let temp = TempDir::new().expect("tempdir");
    write_signal_graph(temp.path(), tier, signal, MINIMAL_SIGNAL_GRAPH_TOML);
    let graph_handler = make_graph_handler(temp.path());
    let handler = make_handler().with_signal_dispatch(
        temp.path().to_path_buf(),
        "dispatch-test-family".to_string(),
        Arc::new(graph_handler),
    );
    (handler, temp)
}

async fn register_tower_atomic(
    handler: &crate::handlers::CapabilityHandler,
    discovery_sock: &Path,
    security_sock: &Path,
) {
    handler
        .register(&Some(json!({
            "capability": "security",
            "primal": "beardog",
            "socket": security_sock.to_str().unwrap(),
            "source": "test"
        })))
        .await
        .expect("register security");
    handler
        .register(&Some(json!({
            "capability": "discovery",
            "primal": "songbird",
            "socket": discovery_sock.to_str().unwrap(),
            "source": "test"
        })))
        .await
        .expect("register discovery");
}

#[tokio::test]
async fn dispatch_signal_intercept_executes_graph() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (handler, _temp) = handler_with_signal_dispatch("tower", "health");
        let params = Some(json!({
            "capability": "tower",
            "operation": "health",
            "args": { "check": true }
        }));
        let out = handler.call(&params).await.expect("signal intercept call");
        assert_eq!(out.result["signal"], "tower.health");
        assert_eq!(out.result["graph_id"], "signals/tower_health");
        assert!(out.result["execution"].is_object());
    })
    .await;
}

#[tokio::test]
async fn dispatch_signal_intercept_routing_trace() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (handler, _temp) = handler_with_signal_dispatch("meta", "observe");
        let params = Some(json!({
            "capability": "meta",
            "operation": "observe",
            "args": {},
            "_routing_trace": true
        }));
        let out = handler.call(&params).await.expect("signal intercept trace");
        let trace = out.routing_trace.expect("trace requested");
        let phases = trace["phases"]
            .as_array()
            .expect("phases")
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        assert_eq!(phases, vec!["route_resolved", "forwarded"]);
        assert_eq!(trace["provider"], "signal_graph");
        assert_eq!(trace["method"], "meta.observe");
    })
    .await;
}

#[tokio::test]
async fn dispatch_signal_tier_without_graph_falls_through_to_direct() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let sock = dir.path().join("tower-direct.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&sock, json!({ "via": "direct" })).await;

        let handler = make_handler();
        handler
            .register(&Some(json!({
                "capability": "tower",
                "primal": "tower-primal",
                "socket": sock.to_str().unwrap(),
                "source": "test"
            })))
            .await
            .expect("register tower capability");

        let params = Some(json!({
            "capability": "tower",
            "operation": "custom.op",
            "args": {}
        }));
        let out = handler.call(&params).await.expect("direct fallback");
        assert_eq!(out.result["via"], "direct");
    })
    .await;
}

#[tokio::test]
async fn dispatch_tower_atomic_relay_success() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let discovery_sock = dir.path().join("tower-discovery.sock");
        let security_sock = dir.path().join("tower-security.sock");
        let _security_server =
            MockJsonRpcServer::spawn_echo_success(&security_sock, json!({ "ok": true })).await;
        let _discovery_server = MockJsonRpcServer::spawn_echo_success(
            &discovery_sock,
            json!({ "relayed": true, "method": "http.get" }),
        )
        .await;

        let handler = make_handler();
        register_tower_atomic(&handler, &discovery_sock, &security_sock).await;

        let params = Some(json!({
            "capability": "http",
            "operation": "get",
            "args": { "url": "https://example.com" }
        }));
        let out = handler.call(&params).await.expect("tower atomic relay");
        assert_eq!(out.result["relayed"], true);
    })
    .await;
}

#[tokio::test]
async fn dispatch_tower_atomic_relay_routing_trace() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let discovery_sock = dir.path().join("tower-trace-discovery.sock");
        let security_sock = dir.path().join("tower-trace-security.sock");
        let _security_server =
            MockJsonRpcServer::spawn_echo_success(&security_sock, json!({ "ok": true })).await;
        let _discovery_server =
            MockJsonRpcServer::spawn_echo_success(&discovery_sock, json!({ "trace": true })).await;

        let handler = make_handler();
        register_tower_atomic(&handler, &discovery_sock, &security_sock).await;

        let params = Some(json!({
            "capability": "http",
            "operation": "get",
            "args": {},
            "_routing_trace": true
        }));
        let out = handler.call(&params).await.expect("tower relay trace");
        let trace = out.routing_trace.expect("trace requested");
        let phases = trace["phases"]
            .as_array()
            .expect("phases")
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            phases,
            vec!["route_resolved", "endpoint_resolved", "forwarded"]
        );
        assert_eq!(trace["provider"], "songbird");
        assert_eq!(trace["method"], "http.get");
        assert_eq!(
            trace["endpoint"],
            TransportEndpoint::UnixSocket {
                path: discovery_sock.clone()
            }
            .display_string()
        );
    })
    .await;
}

#[tokio::test]
async fn dispatch_tower_atomic_relay_failure_falls_back_to_direct() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let discovery_sock = dir.path().join("tower-fail-discovery.sock");
        let security_sock = dir.path().join("tower-fail-security.sock");
        let fallback_sock = dir.path().join("tower-fallback.sock");

        let _security_server =
            MockJsonRpcServer::spawn_echo_success(&security_sock, json!({ "ok": true })).await;
        let _discovery_server =
            MockJsonRpcServer::spawn_echo_error(&discovery_sock, -32000, "relay unavailable").await;
        let _fallback_server =
            MockJsonRpcServer::spawn_echo_success(&fallback_sock, json!({ "fallback": true }))
                .await;

        let handler = make_handler();
        register_tower_atomic(&handler, &discovery_sock, &security_sock).await;
        handler
            .register(&Some(json!({
                "capability": "mesh",
                "primal": "mesh-primal",
                "socket": fallback_sock.to_str().unwrap(),
                "source": "test"
            })))
            .await
            .expect("register fallback capability");

        let params = Some(json!({
            "capability": "mesh",
            "operation": "ping",
            "args": {}
        }));
        let out = handler
            .call(&params)
            .await
            .expect("fallback after relay failure");
        assert_eq!(out.result["fallback"], true);
    })
    .await;
}

#[tokio::test]
async fn dispatch_tower_atomic_relay_failure_without_fallback_errors() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let discovery_sock = dir.path().join("tower-no-fallback-discovery.sock");
        let security_sock = dir.path().join("tower-no-fallback-security.sock");

        let _security_server =
            MockJsonRpcServer::spawn_echo_success(&security_sock, json!({ "ok": true })).await;
        let _discovery_server =
            MockJsonRpcServer::spawn_echo_error(&discovery_sock, -32000, "relay unavailable").await;

        let handler = make_handler();
        register_tower_atomic(&handler, &discovery_sock, &security_sock).await;

        let params = Some(json!({
            "capability": "nonexistent_xyz",
            "operation": "missing",
            "args": {}
        }));
        let err = handler
            .call(&params)
            .await
            .expect_err("unregistered capability should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("not registered")
                || msg.contains("not found")
                || msg.contains("No primal"),
            "unexpected error: {msg}"
        );
    })
    .await;
}
