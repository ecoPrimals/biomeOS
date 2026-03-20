// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Beacon Genetics Manager tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::super::capability::CapabilityCaller;
use super::super::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct MockCapabilityCaller {
    responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MockCapabilityCaller {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn set_response(&self, capability: &str, response: serde_json::Value) {
        self.responses
            .lock()
            .await
            .insert(capability.to_string(), response);
    }
}

#[async_trait::async_trait]
impl CapabilityCaller for MockCapabilityCaller {
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

#[tokio::test]
async fn test_sync_with_lineage_peer_same_lineage() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = Box::new(MockCapabilityCaller::new());

    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

    let mut local_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "same_lineage");
    local_manifest.add_meeting(
        BeaconId::from_hex("peer_a"),
        MeetingRecord {
            node_name: "peer-a".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.168.1.1:9900".to_string()],
            capabilities_hint: vec![],
            notes: "Local meeting".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_a.seed".to_string(),
        },
    );
    manager.set_manifest(local_manifest);

    let mut remote_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "same_lineage");
    remote_manifest.add_meeting(
        BeaconId::from_hex("peer_b"),
        MeetingRecord {
            node_name: "peer-b".to_string(),
            first_met: 2000,
            last_seen: 2000,
            endpoints: vec!["192.168.1.2:9900".to_string()],
            capabilities_hint: vec!["compute".to_string()],
            notes: "Remote meeting".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_b.seed".to_string(),
        },
    );

    let result = manager
        .sync_with_lineage_peer(&remote_manifest)
        .await
        .unwrap();

    assert_eq!(result.added, 1);
    assert_eq!(result.updated, 0);
    let manifest = manager.manifest.as_ref().unwrap();
    assert_eq!(manifest.meetings.len(), 2);
}

#[tokio::test]
async fn test_sync_with_different_lineage_fails() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = Box::new(MockCapabilityCaller::new());
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("local123"),
        "lineage_a",
    ));
    let remote_manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "lineage_b");
    let result = manager.sync_with_lineage_peer(&remote_manifest).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("different lineage")
    );
}

#[test]
fn test_list_meetings_empty() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    assert!(manager.list_meetings().is_empty());
}

#[test]
fn test_our_beacon_id() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    assert!(manager.our_beacon_id().is_none());
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our_beacon_123"),
        "lineage",
    ));
    let id = manager.our_beacon_id().expect("should have ID");
    assert_eq!(id.0, "our_beacon_123");
}

#[tokio::test]
async fn test_initialize_loads_existing_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("existing-beacon"), "lineage-hint");
    manifest
        .save(&temp_dir.path().join(".beacon.genetics.json"))
        .expect("save");
    let mut manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    manager.initialize().await.expect("init should succeed");
    let id = manager.our_beacon_id().expect("should have loaded ID");
    assert_eq!(id.0, "existing-beacon");
}

#[tokio::test]
async fn test_initialize_generates_new_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = Box::new(MockCapabilityCaller::new());
    mock_caller
        .set_response(
            "beacon.generate",
            serde_json::json!({
                "beacon_id": "new-beacon-456",
                "seed_hex": "deadbeefcafebabe"
            }),
        )
        .await;
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.initialize().await.expect("init should succeed");
    let id = manager.our_beacon_id().expect("should have new ID");
    assert_eq!(id.0, "new-beacon-456");
    assert!(temp_dir.path().join(".beacon.genetics.json").exists());
    assert!(temp_dir.path().join(".beacon.seed").exists());
}

#[tokio::test]
async fn test_initialize_generate_fails() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    let result = manager.initialize().await;
    assert!(result.is_err());
}

#[test]
fn test_get_lineage_hint_with_family_seed() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let seed_data = b"abcdefghijklmnop";
    std::fs::write(temp_dir.path().join(".family.seed"), seed_data).expect("write seed");
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    let hint = manager.get_lineage_hint().expect("should succeed");
    assert_eq!(hint, hex::encode(&seed_data[0..8]));
}

#[test]
fn test_get_lineage_hint_no_seed_file() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    assert!(manager.get_lineage_hint().is_err());
}

#[test]
fn test_save_manifest_no_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    manager.save_manifest().expect("no-op save should succeed");
    assert!(!temp_dir.path().join(".beacon.genetics.json").exists());
}

#[test]
fn test_new_manager() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager = BeaconGeneticsManager::new(temp_dir.path());
    assert!(manager.our_beacon_id().is_none());
    assert!(manager.list_meetings().is_empty());
}

#[test]
fn test_with_capability_caller() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock = Box::new(MockCapabilityCaller::new());
    let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock);
    assert!(manager.our_beacon_id().is_none());
    assert_eq!(manager.root_path, temp_dir.path());
}

#[test]
fn test_list_meetings_with_data() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("our123"), "lineage");
    manifest.add_meeting(
        BeaconId::from_hex("peer1"),
        MeetingRecord {
            node_name: "peer-1".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.168.1.1:9900".to_string()],
            capabilities_hint: vec![],
            notes: "Test".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer1.seed".to_string(),
        },
    );
    manager.set_manifest(manifest);

    let meetings = manager.list_meetings();
    assert_eq!(meetings.len(), 1);
    assert_eq!(meetings[0].0.0, "peer1");
    assert_eq!(meetings[0].1.node_name, "peer-1");
}

#[test]
fn test_get_lineage_hint_short_seed() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let seed_data = b"short";
    std::fs::write(temp_dir.path().join(".family.seed"), seed_data).expect("write seed");
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    let result = manager.get_lineage_hint();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_try_decrypt_with_met_seeds_not_initialized() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );

    let result = manager.try_decrypt_with_met_seeds(b"encrypted_data").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not initialized"));
}

#[tokio::test]
async fn test_try_decrypt_with_met_seeds_no_match() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = Box::new(MockCapabilityCaller::new());
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

#[test]
fn test_save_manifest_with_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager = BeaconGeneticsManager::with_capability_caller(
        temp_dir.path(),
        Box::new(MockCapabilityCaller::new()),
    );
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("save-test"),
        "lineage",
    ));

    manager.save_manifest().expect("save should succeed");
    assert!(temp_dir.path().join(".beacon.genetics.json").exists());
}

#[tokio::test]
async fn test_initiate_meeting_full_flow() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = Box::new(MockCapabilityCaller::new());
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
        .initiate_meeting("192.168.1.10:9900", "peer-node")
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
    let mock_caller = Box::new(MockCapabilityCaller::new());
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our"),
        "lineage",
    ));

    let result = manager.initiate_meeting("192.168.1.10:9900", "peer").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("beacon.get_id"));
}

#[tokio::test]
async fn test_sync_with_lineage_peer_updates_existing() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = Box::new(MockCapabilityCaller::new());
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

    let mut local_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "same_lineage");
    local_manifest.add_meeting(
        BeaconId::from_hex("peer_a"),
        MeetingRecord {
            node_name: "peer-a".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.168.1.1:9900".to_string()],
            capabilities_hint: vec![],
            notes: "Local".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_a.seed".to_string(),
        },
    );
    manager.set_manifest(local_manifest);

    let mut remote_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "same_lineage");
    remote_manifest.add_meeting(
        BeaconId::from_hex("peer_a"),
        MeetingRecord {
            node_name: "peer-a".to_string(),
            first_met: 1000,
            last_seen: 2000,
            endpoints: vec![
                "192.168.1.1:9900".to_string(),
                "192.168.1.2:9900".to_string(),
            ],
            capabilities_hint: vec![],
            notes: "Remote".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_a.seed".to_string(),
        },
    );

    let result = manager
        .sync_with_lineage_peer(&remote_manifest)
        .await
        .expect("sync");

    assert_eq!(result.added, 0);
    assert_eq!(result.updated, 1);
    let manifest = manager.manifest.as_ref().expect("manifest");
    let meeting = manifest
        .get_meeting(&BeaconId::from_hex("peer_a"))
        .expect("meeting");
    assert_eq!(meeting.last_seen, 2000);
    assert_eq!(meeting.endpoints.len(), 2);
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

    let mock_caller = Box::new(MockCapabilityCaller::new());
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
