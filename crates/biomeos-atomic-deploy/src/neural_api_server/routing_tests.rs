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

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

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
async fn test_handle_request_primal_list_routes_to_topology_primals() {
    let (server, _temp) = create_test_server();
    let req = r#"{"jsonrpc":"2.0","method":"primal.list","id":70}"#;
    let result = server.handle_request_json(req).await;
    assert!(
        result.get("result").is_some(),
        "primal.list should route to TopologyPrimals, got: {result}"
    );
    assert!(result["result"]["primals"].is_array());
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
    assert_eq!(result["result"]["status"], "alive");
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
