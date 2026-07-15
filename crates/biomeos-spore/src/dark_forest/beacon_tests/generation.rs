// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{make_beacon, MockDarkForestCaller};

#[tokio::test]
async fn test_generate_encrypted_beacon_success() {
    let mock = MockDarkForestCaller::new();
    mock.setup_generate_success("broadcast_key_b64", "blake3_hash_32_chars_minimum!!")
        .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let result = beacon_mgr
        .generate_encrypted_beacon("/tmp/sock", &["compute", "storage"], Some("genesis"))
        .await
        .expect("beacon generation should succeed");

    assert_eq!(result.version, 1);
    assert_eq!(result.ciphertext, "encrypted_payload_b64");
    assert_eq!(result.nonce, "nonce12bytes==");
    assert_eq!(result.tag, "auth_tag_16bytes==");
}

#[tokio::test]
async fn test_generate_encrypted_beacon_missing_derive_key_fails() {
    let mock = MockDarkForestCaller::new();
    // No derive_lineage_key response - will fail
    mock.set_response("crypto.blake3_hash", serde_json::json!({ "hash": "h" }))
        .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let result = beacon_mgr
        .generate_encrypted_beacon("/tmp/sock", &[], None)
        .await;

    assert!(result.is_err());
}
