// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use super::registration::test_graph_node;
use std::path::PathBuf;

#[tokio::test]
async fn test_apoptosis_nonexistent_primal() {
    let manager = LifecycleManager::new("test-family");
    let result = manager
        .apoptosis("nonexistent", ApoptosisReason::UserRequest)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_apoptosis_updates_state() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal("test-primal", PathBuf::from("/tmp/test.sock"), None, None)
        .await
        .expect("register");

    manager
        .apoptosis("test-primal", ApoptosisReason::UserRequest)
        .await
        .expect("apoptosis");

    let status = manager.get_status().await;
    assert!(matches!(
        status.get("test-primal"),
        Some(LifecycleState::Dead { .. })
    ));
}

#[tokio::test]
async fn test_shutdown_all() {
    let manager = LifecycleManager::new("test-family");

    for name in &["beardog", "songbird"] {
        manager
            .register_primal(
                *name,
                PathBuf::from(format!("/tmp/{name}.sock")),
                None,
                None,
            )
            .await
            .expect("register");
    }

    manager.shutdown_all().await.expect("shutdown");

    let status = manager.get_status().await;
    for state in status.values() {
        assert!(
            matches!(state, LifecycleState::Dead { .. }),
            "Expected Dead state, got: {state:?}"
        );
    }
}

#[tokio::test]
async fn test_collect_shutdown_order() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "beardog",
            PathBuf::from("/tmp/beardog.sock"),
            None,
            Some(test_graph_node("beardog", vec![])),
        )
        .await
        .expect("register beardog");

    manager
        .register_primal(
            "songbird",
            PathBuf::from("/tmp/songbird.sock"),
            None,
            Some(test_graph_node("songbird", vec!["beardog".to_string()])),
        )
        .await
        .expect("register songbird");

    let order = manager.collect_shutdown_order("beardog").await;

    assert_eq!(order.len(), 2);
    let names: Vec<&str> = order.iter().map(|(n, _)| n.as_str()).collect();
    let songbird_pos = names
        .iter()
        .position(|&n| n == "songbird")
        .expect("songbird");
    let beardog_pos = names.iter().position(|&n| n == "beardog").expect("beardog");
    assert!(
        songbird_pos < beardog_pos,
        "Songbird should shut down before beardog"
    );
}

#[tokio::test]
async fn test_clone_for_task() {
    let manager = LifecycleManager::new("test-family");
    manager
        .register_primal("test", PathBuf::from("/tmp/test.sock"), None, None)
        .await
        .expect("register");

    let cloned = manager.clone_for_task();
    let status = cloned.get_status().await;
    assert_eq!(status.len(), 1);
    assert!(status.contains_key("test"));
}

#[test]
fn test_managed_primal_serialization() {
    let primal = ManagedPrimal {
        name: "beardog".to_string(),
        family_id: "test-family".to_string(),
        socket_path: PathBuf::from("/tmp/beardog.sock"),
        pid: Some(1234),
        state: LifecycleState::Germinating,
        deployment_node: None,
        binary_path: None,
        node_id: None,
        depends_on: vec!["base".to_string()],
        depended_by: vec!["songbird".to_string()],
        health_config: HealthConfig::default(),
        resurrection_config: ResurrectionConfig::default(),
        metrics: PrimalMetrics::default(),
    };

    let json = serde_json::to_string(&primal).expect("serialize managed primal");
    let parsed: ManagedPrimal = serde_json::from_str(&json).expect("parse managed primal");
    assert_eq!(parsed.name, "beardog");
    assert_eq!(parsed.family_id, "test-family");
    assert_eq!(parsed.pid, Some(1234));
    assert_eq!(parsed.depends_on, vec!["base"]);
    assert_eq!(parsed.depended_by, vec!["songbird"]);
    assert_eq!(
        parsed.binary_path, None,
        "binary_path should round-trip None"
    );
}

#[tokio::test]
async fn test_register_primal_binary_stores_binary_and_node_id() {
    let manager = LifecycleManager::new("test-family");
    manager
        .register_primal_binary(
            "beardog",
            PathBuf::from("/tmp/beardog.sock"),
            Some(42),
            PathBuf::from("/opt/primals/beardog"),
            "east-gate",
        )
        .await
        .expect("register_primal_binary");

    let info = manager.get_primal_info("beardog").await.expect("info");
    assert_eq!(
        info.binary_path,
        Some(PathBuf::from("/opt/primals/beardog"))
    );
    assert_eq!(info.node_id, Some("east-gate".to_string()));
    assert!(info.deployment_node.is_none());
    assert!(matches!(info.state, LifecycleState::Incubating { .. }));
}
