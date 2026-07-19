// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{MockDarkForestCaller, make_beacon};
use crate::dark_forest::EncryptedBeacon;

#[tokio::test]
async fn test_try_decrypt_beacon_capability_call_fails() {
    let mock = MockDarkForestCaller::new();
    // No responses - capability call will fail with "No mock response"

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let encrypted = EncryptedBeacon {
        ciphertext: "ct".to_string(),
        nonce: "n".to_string(),
        tag: "t".to_string(),
        version: 1,
    };

    let result = beacon_mgr.try_decrypt_beacon(&encrypted).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_encrypted_beacon_missing_encrypt_result_fails() {
    let mock = MockDarkForestCaller::new();
    mock.setup_generate_success("key", "hash_16_chars_min!!")
        .await;
    mock.set_response(
        "crypto.chacha20_poly1305_encrypt",
        serde_json::json!({}), // Missing ciphertext, nonce, tag
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let result = beacon_mgr
        .generate_encrypted_beacon("/tmp/sock", &[], None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_lineage_missing_result_returns_false() {
    let mock = MockDarkForestCaller::new();
    mock.set_response("genetic.verify_lineage", serde_json::json!({}))
        .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let valid = beacon_mgr
        .verify_peer_lineage("peer", "proof")
        .await
        .expect("verify lineage");

    assert!(!valid);
}

#[tokio::test]
async fn test_generate_lineage_proof_missing_proof_fails() {
    let mock = MockDarkForestCaller::new();
    mock.set_response("genetic.generate_lineage_proof", serde_json::json!({}))
        .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let result = beacon_mgr.generate_lineage_proof("peer").await;

    assert!(result.is_err());
}
