// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Unit tests for manifest types
//
// Tests BinaryManifest and SporeManifest serialization, deserialization,
// and core functionality.

use biomeos_spore::manifest::{
    BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta, SporeManifest,
};
use chrono::Utc;
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_binary_info_creation() {
    let binary = BinaryInfo {
        name: "test-binary".to_string(),
        version: "1.0.0".to_string(),
        git_commit: "abc123".to_string(),
        build_date: Utc::now(),
        sha256: "test_hash".to_string(),
        size_bytes: 1000,
        source_repo: "test-repo".to_string(),
        features: vec!["feature1".to_string(), "feature2".to_string()],
    };

    assert_eq!(binary.name, "test-binary");
    assert_eq!(binary.version, "1.0.0");
    assert_eq!(binary.sha256, "test_hash");
    assert_eq!(binary.features.len(), 2);
    assert!(binary.features.contains(&"feature1".to_string()));
}

#[test]
fn test_binary_manifest_serialization() {
    let mut binaries = HashMap::new();
    binaries.insert(
        "beardog".to_string(),
        BinaryInfo {
            name: "beardog".to_string(),
            version: "0.15.0".to_string(),
            git_commit: "abc123".to_string(),
            build_date: Utc::now(),
            sha256: "sha256hash".to_string(),
            size_bytes: 1000,
            source_repo: "ecoPrimals/beardog".to_string(),
            features: vec!["btsp".to_string()],
        },
    );

    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            pipeline_run: "test-run".to_string(),
        },
        binaries,
        compatibility: CompatibilityInfo {
            min_tower_version: "0.6.0".to_string(),
            min_beardog_version: "0.15.0".to_string(),
            min_songbird_version: "3.19.0".to_string(),
        },
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&manifest).expect("Failed to serialize");

    // Should contain key fields
    assert!(toml_str.contains("beardog"));
    assert!(toml_str.contains("0.15.0"));
    assert!(toml_str.contains("sha256hash"));
}

#[test]
fn test_binary_manifest_deserialization() {
    let toml_str = r#"
        [manifest]
        version = "1.0"
        created_at = "2026-01-08T16:00:00Z"
        pipeline_run = "test-run"
        
        [compatibility]
        min_tower_version = "0.6.0"
        min_beardog_version = "0.15.0"
        min_songbird_version = "3.19.0"
        
        [binaries.beardog]
        name = "beardog"
        version = "0.15.0"
        sha256 = "test_hash"
        size_bytes = 1000
        git_commit = "abc123"
        build_date = "2026-01-08T16:00:00Z"
        source_repo = "ecoPrimals/beardog"
        features = ["btsp"]
    "#;

    let manifest: BinaryManifest = toml::from_str(toml_str).expect("Failed to deserialize");

    assert_eq!(manifest.binaries.len(), 1);
    assert!(manifest.binaries.contains_key("beardog"));

    let beardog = &manifest.binaries["beardog"];
    assert_eq!(beardog.version, "0.15.0");
    assert_eq!(beardog.sha256, "test_hash");
}

#[test]
fn test_spore_manifest_creation() {
    let manifest = SporeManifest::new(
        "node-alpha".to_string(),
        "test_family".to_string(),
        "live".to_string(),
        "20260108".to_string(),
        "parent_hash".to_string(),
        "child_hash".to_string(),
    );

    assert_eq!(manifest.spore.node_id, "node-alpha");
    assert_eq!(manifest.spore.family_id, "test_family");
    assert_eq!(manifest.spore.spore_type, "live");
    assert_eq!(manifest.spore.deployment_batch, "20260108");
    assert_eq!(manifest.lineage.parent_seed_hash, "parent_hash");
    assert_eq!(manifest.lineage.child_seed_hash, "child_hash");
}

#[test]
fn test_spore_manifest_add_binary() {
    let mut manifest = SporeManifest::new(
        "test-node".to_string(),
        "test-family".to_string(),
        "live".to_string(),
        "20260108".to_string(),
        "parent".to_string(),
        "child".to_string(),
    );

    manifest.add_binary(
        "beardog".to_string(),
        "0.15.0".to_string(),
        "hash123".to_string(),
    );

    assert_eq!(manifest.binaries.len(), 1);
    assert!(manifest.binaries.contains_key("beardog"));

    let beardog = &manifest.binaries["beardog"];
    assert_eq!(beardog.version, "0.15.0");
}

#[test]
fn test_spore_manifest_save_load() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let spore_path = temp_dir.path();

    // Create manifest
    let mut manifest = SporeManifest::new(
        "test-node".to_string(),
        "test-family".to_string(),
        "live".to_string(),
        "20260108".to_string(),
        "parent_hash".to_string(),
        "child_hash".to_string(),
    );

    manifest.add_binary(
        "test-binary".to_string(),
        "1.0.0".to_string(),
        "sha256".to_string(),
    );

    // Save
    manifest.save(spore_path)?;

    // Load
    let loaded = SporeManifest::load(spore_path)?;

    // Verify
    assert_eq!(loaded.spore.node_id, manifest.spore.node_id);
    assert_eq!(loaded.binaries.len(), manifest.binaries.len());
    assert!(loaded.binaries.contains_key("test-binary"));

    Ok(())
}

#[test]
fn test_binary_manifest_load_missing_file() {
    // Test loading from a directory without MANIFEST.toml
    let result = BinaryManifest::load("/nonexistent/path");
    assert!(result.is_err());
}

#[test]
fn test_binary_manifest_default() {
    // from_nucleus() panics on missing directory, which is expected behavior
    // since it should only be called on valid plasmidBin directories
    // This test just confirms the struct can be created manually
    let manifest = BinaryManifest {
        manifest: ManifestMeta {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            pipeline_run: "test".to_string(),
        },
        binaries: HashMap::new(),
        compatibility: CompatibilityInfo {
            min_tower_version: "0.6.0".to_string(),
            min_beardog_version: "0.15.0".to_string(),
            min_songbird_version: "3.19.0".to_string(),
        },
    };
    assert_eq!(manifest.binaries.len(), 0);
}

#[test]
fn test_spore_manifest_genetic_lineage() {
    let manifest = SporeManifest::new(
        "sibling1".to_string(),
        "family-alpha".to_string(),
        "live".to_string(),
        "batch-001".to_string(),
        "parent-seed-hash".to_string(),
        "child-seed-hash-1".to_string(),
    );

    // Genetic lineage fields should be set
    assert_eq!(manifest.lineage.parent_seed_hash, "parent-seed-hash");
    assert_eq!(manifest.lineage.child_seed_hash, "child-seed-hash-1");

    // Should be different from parent (unique sibling)
    assert_ne!(
        manifest.lineage.parent_seed_hash,
        manifest.lineage.child_seed_hash
    );
}

#[test]
fn test_binary_info_size_tracking() {
    let binary = BinaryInfo {
        name: "large-binary".to_string(),
        version: "1.0.0".to_string(),
        sha256: "hash".to_string(),
        size_bytes: 50_000_000, // 50 MB
        git_commit: "abc".to_string(),
        build_date: Utc::now(),
        source_repo: "test/repo".to_string(),
        features: vec![],
    };

    assert_eq!(binary.size_bytes, 50_000_000);

    // Should be able to represent large files
    let large_binary = BinaryInfo {
        name: "very-large".to_string(),
        version: "2.0.0".to_string(),
        sha256: "hash2".to_string(),
        size_bytes: 2_000_000_000, // 2 GB
        git_commit: "def".to_string(),
        build_date: Utc::now(),
        source_repo: "test/repo2".to_string(),
        features: vec![],
    };

    assert!(large_binary.size_bytes > 1_000_000_000);
}

#[test]
fn test_manifest_handles_empty_features() {
    let binary = BinaryInfo {
        name: "simple-binary".to_string(),
        version: "1.0.0".to_string(),
        sha256: "hash".to_string(),
        size_bytes: 1000,
        git_commit: "abc".to_string(),
        build_date: Utc::now(),
        source_repo: "test/repo".to_string(),
        features: vec![], // No features
    };

    assert!(binary.features.is_empty());

    // Should serialize/deserialize correctly
    let toml = toml::to_string(&binary).expect("Serialize failed");
    let deserialized: BinaryInfo = toml::from_str(&toml).expect("Deserialize failed");

    assert_eq!(deserialized.features.len(), 0);
}

#[test]
fn test_spore_manifest_deployment_history() {
    let mut manifest = SporeManifest::new(
        "test".to_string(),
        "family".to_string(),
        "live".to_string(),
        "batch".to_string(),
        "parent".to_string(),
        "child".to_string(),
    );

    // Initially should be empty
    assert_eq!(manifest.deployment_history.len(), 0);

    // Record deployment
    manifest.record_deployment("prod-server-1".to_string(), "admin".to_string(), true);

    assert_eq!(manifest.deployment_history.len(), 1);
    assert_eq!(manifest.deployment_history[0].deployed_to, "prod-server-1");
    assert!(manifest.deployment_history[0].success);
}
