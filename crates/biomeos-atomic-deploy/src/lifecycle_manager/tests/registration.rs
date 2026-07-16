// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use crate::neural_graph::GraphConfig;
use std::path::PathBuf;

#[tokio::test]
async fn test_lifecycle_manager_creation() {
    let manager = LifecycleManager::new("test-family");
    let status = manager.get_status().await;
    assert!(status.is_empty());
}

#[tokio::test]
async fn test_lifecycle_manager_with_config() {
    let nucleation = Arc::new(RwLock::new(SocketNucleation::default()));
    let manager =
        LifecycleManager::with_config("custom-family", Duration::from_secs(5), nucleation);
    let status = manager.get_status().await;
    assert!(status.is_empty());
}

#[tokio::test]
async fn test_register_primal_incubating() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "test-primal",
            PathBuf::from("/tmp/test-primal.sock"),
            Some(12345),
            None,
        )
        .await
        .expect("register primal");

    let status = manager.get_status().await;
    assert!(matches!(
        status.get("test-primal"),
        Some(LifecycleState::Incubating { .. })
    ));
}

#[tokio::test]
async fn test_register_multiple_primals() {
    let manager = LifecycleManager::new("test-family");

    for name in &["beardog", "songbird", "nestgate"] {
        manager
            .register_primal(
                *name,
                PathBuf::from(format!("/tmp/{name}.sock")),
                Some(100),
                None,
            )
            .await
            .expect("register primal");
    }

    let status = manager.get_status().await;
    assert_eq!(status.len(), 3);
    assert!(status.contains_key("beardog"));
    assert!(status.contains_key("songbird"));
    assert!(status.contains_key("nestgate"));
}

#[tokio::test]
async fn test_get_primal_info() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "beardog",
            PathBuf::from("/tmp/beardog.sock"),
            Some(42),
            None,
        )
        .await
        .expect("register");

    let info = manager.get_primal_info("beardog").await;
    assert!(info.is_some());
    let info = info.expect("primal info");
    assert_eq!(info.name, "beardog");
    assert_eq!(info.family_id, "test-family");
    assert_eq!(info.pid, Some(42));
    assert_eq!(info.socket_path, PathBuf::from("/tmp/beardog.sock"));
}

#[tokio::test]
async fn test_get_primal_info_not_found() {
    let manager = LifecycleManager::new("test-family");
    let info = manager.get_primal_info("nonexistent").await;
    assert!(info.is_none());
}

pub(super) fn test_graph_node(id: &str, depends_on: Vec<String>) -> crate::neural_graph::GraphNode {
    crate::neural_graph::GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_register_with_dependencies() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "beardog",
            PathBuf::from("/tmp/beardog.sock"),
            Some(100),
            Some(test_graph_node("beardog", vec![])),
        )
        .await
        .expect("register beardog");

    manager
        .register_primal(
            "songbird",
            PathBuf::from("/tmp/songbird.sock"),
            Some(101),
            Some(test_graph_node("songbird", vec!["beardog".to_string()])),
        )
        .await
        .expect("register songbird");

    let beardog = manager.get_primal_info("beardog").await.expect("beardog");
    assert!(beardog.depended_by.contains(&"songbird".to_string()));

    let songbird = manager.get_primal_info("songbird").await.expect("songbird");
    assert!(songbird.depends_on.contains(&"beardog".to_string()));
}

#[tokio::test]
async fn test_store_deployment_graph() {
    let manager = LifecycleManager::new("test-family");

    let graph = crate::neural_graph::Graph {
        id: "tower".to_string(),
        version: "1.0.0".to_string(),
        description: "Test graph".to_string(),
        nodes: vec![test_graph_node("beardog", vec![])],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    manager.store_deployment_graph("tower-graph", graph).await;
}
