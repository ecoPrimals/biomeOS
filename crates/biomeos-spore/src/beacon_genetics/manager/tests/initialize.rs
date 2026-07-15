// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockCapabilityCaller;
use crate::beacon_genetics::manager::BeaconGeneticsManager;
use crate::beacon_genetics::{BeaconGeneticsManifest, BeaconId};

#[tokio::test]
async fn test_initialize_loads_existing_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let manifest =
        BeaconGeneticsManifest::new(BeaconId::from_hex("existing-beacon"), "lineage-hint");
    manifest
        .save(&temp_dir.path().join(".beacon.genetics.json"))
        .expect("save");
    let mut manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    manager.initialize().await.expect("init should succeed");
    let id = manager.our_beacon_id().expect("should have loaded ID");
    assert_eq!(id.0, "existing-beacon");
}

#[tokio::test]
async fn test_initialize_generates_new_manifest() {
    let temp_dir = tempfile::TempDir::new().expect("create temp dir");
    let mock_caller = MockCapabilityCaller::new();
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
    let mut manager =
        BeaconGeneticsManager::with_capability_caller(temp_dir.path(), MockCapabilityCaller::new());
    let result = manager.initialize().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_initialize_new_manifest_uses_default_lineage_when_no_family_seed() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "beacon.generate",
            serde_json::json!({ "beacon_id": "no-family-seed-beacon" }),
        )
        .await;
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    manager.initialize().await.expect("init");
    let m = manager.manifest.as_ref().expect("manifest");
    assert_eq!(m.lineage_hint, "0000000000000000");
}

#[tokio::test]
async fn test_initialize_beacon_generate_invalid_seed_hex_fails() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mock_caller = MockCapabilityCaller::new();
    mock_caller
        .set_response(
            "beacon.generate",
            serde_json::json!({
                "beacon_id": "bad-hex",
                "seed_hex": "not-hex!!!"
            }),
        )
        .await;
    let mut manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
    let err = manager.initialize().await.expect_err("invalid hex");
    assert!(err.to_string().contains("hex") || err.to_string().contains("Invalid"));
}
