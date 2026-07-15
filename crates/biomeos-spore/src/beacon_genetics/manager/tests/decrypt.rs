// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockCapabilityCaller;
use crate::beacon_genetics::manager::BeaconGeneticsManager;
use crate::beacon_genetics::{
    BeaconGeneticsManifest, BeaconId, MeetingRecord, MeetingRelationship, MeetingVisibility,
};

#[tokio::test]
async fn test_try_decrypt_with_met_seeds_not_initialized() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());

    let result = manager.try_decrypt_with_met_seeds(b"encrypted_data").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not initialized"));
}

#[tokio::test]
async fn test_try_decrypt_with_met_seeds_no_match() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = MockCapabilityCaller::new();
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("our123"), "lineage");
    manifest.add_meeting(
        BeaconId::from_hex("peer1"),
        MeetingRecord {
            node_name: "peer-1".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec![],
            capabilities_hint: vec![],
            notes: String::new(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer1.seed".to_string(),
        },
    );
    manager.set_manifest(manifest);

    let result = manager
        .try_decrypt_with_met_seeds(b"invalid_encrypted_data")
        .await
        .expect("should not error");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_try_decrypt_with_met_seeds_success() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    std::fs::create_dir_all(temp_dir.path().join(".beacon_seeds")).expect("create dir");
    let seed_content = "encrypted_seed_data";
    std::fs::write(
        temp_dir.path().join(".beacon_seeds").join("peer1234.seed"),
        seed_content,
    )
    .expect("write seed");

    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "crypto.decrypt_with_lineage",
            serde_json::json!({"plaintext": "decrypted_seed_hex"}),
        )
        .await;
    mock_caller
        .set_response(
            "beacon.try_decrypt",
            serde_json::json!({
                "decrypted": true,
                "payload": {"data": "decrypted_payload"}
            }),
        )
        .await;

    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("our"), "lineage");
    manifest.add_meeting(
        BeaconId::from_hex("peer12345678"),
        MeetingRecord {
            node_name: "peer".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec![],
            capabilities_hint: vec![],
            notes: String::new(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer1234.seed".to_string(),
        },
    );
    manager.set_manifest(manifest);

    let result = manager
        .try_decrypt_with_met_seeds(b"encrypted_beacon_data")
        .await
        .expect("decrypt");

    let (payload, beacon_id) = result.expect("should decrypt");
    assert_eq!(beacon_id.0, "peer12345678");
    assert_eq!(
        payload.get("data").and_then(|v| v.as_str()),
        Some("decrypted_payload")
    );
}
