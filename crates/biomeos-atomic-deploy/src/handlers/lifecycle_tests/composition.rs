// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::make_graph_node;
use crate::handlers::lifecycle::LifecycleHandler;
use serde_json::json;

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
