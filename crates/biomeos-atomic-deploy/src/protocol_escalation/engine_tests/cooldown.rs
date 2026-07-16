// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for engine.rs

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

use std::sync::Arc;
use std::time::Duration;

use crate::living_graph::{LivingGraph, ProtocolMode};

use super::super::config::{EscalationConfig, EscalationResult};
use super::super::engine::*;

#[tokio::test(start_paused = true)]
async fn test_cooldown_tracking() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;

    let config = EscalationConfig {
        escalation_cooldown_secs: 1,
        ..Default::default()
    };

    let manager = ProtocolEscalationManager::new(graph.clone(), config);

    let conn = graph.get_connection("a", "b").await.unwrap();

    assert!(!manager.is_in_cooldown(&conn).await);

    manager.record_cooldown(&conn.id.to_string()).await;

    assert!(manager.is_in_cooldown(&conn).await);

    tokio::time::advance(Duration::from_secs(2)).await;

    assert!(!manager.is_in_cooldown(&conn).await);
}

#[tokio::test]
async fn test_stop_monitoring() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    manager.stop_monitoring().await;
    assert!(!*manager.running.read().await);
}

#[tokio::test]
async fn test_start_monitoring_disabled() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let config = EscalationConfig {
        auto_escalate: false,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);

    manager.start_monitoring().await;
}

#[tokio::test]
async fn test_auto_escalate_check_no_candidates() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let result = manager.auto_escalate_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fallback_connection_not_found() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let result = manager.fallback_connection("a", "b", "test reason").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("connection not found")
    );
}

#[tokio::test]
async fn test_escalate_connection_not_found() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let result = manager.escalate_connection("a", "b").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("connection not found")
    );
}

#[tokio::test]
async fn test_multiple_cooldowns_different_connections() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;
    graph.register_connection("c", "d").await;

    let config = EscalationConfig {
        escalation_cooldown_secs: 60,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph.clone(), config);

    let conn_ab = graph.get_connection("a", "b").await.unwrap();
    let conn_cd = graph.get_connection("c", "d").await.unwrap();

    assert!(!manager.is_in_cooldown(&conn_ab).await);
    assert!(!manager.is_in_cooldown(&conn_cd).await);

    manager.record_cooldown(&conn_ab.id.to_string()).await;
    assert!(manager.is_in_cooldown(&conn_ab).await);
    assert!(!manager.is_in_cooldown(&conn_cd).await);

    manager.record_cooldown(&conn_cd.id.to_string()).await;
    assert!(manager.is_in_cooldown(&conn_ab).await);
    assert!(manager.is_in_cooldown(&conn_cd).await);
}

#[tokio::test]
async fn test_cooldown_zero_duration() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;

    let config = EscalationConfig {
        escalation_cooldown_secs: 0,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph.clone(), config);

    let conn = graph.get_connection("a", "b").await.unwrap();
    manager.record_cooldown(&conn.id.to_string()).await;

    assert!(!manager.is_in_cooldown(&conn).await);
}
async fn test_record_cooldown_multiple_keys() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;
    graph.register_connection("c", "d").await;

    let config = EscalationConfig {
        escalation_cooldown_secs: 60,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph.clone(), config);

    let conn_ab = graph.get_connection("a", "b").await.unwrap();
    let conn_cd = graph.get_connection("c", "d").await.unwrap();

    manager.record_cooldown(&conn_ab.id.to_string()).await;
    manager.record_cooldown(&conn_cd.id.to_string()).await;

    assert!(manager.is_in_cooldown(&conn_ab).await);
    assert!(manager.is_in_cooldown(&conn_cd).await);
}
