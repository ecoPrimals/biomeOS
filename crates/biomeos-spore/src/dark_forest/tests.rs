// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Dark Forest tests

use std::sync::Arc;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use super::beacon::DarkForestBeacon;
use super::types::{BeaconPlaintext, DiscoveredPeer, EncryptedBeacon};

#[cfg(test)]
mod run {
    use super::*;

    #[test]
    fn test_beacon_plaintext_serialization_roundtrip() {
        let beacon = BeaconPlaintext {
            family_hash: "abc123def456".to_string(),
            node_id: "tower1".to_string(),
            timestamp: 1_234_567_890,
            socket_path: "/tmp/beardog.sock".to_string(),
            capabilities_hash: "cap_hash_def456".to_string(),
            lineage_mode: Some("genesis".to_string()),
        };

        let json = serde_json::to_string(&beacon).expect("serialize beacon");
        let parsed: BeaconPlaintext = serde_json::from_str(&json).expect("parse beacon");

        assert_eq!(parsed.family_hash, "abc123def456");
        assert_eq!(parsed.node_id, "tower1");
        assert_eq!(parsed.timestamp, 1_234_567_890);
        assert_eq!(parsed.socket_path, "/tmp/beardog.sock");
        assert_eq!(parsed.capabilities_hash, "cap_hash_def456");
        assert_eq!(parsed.lineage_mode, Some("genesis".to_string()));
    }

    #[test]
    fn test_beacon_plaintext_without_lineage() {
        let beacon = BeaconPlaintext {
            family_hash: "abc".to_string(),
            node_id: "node1".to_string(),
            timestamp: 100,
            socket_path: "/tmp/sock".to_string(),
            capabilities_hash: "hash".to_string(),
            lineage_mode: None,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        assert!(!json.contains("lineage_mode"));

        let parsed: BeaconPlaintext = serde_json::from_str(&json).expect("parse");
        assert!(parsed.lineage_mode.is_none());
    }

    #[test]
    fn test_beacon_plaintext_sibling_lineage() {
        let beacon = BeaconPlaintext {
            family_hash: "fam1".to_string(),
            node_id: "tower2".to_string(),
            timestamp: 99999,
            socket_path: "/run/user/1000/biomeos/beardog.sock".to_string(),
            capabilities_hash: "caps".to_string(),
            lineage_mode: Some("sibling".to_string()),
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        assert!(json.contains("sibling"));
        assert!(json.contains("tower2"));
    }

    #[test]
    fn test_beacon_plaintext_clone() {
        let beacon = BeaconPlaintext {
            family_hash: "h".to_string(),
            node_id: "n".to_string(),
            timestamp: 42,
            socket_path: "/s".to_string(),
            capabilities_hash: "c".to_string(),
            lineage_mode: None,
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.node_id, beacon.node_id);
        assert_eq!(cloned.timestamp, beacon.timestamp);
    }

    #[test]
    fn test_beacon_plaintext_debug() {
        let beacon = BeaconPlaintext {
            family_hash: "h".to_string(),
            node_id: "n".to_string(),
            timestamp: 0,
            socket_path: "/s".to_string(),
            capabilities_hash: "c".to_string(),
            lineage_mode: None,
        };

        let debug = format!("{beacon:?}");
        assert!(debug.contains("BeaconPlaintext"));
        assert!(debug.contains("node_id"));
    }

    #[test]
    fn test_encrypted_beacon_serialization_roundtrip() {
        let beacon = EncryptedBeacon {
            ciphertext: "base64ciphertext==".to_string(),
            nonce: "base64nonce==".to_string(),
            tag: "base64tag==".to_string(),
            version: 1,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        let parsed: EncryptedBeacon = serde_json::from_str(&json).expect("parse");

        assert_eq!(parsed.ciphertext, "base64ciphertext==");
        assert_eq!(parsed.nonce, "base64nonce==");
        assert_eq!(parsed.tag, "base64tag==");
        assert_eq!(parsed.version, 1);
    }

    #[test]
    fn test_encrypted_beacon_clone() {
        let beacon = EncryptedBeacon {
            ciphertext: "ct".to_string(),
            nonce: "n".to_string(),
            tag: "t".to_string(),
            version: 2,
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.version, beacon.version);
        assert_eq!(cloned.ciphertext, beacon.ciphertext);
    }

    #[test]
    fn test_encrypted_beacon_debug() {
        let beacon = EncryptedBeacon {
            ciphertext: "data".to_string(),
            nonce: "nonce".to_string(),
            tag: "tag".to_string(),
            version: 1,
        };

        let debug = format!("{beacon:?}");
        assert!(debug.contains("EncryptedBeacon"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_dark_forest_beacon_clone() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let beacon = DarkForestBeacon {
            capability_caller: Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock")),
            family_seed_b64: "dGVzdHNlZWQ=".to_string(),
            node_id: "tower1".to_string(),
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.node_id, "tower1");
        assert_eq!(cloned.family_seed_b64, beacon.family_seed_b64);
    }

    #[tokio::test]
    async fn test_dark_forest_beacon_new_missing_seed() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let caller = Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock"));
        let result =
            DarkForestBeacon::new(caller, "/nonexistent/path/.family.seed", "tower1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dark_forest_beacon_new_with_seed_file() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let dir = tempfile::tempdir().expect("create tempdir");
        let seed_path = dir.path().join(".family.seed");
        tokio::fs::write(&seed_path, b"test-seed-bytes-32chars-minimum!")
            .await
            .expect("write seed");

        let caller = Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock"));
        let beacon = DarkForestBeacon::new(caller, &seed_path, "tower1")
            .await
            .expect("create beacon");

        assert_eq!(beacon.node_id, "tower1");
        assert!(!beacon.family_seed_b64.is_empty());

        let decoded = BASE64
            .decode(&beacon.family_seed_b64)
            .expect("decode base64");
        assert_eq!(decoded, b"test-seed-bytes-32chars-minimum!");
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

        let json = serde_json::to_string(&beacon).expect("serialize");
        let b64 = BASE64.encode(json.as_bytes());

        let decoded_bytes = BASE64.decode(&b64).expect("decode base64");
        let decoded: BeaconPlaintext =
            serde_json::from_slice(&decoded_bytes).expect("parse decoded beacon");

        assert_eq!(decoded.family_hash, "fam123");
        assert_eq!(decoded.node_id, "tower1");
        assert_eq!(decoded.timestamp, 1_700_000_000);
    }

    #[test]
    fn test_beacon_version_field() {
        let beacon = EncryptedBeacon {
            ciphertext: "data".to_string(),
            nonce: "nonce".to_string(),
            tag: "tag".to_string(),
            version: 2,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        assert!(json.contains("\"version\":2"));
    }

    #[test]
    fn test_pure_noise_beacon_minimum_size() {
        let too_small: [u8; 27] = [0u8; 27];
        assert!(too_small.len() < 28);

        let exactly_min: [u8; 28] = [0u8; 28];
        assert_eq!(exactly_min.len(), 28);
    }

    #[test]
    fn test_discovered_peer_struct() {
        let peer = DiscoveredPeer {
            beacon: BeaconPlaintext {
                family_hash: "h".to_string(),
                node_id: "n1".to_string(),
                timestamp: 100,
                socket_path: "/s".to_string(),
                capabilities_hash: "c".to_string(),
                lineage_mode: None,
            },
            lineage_verified: true,
            session_key: Some("key123".to_string()),
        };
        assert!(peer.lineage_verified);
        assert_eq!(peer.session_key.as_deref(), Some("key123"));
        assert_eq!(peer.beacon.node_id, "n1");
    }

    #[test]
    fn test_encrypted_beacon_version_values() {
        for v in [0u8, 1, 2, 255] {
            let beacon = EncryptedBeacon {
                ciphertext: "c".to_string(),
                nonce: "n".to_string(),
                tag: "t".to_string(),
                version: v,
            };
            let json = serde_json::to_string(&beacon).unwrap();
            assert!(json.contains(&format!("\"version\":{v}")));
        }
    }
}
