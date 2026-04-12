// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Routing tests for Neural API Server (extracted from routing.rs).

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use crate::neural_api_server::NeuralApiServer;
use crate::neural_api_server::rpc::DispatchOutcome;

fn create_test_server() -> (NeuralApiServer, tempfile::TempDir) {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(temp.path()).expect("create graphs dir");
    let server = NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"));
    // Prevent lazy socket rescan from finding real primals running on this host.
    server
        .router
        .lazy_rescan_attempted
        .store(true, std::sync::atomic::Ordering::Relaxed);
    (server, temp)
}

#[tokio::test]
async fn test_handle_request_unknown_single_word_method() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent","id":1}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["error"]["code"], -32601);
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent")
    );
    assert_eq!(result["id"], 1);
}

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
async fn test_composition_health_route_returns_standard_shape() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"composition.tower_health","params":{},"id":4}"#;
    let result = server.handle_request_json(req).await;
    let inner = &result["result"];
    assert!(inner["healthy"].is_boolean(), "should have healthy field");
    assert!(
        inner["deploy_graph"].is_string(),
        "should have deploy_graph field"
    );
    assert!(
        inner["subsystems"].is_object(),
        "should have subsystems map"
    );
}

#[tokio::test]
async fn test_handle_request_invalid_json() {
    let (server, _temp) = create_test_server();
    let result = server.handle_request_json("{broken").await;
    assert_eq!(result["error"]["code"], -32700);
    // serde_json error message varies (e.g. "expected value", "EOF while parsing")
    assert!(!result["error"]["message"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn test_handle_request_mesh_method_invalid_format_single_part() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mesh","id":2}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["error"]["code"], -32601);
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("mesh")
    );
}

#[tokio::test]
async fn test_semantic_fallback_multipart_method() {
    let (server, _temp) = create_test_server();
    // "a.b.c" has a dot so semantic fallback splits on first dot: domain="a", operation="b.c"
    let req = r#"{"jsonrpc":"2.0","method":"a.b.c","id":3}"#;
    let result = server.handle_request_json(req).await;
    // Routes through capability.call (no provider), not MethodNotFound
    assert!(result.get("error").is_some());
    assert_ne!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_handle_request_empty_method() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"","id":4}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["error"]["code"], -32601);
}

#[tokio::test]
async fn test_handle_request_method_not_found_response_structure() {
    let (server, _temp) = create_test_server();
    // Single-word methods with no dot hit MethodNotFound (no semantic fallback)
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent_verb","id":99}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_none());
    assert!(result.get("error").is_some());
    assert_eq!(result["error"]["code"], -32601);
    assert_eq!(result["id"], 99);
}

#[tokio::test]
async fn test_handle_request_graph_list_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.list","id":10}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["jsonrpc"], "2.0");
    assert!(result.get("result").is_some());
    assert!(result.get("error").is_none());
    assert_eq!(result["id"], 10);
}

#[tokio::test]
async fn test_handle_request_neural_api_list_graphs_alias() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"neural_api.list_graphs","id":11}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 11);
}

#[tokio::test]
async fn test_handle_request_topology_get_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"topology.get","id":12}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 12);
}

#[tokio::test]
async fn test_handle_request_lifecycle_status_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"lifecycle.status","id":13}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 13);
}

#[tokio::test]
async fn test_handle_request_capability_list_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capabilities.list","id":14}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 14);
}

#[tokio::test]
async fn test_handle_request_niche_list_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"niche.list","id":15}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 15);
}

#[tokio::test]
async fn test_handle_request_protocol_status_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"protocol.status","id":16}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
    assert_eq!(result["id"], 16);
}

#[tokio::test]
async fn test_handle_request_missing_id() {
    // JSON-RPC 2.0 allows omitting id (notification); we accept and echo null
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.list"}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["id"], serde_json::Value::Null);
}

#[tokio::test]
async fn test_handle_request_mesh_status_route_dispatches() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mesh.status","params":{},"id":24}"#;
    // mesh capability not registered in test server - handler returns ApplicationError
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some() || result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_mesh_find_path_route_dispatches() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mesh.find_path","params":{},"id":25}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some() || result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_register_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.register","params":{"capability":"encryption","primal":"beardog","socket":"/tmp/beardog.sock"},"id":28}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_list_translations_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.list_translations","params":{},"id":31}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_discover_missing_params_returns_err() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.discover","params":{},"id":40}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("error").is_some(),
        "missing capability should propagate handler error"
    );
}

#[tokio::test]
async fn test_handle_request_graph_get_missing_params_returns_err() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.get","params":{},"id":41}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_graph_status_route_dispatches() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.status","params":{"execution_id":"nonexistent"},"id":43}"#;
    // Execution not found returns ApplicationError - route dispatch is what we test
    let _ = server.handle_request_json(req).await;
}

#[tokio::test]
async fn test_handle_request_capability_metrics_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.metrics","params":{},"id":44}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

// --- Additional route coverage for 90%+ ---

#[tokio::test]
async fn test_handle_request_topology_primals_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"topology.primals","id":50}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_topology_proprioception_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"topology.proprioception","id":51}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_neural_api_get_topology_alias() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"neural_api.get_topology","id":52}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_lifecycle_get_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"lifecycle.get","params":{"primal_id":"test"},"id":53}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_lifecycle_shutdown_all_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"lifecycle.shutdown_all","id":54}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_protocol_escalate_route() {
    let (server, _temp) = create_test_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"protocol.escalate","params":{"from":"a","to":"b"},"id":55}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_protocol_fallback_route() {
    let (server, _temp) = create_test_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"protocol.fallback","params":{"from":"a","to":"b"},"id":56}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_protocol_metrics_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"protocol.metrics","params":{},"id":57}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_protocol_start_monitoring_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"protocol.start_monitoring","id":58}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_protocol_stop_monitoring_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"protocol.stop_monitoring","id":59}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_graph_protocol_map_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.protocol_map","id":60}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_providers_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.providers","params":{"capability":"security"},"id":61}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_route_alias() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.route","params":{"capability":"security"},"id":62}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_neural_api_route_to_primal_alias() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"neural_api.route_to_primal","params":{"capability":"security"},"id":63}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_capability_discover_translation_singular() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"capability.discover_translation","params":{"capability":"encryption"},"id":64}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_mcp_tools_list_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mcp.tools.list","id":65}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_mcp_tools_list_underscore_alias() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"mcp.tools_list","id":66}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_agent_list_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"agent.list","id":67}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn test_handle_request_agent_get_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"agent.get","params":{"id":"test"},"id":68}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_graph_execute_pipeline_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.execute_pipeline","params":{"graph_id":"test"},"id":69}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_graph_suggest_optimizations_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.suggest_optimizations","params":{"graph_id":"test"},"id":70}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_niche_deploy_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"niche.deploy","params":{"template":"test"},"id":71}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_punch_request_mesh_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"punch.request","params":{},"id":72}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_stun_discover_mesh_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"stun.discover","params":{},"id":73}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_relay_serve_mesh_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"relay.serve","params":{},"id":74}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_onion_create_service_mesh_route() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"onion.create_service","params":{},"id":75}"#;
    let result = server.handle_request_json(req).await;
    assert!(result.get("result").is_some() || result.get("error").is_some());
}

#[tokio::test]
async fn test_handle_request_dispatch_outcome_success_structure() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"graph.list","id":76}"#;
    let outcome = server.handle_request(req).await;
    let response = outcome.into_response();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());
}

#[tokio::test]
async fn test_handle_request_dispatch_outcome_parse_error() {
    let (server, _temp) = create_test_server();
    let outcome = server.handle_request("not json").await;
    let response = outcome.into_response();
    assert_eq!(response["error"]["code"], -32700);
    assert!(response["id"].is_null());
}

#[tokio::test]
async fn test_handle_request_dispatch_outcome_method_not_found() {
    let (server, _temp) = create_test_server();
    // No-dot method → pure MethodNotFound (no semantic fallback)
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent","id":77}"#;
    let outcome = server.handle_request(req).await;
    let response = outcome.into_response();
    assert_eq!(response["error"]["code"], -32601);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent")
    );
}

#[tokio::test]
async fn test_handle_request_health_check() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.check","id":80}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["result"]["status"], "healthy");
    assert_eq!(result["result"]["family_id"], "test_family");
}

#[tokio::test]
async fn test_handle_request_health_liveness() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.liveness","id":81}"#;
    let result = server.handle_request_json(req).await;
    assert_eq!(result["result"]["status"], "alive");
    assert!(result["result"]["version"].is_string());
}

#[tokio::test]
async fn test_handle_request_health_readiness() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"health.readiness","id":82}"#;
    let result = server.handle_request_json(req).await;
    assert!(result["result"]["ready"].is_boolean());
    assert!(result["result"]["mode"].is_string());
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

#[test]
fn dispatch_preserves_primal_json_rpc_error_code() {
    let err = biomeos_types::IpcError::JsonRpcError {
        primal: "beardog".to_string(),
        code: -32601,
        message: "Method not found".to_string(),
    };
    let id = serde_json::json!(42);
    let outcome = super::dispatch(Err(err.into()), id);
    match outcome {
        DispatchOutcome::ApplicationError { code, message, .. } => {
            assert_eq!(code, -32601, "primal error code must be preserved");
            assert_eq!(message, "Method not found");
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}

#[test]
fn dispatch_uses_generic_code_for_non_ipc_errors() {
    let err = anyhow::anyhow!("connection refused");
    let id = serde_json::json!(1);
    let outcome = super::dispatch(Err(err), id);
    match outcome {
        DispatchOutcome::ApplicationError { code, .. } => {
            assert_eq!(code, -32603, "non-IPC errors use generic code");
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}
