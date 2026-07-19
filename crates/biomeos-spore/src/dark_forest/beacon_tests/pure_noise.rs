// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use super::common::{MockDarkForestCaller, make_beacon};

#[tokio::test]
async fn test_try_decrypt_pure_noise_beacon_too_small_returns_none() {
    let mock = MockDarkForestCaller::new();
    mock.set_response(
        "genetic.derive_lineage_beacon_key",
        serde_json::json!({ "beacon_key": "key" }),
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let too_small: [u8; 27] = [0u8; 27];
    let result = beacon_mgr
        .try_decrypt_pure_noise_beacon(&too_small)
        .await
        .expect("try_decrypt_pure_noise_beacon");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_try_decrypt_pure_noise_beacon_ciphertext_too_short_returns_none() {
    let mock = MockDarkForestCaller::new();
    mock.set_response(
        "genetic.derive_lineage_beacon_key",
        serde_json::json!({ "beacon_key": "key" }),
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    // 28 bytes total: 12 nonce + 0 ciphertext + 16 tag would need 28, but
    // ciphertext_and_tag = 16 means we have 0 ciphertext bytes (invalid)
    let bytes: [u8; 28] = [0u8; 28];
    let result = beacon_mgr
        .try_decrypt_pure_noise_beacon(&bytes)
        .await
        .expect("try_decrypt_pure_noise_beacon");

    // ciphertext_and_tag.len() = 16, so ciphertext = 0 bytes, tag = 16
    // The decrypt will be called with empty ciphertext - mock has no response
    // for chacha20_poly1305_decrypt, so it returns Err -> Ok(None)
    assert!(result.is_none());
}

#[tokio::test]
async fn test_try_decrypt_pure_noise_beacon_success() {
    let inner = serde_json::json!({
        "node_id": "tower2",
        "timestamp": 1_700_000_000,
        "socket_path": "/tmp/peer.sock",
        "capabilities": ["compute"],
        "lineage_mode": "genesis"
    });
    let plaintext_b64 = BASE64.encode(
        serde_json::to_string(&inner)
            .expect("inner JSON serialization")
            .as_bytes(),
    );

    let mock = MockDarkForestCaller::new();
    mock.set_response(
        "genetic.derive_lineage_beacon_key",
        serde_json::json!({ "beacon_key": "beacon_key_b64" }),
    )
    .await;
    mock.set_response(
        "crypto.chacha20_poly1305_decrypt",
        serde_json::json!({ "plaintext": plaintext_b64 }),
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    // Build valid structure: 12 nonce + N ciphertext + 16 tag (min 28)
    let mut bytes = vec![0u8; 12];
    bytes.extend_from_slice(&[0u8; 16]); // ciphertext (min 0 for 16 tag)
    bytes.extend_from_slice(&[0u8; 16]); // tag

    let result = beacon_mgr
        .try_decrypt_pure_noise_beacon(&bytes)
        .await
        .expect("try_decrypt_pure_noise_beacon");

    assert!(result.is_some());
    let decrypted = result.expect("decrypted pure noise beacon");
    assert_eq!(
        decrypted.get("node_id").and_then(|v| v.as_str()),
        Some("tower2")
    );
    assert_eq!(
        decrypted
            .get("timestamp")
            .and_then(serde_json::Value::as_u64),
        Some(1_700_000_000)
    );
}

#[tokio::test]
async fn test_generate_pure_noise_beacon_success() {
    let nonce_b64 = BASE64.encode([0u8; 12]);
    let cipher_b64 = BASE64.encode([1u8; 32]);
    let tag_b64 = BASE64.encode([2u8; 16]);

    let mock = MockDarkForestCaller::new();
    mock.setup_pure_noise_success(
        "beacon_key",
        (cipher_b64.clone(), nonce_b64.clone(), tag_b64.clone()),
        None,
    )
    .await;

    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");

    let result = beacon_mgr
        .generate_pure_noise_beacon("/tmp/sock", &["compute"], Some("genesis"))
        .await
        .expect("generate pure noise beacon");

    assert_eq!(result.len(), 12 + 32 + 16);
    assert_eq!(&result[0..12], &[0u8; 12]);
    assert_eq!(&result[12..44], &[1u8; 32]);
    assert_eq!(&result[44..60], &[2u8; 16]);
}

#[tokio::test]
async fn test_generate_pure_noise_beacon_missing_beacon_key_fails() {
    let mock = MockDarkForestCaller::new();
    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");
    let result = beacon_mgr
        .generate_pure_noise_beacon("/tmp/sock", &["compute"], None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_try_decrypt_pure_noise_beacon_derive_key_fails_returns_none() {
    let mock = MockDarkForestCaller::new();
    let beacon_mgr = make_beacon(mock, "dGVzdHNlZWQ=", "tower1");
    let bytes = vec![1u8; 40];
    let result = beacon_mgr
        .try_decrypt_pure_noise_beacon(&bytes)
        .await
        .expect("ok wrapper");
    assert!(result.is_none());
}
