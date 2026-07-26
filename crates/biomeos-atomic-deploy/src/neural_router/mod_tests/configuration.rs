// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{NeuralRouter, PerceptronDispatcher, PerceptronPhase, PerceptronWeights};
use biomeos_types::tarpc_types::ProtocolPreference;
use tempfile::tempdir;

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
