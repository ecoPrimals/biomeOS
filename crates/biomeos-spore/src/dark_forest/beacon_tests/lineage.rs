// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{make_beacon, MockDarkForestCaller};

#[tokio::test]
async fn test_verify_peer_lineage_valid() {
    let mock = MockDarkForestCaller::new();
    mock.setup_verify_lineage(true).await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let valid = beacon_mgr
        .verify_peer_lineage("peer_family", "proof_abc123")
        .await
        .expect("verify lineage");

    assert!(valid);
}

#[tokio::test]
async fn test_verify_peer_lineage_invalid() {
    let mock = MockDarkForestCaller::new();
    mock.setup_verify_lineage(false).await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let valid = beacon_mgr
        .verify_peer_lineage("other_family", "bad_proof")
        .await
        .expect("verify lineage");

    assert!(!valid);
}

#[tokio::test]
async fn test_generate_lineage_proof_success() {
    let mock = MockDarkForestCaller::new();
    mock.setup_generate_lineage_proof("our_proof_xyz").await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let proof = beacon_mgr
        .generate_lineage_proof("peer_family")
        .await
        .expect("generate lineage proof");

    assert_eq!(proof, "our_proof_xyz");
}

#[tokio::test]
async fn test_derive_session_key_success() {
    let mock = MockDarkForestCaller::new();
    mock.set_response(
        "genetic.derive_lineage_key",
        serde_json::json!({ "key": "session_key_abc" }),
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let key = beacon_mgr
        .derive_session_key("peer_id", "birdsong-session-v1")
        .await
        .expect("derive session key");

    assert_eq!(key, "session_key_abc");
}

#[tokio::test]
async fn test_derive_session_key_missing_key_in_response_fails() {
    let mock = MockDarkForestCaller::new();
    mock.set_response("genetic.derive_lineage_key", serde_json::json!({}))
        .await;
    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");
    let result = beacon_mgr.derive_session_key("peer_id", "ctx-v1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dark_forest_beacon_clone() {
    let mock = MockDarkForestCaller::new();
    let beacon = make_beacon(mock, "c2VlZA==", "tower1");
    let cloned = beacon.clone();
    assert_eq!(cloned.node_id, beacon.node_id);
    assert_eq!(cloned.family_seed_b64, beacon.family_seed_b64);
}
