// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::NeuralRouter;
use std::path::PathBuf;
use tempfile::tempdir;

use super::{register_crypto_providers, unix_ep};

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
