// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for engine.rs

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

use std::sync::Arc;
use std::time::Duration;

use crate::living_graph::{LivingGraph, ProtocolMode};

use super::config::{EscalationConfig, EscalationResult};
use super::engine::*;

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

#[tokio::test]
async fn test_fallback_connection_not_found() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let result = manager.fallback_connection("a", "b", "test reason").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Connection not found"));
}

#[tokio::test]
async fn test_escalate_connection_not_found() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let result = manager.escalate_connection("a", "b").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Connection not found"));
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

#[tokio::test]
async fn test_fallback_existing_connection() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("songbird", "beardog").await;
    graph
        .update_connection_protocol("songbird", "beardog", ProtocolMode::Tarpc)
        .await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let result = manager
        .fallback_connection("songbird", "beardog", "tarpc failure")
        .await;

    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(r.success);
    assert_eq!(r.current_mode, ProtocolMode::Degraded);
    assert!(r.message.contains("tarpc failure"));
}

#[tokio::test]
async fn test_escalate_existing_connection_no_primal_state() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("songbird", "beardog").await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let result = manager.escalate_connection("songbird", "beardog").await;

    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(!r.success);
    assert!(r.message.contains("Failed to query tarpc endpoint"));
}

#[tokio::test]
async fn test_get_status() {
    let graph = Arc::new(LivingGraph::new("status-family"));
    graph.register_connection("a", "b").await;
    graph.register_connection("b", "c").await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let status = manager.get_status().await;

    assert!(status.get("connections").is_some());
    assert!(status.get("summary").is_some());
    assert!(status.get("config").is_some());
    assert_eq!(status["summary"]["total"], 2);
}

#[tokio::test]
async fn test_get_status_empty_graph() {
    let graph = Arc::new(LivingGraph::new("empty-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);
    let status = manager.get_status().await;

    assert_eq!(status["summary"]["total"], 0);
    assert!(status["connections"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_connection_metrics_existing() {
    let graph = Arc::new(LivingGraph::new("metrics-family"));
    graph.register_connection("songbird", "beardog").await;
    graph.record_request("songbird", "beardog", 150, true).await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let metrics = manager.get_connection_metrics("songbird", "beardog").await;

    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m["connection"]["from"], "songbird");
    assert_eq!(m["connection"]["to"], "beardog");
    assert_eq!(m["metrics"]["request_count"], 1);
}

#[tokio::test]
async fn test_get_connection_metrics_nonexistent() {
    let graph = Arc::new(LivingGraph::new("metrics-family"));
    let manager = ProtocolEscalationManager::with_defaults(graph);

    let metrics = manager.get_connection_metrics("a", "b").await;
    assert!(metrics.is_none());
}

#[tokio::test]
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
async fn test_escalate_connection_tarpc_unavailable() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;

    let from_state = crate::living_graph::PrimalProtocolState::new(
        "a",
        std::path::PathBuf::from("/nonexistent/a.sock"),
    );
    graph.register_primal(from_state).await;

    let to_state = crate::living_graph::PrimalProtocolState::new(
        "b",
        std::path::PathBuf::from("/nonexistent/b.sock"),
    );
    graph.register_primal(to_state).await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let result = manager.escalate_connection("a", "b").await;

    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(!r.success);
    assert!(
        r.message.contains("Failed to query tarpc endpoint")
            || r.message.contains("Target primal does not support tarpc")
    );
}

#[tokio::test]
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

#[tokio::test]
async fn test_escalation_result_serialization_roundtrip() {
    let result = EscalationResult {
        from: "songbird".to_string(),
        to: "beardog".to_string(),
        previous_mode: ProtocolMode::JsonRpc,
        current_mode: ProtocolMode::Tarpc,
        tarpc_socket: Some(std::path::PathBuf::from("/tmp/beardog.sock")),
        success: true,
        message: "Escalated".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: EscalationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.from, result.from);
    assert_eq!(parsed.success, result.success);
}

#[tokio::test]
async fn test_get_status_with_mixed_protocols() {
    let graph = Arc::new(LivingGraph::new("test-family"));
    graph.register_connection("a", "b").await;
    graph.register_connection("b", "c").await;
    graph
        .update_connection_protocol("a", "b", ProtocolMode::Tarpc)
        .await;

    let manager = ProtocolEscalationManager::with_defaults(graph);
    let status = manager.get_status().await;

    assert!(status.get("summary").is_some());
    assert_eq!(status["summary"]["total"], 2);
    assert!(status["summary"]["tarpc"].as_u64().unwrap_or(0) >= 1);
}

#[tokio::test]
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

#[tokio::test]
async fn manager_graph_family_matches_constructor() {
    let graph = Arc::new(LivingGraph::new("lineage-42"));
    let manager = ProtocolEscalationManager::with_defaults(graph);
    assert_eq!(manager.graph().family_id(), "lineage-42");
}

#[tokio::test]
async fn fallback_connection_preserves_message_on_success() {
    let graph = Arc::new(LivingGraph::new("fam"));
    graph.register_connection("src", "dst").await;
    let manager = ProtocolEscalationManager::with_defaults(graph);
    let r = manager
        .fallback_connection("src", "dst", "latency spike")
        .await
        .expect("fallback");
    assert!(r.success);
    assert!(r.message.contains("latency spike"));
    assert_eq!(r.current_mode, ProtocolMode::Degraded);
}
