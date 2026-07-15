// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde_json::json;

use super::super::common::create_test_server;

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
