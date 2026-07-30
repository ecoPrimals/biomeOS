// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_living_graph_creation() {
    let graph = LivingGraph::new("test-family");
    assert_eq!(graph.family_id(), "test-family");
    assert_eq!(graph.primal_count().await, 0);
    assert_eq!(graph.connection_count().await, 0);
}

#[tokio::test]
async fn test_primal_registration() {
    let graph = LivingGraph::new("test-family");

    let state = PrimalProtocolState::new("beardog", PathBuf::from("/tmp/beardog.sock"))
        .with_capabilities(vec!["crypto".to_string(), "identity".to_string()]);

    graph.register_primal(state).await;

    assert!(graph.has_primal("beardog").await);
    assert_eq!(graph.primal_count().await, 1);

    let retrieved = graph.get_primal_state("beardog").await.unwrap();
    assert_eq!(retrieved.primal_id, "beardog");
    assert_eq!(retrieved.capabilities, vec!["crypto", "identity"]);
}

#[tokio::test]
async fn test_connection_registration() {
    let graph = LivingGraph::new("test-family");

    graph.register_connection("songbird", "beardog").await;
    assert_eq!(graph.connection_count().await, 1);

    let conn = graph.get_connection("songbird", "beardog").await.unwrap();
    assert_eq!(conn.from, "songbird");
    assert_eq!(conn.to, "beardog");
    assert_eq!(conn.protocol, ProtocolMode::JsonRpc);
}

#[tokio::test]
async fn test_protocol_escalation() {
    let graph = LivingGraph::new("test-family");

    graph.register_connection("songbird", "beardog").await;

    // Escalate to tarpc
    graph
        .update_connection_protocol("songbird", "beardog", ProtocolMode::Tarpc)
        .await;

    let conn = graph.get_connection("songbird", "beardog").await.unwrap();
    assert_eq!(conn.protocol, ProtocolMode::Tarpc);
    assert_eq!(conn.escalation_attempts, 1);
}

#[tokio::test]
async fn test_connection_metrics() {
    let graph = LivingGraph::new("test-family");
    graph.register_connection("a", "b").await;

    // Record some requests
    for i in 0..10 {
        graph.record_request("a", "b", 100 + i * 10, true).await;
    }

    let conn = graph.get_connection("a", "b").await.unwrap();
    assert_eq!(conn.metrics.request_count, 10);
    assert!(conn.metrics.avg_latency_us > 100.0);
    assert!(conn.metrics.avg_latency_us < 200.0);
}

#[tokio::test]
async fn test_protocol_summary() {
    let graph = LivingGraph::new("test-family");

    graph.register_connection("a", "b").await;
    graph.register_connection("b", "c").await;
    graph.register_connection("c", "d").await;

    // Escalate one
    graph
        .update_connection_protocol("b", "c", ProtocolMode::Tarpc)
        .await;

    let summary = graph.get_protocol_summary().await;
    assert_eq!(summary.json_rpc, 2);
    assert_eq!(summary.tarpc, 1);
    assert_eq!(summary.total(), 3);
}

#[tokio::test]
async fn test_escalation_candidates() {
    let graph = LivingGraph::new("test-family");
    graph.register_connection("slow", "target").await;
    graph.register_connection("fast", "target").await;

    // Make "slow" connection have high latency
    for _ in 0..100 {
        graph.record_request("slow", "target", 1000, true).await; // 1ms
    }

    // Make "fast" connection have low latency
    for _ in 0..100 {
        graph.record_request("fast", "target", 50, true).await; // 50μs
    }

    let candidates = graph.get_escalation_candidates(50, 500.0).await;
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].from, "slow");
}

#[test]
fn connection_metrics_error_rate_and_p50_ema() {
    let mut m = ConnectionMetrics::default();
    assert!(m.error_rate() <= f64::EPSILON);
    m.record_request(100, true);
    m.record_request(200, false);
    assert!((m.error_rate() - 0.5).abs() < f64::EPSILON);
    assert!(m.p50_latency_us > 0);
    assert_eq!(m.max_latency_us, 200);
    assert!(m.avg_latency_us > 0.0);
}

#[test]
fn connection_metrics_many_samples_percentile_caps() {
    let mut m = ConnectionMetrics::default();
    m.record_request(1000, true);
    assert!(m.p95_latency_us >= 950);
    assert!(m.p99_latency_us >= 990);
}

#[test]
fn protocol_summary_total_sum() {
    let s = ProtocolSummary {
        json_rpc: 2,
        tarpc: 1,
        hybrid: 1,
        degraded: 0,
    };
    assert_eq!(s.total(), 4);
}

#[test]
fn connection_id_and_protocol_mode_display() {
    let id = ConnectionId::new("a", "b");
    assert!(id.to_string().contains('→'));
    assert_eq!(ProtocolMode::JsonRpc.to_string(), "JSON-RPC");
    assert_eq!(ProtocolMode::Tarpc.to_string(), "tarpc");
    assert_eq!(ProtocolMode::Hybrid.to_string(), "Hybrid");
    assert_eq!(ProtocolMode::Degraded.to_string(), "Degraded");
}

#[test]
fn primal_protocol_state_tarpc_counters() {
    let mut p = PrimalProtocolState::new("p", std::path::PathBuf::from("/x.sock"));
    assert!(!p.tarpc_available());
    p.record_tarpc_failure();
    assert_eq!(p.tarpc_failure_count, 1);
    p.reset_tarpc_failures();
    assert_eq!(p.tarpc_failure_count, 0);
}

#[tokio::test]
async fn test_primal_removal() {
    let graph = LivingGraph::new("test-family");

    graph
        .register_primal(PrimalProtocolState::new(
            "primal-a",
            PathBuf::from("/tmp/a.sock"),
        ))
        .await;
    graph.register_connection("primal-a", "primal-b").await;
    graph.register_connection("primal-b", "primal-a").await;
    graph.register_connection("primal-b", "primal-c").await;

    assert_eq!(graph.connection_count().await, 3);

    graph.remove_primal("primal-a").await;

    assert!(!graph.has_primal("primal-a").await);
    assert_eq!(graph.connection_count().await, 1); // Only b→c remains
}
