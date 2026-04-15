// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `capability.call` routing contract tests.

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test assertions")]

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use tempfile::tempdir;

use crate::handlers::capability_tests::{handler_with_registration, make_handler};

#[tokio::test]
async fn test_call_missing_params() {
    let handler = make_handler();
    let result = handler.call(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_capability() {
    let handler = make_handler();
    let params = Some(json!({ "operation": "sha256" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_operation() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_dotted_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto.sha256",
        "args": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing 'operation'"));
}

#[tokio::test]
async fn test_call_params_alias_for_args() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "params": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing"));
}

#[tokio::test]
async fn test_call_uses_translation_when_present() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-tr.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "abc" })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": sock.to_str().unwrap(),
        "source": "test",
        "semantic_mappings": { "sha256": "crypto.blake3_hash" }
    }));
    handler.register(&reg).await.expect("reg");

    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "args": { "data": "x" }
    }));
    let out = handler.call(&params).await.expect("call").result;
    assert_eq!(out["hashed"], "abc");
}

#[tokio::test]
async fn test_call_direct_without_translation_warn_path() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-direct.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "direct": true })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "custom",
        "primal": "p",
        "socket": sock.to_str().unwrap(),
        "source": "test"
    }));
    handler.register(&reg).await.expect("register");

    let params = Some(json!({
        "capability": "custom",
        "operation": "custom.op",
        "args": {}
    }));
    let out = handler.call(&params).await.expect("call").result;
    assert_eq!(out["direct"], true);
}

#[tokio::test]
async fn test_call_routing_trace_when_enabled() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-trace.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "abc" })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": sock.to_str().unwrap(),
        "source": "test",
        "semantic_mappings": { "sha256": "crypto.blake3_hash" }
    }));
    handler.register(&reg).await.expect("reg");

    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "args": { "data": "x" },
        "_routing_trace": true
    }));
    let out = handler.call(&params).await.expect("call");
    assert_eq!(out.result["hashed"], "abc");
    let trace = out.routing_trace.expect("trace requested");
    let phases = trace["phases"].as_array().expect("phases array");
    assert_eq!(
        phases.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>(),
        vec!["route_resolved", "endpoint_resolved", "forwarded"]
    );
    assert_eq!(trace["capability"], "crypto");
    assert!(trace["elapsed_ms"].as_u64().is_some());
}

#[tokio::test]
async fn test_call_routing_trace_absent_when_disabled() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-notrace.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "abc" })).await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": sock.to_str().unwrap(),
        "source": "test",
        "semantic_mappings": { "sha256": "crypto.blake3_hash" }
    }));
    handler.register(&reg).await.expect("reg");

    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "args": { "data": "x" }
    }));
    let out = handler.call(&params).await.expect("call");
    assert!(out.routing_trace.is_none());
}

#[tokio::test]
async fn test_call_with_unknown_gate_returns_error() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-gate.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "via": "local" })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "zcap",
            "primal": "z",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "zcap",
        "operation": "zcap.op",
        "args": {},
        "gate": "nonexistent_gate_label"
    }));
    let err = handler
        .call(&params)
        .await
        .expect_err("unknown gate should error, not fall through");
    let msg = err.to_string();
    assert!(
        msg.contains("not registered"),
        "Error should mention gate not registered: {msg}"
    );
}

#[tokio::test]
async fn test_call_with_local_gate_routes_locally() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("call-local-gate.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "via": "local" })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "zcap",
            "primal": "z",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "zcap",
        "operation": "zcap.op",
        "args": {},
        "gate": "local"
    }));
    let out = handler
        .call(&params)
        .await
        .expect("gate='local' should route locally")
        .result;
    assert_eq!(out["via"], "local");
}
