// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional protocol handler coverage (sibling to `protocol.rs`).

#![expect(clippy::unwrap_used, reason = "test")]

use super::protocol::ProtocolHandler;
use crate::living_graph::{LivingGraph, PrimalProtocolState};
use crate::protocol_escalation::{EscalationConfig, ProtocolEscalationManager};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

fn make_handler() -> ProtocolHandler {
    let graph = Arc::new(LivingGraph::new("protocol-cov-family"));
    let manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
        graph.clone(),
        EscalationConfig::default(),
    )));
    ProtocolHandler::new(graph, manager)
}

fn create_test_handler() -> ProtocolHandler {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
        graph.clone(),
        EscalationConfig::default(),
    )));
    ProtocolHandler::new(graph, manager)
}

#[tokio::test]
async fn register_primal_missing_primal_id_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "json_rpc_socket": "/tmp/a.sock"
    }));
    let err = h.register_primal(&params).await.unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("primal_id"),
        "{err}"
    );
}

#[tokio::test]
async fn register_primal_missing_json_rpc_socket_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "primal_id": "only-id"
    }));
    let err = h.register_primal(&params).await.unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("json_rpc_socket"),
        "{err}"
    );
}

#[tokio::test]
async fn record_request_missing_latency_us_is_error() {
    let h = make_handler();
    h.register_connection(&Some(json!({ "from": "a", "to": "b" })))
        .await
        .unwrap();
    let params = Some(json!({
        "from": "a",
        "to": "b"
    }));
    let err = h.record_request(&params).await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("latency"), "{err}");
}

#[tokio::test]
async fn record_request_missing_from_is_error() {
    let h = make_handler();
    let params = Some(json!({
        "to": "b",
        "latency_us": 1
    }));
    assert!(h.record_request(&params).await.is_err());
}

#[tokio::test]
async fn register_connection_preserves_ids_in_json() {
    let h = make_handler();
    let params = Some(json!({
        "from": "left",
        "to": "right"
    }));
    let v = h.register_connection(&params).await.unwrap();
    assert_eq!(v["from"], "left");
    assert_eq!(v["to"], "right");
}

#[tokio::test]
async fn protocol_map_includes_edge_latency_and_counts() {
    let h = make_handler();
    h.register_primal(&Some(json!({
        "primal_id": "p1",
        "json_rpc_socket": "/tmp/p1.sock"
    })))
    .await
    .unwrap();
    h.register_connection(&Some(json!({ "from": "p1", "to": "p2" })))
        .await
        .unwrap();
    h.record_request(&Some(json!({
        "from": "p1",
        "to": "p2",
        "latency_us": 42,
        "success": true
    })))
    .await
    .unwrap();
    let v = h.protocol_map().await.unwrap();
    let edges = v["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["latency_us"].as_f64(), Some(42.0));
    assert_eq!(v["summary"]["connection_count"], 1);
}

#[tokio::test]
async fn escalate_and_fallback_return_mode_strings() {
    let h = make_handler();
    h.register_connection(&Some(json!({ "from": "u", "to": "v" })))
        .await
        .unwrap();
    let esc = h
        .escalate(&Some(json!({ "from": "u", "to": "v" })))
        .await
        .unwrap();
    assert!(esc.get("previous_mode").is_some());
    assert!(esc.get("current_mode").is_some());
    let fb = h
        .fallback(&Some(json!({ "from": "u", "to": "v", "reason": "t" })))
        .await
        .unwrap();
    assert!(fb.get("message").is_some());
}

#[tokio::test]
async fn register_primal_empty_capabilities_ok() {
    let h = make_handler();
    let params = Some(json!({
        "primal_id": "bare",
        "json_rpc_socket": "/tmp/bare.sock"
    }));
    h.register_primal(&params).await.unwrap();
    assert!(h.living_graph().has_primal("bare").await);
}

#[tokio::test]
async fn status_empty_graph_has_zero_summary_total() {
    let h = make_handler();
    let v = h.status().await.unwrap();
    let summary = v["summary"].as_object().unwrap();
    assert_eq!(summary.get("total").and_then(|x| x.as_u64()), Some(0));
}

#[tokio::test]
async fn escalate_missing_to_parameter() {
    let h = make_handler();
    let err = h
        .escalate(&Some(json!({ "from": "only-from" })))
        .await
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("to"));
}

#[tokio::test]
async fn record_request_missing_to_is_error() {
    let h = make_handler();
    let err = h
        .record_request(&Some(json!({
            "from": "a",
            "latency_us": 1
        })))
        .await
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("to"));
}

#[tokio::test]
async fn protocol_map_two_nodes_and_edge() {
    let h = make_handler();
    h.register_primal(&Some(json!({
        "primal_id": "n1",
        "json_rpc_socket": "/tmp/n1.sock"
    })))
    .await
    .unwrap();
    h.register_primal(&Some(json!({
        "primal_id": "n2",
        "json_rpc_socket": "/tmp/n2.sock"
    })))
    .await
    .unwrap();
    h.register_connection(&Some(json!({ "from": "n1", "to": "n2" })))
        .await
        .unwrap();
    let v = h.protocol_map().await.unwrap();
    assert_eq!(v["summary"]["primal_count"], 2);
    assert_eq!(v["summary"]["connection_count"], 1);
}

#[tokio::test]
async fn register_primal_capabilities_non_array_treated_as_empty() {
    let h = make_handler();
    let params = Some(json!({
        "primal_id": "raw-cap",
        "json_rpc_socket": "/tmp/raw.sock",
        "capabilities": "not-an-array"
    }));
    h.register_primal(&params).await.unwrap();
    let st = h
        .living_graph()
        .get_primal_state("raw-cap")
        .await
        .expect("state");
    assert!(st.capabilities.is_empty());
}

#[tokio::test]
async fn stop_monitoring_idempotent() {
    let h = make_handler();
    let a = h.stop_monitoring().await.unwrap();
    let b = h.stop_monitoring().await.unwrap();
    assert_eq!(a["status"], "stopped");
    assert_eq!(b["status"], "stopped");
}

#[tokio::test]
async fn test_status() {
    let handler = create_test_handler();

    handler.living_graph().register_connection("a", "b").await;

    let result = handler.status().await.unwrap();

    assert!(result.get("connections").is_some());
    assert!(result.get("summary").is_some());
    assert_eq!(result["summary"]["total"], 1);
}

#[tokio::test]
async fn test_protocol_map() {
    let handler = create_test_handler();

    handler
        .living_graph()
        .register_primal(PrimalProtocolState::new(
            "beardog",
            std::path::PathBuf::from("/tmp/beardog.sock"),
        ))
        .await;

    handler
        .living_graph()
        .register_connection("songbird", "beardog")
        .await;

    let result = handler.protocol_map().await.unwrap();

    assert_eq!(result["family_id"], "test-family");
    assert_eq!(result["summary"]["primal_count"], 1);
    assert_eq!(result["summary"]["connection_count"], 1);
}

#[tokio::test]
async fn test_register_primal() {
    let handler = create_test_handler();

    let params = Some(json!({
        "primal_id": "test-primal",
        "json_rpc_socket": "/tmp/test.sock",
        "capabilities": ["capability1", "capability2"]
    }));

    let result = handler.register_primal(&params).await.unwrap();

    assert_eq!(result["status"], "registered");
    assert_eq!(result["primal_id"], "test-primal");

    assert!(handler.living_graph().has_primal("test-primal").await);
}

#[tokio::test]
async fn test_register_connection() {
    let handler = create_test_handler();

    let params = Some(json!({
        "from": "primal-a",
        "to": "primal-b"
    }));

    let result = handler.register_connection(&params).await.unwrap();

    assert_eq!(result["status"], "registered");
    assert_eq!(result["from"], "primal-a");
    assert_eq!(result["to"], "primal-b");

    assert!(
        handler
            .living_graph()
            .get_connection("primal-a", "primal-b")
            .await
            .is_some()
    );
}

#[tokio::test]
async fn test_record_request() {
    let handler = create_test_handler();

    handler.living_graph().register_connection("a", "b").await;

    let params = Some(json!({
        "from": "a",
        "to": "b",
        "latency_us": 150,
        "success": true
    }));

    let result = handler.record_request(&params).await.unwrap();

    assert_eq!(result["status"], "recorded");
    assert_eq!(result["latency_us"], 150);

    let conn = handler
        .living_graph()
        .get_connection("a", "b")
        .await
        .unwrap();
    assert_eq!(conn.metrics.request_count, 1);
}

#[tokio::test]
async fn test_metrics() {
    let handler = create_test_handler();

    handler.living_graph().register_connection("x", "y").await;

    for i in 0..10 {
        handler
            .living_graph()
            .record_request("x", "y", 100 + i * 10, true)
            .await;
    }

    let params = Some(json!({
        "from": "x",
        "to": "y"
    }));

    let result = handler.metrics(&params).await.unwrap();

    assert!(result.get("connection").is_some());
    assert!(result.get("metrics").is_some());
    assert_eq!(result["metrics"]["request_count"], 10);
}

#[tokio::test]
async fn test_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.escalate(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.escalate(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_escalate_none_params() {
    let handler = create_test_handler();
    let result = handler.escalate(&None).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("missing"),
        "Error should mention missing params"
    );
}

#[tokio::test]
async fn test_fallback_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.fallback(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.fallback(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fallback_none_params() {
    let handler = create_test_handler();
    let result = handler.fallback(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fallback_with_reason() {
    let handler = create_test_handler();
    handler
        .living_graph()
        .register_connection("src", "dst")
        .await;

    let params = Some(json!({
        "from": "src",
        "to": "dst",
        "reason": "manual_test"
    }));

    let result = handler.fallback(&params).await.unwrap();
    assert!(result.get("status").is_some());
    assert_eq!(result["from"], "src");
    assert_eq!(result["to"], "dst");
}

#[tokio::test]
async fn test_fallback_default_reason() {
    let handler = create_test_handler();
    handler.living_graph().register_connection("a", "b").await;

    let params = Some(json!({ "from": "a", "to": "b" }));
    let result = handler.fallback(&params).await.unwrap();
    assert_eq!(result["status"], "degraded");
}

#[tokio::test]
async fn test_metrics_missing_params() {
    let handler = create_test_handler();

    let params = Some(json!({ "to": "b" }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_none_params() {
    let handler = create_test_handler();
    let result = handler.metrics(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_connection_not_found() {
    let handler = create_test_handler();

    let params = Some(json!({
        "from": "nonexistent",
        "to": "also-nonexistent"
    }));
    let result = handler.metrics(&params).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("not found"),
        "Error should mention connection not found"
    );
}

#[tokio::test]
async fn test_register_primal_with_tarpc_socket() {
    let handler = create_test_handler();

    let params = Some(json!({
        "primal_id": "tarpc-primal",
        "json_rpc_socket": "/tmp/json.sock",
        "tarpc_socket": "/tmp/tarpc.sock",
        "capabilities": ["rpc"]
    }));

    let result = handler.register_primal(&params).await.unwrap();
    assert_eq!(result["status"], "registered");
    assert_eq!(result["primal_id"], "tarpc-primal");

    assert!(handler.living_graph().has_primal("tarpc-primal").await);
    let state = handler
        .living_graph()
        .get_primal_state("tarpc-primal")
        .await
        .expect("primal state");
    assert!(state.tarpc_socket.is_some());
    assert_eq!(
        state.tarpc_socket.as_ref().unwrap().to_string_lossy(),
        "/tmp/tarpc.sock"
    );
}

#[tokio::test]
async fn test_register_primal_missing_params() {
    let handler = create_test_handler();

    let result = handler.register_primal(&None).await;
    assert!(result.is_err());

    let params = Some(json!({}));
    let result = handler.register_primal(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_connection_missing_params() {
    let handler = create_test_handler();

    let result = handler.register_connection(&None).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a" }));
    let result = handler.register_connection(&params).await;
    assert!(result.is_err());

    let params = Some(json!({ "to": "b" }));
    let result = handler.register_connection(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_record_request_success_false() {
    let handler = create_test_handler();
    handler.living_graph().register_connection("a", "b").await;

    let params = Some(json!({
        "from": "a",
        "to": "b",
        "latency_us": 200,
        "success": false
    }));

    let result = handler.record_request(&params).await.unwrap();
    assert_eq!(result["success"], false);

    let conn = handler
        .living_graph()
        .get_connection("a", "b")
        .await
        .unwrap();
    assert_eq!(conn.metrics.error_count, 1);
}

#[tokio::test]
async fn test_record_request_default_success() {
    let handler = create_test_handler();
    handler.living_graph().register_connection("x", "y").await;

    let params = Some(json!({
        "from": "x",
        "to": "y",
        "latency_us": 100
    }));

    let result = handler.record_request(&params).await.unwrap();
    assert_eq!(result["success"], true);
}

#[tokio::test]
async fn test_record_request_missing_params() {
    let handler = create_test_handler();

    let result = handler.record_request(&None).await;
    assert!(result.is_err());

    let params = Some(json!({ "from": "a", "to": "b" }));
    let result = handler.record_request(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_start_monitoring() {
    let handler = create_test_handler();
    let result = handler.start_monitoring().await.unwrap();
    assert_eq!(result["status"], "started");
    assert!(result["message"].as_str().unwrap().contains("started"));
}

#[tokio::test]
async fn test_stop_monitoring() {
    let handler = create_test_handler();
    let result = handler.stop_monitoring().await.unwrap();
    assert_eq!(result["status"], "stopped");
    assert!(result["message"].as_str().unwrap().contains("stopped"));
}

#[tokio::test]
async fn test_protocol_map_empty() {
    let handler = create_test_handler();
    let result = handler.protocol_map().await.unwrap();

    assert_eq!(result["family_id"], "test-family");
    assert_eq!(result["summary"]["primal_count"], 0);
    assert_eq!(result["summary"]["connection_count"], 0);
    assert!(result["nodes"].as_array().unwrap().is_empty());
    assert!(result["edges"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_protocol_map_node_structure() {
    let handler = create_test_handler();
    handler
        .living_graph()
        .register_primal(PrimalProtocolState::new(
            "test-node",
            std::path::PathBuf::from("/tmp/test.sock"),
        ))
        .await;

    let result = handler.protocol_map().await.unwrap();
    let nodes = result["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["id"], "test-node");
    assert!(nodes[0].get("tarpc_available").is_some());
    assert!(nodes[0].get("current_mode").is_some());
}

#[tokio::test]
async fn test_escalate_registered_connection() {
    let handler = create_test_handler();
    handler
        .living_graph()
        .register_connection("client", "server")
        .await;

    let params = Some(json!({
        "from": "client",
        "to": "server"
    }));

    let result = handler.escalate(&params).await.unwrap();
    assert!(result.get("status").is_some());
    assert_eq!(result["from"], "client");
    assert_eq!(result["to"], "server");
}
