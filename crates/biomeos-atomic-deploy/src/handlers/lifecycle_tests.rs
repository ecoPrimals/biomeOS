// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Sibling tests for `lifecycle.rs` (handler API surface).

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

use serde_json::json;

use super::lifecycle::LifecycleHandler;
use crate::neural_graph::GraphNode;

#[tokio::test]
async fn status_empty_reports_zero_healthy() {
    let handler = LifecycleHandler::new("sibling-lc-fam");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
    assert_eq!(status["healthy"], 0);
    assert!(status["primals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn shutdown_all_on_empty_manager_completes() {
    let handler = LifecycleHandler::new("sibling-lc-shut");
    let out = handler.shutdown_all().await.expect("shutdown_all");
    assert_eq!(out["shutdown"], "complete");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
}

#[tokio::test]
async fn resurrect_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-res");
    let err = handler
        .resurrect(&Some(json!({})))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn apoptosis_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-apo");
    let err = handler
        .apoptosis(&Some(json!({ "reason": "user_request" })))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn get_success_includes_family_and_metrics_block() {
    let handler = LifecycleHandler::new("sibling-lc-get");
    handler
        .register(&Some(json!({
            "name": "p-sibling",
            "socket_path": "/tmp/p-sibling.sock",
            "pid": 7
        })))
        .await
        .expect("register");
    let g = handler
        .get(&Some(json!({ "name": "p-sibling" })))
        .await
        .expect("get");
    assert_eq!(g["family_id"], "sibling-lc-get");
    assert!(g.get("metrics").is_some());
    assert_eq!(g["state"], "incubating");
}

#[tokio::test]
async fn status_increments_count_after_register() {
    let handler = LifecycleHandler::new("sibling-lc-st");
    handler
        .register(&Some(json!({
            "name": "only-one",
            "socket_path": "/tmp/only-one.sock"
        })))
        .await
        .expect("register");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    let primals = status["primals"].as_array().unwrap();
    assert_eq!(primals[0]["name"], "only-one");
}

#[tokio::test]
async fn apoptosis_default_reason_string_when_omitted() {
    let handler = LifecycleHandler::new("sibling-lc-reason");
    handler
        .register(&Some(json!({
            "name": "r1",
            "socket_path": "/tmp/r1.sock",
            "pid": 1
        })))
        .await
        .expect("register");
    let out = handler
        .apoptosis(&Some(json!({ "name": "r1" })))
        .await
        .expect("apoptosis");
    assert_eq!(out["reason"], "user_request");
}

// =========================================================================
// Enriched composition dashboard (lifecycle.composition)
// =========================================================================

fn make_graph_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}

#[tokio::test]
async fn composition_enriched_includes_capabilities_and_edges() {
    let handler = LifecycleHandler::new("test-family");

    let mut node_with_caps = make_graph_node("beardog", vec![]);
    node_with_caps.capabilities = vec!["crypto".to_string(), "security".to_string()];

    handler
        .register(&Some(json!({
            "name": "beardog",
            "socket_path": "/tmp/beardog.sock",
            "pid": 100,
            "deployment_node": serde_json::to_value(&node_with_caps).unwrap()
        })))
        .await
        .expect("register beardog");

    let songbird_node = make_graph_node("songbird", vec!["beardog".to_string()]);
    handler
        .register(&Some(json!({
            "name": "songbird",
            "socket_path": "/tmp/songbird.sock",
            "pid": 200,
            "deployment_node": serde_json::to_value(&songbird_node).unwrap()
        })))
        .await
        .expect("register songbird");

    let comp = handler.composition().await.expect("composition");

    assert_eq!(comp["total"], 2);
    let caps = comp["capabilities_live"]
        .as_array()
        .expect("capabilities array");
    assert!(caps.iter().any(|c| c == "crypto"));
    assert!(caps.iter().any(|c| c == "security"));

    let edges = comp["dependency_graph"].as_array().expect("edges array");
    assert!(
        edges
            .iter()
            .any(|e| e["from"] == "beardog" && e["to"] == "songbird"),
        "dependency edge beardog→songbird should exist"
    );

    let all_primals: Vec<&serde_json::Value> = comp["degraded"]
        .as_array()
        .unwrap()
        .iter()
        .chain(comp["active"].as_array().unwrap().iter())
        .chain(comp["dead"].as_array().unwrap().iter())
        .collect();
    for p in &all_primals {
        assert!(p.get("capabilities").is_some());
        assert!(p.get("health").is_some());
        assert!(p.get("state_details").is_some());
        assert!(p.get("depends_on").is_some());
    }
}

#[tokio::test]
async fn composition_empty_returns_healthy_defaults() {
    let handler = LifecycleHandler::new("test-family");
    let comp = handler.composition().await.expect("composition");
    assert_eq!(comp["total"], 0);
    assert_eq!(comp["health_ratio"], 1.0);
    assert!(comp["composition_healthy"].as_bool().unwrap());
    assert!(comp["capabilities_live"].as_array().unwrap().is_empty());
    assert!(comp["dependency_graph"].as_array().unwrap().is_empty());
}

// =========================================================================
// Composition health (COMPOSITION_HEALTH_STANDARD)
// =========================================================================

#[tokio::test]
async fn composition_health_empty_returns_unavailable_subsystems() {
    let handler = LifecycleHandler::new("test-family");
    let health = handler.composition_health(&None).await.expect("health");
    assert!(health["healthy"].as_bool().is_some());
    assert!(health["deploy_graph"].as_str().is_some());
    assert!(health["subsystems"].is_object());
    let subs = health["subsystems"].as_object().unwrap();
    assert_eq!(subs["tower"], "unavailable");
    assert_eq!(subs["mesh"], "unavailable");
}

#[tokio::test]
async fn composition_health_with_incubating_tower_shows_degraded() {
    let handler = LifecycleHandler::new("test-family");

    handler
        .register(&Some(json!({
            "name": "beardog-server",
            "socket_path": "/tmp/beardog.sock",
            "pid": 1
        })))
        .await
        .expect("register beardog");
    handler
        .register(&Some(json!({
            "name": "songbird-orch",
            "socket_path": "/tmp/songbird.sock",
            "pid": 2
        })))
        .await
        .expect("register songbird");

    let health = handler.composition_health(&None).await.expect("health");
    let subs = health["subsystems"].as_object().unwrap();

    assert_eq!(subs["tower"], "degraded");
    assert_eq!(subs["mesh"], "degraded");
    assert_eq!(subs["node"], "unavailable");
    assert_eq!(subs["nest"], "unavailable");
    assert!(!health["healthy"].as_bool().unwrap());
}
