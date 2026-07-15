// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::helpers::make_handler;
use super::super::super::*;
use crate::neural_router::NeuralRouter;
use std::sync::Arc;

#[tokio::test]
async fn test_topology_handler_creation() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let handler = make_handler("test-family", router, "/tmp");

    let result = handler
        .get_proprioception()
        .await
        .expect("get_proprioception");
    assert_eq!(result["family_id"], "test-family");
}

#[tokio::test]
async fn test_topology_get_response_structure() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get().await.expect("topology.get");

    assert!(
        result.get("primals").is_some(),
        "Response must have primals"
    );
    assert!(
        result.get("connections").is_some(),
        "Response must have connections"
    );
    assert!(
        result.get("timestamp").is_some(),
        "Response must have timestamp"
    );

    let primals = result["primals"].as_array().expect("primals is array");
    let connections = result["connections"]
        .as_array()
        .expect("connections is array");

    for p in primals {
        // Wave 20 canonical fields (primalSpring s_schema_standard)
        assert!(p.get("name").is_some(), "Primal must have name (Wave 20)");
        assert!(
            p.get("socket").is_some(),
            "Primal must have socket (Wave 20)"
        );
        assert!(
            p.get("status").is_some(),
            "Primal must have status (Wave 20)"
        );
        assert!(
            p.get("capabilities").is_some(),
            "Primal must have capabilities"
        );
        // Backward-compat fields
        assert!(p.get("id").is_some(), "Primal must have id");
        assert!(
            p.get("primal_type").is_some(),
            "Primal must have primal_type"
        );
        assert!(
            p.get("socket_path").is_some(),
            "Primal must have socket_path"
        );
        assert!(p.get("health").is_some(), "Primal must have health");
    }

    for c in connections {
        assert!(c.get("from").is_some(), "Connection must have from");
        assert!(c.get("to").is_some(), "Connection must have to");
        assert_eq!(
            c["connection_type"].as_str(),
            Some("security-provider"),
            "Connection type"
        );
    }
}

#[tokio::test]
async fn test_topology_get_with_registered_capabilities() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    router
        .register_capability_unix(
            "security",
            "beardog",
            "/tmp/beardog-test-family.sock",
            "test",
        )
        .await
        .expect("register security");
    router
        .register_capability_unix(
            "discovery",
            "songbird",
            "/tmp/songbird-test-family.sock",
            "test",
        )
        .await
        .expect("register discovery");

    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get().await.expect("topology.get");
    let primals = result["primals"].as_array().expect("primals");
    let connections = result["connections"].as_array().expect("connections");

    assert!(
        primals.len() >= 2,
        "Should discover beardog and songbird from registry, got {}",
        primals.len()
    );

    let primal_ids: Vec<&str> = primals
        .iter()
        .map(|p| p["id"].as_str().unwrap_or(""))
        .collect();
    assert!(
        primal_ids.contains(&"beardog-test-family"),
        "Should have beardog, got {primal_ids:?}"
    );
    assert!(
        primal_ids.contains(&"songbird-test-family"),
        "Should have songbird, got {primal_ids:?}"
    );

    if !connections.is_empty() {
        let conn = &connections[0];
        assert_eq!(conn["connection_type"].as_str(), Some("security-provider"));
        assert!(conn["from"].as_str().is_some());
        assert!(conn["to"].as_str().is_some());
    }
}

#[tokio::test]
async fn test_get_primals_response_format() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_primals().await.expect("get_primals");

    assert_eq!(result["family_id"], "test-family");
    assert!(result.get("timestamp").is_some());
    assert!(result.get("primals").is_some());
    assert!(result.get("count").is_some());

    let count = result["count"].as_u64().expect("count is number");
    let primals = result["primals"].as_array().expect("primals is array");
    assert_eq!(count as usize, primals.len());
}
