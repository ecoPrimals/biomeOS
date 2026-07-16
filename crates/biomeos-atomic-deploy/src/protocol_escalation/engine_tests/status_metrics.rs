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

