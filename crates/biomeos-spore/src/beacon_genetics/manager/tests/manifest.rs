// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockCapabilityCaller;
use crate::beacon_genetics::manager::BeaconGeneticsManager;
use crate::beacon_genetics::{
    BeaconGeneticsManifest, BeaconId, MeetingRecord, MeetingRelationship, MeetingVisibility,
};

#[test]
fn test_list_meetings_empty() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    assert!(manager.list_meetings().is_empty());
}

#[test]
fn test_our_beacon_id() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    assert!(manager.our_beacon_id().is_none());
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("our_beacon_123"),
        "lineage",
    ));
    let id = manager.our_beacon_id().expect("should have ID");
    assert_eq!(id.0, "our_beacon_123");
}

#[test]
fn test_get_lineage_hint_with_family_seed() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let seed_data = b"abcdefghijklmnop";
    std::fs::write(temp_dir.path().join(".family.seed"), seed_data).expect("write seed");
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    let hint = manager.get_lineage_hint().expect("should succeed");
    assert_eq!(hint, hex::encode(&seed_data[0..8]));
}

#[test]
fn test_get_lineage_hint_no_seed_file() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    assert!(manager.get_lineage_hint().is_err());
}

#[test]
fn test_save_manifest_no_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
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
    let mock = MockCapabilityCaller::new();
    let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock);
    assert!(manager.our_beacon_id().is_none());
    assert_eq!(manager.root_path, temp_dir.path());
}

#[test]
fn test_list_meetings_with_data() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("our123"), "lineage");
    manifest.add_meeting(
        BeaconId::from_hex("peer1"),
        MeetingRecord {
            node_name: "peer-1".to_string(),
            first_met: 1000,
            last_seen: 1000,
            endpoints: vec!["192.0.2.1:9900".to_string()],
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
    let manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    let result = manager.get_lineage_hint();
    assert!(result.is_err());
}

#[test]
fn test_save_manifest_with_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mut manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    manager.set_manifest(BeaconGeneticsManifest::new(
        BeaconId::from_hex("save-test"),
        "lineage",
    ));

    manager.save_manifest().expect("save should succeed");
    assert!(temp_dir.path().join(".beacon.genetics.json").exists());
}
