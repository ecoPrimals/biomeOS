// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{
    CompositionPattern, CompositionTier, NeuralRouter, PerceptronDispatcher,
};
use biomeos_types::tarpc_types::ProtocolPreference;
use std::sync::Arc;

use super::register_crypto_providers;

#[tokio::test]
async fn utilization_tracking_hot_cold_and_json() {
    let router = NeuralRouter::new("util-test");

    for _ in 0..20 {
        router.record_utilization("crypto.hash").await;
    }
    router.record_utilization("storage.put").await;

    let summary = router.utilization_summary().await;
    assert_eq!(summary.tracked_methods, 2);
    assert_eq!(summary.total_calls, 21);

    let hot = router.hot_methods(1).await;
    assert_eq!(hot.len(), 1);
    assert_eq!(&*hot[0].method, "crypto.hash");

    let cold = router.cold_methods(5).await;
    assert_eq!(cold.len(), 1);
    assert_eq!(&*cold[0].method, "storage.put");

    let json = router.utilization_json().await;
    assert_eq!(json["tracked_methods"], 2);
    assert_eq!(json["total_calls"], 21);
}
#[tokio::test]
async fn metrics_log_get_and_clear() {
    let router = NeuralRouter::new("metrics-test");

    let metric = super::super::types::RoutingMetrics {
        request_id: Arc::from("req-1"),
        capability: Arc::from("crypto"),
        method: Arc::from("crypto.hash"),
        routed_through: vec![Arc::from("beardog")],
        latency_ms: 7,
        success: true,
        timestamp: chrono::Utc::now(),
        error: None,
    };

    router.log_metric(metric).await;
    assert_eq!(router.get_metrics().await.len(), 1);

    router.clear_metrics().await;
    assert!(router.get_metrics().await.is_empty());
}

#[test]
fn builder_methods_attach_living_graph_and_protocol_override() {
    let graph = std::sync::Arc::new(crate::living_graph::LivingGraph::new("g"));
    let router = NeuralRouter::new("builder-test")
        .with_protocol_preference(ProtocolPreference::PreferTarpc)
        .with_living_graph(graph);
    assert_eq!(router.protocol_preference, ProtocolPreference::PreferTarpc);
    assert!(router.living_graph.is_some());
}

#[tokio::test]
async fn perceptron_with_remote_infer_reports_capability() {
    let dispatcher =
        PerceptronDispatcher::shadow_default().with_remote_infer("/tmp/neural-api.sock".to_owned());
    let router = NeuralRouter::new("remote-perc").with_perceptron(dispatcher);

    assert!(router.perceptron_has_remote_infer());
    assert_eq!(router.perceptron_shadow_stats(), Some((0, 0)));
}

#[tokio::test]
async fn select_primary_with_perceptron_runs_shadow_path() {
    let router =
        NeuralRouter::new("perc-primary").with_perceptron(PerceptronDispatcher::shadow_default());
    register_crypto_providers(&router, "p_slow", "p_fast").await;

    for _ in 0..5 {
        router
            .record_dispatch_outcome("crypto", "p_fast", true, 3)
            .await;
    }

    let providers = router.get_capability_providers("crypto").await.unwrap();
    let idx = router.select_primary("crypto", &providers).await;
    assert_eq!(providers[idx].primal_name.as_ref(), "p_fast");

    let stats = router
        .perceptron_shadow_stats()
        .expect("perceptron attached");
    assert!(stats.0 >= 1);
}

#[tokio::test]
async fn record_dispatch_outcome_without_pending_still_records_weights() {
    let router = NeuralRouter::new("no-pending");
    register_crypto_providers(&router, "solo", "other").await;

    router
        .record_dispatch_outcome("crypto", "solo", true, 11)
        .await;

    let summary = router.get_weight_summary().await;
    assert_eq!(summary.total_dispatches, 1);
    assert_eq!(router.training_data_count().await, 0);
}

#[tokio::test]
async fn get_pattern_returns_none_for_unknown_name() {
    let router = NeuralRouter::new("pattern-miss");
    assert!(router.get_pattern("does_not_exist_xyz").await.is_none());
}

#[tokio::test]
async fn training_log_evicts_oldest_row_at_capacity() {
    let router = NeuralRouter::new("train-cap");
    register_crypto_providers(&router, "a", "b").await;
    let providers = router.get_capability_providers("crypto").await.unwrap();

    for i in 0..10_001u64 {
        let idx = router.select_primary("crypto", &providers).await;
        let provider = providers[idx].primal_name.clone();
        router
            .record_dispatch_outcome("crypto", &provider, true, i)
            .await;
    }

    assert_eq!(router.training_data_count().await, 10_000);
    let rows = router.drain_training_data().await;
    assert_eq!(rows.len(), 10_000);
    assert_eq!(rows[0].latency_ms, 1, "oldest row should have been evicted");
}

#[tokio::test]
async fn composition_patterns_register_lookup_reload_and_plan() {
    let router = NeuralRouter::new("comp-test");

    let patterns = router.get_composition_patterns().await;
    assert!(!patterns.is_empty());
    assert!(router.get_pattern("rootpulse_commit").await.is_some());

    let custom = CompositionPattern {
        name: Arc::from("test_pattern_xyz"),
        methods: vec![Arc::from("crypto.hash")],
        primals: vec![Arc::from("beardog")],
        tier: CompositionTier::Tower,
        graph_file: None,
    };
    router.register_composition_pattern(custom.clone()).await;
    assert_eq!(
        router
            .get_pattern("test_pattern_xyz")
            .await
            .unwrap()
            .name
            .as_ref(),
        "test_pattern_xyz"
    );

    let reloaded = router.reload_composition_patterns().await;
    assert!(reloaded >= patterns.len());

    let plan = router.plan_tier(CompositionTier::Tower).await;
    assert_eq!(plan.tier, CompositionTier::Tower);
    assert!(!plan.required_primals.is_empty());

    let json = router.composition_patterns_json().await;
    assert!(json.get("patterns").is_some());
}
