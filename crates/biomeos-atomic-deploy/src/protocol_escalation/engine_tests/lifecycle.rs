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

#[tokio::test]
async fn test_escalation_manager_creation() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    assert_eq!(manager.graph().family_id(), "test-family");
    assert!(manager.config().auto_escalate);
}

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

async fn test_graph_accessor() {
    let graph = Arc::new(LivingGraph::new("my-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);
    assert_eq!(manager.graph().family_id(), "my-family");
}

#[tokio::test]
async fn test_config_accessor() {
    let config = EscalationConfig {
        min_requests: 77,
        latency_threshold_us: 333,
        auto_escalate: false,
        ..Default::default()
    };
    let graph = Arc::new(LivingGraph::new("test"));
    let manager = ProtocolEscalationManager::new(graph, config);
    assert_eq!(manager.config().min_requests, 77);
    assert_eq!(manager.config().latency_threshold_us, 333);
    assert!(!manager.config().auto_escalate);
}

#[tokio::test]
async fn test_stop_then_check_running_flag() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    assert!(!*manager.running.read().await);

    manager.stop_monitoring().await;
    assert!(!*manager.running.read().await);
}
async fn test_start_monitoring_already_running() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let config = EscalationConfig {
        check_interval_secs: 1,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);

    *manager.running.write().await = true;
    manager.start_monitoring().await;
    assert!(*manager.running.read().await);
}
async fn test_new_with_custom_config() {
    let graph = Arc::new(LivingGraph::new("test"));
    let config = EscalationConfig {
        min_requests: 50,
        latency_threshold_us: 250,
        check_interval_secs: 5,
        escalation_cooldown_secs: 30,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);
    assert_eq!(manager.config().min_requests, 50);
    assert_eq!(manager.config().latency_threshold_us, 250);
    assert_eq!(manager.config().check_interval_secs, 5);
}
async fn manager_graph_family_matches_constructor() {
    let graph = Arc::new(LivingGraph::new("lineage-42"));
    let manager = ProtocolEscalationManager::with_defaults(graph);
    assert_eq!(manager.graph().family_id(), "lineage-42");
}
