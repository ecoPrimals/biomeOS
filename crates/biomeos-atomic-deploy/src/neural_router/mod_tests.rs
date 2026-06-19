// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::NeuralRouter`] core API in `mod.rs`:
//! provider selection, fallback routing, dispatch outcomes, and configuration.

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::{
    CompositionPattern, CompositionTier, NeuralRouter, PerceptronDispatcher, PerceptronPhase,
    PerceptronWeights,
};
use biomeos_core::TransportEndpoint;
use biomeos_types::tarpc_types::ProtocolPreference;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn unix_ep(path: &std::path::Path) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: path.to_path_buf(),
    }
}

async fn register_crypto_providers(router: &NeuralRouter, slow: &str, fast: &str) {
    let dir = tempdir().expect("tempdir");
    let slow_sock = dir.path().join(format!("{slow}.sock"));
    let fast_sock = dir.path().join(format!("{fast}.sock"));
    router
        .register_capability_unix("crypto", slow, &slow_sock, "test")
        .await
        .expect("register slow");
    router
        .register_capability_unix("crypto", fast, &fast_sock, "test")
        .await
        .expect("register fast");
    std::mem::forget(dir);
}

// --- Configuration loading ---

#[test]
fn new_router_uses_in_memory_weights() {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let router = NeuralRouter::new("cfg-test");
        assert!(!router.weights_are_persistent().await);
    });
}

#[tokio::test]
async fn new_reads_protocol_preference_from_env() {
    temp_env::async_with_vars([("IPC_PROTOCOL", Some("tarpc"))], async {
        let router = NeuralRouter::new("env-test");
        assert_eq!(router.protocol_preference, ProtocolPreference::TarpcOnly);
    })
    .await;
}

#[tokio::test]
async fn with_persistent_weights_loads_and_persists_outcomes() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("routing_weights.redb");

    {
        let router = NeuralRouter::with_persistent_weights("persist-fam", &path);
        assert!(router.weights_are_persistent().await);
        router
            .record_dispatch_outcome("crypto", "beardog", true, 12)
            .await;
        router.flush_weights().await;
    }

    {
        let router = NeuralRouter::with_persistent_weights("persist-fam", &path);
        assert!(router.weights_are_persistent().await);
        let summary = router.get_weight_summary().await;
        assert_eq!(summary.total_dispatches, 1);
        assert_eq!(summary.unique_providers, 1);
    }
}

#[tokio::test]
async fn with_perceptron_exposes_shadow_metadata() {
    let dispatcher = PerceptronDispatcher::new(
        PerceptronWeights::neutral_default(),
        PerceptronPhase::Shadow,
    );
    let router = NeuralRouter::new("perc-test").with_perceptron(dispatcher);

    assert_eq!(router.perceptron_phase(), Some(PerceptronPhase::Shadow));
    assert!(!router.perceptron_has_remote_infer());
    assert_eq!(router.perceptron_shadow_stats(), Some((0, 0)));
}

// --- Provider selection ---

#[tokio::test]
async fn select_weighted_provider_returns_none_for_unknown_capability() {
    let router = NeuralRouter::new("sel-test");
    assert!(
        router
            .select_weighted_provider("missing.cap")
            .await
            .is_none()
    );
}

#[tokio::test]
async fn select_weighted_provider_falls_back_to_first_without_observations() {
    let router = NeuralRouter::new("sel-test");
    register_crypto_providers(&router, "alpha", "beta").await;

    let chosen = router
        .select_weighted_provider("crypto")
        .await
        .expect("should fall back to first registered provider");
    let providers = router.get_capability_providers("crypto").await.unwrap();
    assert_eq!(chosen.as_ref(), providers[0].primal_name.as_ref());
}

#[tokio::test]
async fn select_weighted_provider_prefers_lower_latency_provider() {
    let router = NeuralRouter::new("sel-test");
    register_crypto_providers(&router, "slow_beardog", "fast_beardog").await;

    for _ in 0..10 {
        router
            .record_dispatch_outcome("crypto", "fast_beardog", true, 5)
            .await;
        router
            .record_dispatch_outcome("crypto", "slow_beardog", true, 500)
            .await;
    }

    let chosen = router
        .select_weighted_provider("crypto")
        .await
        .expect("weighted selection");
    assert_eq!(chosen.as_ref(), "fast_beardog");
}

#[tokio::test]
async fn select_primary_single_provider_returns_zero() {
    let router = NeuralRouter::new("sel-test");
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("solo.sock");
    router
        .register_capability_unix("crypto", "solo", &sock, "test")
        .await
        .expect("register");

    let providers = router.get_capability_providers("crypto").await.unwrap();
    assert_eq!(router.select_primary("crypto", &providers).await, 0);
}

#[tokio::test]
async fn select_primary_prefers_weighted_provider_over_first_match() {
    let router = NeuralRouter::new("sel-test");
    register_crypto_providers(&router, "first_match", "weighted_winner").await;

    for _ in 0..10 {
        router
            .record_dispatch_outcome("crypto", "weighted_winner", true, 5)
            .await;
        router
            .record_dispatch_outcome("crypto", "first_match", true, 800)
            .await;
    }

    let providers = router.get_capability_providers("crypto").await.unwrap();
    let idx = router.select_primary("crypto", &providers).await;
    assert_eq!(providers[idx].primal_name.as_ref(), "weighted_winner");

    let (total, disagreements) = router.shadow_stats();
    assert!(total >= 1);
    assert!(disagreements >= 1);
}

#[tokio::test]
async fn set_provider_affinity_biases_weighted_selection() {
    let router = NeuralRouter::new("affinity-test");
    register_crypto_providers(&router, "low_affinity", "high_affinity").await;

    router
        .set_provider_affinity("crypto", "high_affinity", 0.99)
        .await;
    router
        .set_provider_affinity("crypto", "low_affinity", 0.01)
        .await;

    for _ in 0..5 {
        router
            .record_dispatch_outcome("crypto", "low_affinity", true, 50)
            .await;
        router
            .record_dispatch_outcome("crypto", "high_affinity", true, 50)
            .await;
    }

    let chosen = router
        .select_weighted_provider("crypto")
        .await
        .expect("affinity-weighted selection");
    assert_eq!(chosen.as_ref(), "high_affinity");
}

#[tokio::test]
async fn set_provider_cost_hint_and_topology_affinity_are_queryable() {
    let router = NeuralRouter::new("hints-test");
    let endpoint = unix_ep(&PathBuf::from("/tmp/hints.sock"));

    router
        .register_capability("crypto", "beardog", endpoint.clone(), "test")
        .await
        .expect("register");
    router
        .set_provider_cost_hint("crypto", "beardog", 42.0)
        .await;
    router
        .set_provider_topology_affinity("crypto", "beardog", &endpoint)
        .await;

    let weights = router.get_routing_weights().await;
    let w = weights
        .iter()
        .find(|w| w.provider.as_ref() == "beardog")
        .expect("provider weight");
    assert_eq!(w.cost_hint, Some(42.0));
    assert!(w.topology_affinity > 0.0);
}

// --- Fallback routing (circuit-broken provider skipped) ---

#[tokio::test]
async fn select_weighted_provider_skips_circuit_broken_provider() {
    let router = NeuralRouter::new("fallback-test");
    register_crypto_providers(&router, "healthy", "broken").await;

    for _ in 0..10 {
        router
            .record_dispatch_outcome("crypto", "healthy", true, 20)
            .await;
    }
    for _ in 0..5 {
        router
            .record_dispatch_outcome("crypto", "broken", false, 0)
            .await;
    }

    let chosen = router
        .select_weighted_provider("crypto")
        .await
        .expect("healthy fallback");
    assert_eq!(chosen.as_ref(), "healthy");
}

// --- Error propagation / dispatch outcomes ---

#[tokio::test]
async fn record_dispatch_outcome_tracks_failures_in_weight_table() {
    let router = NeuralRouter::new("outcome-test");
    register_crypto_providers(&router, "p1", "p2").await;

    router
        .record_dispatch_outcome("crypto", "p1", false, 0)
        .await;
    router
        .record_dispatch_outcome("crypto", "p1", false, 0)
        .await;

    let weights = router.get_routing_weights().await;
    let w = weights
        .iter()
        .find(|w| w.provider.as_ref() == "p1")
        .expect("p1 weight");
    assert!(w.ewma_error_rate > 0.0);
}

#[tokio::test]
async fn record_dispatch_outcome_completes_stashed_training_row() {
    let router = NeuralRouter::new("train-test");
    register_crypto_providers(&router, "first", "second").await;

    let providers = router.get_capability_providers("crypto").await.unwrap();
    let idx = router.select_primary("crypto", &providers).await;
    let provider = providers[idx].primal_name.clone();

    assert_eq!(router.training_data_count().await, 0);

    router
        .record_dispatch_outcome("crypto", &provider, true, 99)
        .await;

    assert_eq!(router.training_data_count().await, 1);
    let rows = router.drain_training_data().await;
    assert_eq!(rows.len(), 1);
    assert!(rows[0].success);
    assert_eq!(rows[0].latency_ms, 99);
    assert_eq!(rows[0].chosen_idx, idx);
    assert_eq!(router.training_data_count().await, 0);
}

// --- Utilization tracking ---

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
    assert_eq!(hot[0].method, "crypto.hash");

    let cold = router.cold_methods(5).await;
    assert_eq!(cold.len(), 1);
    assert_eq!(cold[0].method, "storage.put");

    let json = router.utilization_json().await;
    assert_eq!(json["tracked_methods"], 2);
    assert_eq!(json["total_calls"], 21);
}

// --- Composition API ---

#[test]
fn classify_tier_delegates_to_composition_module() {
    let router = NeuralRouter::new("tier-test");
    assert_eq!(
        router.classify_tier("crypto", "beardog"),
        CompositionTier::Tower
    );
    assert_eq!(
        router.classify_tier("unknown_domain", "beardog"),
        CompositionTier::Tower
    );
}

#[tokio::test]
async fn select_weighted_provider_falls_back_to_first_when_all_circuits_open() {
    let router = NeuralRouter::new("circuit-test");
    register_crypto_providers(&router, "broken_a", "broken_b").await;

    for _ in 0..5 {
        router
            .record_dispatch_outcome("crypto", "broken_a", false, 0)
            .await;
        router
            .record_dispatch_outcome("crypto", "broken_b", false, 0)
            .await;
    }

    let chosen = router
        .select_weighted_provider("crypto")
        .await
        .expect("falls back to first when scoring returns None");
    let providers = router.get_capability_providers("crypto").await.unwrap();
    assert_eq!(chosen.as_ref(), providers[0].primal_name.as_ref());
}

#[tokio::test]
async fn metrics_log_get_and_clear() {
    let router = NeuralRouter::new("metrics-test");

    let metric = super::types::RoutingMetrics {
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
