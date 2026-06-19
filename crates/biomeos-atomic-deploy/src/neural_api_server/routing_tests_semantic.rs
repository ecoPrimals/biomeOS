// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Semantic fallback, capability forwarding, composition, and method registration.

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::common::create_test_server;

#[tokio::test]
async fn test_semantic_fallback_routes_through_capability_call() {
    let (server, _temp) = create_test_server();
    // "provenance.begin" is not in ROUTE_TABLE but has domain.operation format,
    // so it routes through capability.call via the semantic fallback.
    let req = r#"{"jsonrpc":"2.0","method":"provenance.begin","params":{},"id":1}"#;
    let result = server.handle_request_json(req).await;
    // No provider registered in test server → ApplicationError (-32603), not MethodNotFound
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "semantic fallback should route through capability.call, not MethodNotFound"
    );
}

#[tokio::test]
async fn test_semantic_fallback_birdsong_decrypt() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"birdsong.decrypt","params":{"data":"test"},"id":2}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_semantic_fallback_dag_dehydrate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"dag.dehydrate","params":{"session_id":"s1"},"id":3}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_semantic_fallback_content_get_routes_to_nestgate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"content.get","params":{"key":"test"},"id":106}"#;
    let result = server.handle_request_json(req).await;
    // No NestGate registered → ApplicationError, not MethodNotFound.
    // This confirms content.get routes through semantic fallback → capability.call.
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "content.get should route through capability.call, not MethodNotFound"
    );
}

#[tokio::test]
async fn test_semantic_fallback_content_resolve_routes_to_nestgate() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"content.resolve","params":{"path":"/test"},"id":107}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
    assert_ne!(
        result["error"]["code"], -32601,
        "content.resolve should route through capability.call"
    );
}

#[tokio::test]
async fn test_content_routes_to_nestgate_with_registered_provider() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("nestgate.sock");
    let _mock =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({"found": true, "data": "hello"})).await;

    server
        .capability_handler
        .register(&Some(json!({
            "capability": "content",
            "primal": "nestgate",
            "socket": sock.to_str().unwrap(),
            "source": "content-routing-test"
        })))
        .await
        .expect("register content capability");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "content.get",
        "params": { "key": "test-content" },
        "id": 108
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(
        result.get("result").is_some(),
        "content.get should route to registered NestGate: {result}"
    );
    assert_eq!(result["result"]["found"], true);
}
// --- capability.resolve route tests ---

#[tokio::test]
async fn test_handle_request_capability_resolve_route() {
    let (server, _temp) = create_test_server();
    server
        .capability_handler
        .register(&Some(serde_json::json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock",
            "source": "test"
        })))
        .await
        .unwrap();

    let req = r#"{"jsonrpc":"2.0","method":"capability.resolve","params":{"capability":"crypto"},"id":90}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("result").is_some(),
        "resolve should succeed: {result}"
    );
    assert_eq!(result["result"]["resolved"], true);
    assert_eq!(result["result"]["primal"], "beardog");
    assert_eq!(result["id"], 90);
}

#[tokio::test]
async fn test_handle_request_capability_resolve_missing_capability_errors() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.resolve","params":{},"id":91}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

// --- inference.* canonical namespace route tests ---

#[tokio::test]
async fn test_handle_request_inference_register_provider_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.register_provider","params":{"name":"neuralSpring","endpoint":"/tmp/neural.sock"},"id":92}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("result").is_some(),
        "register_provider should succeed: {result}"
    );
    assert_eq!(result["result"]["registered"], true);
    assert_eq!(result["result"]["name"], "neuralSpring");
}

#[tokio::test]
async fn test_handle_request_inference_providers_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.providers","id":93}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["result"]["count"], 0);
}

#[tokio::test]
async fn test_handle_request_inference_complete_no_provider() {
    let (server, _temp) = create_test_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"inference.complete","params":{"prompt":"hello"},"id":94}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("error").is_some(),
        "inference.complete with no provider should error"
    );
}

#[tokio::test]
async fn test_handle_request_inference_embed_no_provider() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.embed","params":{"text":"test"},"id":95}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_inference_models_no_provider() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.models","id":96}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_inference_register_provider_missing_name_errors() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"inference.register_provider","params":{"endpoint":"/tmp/x.sock"},"id":97}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_call_includes_routing_trace_when_enabled() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("routing-trace.sock");
    let _mock = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "hashed": "z" })).await;

    server
        .capability_handler
        .register(&Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": sock.to_str().unwrap(),
            "source": "routing_test",
            "semantic_mappings": { "sha256": "crypto.blake3_hash" }
        })))
        .await
        .expect("register");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": "crypto",
            "operation": "sha256",
            "args": {},
            "_routing_trace": true
        },
        "id": 42
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(
        result.get("_routing_trace").is_some(),
        "expected top-level _routing_trace: {result}"
    );
    assert_eq!(result["result"]["hashed"], "z");
    assert_eq!(result["id"], 42);
    let phases = result["_routing_trace"]["phases"]
        .as_array()
        .expect("phases");
    assert_eq!(phases.len(), 3);
}

// --- composition.status route tests ---

#[tokio::test]
async fn test_handle_request_composition_status_returns_expected_shape() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"composition.status","id":100}"#;
    let result = server.handle_request_json(req).await;
    let inner = &result["result"];
    assert!(inner["active_users"].is_number(), "expected active_users");
    assert!(
        inner["primal_health"].is_array(),
        "expected primal_health array"
    );
    assert!(
        inner["resource_pressure"].is_object(),
        "expected resource_pressure object"
    );
    assert!(
        inner["total_primals"].is_number(),
        "expected total_primals count"
    );
    assert!(
        inner["topology_version"].is_number(),
        "expected topology_version"
    );
}

// --- composition.deploy route tests ---

#[tokio::test]
async fn test_handle_request_composition_deploy_routes_to_graph_execute() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"composition.deploy","params":{"graph_id":"nonexistent"},"id":101}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("error").is_some() || result.get("result").is_some(),
        "composition.deploy should route to graph.execute"
    );
    assert_ne!(
        result.get("error").and_then(|e| e["code"].as_i64()),
        Some(-32601),
        "composition.deploy must not be MethodNotFound"
    );
}

// --- composition.deploy.shadow route tests ---

#[tokio::test]
async fn test_shadow_deploy_nonexistent_graph_returns_error() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "composition.deploy.shadow",
        "params": { "graph_id": "nonexistent" },
        "id": 110
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(
        result.get("error").is_some(),
        "shadow deploy of nonexistent graph should error: {result}"
    );
}

#[tokio::test]
async fn test_shadow_deploy_valid_graph_returns_plan() {
    let (server, temp) = create_test_server();

    let graph_toml = r#"
[graph]
id = "test_shadow"
version = "1.0"

[[graph.nodes]]
id = "security"
capability = "security"
capabilities = ["security"]

[[graph.nodes]]
id = "discovery"
capability = "discovery"
depends_on = ["security"]
capabilities = ["discovery"]
"#;

    std::fs::write(temp.path().join("test_shadow.toml"), graph_toml).expect("write graph");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "composition.deploy.shadow",
        "params": { "graph_id": "test_shadow" },
        "id": 111
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];

    assert!(
        inner["valid"].as_bool().unwrap_or(false),
        "graph should be valid: {result}"
    );
    assert_eq!(inner["graph_id"], "test_shadow");
    assert_eq!(inner["version"], "1.0");
    assert_eq!(inner["node_count"], 2);
    assert!(inner["phases"].is_array(), "should have phases");
    assert!(
        inner["phase_count"].as_u64().unwrap() > 0,
        "should have phases"
    );
    assert!(
        inner["capability_resolution"].is_array(),
        "should have capability_resolution"
    );
    assert!(inner["integrity"].is_object(), "should have integrity");
    assert!(
        inner["validation_errors"].as_array().unwrap().is_empty(),
        "no errors expected"
    );
}

#[tokio::test]
async fn test_shadow_deploy_does_not_register_capabilities() {
    let (server, temp) = create_test_server();

    let graph_toml = r#"
[graph]
id = "shadow_no_register"
version = "1.0"

[[graph.nodes]]
id = "testprimal"
capability = "security"
capabilities = ["security"]
"#;

    std::fs::write(temp.path().join("shadow_no_register.toml"), graph_toml).expect("write");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "composition.deploy.shadow",
        "params": { "graph_id": "shadow_no_register" },
        "id": 112
    })
    .to_string();

    server.handle_request_json(&req).await;

    // Verify no capabilities were registered by checking resolve fails
    let resolve_req = json!({
        "jsonrpc": "2.0",
        "method": "capability.resolve",
        "params": { "capability": "security" },
        "id": 113
    })
    .to_string();

    let resolve_result = server.handle_request_json(&resolve_req).await;
    assert!(
        resolve_result.get("error").is_some() || resolve_result["result"]["resolved"] == false,
        "shadow deploy must not register capabilities: {resolve_result}"
    );
}

#[tokio::test]
async fn test_shadow_deploy_membrane_graph_flags_compute_violations() {
    let (server, temp) = create_test_server();

    let graph_toml = r#"
[graph]
id = "membrane_bad"
version = "1.0"

[graph.metadata]
composition_model = "membrane"

[[graph.nodes]]
id = "beardog"
capabilities = ["security.sign"]

[[graph.nodes]]
id = "toadstool"
capabilities = ["compute.dispatch"]
"#;

    std::fs::write(temp.path().join("membrane_bad.toml"), graph_toml).expect("write");

    let req = json!({
        "jsonrpc": "2.0",
        "method": "composition.deploy.shadow",
        "params": { "graph_id": "membrane_bad" },
        "id": 114
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];

    assert_eq!(
        inner["valid"], false,
        "membrane graph with compute node should be invalid"
    );
    let errors = inner["validation_errors"].as_array().expect("errors array");
    assert!(
        errors.iter().any(|e| {
            let s = e.as_str().unwrap_or("");
            s.contains("Membrane") && s.contains("toadstool")
        }),
        "should flag toadstool compute node: {errors:?}"
    );
}

// --- biomeos.spring_status route tests ---

#[tokio::test]
async fn test_spring_status_returns_expected_shape() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 120
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];

    assert!(
        inner["primals"].is_array(),
        "expected primals array: {result}"
    );
    let primals = inner["primals"].as_array().unwrap();
    assert!(
        !primals.is_empty(),
        "primals array should list known primals"
    );

    // Every entry should have required fields
    for p in primals {
        assert!(p["name"].is_string(), "missing name: {p}");
        assert!(p["display_name"].is_string(), "missing display_name: {p}");
        assert!(
            p["binary_available"].is_boolean(),
            "missing binary_available: {p}"
        );
        assert!(p["capabilities"].is_array(), "missing capabilities: {p}");
    }

    assert!(
        inner["workload_count"].is_number(),
        "expected workload_count"
    );
    assert!(
        inner["workloads_running"].is_number(),
        "expected workloads_running"
    );
    assert!(
        inner["topology_version"].is_number(),
        "expected topology_version"
    );
}

#[tokio::test]
async fn test_spring_status_includes_core_primals() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 121
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let primals = result["result"]["primals"].as_array().unwrap();
    let names: Vec<&str> = primals.iter().filter_map(|p| p["name"].as_str()).collect();

    // Core primals must appear
    assert!(names.contains(&"beardog"), "missing beardog");
    assert!(names.contains(&"songbird"), "missing songbird");
    assert!(names.contains(&"nestgate"), "missing nestgate");
    assert!(names.contains(&"toadstool"), "missing toadstool");
    // Provenance trio
    assert!(names.contains(&"rhizocrypt"), "missing rhizocrypt");
    assert!(names.contains(&"loamspine"), "missing loamspine");
    assert!(names.contains(&"sweetgrass"), "missing sweetgrass");
}

#[tokio::test]
async fn test_spring_status_has_display_names() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "biomeos.spring_status",
        "params": {},
        "id": 122
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let primals = result["result"]["primals"].as_array().unwrap();

    let beardog = primals.iter().find(|p| p["name"] == "beardog").unwrap();
    assert_eq!(beardog["display_name"], "BearDog");

    let nestgate = primals.iter().find(|p| p["name"] == "nestgate").unwrap();
    assert_eq!(nestgate["display_name"], "NestGate");
}

// --- method.register route tests (GAP-09) ---

#[tokio::test]
async fn test_handle_request_method_register_registers_domains() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "ludoSpring",
            "transport": "/tmp/ludo.sock",
            "methods": ["game.start", "game.join", "game.end", "score.get", "score.leaderboard"]
        },
        "id": 102
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    let inner = &result["result"];
    assert_eq!(inner["registered"], 5, "should register 5 methods");
    assert_eq!(inner["primal"], "ludoSpring");
    let domains = inner["domains"].as_array().expect("domains array");
    assert!(domains.len() == 2, "should have 2 domains: game + score");
}

#[tokio::test]
async fn test_handle_request_method_register_empty_methods_errors() {
    let (server, _temp) = create_test_server();
    let req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "test",
            "transport": "/tmp/test.sock",
            "methods": []
        },
        "id": 103
    })
    .to_string();

    let result = server.handle_request_json(&req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_method_register_makes_methods_semantically_routable() {
    let (server, _temp) = create_test_server();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ludo.sock");
    let _mock = MockJsonRpcServer::spawn_echo_success(&sock, json!({"started": true})).await;

    let reg_req = json!({
        "jsonrpc": "2.0",
        "method": "method.register",
        "params": {
            "primal": "ludoSpring",
            "transport": sock.to_str().unwrap(),
            "methods": ["game.start"]
        },
        "id": 104
    })
    .to_string();

    let reg_result = server.handle_request_json(&reg_req).await;
    assert!(
        reg_result.get("result").is_some(),
        "registration should succeed"
    );

    let call_req = json!({
        "jsonrpc": "2.0",
        "method": "game.start",
        "params": {},
        "id": 105
    })
    .to_string();

    let call_result = server.handle_request_json(&call_req).await;
    assert!(
        call_result.get("result").is_some(),
        "game.start should route via semantic fallback after method.register: {call_result}"
    );
}
