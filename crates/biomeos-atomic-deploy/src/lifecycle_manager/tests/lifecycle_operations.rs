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
        boot_order_index: None,
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

#[tokio::test]
async fn test_rapid_restart_detection_carries_forward_attempts() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "skunkbat",
            PathBuf::from("/tmp/skunkbat.sock"),
            Some(999),
            None,
        )
        .await
        .expect("register");

    // Simulate: primal was recently resurrected (30s ago) and has resurrection_count=3
    {
        let mut primals = manager.primals.write().await;
        let primal = primals.get_mut("skunkbat").unwrap();
        primal.metrics.resurrection_count = 3;
        primal.metrics.last_resurrection_at =
            Some(chrono::Utc::now() - chrono::Duration::seconds(30));
        primal.state = LifecycleState::Active {
            since: chrono::Utc::now(),
            last_health_check: chrono::Utc::now(),
        };
        primal.metrics.health_failures = 0;
    }

    // Simulate 3 health failures → triggers degradation
    for _ in 0..3 {
        manager.check_primal_health("skunkbat").await.unwrap();
    }

    // Verify: resurrection_attempts should be carried forward (3), not reset to 0
    let info = manager.get_primal_info("skunkbat").await.unwrap();
    match info.state {
        LifecycleState::Degraded {
            resurrection_attempts,
            ..
        } => {
            assert_eq!(
                resurrection_attempts, 3,
                "rapid restart must carry forward cumulative resurrection_count"
            );
        }
        other => panic!("expected Degraded, got {other:?}"),
    }
}

#[tokio::test]
async fn test_stable_primal_gets_fresh_resurrection_attempts() {
    let manager = LifecycleManager::new("test-family");

    manager
        .register_primal(
            "skunkbat",
            PathBuf::from("/tmp/skunkbat.sock"),
            Some(999),
            None,
        )
        .await
        .expect("register");

    // Simulate: primal was resurrected long ago (5 minutes) — has been stable
    {
        let mut primals = manager.primals.write().await;
        let primal = primals.get_mut("skunkbat").unwrap();
        primal.metrics.resurrection_count = 3;
        primal.metrics.last_resurrection_at =
            Some(chrono::Utc::now() - chrono::Duration::seconds(300));
        primal.state = LifecycleState::Active {
            since: chrono::Utc::now(),
            last_health_check: chrono::Utc::now(),
        };
        primal.metrics.health_failures = 0;
    }

    // Simulate 3 health failures → triggers degradation
    for _ in 0..3 {
        manager.check_primal_health("skunkbat").await.unwrap();
    }

    // Verify: resurrection_attempts reset to 0 (was stable long enough)
    let info = manager.get_primal_info("skunkbat").await.unwrap();
    match info.state {
        LifecycleState::Degraded {
            resurrection_attempts,
            ..
        } => {
            assert_eq!(
                resurrection_attempts, 0,
                "stable primal should get fresh resurrection attempts"
            );
        }
        other => panic!("expected Degraded, got {other:?}"),
    }
}
