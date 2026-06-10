// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test code")]

use super::*;

#[test]
fn mock_weights_are_neutral() {
    let w = PerceptronWeights::neutral_default();
    assert_eq!(w.weights.len(), WEIGHT_VEC_LEN);
    assert!(
        (w.weights[FEATURE_DIM]).abs() < f32::EPSILON,
        "bias should be 0"
    );
}

#[test]
fn feature_dim_is_36() {
    assert_eq!(FEATURE_DIM, 36);
    assert_eq!(WEIGHT_VEC_LEN, 37);
}

#[test]
fn build_features_sets_onehot() {
    let f = DispatchFeatures::build(5, None, 0.3);
    assert!((f.values[5] - 1.0).abs() < f32::EPSILON);
    for i in 0..32 {
        if i != 5 {
            assert!(f.values[i].abs() < f32::EPSILON, "slot {i} should be 0");
        }
    }
    assert!((f.values[35] - 0.3).abs() < f32::EPSILON, "gate_load");
}

#[test]
fn build_features_with_provider_weight() {
    let mut pw = ProviderWeight::new("test-provider", "test-cap");
    pw.ewma_latency_ms = 100.0;
    pw.ewma_error_rate = 0.05;
    pw.topology_affinity = 0.9;

    let f = DispatchFeatures::build(0, Some(&pw), 0.0);
    assert!((f.values[32] - 0.2).abs() < 0.01, "latency 100/500=0.2");
    assert!((f.values[33] - 0.05).abs() < f32::EPSILON, "error rate");
    assert!((f.values[34] - 0.9).abs() < f32::EPSILON, "topology");
}

#[test]
fn domain_to_index_is_stable() {
    let idx1 = domain_to_index("crypto.hash");
    let idx2 = domain_to_index("crypto.verify");
    assert_eq!(idx1, idx2, "same domain prefix → same slot");
    assert!(idx1 < 32);

    let idx3 = domain_to_index("storage.put");
    assert!(idx3 < 32);
}

#[test]
fn domain_to_index_varies_across_domains() {
    let domains = ["crypto", "storage", "compute", "mesh", "relay", "security"];
    let indices: Vec<usize> = domains.iter().map(|d| domain_to_index(d)).collect();
    let unique: std::collections::HashSet<_> = indices.iter().collect();
    assert!(
        unique.len() >= 4,
        "most domains should hash to different slots"
    );
}

#[test]
fn score_forward_pass() {
    let w = PerceptronWeights::neutral_default();
    let f = DispatchFeatures::build(0, None, 0.0);
    let score = w.score(&f);
    // With mock weights and default features (latency=0.5, error=0, topo=1.0, load=0):
    // = (-0.3 * 0.5) + (-0.5 * 0.0) + (0.4 * 1.0) + (-0.1 * 0.0) + 0 (bias) = 0.25
    assert!((score - 0.25).abs() < 0.01, "expected ~0.25, got {score}");
}

#[test]
fn recommend_picks_highest_score() {
    let dispatcher = PerceptronDispatcher::shadow_default();

    let mut low_latency = DispatchFeatures::build(0, None, 0.0);
    low_latency.values[32] = 0.1; // fast
    low_latency.values[34] = 1.0; // same gate

    let mut high_latency = DispatchFeatures::build(0, None, 0.0);
    high_latency.values[32] = 0.9; // slow
    high_latency.values[34] = 0.3; // remote

    let idx = dispatcher.recommend(&[high_latency, low_latency]);
    assert_eq!(idx, 1, "should prefer low latency + high topology");
}

#[test]
fn shadow_compare_tracks_disagreement() {
    let dispatcher = PerceptronDispatcher::shadow_default();

    let same = DispatchFeatures::build(0, None, 0.0);
    dispatcher.shadow_compare(0, &[same.clone(), same.clone()], "test.cap");
    let (total, disagree) = dispatcher.shadow_stats();
    assert_eq!(total, 1);
    assert_eq!(disagree, 0, "identical features → agreement");

    let mut fast = DispatchFeatures::build(0, None, 0.0);
    fast.values[32] = 0.0; // very fast
    fast.values[34] = 1.0;
    let mut slow = DispatchFeatures::build(0, None, 0.0);
    slow.values[32] = 1.0; // very slow
    slow.values[34] = 0.3;

    // Rule chose idx=0 (slow), perceptron should prefer idx=1 (fast)
    dispatcher.shadow_compare(0, &[slow, fast], "test.cap");
    let (total, disagree) = dispatcher.shadow_stats();
    assert_eq!(total, 2);
    assert_eq!(
        disagree, 1,
        "perceptron disagrees with rule picking slow provider"
    );
}

#[test]
fn shadow_default_starts_in_shadow_phase() {
    let d = PerceptronDispatcher::shadow_default();
    assert_eq!(d.phase(), PerceptronPhase::Shadow);
}

#[test]
fn weight_file_load_wrong_size_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad_weights.bin");
    std::fs::write(&path, &[0u8; 10]).unwrap();
    assert!(PerceptronWeights::load_from_file(&path).is_none());
}

#[test]
fn weight_file_load_correct_size() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("good_weights.bin");
    let data: Vec<u8> = (0..WEIGHT_VEC_LEN)
        .flat_map(|i| (i as f32 * 0.01).to_le_bytes())
        .collect();
    std::fs::write(&path, &data).unwrap();
    let w = PerceptronWeights::load_from_file(&path).unwrap();
    assert!((w.weights[0]).abs() < f32::EPSILON);
    assert!((w.weights[1] - 0.01).abs() < 0.001);
}

#[test]
fn build_candidate_features_produces_correct_count() {
    let table = RoutingWeightTable::new();
    let candidates: Vec<Arc<str>> = vec![Arc::from("provider-a"), Arc::from("provider-b")];
    let features = build_candidate_features("crypto.hash", &candidates, &table, 0.5);
    assert_eq!(features.len(), 2);
    assert!((features[0].values[35] - 0.5).abs() < f32::EPSILON);
}

#[test]
fn remote_infer_default_is_none() {
    let d = PerceptronDispatcher::shadow_default();
    assert!(!d.has_remote_infer());
}

#[test]
fn with_remote_infer_enables_remote() {
    let d = PerceptronDispatcher::shadow_default()
        .with_remote_infer("/tmp/neural-api.sock".to_string());
    assert!(d.has_remote_infer());
}

#[tokio::test]
async fn shadow_compare_remote_falls_back_without_socket() {
    let d = PerceptronDispatcher::shadow_default();
    let f = DispatchFeatures::build(0, None, 0.0);
    let idx = d
        .shadow_compare_remote(0, &[f.clone(), f], "test.cap")
        .await;
    assert_eq!(idx, 0, "without remote socket, falls back to local");
}

#[tokio::test]
async fn shadow_compare_remote_falls_back_on_unreachable_socket() {
    let d = PerceptronDispatcher::shadow_default()
        .with_remote_infer("/tmp/definitely-nonexistent-biomeos-test.sock".to_string());
    let f = DispatchFeatures::build(0, None, 0.0);
    let idx = d
        .shadow_compare_remote(0, &[f.clone(), f], "test.cap")
        .await;
    assert_eq!(idx, 0, "unreachable socket gracefully falls back to local");
}
