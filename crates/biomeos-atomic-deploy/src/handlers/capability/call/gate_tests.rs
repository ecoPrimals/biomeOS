// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for `handlers/capability/call/gate.rs`.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::sync::{Arc, Mutex};

use biomeos_core::TransportEndpoint;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::handlers::capability_tests::make_handler;

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

fn parse_request(req: &str) -> (String, Value) {
    let value = serde_json::from_str::<Value>(req).expect("valid json");
    let method = value
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();
    let params = value.get("params").cloned().unwrap_or(json!({}));
    (method, params)
}

#[tokio::test]
async fn gate_cross_gate_routes_to_registered_endpoint() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let remote_sock = dir.path().join("remote-gate.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&remote_sock, json!({ "remote": true })).await;

        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "westgate",
                "endpoint": remote_sock.to_str().unwrap()
            })))
            .await
            .expect("register gate");

        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": { "data": "abc" },
            "gate": "westgate"
        }));
        let out = handler.call(&params).await.expect("cross-gate call");
        assert_eq!(out.result["remote"], true);
    })
    .await;
}

#[tokio::test]
async fn gate_cross_gate_forwards_capability_call_payload() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let remote_sock = dir.path().join("remote-payload.sock");
        let captured = Arc::new(Mutex::new(None::<(String, Value)>));
        let captured_clone = Arc::clone(&captured);

        let _server = MockJsonRpcServer::spawn(&remote_sock, move |req| {
            let id = serde_json::from_str::<Value>(req)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(Value::Null);
            *captured_clone.lock().expect("lock") = Some(parse_request(req));
            format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"seen":true}}}}"#)
        })
        .await;

        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "pixel",
                "endpoint": remote_sock.to_str().unwrap()
            })))
            .await
            .expect("register gate");

        let params = Some(json!({
            "capability": "storage",
            "operation": "put",
            "args": { "key": "k1", "value": "v1" },
            "gate": "pixel"
        }));
        let out = handler
            .call(&params)
            .await
            .expect("cross-gate payload call");
        assert_eq!(out.result["seen"], true);

        let (method, forwarded) = captured
            .lock()
            .expect("lock")
            .take()
            .expect("captured request");
        assert_eq!(method, "capability.call");
        assert_eq!(forwarded["capability"], "storage");
        assert_eq!(forwarded["operation"], "put");
        assert_eq!(forwarded["args"]["key"], "k1");
    })
    .await;
}

#[tokio::test]
async fn gate_forwards_bearer_token_in_remote_args() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let remote_sock = dir.path().join("remote-auth.sock");
        let captured = Arc::new(Mutex::new(None::<Value>));
        let captured_clone = Arc::clone(&captured);

        let _server = MockJsonRpcServer::spawn(&remote_sock, move |req| {
            let (_, params) = parse_request(req);
            *captured_clone.lock().expect("lock") = Some(params);
            r#"{"jsonrpc":"2.0","id":1,"result":{"authorized":true}}"#.to_string()
        })
        .await;

        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "eastgate",
                "endpoint": remote_sock.to_str().unwrap()
            })))
            .await
            .expect("register gate");

        let params = Some(json!({
            "capability": "security",
            "operation": "verify",
            "args": { "payload": "secret" },
            "_bearer_token": "ionic-token-abc",
            "gate": "eastgate"
        }));
        let out = handler
            .call(&params)
            .await
            .expect("authenticated gate call");
        assert_eq!(out.result["authorized"], true);

        let forwarded = captured
            .lock()
            .expect("lock")
            .take()
            .expect("captured params");
        assert_eq!(
            forwarded["args"]["_bearer_token"], "ionic-token-abc",
            "bearer token should be forwarded inside remote args"
        );
    })
    .await;
}

#[tokio::test]
async fn gate_cross_gate_routing_trace() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let remote_sock = dir.path().join("remote-trace.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&remote_sock, json!({ "trace": true })).await;

        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "gate2",
                "endpoint": remote_sock.to_str().unwrap()
            })))
            .await
            .expect("register gate");

        let params = Some(json!({
            "capability": "network",
            "operation": "ping",
            "args": {},
            "gate": "gate2",
            "_routing_trace": true
        }));
        let out = handler.call(&params).await.expect("gate trace call");
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
        assert_eq!(trace["provider"], "gate2");
        assert_eq!(trace["method"], "capability.call");
        assert_eq!(
            trace["endpoint"],
            TransportEndpoint::UnixSocket {
                path: remote_sock.clone()
            }
            .display_string()
        );
    })
    .await;
}

#[tokio::test]
async fn gate_remote_call_error_propagates() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let remote_sock = dir.path().join("remote-error.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_error(&remote_sock, -32001, "PERMISSION_DENIED").await;

        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "remote",
                "endpoint": remote_sock.to_str().unwrap()
            })))
            .await
            .expect("register gate");

        let params = Some(json!({
            "capability": "security",
            "operation": "verify",
            "args": {},
            "gate": "remote"
        }));
        let err = handler
            .call(&params)
            .await
            .expect_err("remote error should propagate");
        let msg = err.to_string();
        assert!(
            msg.contains("PERMISSION_DENIED") || msg.contains("-32001"),
            "unexpected error: {msg}"
        );
    })
    .await;
}

#[tokio::test]
async fn gate_local_defers_to_local_dispatch() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let dir = TempDir::new().expect("tempdir");
        let local_sock = dir.path().join("local-gate.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&local_sock, json!({ "via": "local" })).await;

        let handler = make_handler();
        handler
            .register(&Some(json!({
                "capability": "localcap",
                "primal": "local-primal",
                "socket": local_sock.to_str().unwrap(),
                "source": "test"
            })))
            .await
            .expect("register local capability");

        let params = Some(json!({
            "capability": "localcap",
            "operation": "ping",
            "args": {},
            "gate": "local"
        }));
        let out = handler.call(&params).await.expect("local gate call");
        assert_eq!(out.result["via"], "local");
    })
    .await;
}

#[tokio::test]
async fn gate_unregistered_without_mesh_relay_errors_with_known_gates() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let handler = make_handler();
        handler
            .register_gate(&Some(json!({
                "gate": "known-gate",
                "endpoint": "/tmp/unused-gate.sock"
            })))
            .await
            .expect("register one gate");

        let params = Some(json!({
            "capability": "crypto",
            "operation": "hash",
            "args": {},
            "gate": "missing-gate"
        }));
        let err = handler.call(&params).await.expect_err("unknown gate");
        let msg = err.to_string();
        assert!(msg.contains("not registered"));
        assert!(msg.contains("relay fallback unavailable"));
        assert!(msg.contains("Known gates"));
    })
    .await;
}
