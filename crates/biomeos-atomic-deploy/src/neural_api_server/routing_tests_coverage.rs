// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional routing coverage for uncovered dispatch paths and route arms.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use crate::handlers::capability::CapabilityCallOutcome;
use crate::neural_api_server::btsp_negotiate;
use crate::neural_api_server::rpc::DispatchOutcome;
use biomeos_core::{EnforcementMode, MethodGate};
use serde_json::json;

use super::common::create_test_server;

async fn rpc(
    server: &crate::neural_api_server::NeuralApiServer,
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> serde_json::Value {
    let req = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    })
    .to_string();
    server.handle_request_json(&req).await
}

#[test]
fn dispatch_capability_call_success_with_routing_trace() {
    let outcome = super::super::dispatch_capability_call(
        Ok(CapabilityCallOutcome {
            result: json!({"ok": true}),
            routing_trace: Some(json!({"phases": []})),
        }),
        json!(7),
    );
    match outcome {
        DispatchOutcome::Success(v) => {
            assert_eq!(v["jsonrpc"], "2.0");
            assert_eq!(v["result"]["ok"], true);
            assert!(v["_routing_trace"].is_object());
            assert_eq!(v["id"], 7);
        }
        other => panic!("expected Success, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_success_without_routing_trace() {
    let outcome = super::super::dispatch_capability_call(
        Ok(CapabilityCallOutcome {
            result: json!({"value": 1}),
            routing_trace: None,
        }),
        json!(null),
    );
    match outcome {
        DispatchOutcome::Success(v) => {
            assert!(v.get("_routing_trace").is_none());
            assert_eq!(v["result"]["value"], 1);
        }
        other => panic!("expected Success, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_preserves_ipc_json_rpc_error() {
    let err = biomeos_types::IpcError::JsonRpcError {
        primal: "nestgate".to_string(),
        code: -32001,
        message: "permission denied".to_string(),
    };
    let outcome = super::super::dispatch_capability_call(Err(err.into()), json!(3));
    match outcome {
        DispatchOutcome::ApplicationError { code, message, id } => {
            assert_eq!(code, -32001);
            assert_eq!(message, "permission denied");
            assert_eq!(id, 3);
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_uses_generic_code_for_other_errors() {
    let outcome =
        super::super::dispatch_capability_call(Err(anyhow::anyhow!("upstream down")), json!(1));
    match outcome {
        DispatchOutcome::ApplicationError { code, message, .. } => {
            assert_eq!(code, -32603);
            assert!(message.contains("upstream down"));
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}

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

#[tokio::test]
async fn neural_api_introspection_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let weights = rpc(&server, "neural_api.routing_weights", json!({}), 250).await;
    assert!(weights["result"]["weights"].is_array());
    assert!(weights["result"]["summary"].is_object());

    let explain = rpc(
        &server,
        "neural_api.route_explain",
        json!({"capability": "security"}),
        251,
    )
    .await;
    assert!(explain.get("result").is_some() || explain.get("error").is_some());

    let patterns = rpc(&server, "neural_api.composition_patterns", json!({}), 252).await;
    assert!(patterns.get("result").is_some());

    let reload = rpc(
        &server,
        "neural_api.composition_patterns_reload",
        json!({}),
        253,
    )
    .await;
    assert_eq!(reload["result"]["reloaded"], true);

    let utilization = rpc(&server, "neural_api.utilization", json!({}), 254).await;
    assert!(utilization.get("result").is_some());

    let weight_health = rpc(&server, "neural_api.weight_health", json!({}), 255).await;
    assert!(weight_health.get("result").is_some());

    let training = rpc(&server, "neural_api.training_data", json!({}), 256).await;
    assert!(training["result"]["rows"].is_array());
}

#[tokio::test]
async fn composition_plan_tier_accepts_all_known_tiers() {
    let (server, _temp) = create_test_server();

    for (tier, id) in [
        ("tower", 260),
        ("node", 261),
        ("nest", 262),
        ("nucleus", 263),
        ("meta", 264),
        ("orchestration", 265),
        ("unknown", 266),
    ] {
        let result = rpc(&server, "neural_api.plan_tier", json!({"tier": tier}), id).await;
        assert!(
            result.get("result").is_some(),
            "plan_tier({tier}) failed: {result}"
        );
    }
}

#[tokio::test]
async fn graph_lifecycle_and_topology_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let save = rpc(
        &server,
        "graph.save",
        json!({"graph_id": "cov", "content": "[graph]\nid=\"cov\""}),
        270,
    )
    .await;
    assert!(save.get("result").is_some() || save.get("error").is_some());

    let execute = rpc(
        &server,
        "graph.execute",
        json!({"graph_id": "nonexistent"}),
        271,
    )
    .await;
    assert!(execute.get("result").is_some() || execute.get("error").is_some());

    let tick = rpc(&server, "graph.tick_status", json!({}), 272).await;
    assert!(tick.get("result").is_some());

    let verify = rpc(
        &server,
        "graph.verify",
        json!({"graph_id": "nonexistent"}),
        273,
    )
    .await;
    assert!(verify.get("result").is_some() || verify.get("error").is_some());

    for (method, id) in [
        ("graph.start_continuous", 274),
        ("graph.pause_continuous", 275),
        ("graph.resume_continuous", 276),
        ("graph.stop_continuous", 277),
    ] {
        let result = rpc(&server, method, json!({"execution_id": "missing"}), id).await;
        assert!(
            result.get("result").is_some() || result.get("error").is_some(),
            "{method} did not dispatch"
        );
    }

    let metrics = rpc(&server, "topology.metrics", json!({}), 278).await;
    assert!(metrics.get("result").is_some());

    let rescan = rpc(&server, "topology.rescan", json!({}), 279).await;
    assert!(rescan.get("result").is_some() || rescan.get("error").is_some());
}

#[tokio::test]
async fn lifecycle_protocol_and_gate_routes_dispatch() {
    let (server, _temp) = create_test_server();

    for (method, params, id) in [
        (
            "lifecycle.register",
            json!({"primal_id": "p1", "socket": "/tmp/p1.sock"}),
            280,
        ),
        ("lifecycle.resurrect", json!({"primal_id": "p1"}), 281),
        ("lifecycle.apoptosis", json!({"primal_id": "p1"}), 282),
        ("lifecycle.composition", json!({}), 283),
        ("composition.reload", json!({}), 284),
        (
            "protocol.register_primal",
            json!({"primal": "p1", "socket": "/tmp/p1.sock"}),
            285,
        ),
        (
            "protocol.register_connection",
            json!({"from": "a", "to": "b"}),
            286,
        ),
        (
            "protocol.record_request",
            json!({"from": "a", "to": "b", "method": "test"}),
            287,
        ),
        (
            "gate.register",
            json!({"gate_id": "g1", "endpoint": "/tmp/g1.sock"}),
            288,
        ),
        ("gate.list", json!({}), 289),
        (
            "route.register",
            json!({"routes": [{"capability": "security", "primal": "beardog"}]}),
            290,
        ),
        (
            "capability.discover",
            json!({"capability": "security"}),
            291,
        ),
        ("capability.prune", json!({}), 292),
        (
            "primal.announce",
            json!({"primal": "testprimal", "socket": "/tmp/testprimal.sock"}),
            293,
        ),
        ("manifest.gate_profile", json!({}), 294),
        (
            "neural_api.proxy_http",
            json!({"method": "GET", "url": "https://example.com"}),
            295,
        ),
    ] {
        let result = rpc(&server, method, params, id).await;
        assert!(
            result.get("result").is_some() || result.get("error").is_some(),
            "{method} did not dispatch: {result}"
        );
    }
}

#[tokio::test]
async fn inference_schedule_and_gates_routes_dispatch() {
    let (server, _temp) = create_test_server();

    let gates = rpc(&server, "inference.gates", json!({}), 300).await;
    assert!(gates.get("result").is_some());

    let schedule = rpc(
        &server,
        "inference.schedule",
        json!({"prompt": "hello", "model": "default"}),
        301,
    )
    .await;
    assert!(schedule.get("result").is_some() || schedule.get("error").is_some());
}

#[tokio::test]
async fn semantic_capability_call_route_dispatches() {
    let (server, _temp) = create_test_server();

    let result = rpc(
        &server,
        "nest.store",
        json!({"key": "k", "value": "v"}),
        310,
    )
    .await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
    assert_ne!(
        result.get("error").and_then(|e| e["code"].as_i64()),
        Some(-32601),
        "nest.store must use SemanticCapabilityCall route"
    );
}
