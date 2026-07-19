// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::common::create_test_server;
use super::rpc;
use serde_json::json;

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
        ("cleanup.sockets", json!({}), 293),
        (
            "primal.announce",
            json!({"primal": "testprimal", "socket": "/tmp/testprimal.sock"}),
            294,
        ),
        ("manifest.gate_profile", json!({}), 295),
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
