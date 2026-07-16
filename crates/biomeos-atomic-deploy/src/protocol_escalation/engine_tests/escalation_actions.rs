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
