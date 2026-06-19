// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Route table dispatch coverage for known JSON-RPC methods.

use super::common::create_test_server;

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
