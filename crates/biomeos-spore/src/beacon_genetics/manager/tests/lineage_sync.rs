// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockCapabilityCaller;
use crate::beacon_genetics::manager::BeaconGeneticsManager;
use crate::beacon_genetics::{
    BeaconGeneticsManifest, BeaconId, MeetingRecord, MeetingRelationship, MeetingVisibility,
};

#[tokio::test]
async fn test_sync_with_lineage_peer_same_lineage() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();

    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

    let mut local_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "same_lineage");
    local_manifest.add_meeting(
        BeaconId::from_hex("peer_a"),
        MeetingRecord {
            node_name: "peer-a".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.0.2.1:9900".to_string()],
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
            endpoints: vec!["192.0.2.2:9900".to_string()],
            capabilities_hint: vec!["compute".to_string()],
            notes: "Remote meeting".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_b.seed".to_string(),
        },
    );

    let result = manager.sync_with_lineage_peer(&remote_manifest).unwrap();

    assert_eq!(result.added, 1);
    assert_eq!(result.updated, 0);
    let manifest = manager.manifest.as_ref().unwrap();
    assert_eq!(manifest.meetings.len(), 2);
}

#[tokio::test]
async fn test_sync_with_different_lineage_fails() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("local123"),
        "lineage_a",
    ));
    let remote_manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "lineage_b");
    let result = manager.sync_with_lineage_peer(&remote_manifest);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("different lineage")
    );
}

#[tokio::test]
async fn test_sync_with_lineage_peer_updates_existing() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = MockCapabilityCaller::new();
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

    let mut local_manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "same_lineage");
    local_manifest.add_meeting(
        BeaconId::from_hex("peer_a"),
        MeetingRecord {
            node_name: "peer-a".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.0.2.1:9900".to_string()],
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
            endpoints: vec!["192.0.2.1:9900".to_string(), "192.0.2.2:9900".to_string()],
            capabilities_hint: vec![],
            notes: "Remote".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer_a.seed".to_string(),
        },
    );

    let result = manager
        .sync_with_lineage_peer(&remote_manifest)
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
