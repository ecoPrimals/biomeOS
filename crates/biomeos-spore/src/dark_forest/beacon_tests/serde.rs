// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use crate::dark_forest::{BeaconPlaintext, EncryptedBeacon};

#[test]
fn test_beacon_plaintext_serde_roundtrip() {
    let beacon = BeaconPlaintext {
        family_hash: "abc123def4567890".to_string(),
        node_id: "tower1".to_string(),
        timestamp: 1_234_567_890,
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
        timestamp: 1_700_000_000,
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
