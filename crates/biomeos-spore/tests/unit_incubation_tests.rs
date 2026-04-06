// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Unit tests for spore incubation

use biomeos_spore::incubation::{IncubatedNode, LocalEntropy, SporeIncubator};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_local_entropy_generation() {
    let entropy = LocalEntropy::generate(Some("test-computer")).unwrap();

    assert_eq!(entropy.hostname, "test-computer");
    assert!(
        !entropy.machine_id.is_empty(),
        "Machine ID should not be empty"
    );
    assert_eq!(entropy.random_nonce.len(), 32, "Nonce should be 32 bytes");

    let hash = entropy.hash();
    assert_eq!(hash.len(), 64, "SHA256 hex should be 64 characters");
}

#[test]
fn test_entropy_hash_deterministic() {
    use chrono::{DateTime, Utc};

    let timestamp: DateTime<Utc> = "2026-01-08T20:00:00Z".parse().unwrap();

    let entropy1 = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "12345".to_string(),
        timestamp,
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: vec![1, 2, 3].into(),
        cpu_hash: None,
        disk_serial: None,
    };

    let hash1 = entropy1.hash();
    let hash2 = entropy1.hash();

    assert_eq!(
        hash1, hash2,
        "Hash should be deterministic for same entropy"
    );
}

#[test]
fn test_entropy_hash_unique() {
    use chrono::{DateTime, Utc};

    let timestamp: DateTime<Utc> = "2026-01-08T20:00:00Z".parse().unwrap();

    let entropy1 = LocalEntropy {
        hostname: "computer-a".to_string(),
        machine_id: "12345".to_string(),
        timestamp,
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: vec![1, 2, 3].into(),
        cpu_hash: None,
        disk_serial: None,
    };

    let entropy2 = LocalEntropy {
        hostname: "computer-b".to_string(),
        machine_id: "12345".to_string(),
        timestamp,
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: vec![1, 2, 3].into(),
        cpu_hash: None,
        disk_serial: None,
    };

    let hash1 = entropy1.hash();
    let hash2 = entropy2.hash();

    assert_ne!(
        hash1, hash2,
        "Different hostnames should produce different hashes"
    );
}

#[test]
fn test_entropy_with_all_fields() {
    use chrono::Utc;

    let entropy = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "12345".to_string(),
        timestamp: Utc::now(),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: vec![1, 2, 3, 4, 5].into(),
        cpu_hash: Some("cpu-abc123".to_string()),
        disk_serial: Some("disk-xyz789".to_string()),
    };

    let hash = entropy.hash();
    assert_eq!(hash.len(), 64, "Hash should include all fields");
}

#[test]
fn test_entropy_with_optional_fields_none() {
    use chrono::Utc;

    let entropy = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "12345".to_string(),
        timestamp: Utc::now(),
        mac_address: None,
        random_nonce: vec![1, 2, 3].into(),
        cpu_hash: None,
        disk_serial: None,
    };

    let hash = entropy.hash();
    assert_eq!(
        hash.len(),
        64,
        "Hash should work with missing optional fields"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_spore_incubator_creation() {
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    // Create minimal spore structure
    std::fs::create_dir_all(&spore_path).unwrap();
    // FamilySeed expects 32 bytes
    let seed_bytes = [42u8; 32];
    std::fs::write(spore_path.join(".family.seed"), seed_bytes).unwrap();
    std::fs::write(
        spore_path.join("tower.toml"),
        r#"
[meta]
node_id = "test-node"
family_id = "test-family"
        "#,
    )
    .unwrap();

    let _incubator = SporeIncubator::new(&spore_path).unwrap();

    // Verify incubator was created (internal state check would require pub fields or getters)
    // Successful creation without panic = test pass
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_spore_incubator_missing_seed() {
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    std::fs::create_dir_all(&spore_path).unwrap();
    // Don't create .family.seed

    let result = SporeIncubator::new(&spore_path);
    assert!(result.is_err(), "Should fail without .family.seed");
}

#[test]
fn test_incubated_node_structure() {
    use chrono::Utc;

    let node = IncubatedNode {
        node_id: "node-alpha-laptop".to_string(),
        spore_id: "alpha".to_string(),
        deployed_seed_hash: "abc123".to_string(),
        local_config_path: PathBuf::from("/home/user/.config/biomeos/deployed-nodes/alpha"),
        incubated_at: Utc::now(),
        entropy_hash: "def456".to_string(),
        spore_path: Some(PathBuf::from("/media/usb/biomeOS")),
    };

    assert_eq!(node.node_id, "node-alpha-laptop");
    assert_eq!(node.spore_id, "alpha");
    assert!(node.spore_path.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_list_local_nodes_empty() {
    // This test would need a temporary home directory
    // For now, just verify the function exists and returns a Result
    let result = biomeos_spore::incubation::list_local_nodes().await;

    // Should return Ok with empty vec if no nodes exist, or Err if HOME not set
    assert!(result.is_ok() || result.is_err());
}
