// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use crate::dark_forest::{BeaconPlaintext, EncryptedBeacon};
use super::common::{make_beacon, MockDarkForestCaller};

#[tokio::test]
async fn test_try_decrypt_beacon_success() {
    let plaintext = BeaconPlaintext {
        family_hash: "famhash1_234_567_890".to_string(),
        node_id: "peer_tower".to_string(),
        timestamp: 1_700_000_000,
        socket_path: "/run/peer/beardog.sock".to_string(),
        capabilities_hash: "capshash12345678".to_string(),
        lineage_mode: Some("sibling".to_string()),
    };
    let json = serde_json::to_string(&plaintext).expect("BeaconPlaintext serialization");
    let plaintext_b64 = BASE64.encode(json.as_bytes());

    let mock = MockDarkForestCaller::new();
    mock.setup_decrypt_success(&plaintext_b64).await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let encrypted = EncryptedBeacon {
        ciphertext: "ct".to_string(),
        nonce: "n".to_string(),
        tag: "t".to_string(),
        version: 1,
    };

    let result = beacon_mgr
        .try_decrypt_beacon(&encrypted)
        .await
        .expect("decrypt should succeed");

    assert!(result.is_some());
    let decrypted = result.expect("decrypted beacon");
    assert_eq!(decrypted.node_id, "peer_tower");
    assert_eq!(decrypted.socket_path, "/run/peer/beardog.sock");
    assert_eq!(decrypted.timestamp, 1_700_000_000);
    assert_eq!(decrypted.lineage_mode, Some("sibling".to_string()));
}

#[tokio::test]
async fn test_try_decrypt_beacon_not_family_returns_err() {
    let mock = MockDarkForestCaller::new();
    mock.set_response(
        "genetic.derive_lineage_key",
        serde_json::json!({ "key": "our_key" }),
    )
    .await;
    // No decrypt response - capability call returns Err (simulates BearDog auth failure)
    // When decrypt fails, AtomicClient returns Err, so we get Err from try_decrypt_beacon

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let encrypted = EncryptedBeacon {
        ciphertext: "attacker_ct".to_string(),
        nonce: "attacker_nonce".to_string(),
        tag: "attacker_tag".to_string(),
        version: 1,
    };

    let result = beacon_mgr.try_decrypt_beacon(&encrypted).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_try_decrypt_beacon_invalid_base64_plaintext_fails() {
    let mock = MockDarkForestCaller::new();
    mock.setup_decrypt_success("not-valid-base64!!!").await;

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
async fn test_try_decrypt_beacon_malformed_json_fails() {
    let plaintext_b64 = BASE64.encode(b"{ invalid json }");
    let mock = MockDarkForestCaller::new();
    mock.setup_decrypt_success(&plaintext_b64).await;

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
