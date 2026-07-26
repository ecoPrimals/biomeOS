// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for engine.rs

#![expect(clippy::unwrap_used, reason = "test")]

use std::sync::Arc;

use crate::living_graph::LivingGraph;

use super::super::config::EscalationConfig;
use super::super::engine::*;

#[tokio::test]
async fn test_auto_escalate_check_with_low_traffic_connections() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("songbird", "beardog").await;
    graph
        .record_request("songbird", "beardog", 1000, true)
        .await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let result = manager.auto_escalate_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auto_escalate_check_with_candidates_in_cooldown() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;
    graph.record_request("a", "b", 2000, true).await;
    graph.record_request("a", "b", 2000, true).await;

    let config = EscalationConfig {
        min_requests: 1,
        latency_threshold_us: 100,
        escalation_cooldown_secs: 60,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph.clone(), config);

    let conn = graph.get_connection("a", "b").await.unwrap();
    manager.record_cooldown(&conn.id.to_string()).await;

    let result = manager.auto_escalate_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auto_escalate_check_candidates_unhealthy_skipped() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;
    graph.record_request("a", "b", 2000, true).await;

    let from_state =
        crate::living_graph::PrimalProtocolState::new("a", std::path::PathBuf::from("/tmp/a.sock"))
            .with_capabilities(vec!["test".to_string()]);
    graph.register_primal(from_state).await;

    let to_state =
        crate::living_graph::PrimalProtocolState::new("b", std::path::PathBuf::from("/tmp/b.sock"))
            .with_capabilities(vec!["test".to_string()]);
    graph.register_primal(to_state).await;

    let config = EscalationConfig {
        min_requests: 1,
        latency_threshold_us: 100,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);

    let result = manager.auto_escalate_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn auto_escalate_skips_when_below_min_requests() {
    let graph = Arc::new(LivingGraph::new("fam"));
    graph.register_connection("a", "b").await;
    graph.record_request("a", "b", 5000, true).await;

    let config = EscalationConfig {
        min_requests: 100,
        latency_threshold_us: 1,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);
    assert!(manager.auto_escalate_check().await.is_ok());
}

#[tokio::test]
async fn auto_escalate_skips_when_latency_below_threshold() {
    let graph = Arc::new(LivingGraph::new("fam"));
    graph.register_connection("a", "b").await;
    for _ in 0..200 {
        graph.record_request("a", "b", 10, true).await;
    }

    let config = EscalationConfig {
        min_requests: 1,
        latency_threshold_us: 1_000_000,
        ..Default::default()
    };
    let manager = ProtocolEscalationManager::new(graph, config);
    assert!(manager.auto_escalate_check().await.is_ok());
}

#[tokio::test]
async fn escalation_candidates_empty_for_json_rpc_only_without_volume() {
    let graph = Arc::new(LivingGraph::new("fam"));
    graph.register_connection("x", "y").await;
    let c = graph.get_escalation_candidates(100, 100.0).await;
    assert!(c.is_empty());
}
