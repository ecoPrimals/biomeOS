// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use super::*;
use crate::beacon_genetics::CapabilityCaller;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock capability caller for Dark Forest beacon tests.
/// Returns preset responses keyed by BearDog method name.
struct MockDarkForestCaller {
    responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MockDarkForestCaller {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn set_response(&self, method: &str, result: serde_json::Value) {
        self.responses
            .lock()
            .await
            .insert(method.to_string(), result);
    }

    /// Configure mock for successful beacon generation flow
    async fn setup_generate_success(&self, broadcast_key: &str, hash: &str) {
        self.set_response(
            "genetic.derive_lineage_key",
            serde_json::json!({ "key": broadcast_key }),
        )
        .await;
        self.set_response("crypto.blake3_hash", serde_json::json!({ "hash": hash }))
            .await;
        self.set_response(
            "crypto.chacha20_poly1305_encrypt",
            serde_json::json!({
                "ciphertext": "encrypted_payload_b64",
                "nonce": "nonce12bytes==",
                "tag": "auth_tag_16bytes=="
            }),
        )
        .await;
    }

    /// Configure mock for successful beacon decryption
    async fn setup_decrypt_success(&self, plaintext_b64: &str) {
        self.set_response(
            "genetic.derive_lineage_key",
            serde_json::json!({ "key": "same_broadcast_key" }),
        )
        .await;
        self.set_response(
            "crypto.chacha20_poly1305_decrypt",
            serde_json::json!({ "plaintext": plaintext_b64 }),
        )
        .await;
    }

    /// Configure mock for lineage verification
    async fn setup_verify_lineage(&self, valid: bool) {
        self.set_response(
            "genetic.verify_lineage",
            serde_json::json!({ "valid": valid }),
        )
        .await;
    }

    /// Configure mock for lineage proof generation
    async fn setup_generate_lineage_proof(&self, proof: &str) {
        self.set_response(
            "genetic.generate_lineage_proof",
            serde_json::json!({ "proof": proof }),
        )
        .await;
    }

    /// Configure mock for pure noise beacon (derive + encrypt/decrypt)
    async fn setup_pure_noise_success(
        &self,
        beacon_key: &str,
        encrypt_result: (String, String, String),
        decrypt_plaintext: Option<&str>,
    ) {
        self.set_response(
            "genetic.derive_lineage_beacon_key",
            serde_json::json!({ "beacon_key": beacon_key }),
        )
        .await;
        self.set_response(
            "crypto.chacha20_poly1305_encrypt",
            serde_json::json!({
                "ciphertext": encrypt_result.0,
                "nonce": encrypt_result.1,
                "tag": encrypt_result.2
            }),
        )
        .await;
        if let Some(pt) = decrypt_plaintext {
            self.set_response(
                "crypto.chacha20_poly1305_decrypt",
                serde_json::json!({ "plaintext": pt }),
            )
            .await;
        }
    }
}

#[async_trait::async_trait]
impl CapabilityCaller for MockDarkForestCaller {
    async fn call(
        &self,
        capability: &str,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let responses = self.responses.lock().await;
        responses
            .get(capability)
            .cloned()
            .ok_or_else(|| format!("No mock response for {capability}"))
    }
}

fn make_beacon(
    caller: MockDarkForestCaller,
    family_seed_b64: &str,
    node_id: &str,
) -> DarkForestBeacon {
    DarkForestBeacon {
        capability_caller: Arc::new(caller),
        family_seed_b64: family_seed_b64.to_string(),
        node_id: node_id.to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. Beacon generation
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// 2. Beacon decryption logic
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_try_decrypt_beacon_success() {
    let plaintext = BeaconPlaintext {
        family_hash: "famhash1234567890".to_string(),
        node_id: "peer_tower".to_string(),
        timestamp: 1700000000,
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
    assert_eq!(decrypted.timestamp, 1700000000);
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

// ═══════════════════════════════════════════════════════════════════
// 3. Lineage verification
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// 4. Serde serialization roundtrips for beacon types
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_beacon_plaintext_serde_roundtrip() {
    let beacon = BeaconPlaintext {
        family_hash: "abc123def4567890".to_string(),
        node_id: "tower1".to_string(),
        timestamp: 1234567890,
        socket_path: "/tmp/beardog.sock".to_string(),
        capabilities_hash: "cap_hash_def4567890".to_string(),
        lineage_mode: Some("genesis".to_string()),
    };

    let json = serde_json::to_string(&beacon).expect("BeaconPlaintext serialization");
    let parsed: BeaconPlaintext =
        serde_json::from_str(&json).expect("BeaconPlaintext deserialization");

    assert_eq!(parsed.family_hash, beacon.family_hash);
    assert_eq!(parsed.node_id, beacon.node_id);
    assert_eq!(parsed.timestamp, beacon.timestamp);
    assert_eq!(parsed.socket_path, beacon.socket_path);
    assert_eq!(parsed.capabilities_hash, beacon.capabilities_hash);
    assert_eq!(parsed.lineage_mode, beacon.lineage_mode);
}

#[test]
fn test_beacon_plaintext_serde_roundtrip_no_lineage() {
    let beacon = BeaconPlaintext {
        family_hash: "fam".to_string(),
        node_id: "n1".to_string(),
        timestamp: 100,
        socket_path: "/s".to_string(),
        capabilities_hash: "c".to_string(),
        lineage_mode: None,
    };

    let json = serde_json::to_string(&beacon).expect("BeaconPlaintext serialization");
    assert!(!json.contains("lineage_mode"));
    let parsed: BeaconPlaintext =
        serde_json::from_str(&json).expect("BeaconPlaintext deserialization");
    assert!(parsed.lineage_mode.is_none());
}

#[test]
fn test_encrypted_beacon_serde_roundtrip() {
    let beacon = EncryptedBeacon {
        ciphertext: "base64ciphertext==".to_string(),
        nonce: "base64nonce==".to_string(),
        tag: "base64tag==".to_string(),
        version: 1,
    };

    let json = serde_json::to_string(&beacon).expect("EncryptedBeacon serialization");
    let parsed: EncryptedBeacon =
        serde_json::from_str(&json).expect("EncryptedBeacon deserialization");

    assert_eq!(parsed.ciphertext, beacon.ciphertext);
    assert_eq!(parsed.nonce, beacon.nonce);
    assert_eq!(parsed.tag, beacon.tag);
    assert_eq!(parsed.version, beacon.version);
}

#[test]
fn test_beacon_plaintext_json_to_base64_roundtrip() {
    let beacon = BeaconPlaintext {
        family_hash: "fam123".to_string(),
        node_id: "tower1".to_string(),
        timestamp: 1700000000,
        socket_path: "/run/user/1000/biomeos/beardog.sock".to_string(),
        capabilities_hash: "cap456".to_string(),
        lineage_mode: Some("genesis".to_string()),
    };

    let json = serde_json::to_string(&beacon).expect("BeaconPlaintext serialization");
    let b64 = BASE64.encode(json.as_bytes());
    let decoded_bytes = BASE64.decode(&b64).expect("base64 decode");
    let decoded: BeaconPlaintext =
        serde_json::from_slice(&decoded_bytes).expect("BeaconPlaintext deserialization");

    assert_eq!(decoded.family_hash, beacon.family_hash);
    assert_eq!(decoded.node_id, beacon.node_id);
    assert_eq!(decoded.timestamp, beacon.timestamp);
}

// ═══════════════════════════════════════════════════════════════════
// 5. Error cases (invalid beacons, wrong keys, malformed data)
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// 6. Pure noise beacon - size checks and decryption
// ═══════════════════════════════════════════════════════════════════

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
        "timestamp": 1700000000,
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
        decrypted.get("timestamp").and_then(|v| v.as_u64()),
        Some(1700000000)
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
async fn test_dark_forest_beacon_clone() {
    let mock = MockDarkForestCaller::new();
    let beacon = make_beacon(mock, "c2VlZA==", "tower1");
    let cloned = beacon.clone();
    assert_eq!(cloned.node_id, "tower1");
    assert_eq!(cloned.family_seed_b64, "c2VlZA==");
}
