// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{
    CompositionPattern, CompositionTier, NeuralRouter, PerceptronDispatcher, PerceptronPhase,
    PerceptronWeights,
};
use biomeos_core::TransportEndpoint;
use biomeos_types::tarpc_types::ProtocolPreference;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

use super::register_crypto_providers;

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
