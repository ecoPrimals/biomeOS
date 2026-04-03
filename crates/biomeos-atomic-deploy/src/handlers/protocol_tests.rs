// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Additional protocol handler coverage (sibling to `protocol.rs`).

#![expect(clippy::unwrap_used, reason = "test")]

use super::protocol::ProtocolHandler;
use crate::living_graph::LivingGraph;
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
