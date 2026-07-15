// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockCapabilityCaller;
use crate::beacon_genetics::manager::BeaconGeneticsManager;
use crate::beacon_genetics::{BeaconGeneticsManifest, BeaconId};

#[tokio::test]
async fn test_initiate_meeting_full_flow() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "beacon.get_id",
            serde_json::json!({"beacon_id": "our-beacon-123"}),
        )
        .await;
    mock_caller
        .set_response(
            "beacon.get_seed",
            serde_json::json!({"seed_hex": "deadbeefcafebabe"}),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.encrypt",
            serde_json::json!({"ciphertext": "encrypted_our_seed"}),
        )
        .await;
    mock_caller
        .set_response(
            "network.beacon_exchange",
            serde_json::json!({
                "peer_beacon_id": "peer-beacon-456",
                "peer_encrypted_seed": "encrypted_peer_seed"
            }),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.decrypt",
            serde_json::json!({"plaintext": "peer_seed_hex"}),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.encrypt_with_lineage",
            serde_json::json!({"ciphertext": "encrypted_for_storage"}),
        )
        .await;

    std::fs::create_dir_all(temp_dir.path().join(".beacon_seeds")).expect("create seeds dir");
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our-beacon-123"),
        "lineage",
    ));

    let result = manager
        .initiate_meeting("192.0.2.10:9900", "peer-node")
        .await
        .expect("initiate meeting");

    assert_eq!(result.0, "peer-beacon-456");
    let meetings = manager.list_meetings();
    assert_eq!(meetings.len(), 1);
    assert_eq!(meetings[0].0.0, "peer-beacon-456");
    assert_eq!(meetings[0].1.node_name, "peer-node");
}

#[tokio::test]
async fn test_initiate_meeting_beacon_get_id_fails() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = MockCapabilityCaller::new();
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our"),
        "lineage",
    ));

    let result = manager.initiate_meeting("192.0.2.10:9900", "peer").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("beacon.get_id"));
}

#[tokio::test]
async fn test_initiate_meeting_short_peer_id_uses_full_id_for_seed_filename() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "beacon.get_id",
            serde_json::json!({ "beacon_id": "our-beacon-123" }),
        )
        .await;
    mock_caller
        .set_response(
            "beacon.get_seed",
            serde_json::json!({ "seed_hex": "deadbeefcafebabe" }),
        )
        .await;
    mock_caller
        .set_response("crypto.encrypt", serde_json::json!({ "ciphertext": "enc" }))
        .await;
    mock_caller
        .set_response(
            "network.beacon_exchange",
            serde_json::json!({
                "peer_beacon_id": "short",
                "peer_encrypted_seed": "peer_enc"
            }),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.decrypt",
            serde_json::json!({ "plaintext": "peer_seed_hex" }),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.encrypt_with_lineage",
            serde_json::json!({ "ciphertext": "stored" }),
        )
        .await;

    std::fs::create_dir_all(temp_dir.path().join(".beacon_seeds")).unwrap();
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our-beacon-123"),
        "lineage",
    ));

    manager
        .initiate_meeting("127.0.0.1:1", "peer")
        .await
        .expect("meeting");
    let rec = manager
        .list_meetings()
        .into_iter()
        .find(|(id, _)| id.0 == "short")
        .expect("record")
        .1;
    assert_eq!(rec.seed_file, "short.seed");
}

#[tokio::test]
async fn test_initiate_meeting_encrypt_with_lineage_fails() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "beacon.get_id",
            serde_json::json!({ "beacon_id": "our-beacon-123" }),
        )
        .await;
    mock_caller
        .set_response(
            "beacon.get_seed",
            serde_json::json!({ "seed_hex": "deadbeefcafebabe" }),
        )
        .await;
    mock_caller
        .set_response("crypto.encrypt", serde_json::json!({ "ciphertext": "enc" }))
        .await;
    mock_caller
        .set_response(
            "network.beacon_exchange",
            serde_json::json!({
                "peer_beacon_id": "peer-beacon-xyz",
                "peer_encrypted_seed": "peer_enc"
            }),
        )
        .await;
    mock_caller
        .set_response(
            "crypto.decrypt",
            serde_json::json!({ "plaintext": "peer_seed_hex" }),
        )
        .await;

    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our-beacon-123"),
        "lineage",
    ));

    let err = manager
        .initiate_meeting("127.0.0.1:1", "peer")
        .await
        .expect_err("encrypt storage");
    assert!(err.to_string().contains("encrypt_with_lineage"));
}
