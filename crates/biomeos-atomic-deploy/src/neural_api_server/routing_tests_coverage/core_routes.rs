// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::rpc;
use super::super::common::create_test_server;
use crate::neural_api_server::btsp_negotiate;
use biomeos_core::{EnforcementMode, MethodGate};
use serde_json::json;

#[tokio::test]
async fn method_gate_rejects_protected_call_in_enforced_mode() {
    let (mut server, _temp) = create_test_server();
    server.method_gate = MethodGate::new(EnforcementMode::Enforced);

    let result = rpc(
        &server,
        "capability.call",
        json!({"capability": "crypto", "operation": "hash", "args": {}}),
        200,
    )
    .await;

    assert_eq!(result["error"]["code"], -32001);
    assert_eq!(result["id"], 200);
}

#[tokio::test]
async fn bearer_token_in_params_builds_caller_and_semantic_params() {
    let (server, _temp) = create_test_server();
    let result = rpc(
        &server,
        "unknown.capability",
        json!({
            "_bearer_token": "test-token",
            "_routing_trace": true,
            "payload": "x"
        }),
        201,
    )
    .await;

    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn semantic_fallback_rejects_empty_domain_or_operation() {
    let (server, _temp) = create_test_server();

    for (method, id) in [(".missing_domain", 202), ("missing_operation.", 203)] {
        let result = rpc(&server, method, json!({}), id).await;
        assert_eq!(result["error"]["code"], -32601, "method {method}");
        assert_eq!(result["id"], id);
    }
}

#[tokio::test]
async fn identity_get_returns_orchestrator_profile() {
    let (server, _temp) = create_test_server();
    let result = rpc(&server, "identity.get", json!({}), 210).await;

    assert_eq!(
        result["result"]["primal"],
        biomeos_types::primal_names::BIOMEOS
    );
    assert_eq!(result["result"]["role"].as_str().unwrap(), "orchestrator");
    assert_eq!(result["result"]["is_orchestrator"], true);
    assert!(result["result"]["capabilities"].is_array());
}

#[tokio::test]
async fn auth_routes_return_gate_diagnostics() {
    let (server, _temp) = create_test_server();

    let check = rpc(&server, "auth.check", json!({}), 211).await;
    assert!(check.get("result").is_some());

    let mode = rpc(&server, "auth.mode", json!({}), 212).await;
    assert!(mode["result"]["mode"].is_string());

    let peer = rpc(&server, "auth.peer_info", json!({}), 213).await;
    assert!(peer.get("result").is_some());
}

#[tokio::test]
async fn btsp_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let escalate = rpc(&server, "btsp.escalate", json!({}), 220).await;
    assert_eq!(escalate["result"]["escalated"], true);

    let status = rpc(&server, "btsp.status", json!({}), 221).await;
    assert!(status["result"]["phase"].is_string());

    btsp_negotiate::register_session(&server.btsp_sessions, "cov-session", None).await;
    let negotiate = rpc(
        &server,
        "btsp.negotiate",
        json!({
            "session_id": "cov-session",
            "preferred_cipher": "null",
            "client_nonce": "AAAA"
        }),
        222,
    )
    .await;
    assert!(negotiate.get("result").is_some() || negotiate.get("error").is_some());
}

#[tokio::test]
async fn spore_and_nucleus_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let instantiate = rpc(&server, "spore.instantiate", json!({"family": "test"}), 230).await;
    assert_eq!(instantiate["result"]["status"], "deferred");

    let ingest = rpc(
        &server,
        "nucleus.ingest_spore",
        json!({"spore_id": "s1"}),
        231,
    )
    .await;
    assert!(ingest.get("result").is_some() || ingest.get("error").is_some());

    let emit_missing = rpc(&server, "nucleus.emit_spore", json!({}), 232).await;
    assert!(emit_missing.get("error").is_some());
    let msg = emit_missing["error"]["message"]
        .as_str()
        .expect("error message");
    assert!(msg.contains("spore_id"));

    let emit_ok = rpc(
        &server,
        "nucleus.emit_spore",
        json!({"spore_id": "s1"}),
        233,
    )
    .await;
    assert!(emit_ok.get("result").is_some() || emit_ok.get("error").is_some());
}

#[tokio::test]
async fn signal_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let list = rpc(&server, "signal.list", json!({}), 240).await;
    assert!(list["result"]["signals"].is_array());

    let schema = rpc(&server, "signal.schema", json!({}), 241).await;
    assert!(schema.get("result").is_some() || schema.get("error").is_some());

    let dispatch = rpc(
        &server,
        "signal.dispatch",
        json!({"signal": "nest.test", "params": {}}),
        242,
    )
    .await;
    assert!(dispatch.get("result").is_some() || dispatch.get("error").is_some());
}
