// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for `handlers/capability/call/mesh.rs`.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::sync::{Arc, Mutex};

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::handlers::capability_tests::make_handler;

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

fn parse_request_params(req: &str) -> Value {
    serde_json::from_str::<Value>(req)
        .ok()
        .and_then(|v| v.get("params").cloned())
        .unwrap_or(json!({}))
}

async fn register_relay(
    handler: &crate::handlers::CapabilityHandler,
    relay_sock: &std::path::Path,
) {
    handler
        .register(&Some(json!({
            "capability": "relay",
            "primal": "songbird",
            "socket": relay_sock.to_str().unwrap(),
            "source": "test"
        })))
        .await
        .expect("register relay capability");
}

#[tokio::test]
async fn mesh_fallback_via_unknown_gate_success() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-relay.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &relay_sock,
            json!({
                "provider": "remote-crypto",
                "gate": "westgate",
                "result": { "hashed": "deadbeef" }
            }),
        )
        .await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": { "data": "hello" },
            "gate": "westgate"
        }));
        let out = handler.call(&params).await.expect("mesh fallback call");
        assert_eq!(out.result["hashed"], "deadbeef");
    })
    .await;
}

#[tokio::test]
async fn mesh_unwraps_inner_result_from_songbird_wrapper() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-unwrap.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &relay_sock,
            json!({
                "provider": "p",
                "gate": "g",
                "result": { "inner": 42 }
            }),
        )
        .await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "storage",
            "operation": "get",
            "args": { "key": "k" },
            "gate": "any-gate"
        }));
        let out = handler.call(&params).await.expect("mesh unwrap call");
        assert_eq!(out.result["inner"], 42);
        assert!(out.result.get("provider").is_none());
        assert!(out.result.get("gate").is_none());
    })
    .await;
}

#[tokio::test]
async fn mesh_forwards_routing_any_in_songbird_params() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-routing.sock");
        let captured = Arc::new(Mutex::new(None::<Value>));
        let captured_clone = Arc::clone(&captured);

        let _server = MockJsonRpcServer::spawn(&relay_sock, move |req| {
            *captured_clone.lock().expect("lock") = Some(parse_request_params(req));
            r#"{"jsonrpc":"2.0","id":1,"result":{"provider":"p","gate":"g","result":{"ok":true}}}"#
                .to_string()
        })
        .await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "network",
            "operation": "connect",
            "args": { "host": "10.0.0.1" },
            "gate": "remote-gate"
        }));
        handler.call(&params).await.expect("mesh routing call");

        let forwarded = captured
            .lock()
            .expect("lock")
            .take()
            .expect("captured params");
        assert_eq!(forwarded["routing"], "any");
        assert_eq!(forwarded["capability"], "network");
        assert_eq!(forwarded["operation"], "connect");
        assert_eq!(forwarded["params"]["host"], "10.0.0.1");
    })
    .await;
}

#[tokio::test]
async fn mesh_discovery_failure_when_relay_unavailable() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": {},
            "gate": "westgate"
        }));
        let err = handler
            .call(&params)
            .await
            .expect_err("mesh unavailable without relay");
        let msg = err.to_string();
        assert!(msg.contains("relay fallback unavailable"));
        assert!(msg.contains("not registered"));
    })
    .await;
}

#[tokio::test]
async fn mesh_forward_failure_returns_gate_error() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-fail.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_error(&relay_sock, -32000, "mesh unreachable").await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": {},
            "gate": "westgate"
        }));
        let err = handler
            .call(&params)
            .await
            .expect_err("mesh forward failure");
        let msg = err.to_string();
        assert!(
            msg.contains("not registered") && msg.contains("relay fallback unavailable"),
            "should fall through to gate error when mesh returns None: {msg}"
        );
    })
    .await;
}

#[tokio::test]
async fn mesh_routing_trace_via_gate_fallback() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-trace.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &relay_sock,
            json!({
                "provider": "remote",
                "gate": "westgate",
                "result": { "ok": true }
            }),
        )
        .await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": {},
            "gate": "westgate",
            "_routing_trace": true
        }));
        let out = handler.call(&params).await.expect("mesh trace call");
        let trace = out.routing_trace.expect("trace requested");
        let phases = trace["phases"]
            .as_array()
            .expect("phases")
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        assert_eq!(phases, vec!["route_resolved", "forwarded"]);
        assert_eq!(trace["provider"], "relay:westgate");
        assert_eq!(trace["method"], "capability.call");
    })
    .await;
}

#[tokio::test]
async fn mesh_success_via_direct_dispatch_when_local_forward_fails() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let local_sock = dir.path().join("mesh-local-fail.sock");
        let relay_sock = dir.path().join("mesh-direct-fallback.sock");

        let _local_server =
            MockJsonRpcServer::spawn_echo_error(&local_sock, -32000, "partition").await;
        let _relay_server = MockJsonRpcServer::spawn_echo_success(
            &relay_sock,
            json!({
                "provider": "mesh",
                "gate": "remote",
                "result": { "via_mesh": true }
            }),
        )
        .await;

        let handler = make_handler();
        handler
            .register(&Some(json!({
                "capability": "compute",
                "primal": "compute-primal",
                "socket": local_sock.to_str().unwrap(),
                "source": "test"
            })))
            .await
            .expect("register compute");
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "compute",
            "operation": "run",
            "args": { "job": "x" }
        }));
        let out = handler
            .call(&params)
            .await
            .expect("mesh fallback after local forward failure");
        assert_eq!(out.result["via_mesh"], true);
    })
    .await;
}

#[tokio::test]
async fn mesh_response_without_result_field_returns_full_payload() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let relay_sock = dir.path().join("mesh-no-result.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&relay_sock, json!({ "flat": "payload" })).await;

        let handler = make_handler();
        register_relay(&handler, &relay_sock).await;

        let params = Some(json!({
            "capability": "storage",
            "operation": "list",
            "args": {},
            "gate": "remote"
        }));
        let out = handler.call(&params).await.expect("flat mesh response");
        assert_eq!(out.result["flat"], "payload");
    })
    .await;
}
