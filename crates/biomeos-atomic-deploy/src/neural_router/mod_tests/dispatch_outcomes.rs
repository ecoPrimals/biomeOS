// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::NeuralRouter;

use super::register_crypto_providers;

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
