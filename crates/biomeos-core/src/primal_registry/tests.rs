// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for primal_registry/mod.rs

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

use super::*;

#[tokio::test]
async fn test_registry_creation() {
    let registry = PrimalRegistry::new("/tmp/test-bins");
    assert_eq!(registry.list_primals().len(), 0);
}

#[test]
fn test_primal_name_detection() {
    let _registry = PrimalRegistry::new("/tmp");
    assert_eq!(PrimalRegistry::detect_primal_name("beardog"), "beardog");
    assert_eq!(
        PrimalRegistry::detect_primal_name("beardog-linux"),
        "beardog"
    );
    assert_eq!(
        PrimalRegistry::detect_primal_name("songbird.exe"),
        "songbird"
    );
}

#[test]
fn test_find_by_capability() {
    let mut registry = PrimalRegistry::new("/tmp");

    // Add test binaries
    let beardog = PrimalBinary {
        name: "beardog".to_string(),
        version: "1.0.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/beardog")),
        checksum: None,
        metadata: PrimalMetadata {
            description: "Security primal".to_string(),
            capabilities: vec!["encryption".to_string(), "crypto".to_string()],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };

    let songbird = PrimalBinary {
        name: "songbird".to_string(),
        version: "1.0.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/songbird")),
        checksum: None,
        metadata: PrimalMetadata {
            description: "Discovery primal".to_string(),
            capabilities: vec!["discovery".to_string(), "federation".to_string()],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };

    registry
        .binaries
        .insert("beardog".to_string(), vec![beardog]);
    registry
        .binaries
        .insert("songbird".to_string(), vec![songbird]);

    // Test finding by capability
    let encryption_primals = registry.find_by_capability(&CapabilityTaxonomy::Encryption);
    assert_eq!(encryption_primals.len(), 1);
    assert_eq!(encryption_primals[0].name, "beardog");

    let discovery_primals = registry.find_by_capability(&CapabilityTaxonomy::Discovery);
    assert_eq!(discovery_primals.len(), 1);
    assert_eq!(discovery_primals[0].name, "songbird");

    // Test finding non-existent capability
    let compute_primals = registry.find_by_capability(&CapabilityTaxonomy::WorkloadExecution);
    assert_eq!(compute_primals.len(), 0);
}

#[test]
fn test_find_by_multiple_capabilities() {
    let mut registry = PrimalRegistry::new("/tmp");

    // Add a primal that provides multiple capabilities
    let multi_cap = PrimalBinary {
        name: "multi".to_string(),
        version: "1.0.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/multi")),
        checksum: None,
        metadata: PrimalMetadata {
            description: "Multi-capability primal".to_string(),
            capabilities: vec![
                "encryption".to_string(),
                "discovery".to_string(),
                "compute".to_string(),
            ],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };

    registry
        .binaries
        .insert("multi".to_string(), vec![multi_cap]);

    // Test finding by multiple capabilities (all match)
    let primals = registry.find_by_capabilities(&[
        CapabilityTaxonomy::Encryption,
        CapabilityTaxonomy::Discovery,
    ]);
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].name, "multi");

    // Test finding by multiple capabilities (not all match)
    let primals = registry.find_by_capabilities(&[
        CapabilityTaxonomy::Encryption,
        CapabilityTaxonomy::DataStorage, // multi doesn't have storage
    ]);
    assert_eq!(primals.len(), 0);
}

#[test]
fn test_get_best_for_capability() {
    let mut registry = PrimalRegistry::new("/tmp");

    // Add multiple versions of the same primal
    let v1 = PrimalBinary {
        name: "beardog".to_string(),
        version: "1.0.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/beardog-1.0.0")),
        checksum: None,
        metadata: PrimalMetadata {
            description: "Security primal".to_string(),
            capabilities: vec!["encryption".to_string()],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };

    let v2 = PrimalBinary {
        name: "beardog".to_string(),
        version: "2.0.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/beardog-2.0.0")),
        checksum: None,
        metadata: PrimalMetadata {
            description: "Security primal".to_string(),
            capabilities: vec!["encryption".to_string()],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };

    registry
        .binaries
        .insert("beardog".to_string(), vec![v1, v2]);

    // Get best (should return v2.0.0)
    let best = registry.get_best_for_capability(&CapabilityTaxonomy::Encryption);
    assert!(best.is_some());
    assert_eq!(best.unwrap().version, "2.0.0");
}

#[test]
fn test_capability_fuzzy_matching() {
    let _registry = PrimalRegistry::new("/tmp");

    // Test encryption/crypto match
    assert!(PrimalRegistry::capability_matches("crypto", "encryption"));
    assert!(PrimalRegistry::capability_matches("security", "encryption"));

    // Test discovery/federation match
    assert!(PrimalRegistry::capability_matches("mesh", "discovery"));
    assert!(PrimalRegistry::capability_matches(
        "federation",
        "discovery"
    ));

    // Test compute/orchestration match
    assert!(PrimalRegistry::capability_matches(
        "orchestration",
        "compute"
    ));

    // Test non-matches
    assert!(!PrimalRegistry::capability_matches("storage", "encryption"));
    assert!(!PrimalRegistry::capability_matches("random", "discovery"));
}

#[tokio::test]
async fn test_scan_local_nonexistent_dir() {
    let mut registry = PrimalRegistry::new("/nonexistent/path/12345");
    let result = registry.scan_local().await;
    assert!(result.is_ok());
    assert!(registry.list_primals().is_empty());
}

#[tokio::test]
async fn test_deploy_to_target_not_found() {
    let registry = PrimalRegistry::new("/tmp");
    let result = registry
        .deploy_to_target("nonexistent-primal", None, "/tmp/target")
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_deploy_to_target_version_not_found() {
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let bin_path = temp_dir.path().join("testbin");
    std::fs::write(&bin_path, b"binary").expect("write");
    let mut registry = PrimalRegistry::new(temp_dir.path());
    registry.scan_local().await.expect("scan");
    let result = registry
        .deploy_to_target(
            "testbin",
            Some("2.0.0"),
            temp_dir.path().join("target").to_str().unwrap(),
        )
        .await;
    assert!(result.is_err());
}

#[test]
fn test_binary_location_serde() {
    let local = BinaryLocation::Local(PathBuf::from("/tmp/beardog"));
    let json = serde_json::to_string(&local).expect("serialize");
    let _: BinaryLocation = serde_json::from_str(&json).expect("deserialize");

    let github = BinaryLocation::GitHub {
        org: "eco".to_string(),
        repo: "beardog".to_string(),
        tag: "v1.0".to_string(),
        asset: "beardog-linux".to_string(),
    };
    let json = serde_json::to_string(&github).expect("serialize");
    let _: BinaryLocation = serde_json::from_str(&json).expect("deserialize");
}

#[tokio::test]
async fn test_scan_local_with_binaries() {
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let bin_path = temp_dir.path().join("test-primal");
    std::fs::write(&bin_path, b"#!/bin/sh\necho test").expect("write");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin_path, perms).unwrap();
    }

    let mut registry = PrimalRegistry::new(temp_dir.path());
    let result = registry.scan_local().await;
    assert!(result.is_ok());
    let primals = registry.list_primals();
    assert!(!primals.is_empty());
}

#[tokio::test]
async fn test_deploy_to_target_local_success() {
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let bin_path = temp_dir.path().join("deployable");
    std::fs::write(&bin_path, b"binary").expect("write");

    let mut registry = PrimalRegistry::new(temp_dir.path());
    registry.scan_local().await.expect("scan");

    let target = temp_dir.path().join("target");
    std::fs::create_dir_all(&target).unwrap();
    let result = registry
        .deploy_to_target("deployable", None, target.to_str().unwrap())
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), bin_path);
}

#[tokio::test]
async fn test_fetch_from_github() {
    let mut registry = PrimalRegistry::new("/tmp");
    let result = registry
        .fetch_from_github("ecoPrimals", &["beardog", "songbird"])
        .await;
    assert!(result.is_ok());
}

#[test]
fn test_get_primal_versions_empty() {
    let registry = PrimalRegistry::new("/tmp");
    let versions = registry.get_primal_versions("nonexistent");
    assert!(versions.is_empty());
}

#[test]
fn test_get_latest_empty() {
    let registry = PrimalRegistry::new("/tmp");
    assert!(registry.get_latest("nonexistent").is_none());
}

#[test]
fn test_primal_name_detection_variants() {
    assert_eq!(
        PrimalRegistry::detect_primal_name("biomeos-beardog-bin"),
        "beardog"
    );
    assert_eq!(
        PrimalRegistry::detect_primal_name("songbird-server"),
        "songbird"
    );
    assert_eq!(
        PrimalRegistry::detect_primal_name("toadstool-cli"),
        "toadstool"
    );
}

#[test]
fn test_primal_binary_serde() {
    let binary = PrimalBinary {
        name: "test".to_string(),
        version: "1.0".to_string(),
        path: BinaryLocation::Local(PathBuf::from("/tmp/test")),
        checksum: Some("abc123".to_string()),
        metadata: PrimalMetadata {
            description: "Test".to_string(),
            capabilities: vec!["compute".to_string()],
            default_ports: HashMap::new(),
            config_hints: HashMap::new(),
        },
    };
    let json = serde_json::to_string(&binary).expect("serialize");
    let _: PrimalBinary = serde_json::from_str(&json).expect("deserialize");
}
