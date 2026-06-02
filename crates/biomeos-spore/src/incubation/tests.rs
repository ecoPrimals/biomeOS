#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
use super::*;
use chrono::Utc;
use std::path::PathBuf;

// ========== LocalEntropy Tests ==========

#[test]
fn test_local_entropy_generation() {
    let entropy = LocalEntropy::generate(Some("test-computer")).expect("generate entropy");

    assert_eq!(entropy.hostname, "test-computer");
    assert!(!entropy.machine_id.is_empty());
    assert_eq!(entropy.random_nonce.len(), 32);

    let hash = entropy.hash();
    assert_eq!(hash.len(), 64); // SHA256 hex
}

#[test]
fn test_local_entropy_generation_no_name() {
    let entropy = LocalEntropy::generate(None).expect("generate entropy");

    // Should use system hostname or fallback
    assert!(!entropy.hostname.is_empty());
    assert_eq!(entropy.random_nonce.len(), 32);
}

#[test]
fn test_entropy_hash_deterministic() {
    let entropy = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "12345".to_string(),
        timestamp: Utc::now(),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: bytes::Bytes::from_static(&[1, 2, 3]),
        cpu_hash: None,
        disk_serial: None,
    };

    let hash1 = entropy.hash();
    let hash2 = entropy.hash();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_entropy_hash_varies_with_hostname() {
    let now = Utc::now();

    let entropy1 = LocalEntropy {
        hostname: "host-a".to_string(),
        machine_id: "same-id".to_string(),
        timestamp: now,
        mac_address: None,
        random_nonce: bytes::Bytes::from_static(&[1, 2, 3]),
        cpu_hash: None,
        disk_serial: None,
    };

    let entropy2 = LocalEntropy {
        hostname: "host-b".to_string(),
        machine_id: "same-id".to_string(),
        timestamp: now,
        mac_address: None,
        random_nonce: bytes::Bytes::from_static(&[1, 2, 3]),
        cpu_hash: None,
        disk_serial: None,
    };

    assert_ne!(entropy1.hash(), entropy2.hash());
}

#[test]
fn test_entropy_hash_includes_optional_fields() {
    let now = Utc::now();

    let entropy_without = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "id".to_string(),
        timestamp: now,
        mac_address: None,
        random_nonce: bytes::Bytes::from_static(&[1]),
        cpu_hash: None,
        disk_serial: None,
    };

    let entropy_with_mac = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "id".to_string(),
        timestamp: now,
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: bytes::Bytes::from_static(&[1]),
        cpu_hash: None,
        disk_serial: None,
    };

    let entropy_with_cpu = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "id".to_string(),
        timestamp: now,
        mac_address: None,
        random_nonce: bytes::Bytes::from_static(&[1]),
        cpu_hash: Some("cpu_hash_val".to_string()),
        disk_serial: None,
    };

    let entropy_with_disk = LocalEntropy {
        hostname: "test".to_string(),
        machine_id: "id".to_string(),
        timestamp: now,
        mac_address: None,
        random_nonce: bytes::Bytes::from_static(&[1]),
        cpu_hash: None,
        disk_serial: Some("disk_serial_val".to_string()),
    };

    // All should produce different hashes
    let h0 = entropy_without.hash();
    let h1 = entropy_with_mac.hash();
    let h2 = entropy_with_cpu.hash();
    let h3 = entropy_with_disk.hash();

    assert_ne!(h0, h1);
    assert_ne!(h0, h2);
    assert_ne!(h0, h3);
    assert_ne!(h1, h2);
}

#[test]
fn test_entropy_clone() {
    let entropy = LocalEntropy {
        hostname: "clone-test".to_string(),
        machine_id: "id".to_string(),
        timestamp: Utc::now(),
        mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
        random_nonce: bytes::Bytes::from_static(&[10, 20, 30]),
        cpu_hash: Some("cpu".to_string()),
        disk_serial: Some("disk".to_string()),
    };

    let cloned = entropy.clone();
    assert_eq!(cloned.hostname, entropy.hostname);
    assert_eq!(cloned.machine_id, entropy.machine_id);
    assert_eq!(cloned.random_nonce, entropy.random_nonce);
    assert_eq!(cloned.hash(), entropy.hash());
}

#[test]
fn test_entropy_serialization_json() {
    let entropy = LocalEntropy {
        hostname: "test-host".to_string(),
        machine_id: "abc123".to_string(),
        timestamp: Utc::now(),
        mac_address: Some("00:11:22:33:44:55".to_string()),
        random_nonce: bytes::Bytes::from_static(&[1, 2, 3, 4]),
        cpu_hash: None,
        disk_serial: None,
    };

    let json = serde_json::to_string(&entropy).expect("serialize");
    let deserialized: LocalEntropy = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.hostname, "test-host");
    assert_eq!(deserialized.machine_id, "abc123");
    assert_eq!(
        deserialized.mac_address,
        Some("00:11:22:33:44:55".to_string())
    );
}

// ========== IncubatedNode Tests ==========

#[test]
fn test_incubated_node_creation() {
    let node = IncubatedNode {
        node_id: "node-spore1-laptop".to_string(),
        spore_id: "spore1".to_string(),
        deployed_seed_hash: "hash123".to_string(),
        local_config_path: PathBuf::from("/home/user/.config/biomeos/deployed-nodes/spore1"),
        incubated_at: Utc::now(),
        entropy_hash: "entropy_hash_456".to_string(),
        spore_path: Some(PathBuf::from("/media/usb/biomeOS")),
    };

    assert_eq!(node.node_id, "node-spore1-laptop");
    assert_eq!(node.spore_id, "spore1");
    assert!(node.spore_path.is_some());
}

#[test]
fn test_incubated_node_serialization() {
    let node = IncubatedNode {
        node_id: "node-test".to_string(),
        spore_id: "test-spore".to_string(),
        deployed_seed_hash: "deadbeef".to_string(),
        local_config_path: PathBuf::from("/tmp/test"),
        incubated_at: Utc::now(),
        entropy_hash: "abc123".to_string(),
        spore_path: None,
    };

    let json = serde_json::to_string(&node).expect("serialize");
    let deserialized: IncubatedNode = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.node_id, "node-test");
    assert!(deserialized.spore_path.is_none());
}

// ========== NodeConfig / NodeInfo Tests ==========

#[test]
fn test_node_info_creation() {
    let info = NodeInfo {
        spore_id: "spore-abc".to_string(),
        node_id: "node-abc-laptop".to_string(),
        deployed_at: Utc::now(),
        computer_name: "my-laptop".to_string(),
        entropy_hash: "hash".to_string(),
    };

    assert_eq!(info.spore_id, "spore-abc");
    assert_eq!(info.computer_name, "my-laptop");
}

#[test]
fn test_lineage_info_creation() {
    let lineage = LineageInfo {
        parent_seed_hash: "parent".to_string(),
        spore_seed_hash: "spore".to_string(),
        deployed_seed_hash: "deployed".to_string(),
    };

    assert_eq!(lineage.parent_seed_hash, "parent");
    assert_eq!(lineage.spore_seed_hash, "spore");
    assert_eq!(lineage.deployed_seed_hash, "deployed");
}

#[test]
fn test_federation_info_creation() {
    let fed = FederationInfo {
        family_id: "fam-123".to_string(),
        sub_federations: vec!["sub-1".to_string(), "sub-2".to_string()],
    };

    assert_eq!(fed.family_id, "fam-123");
    assert_eq!(fed.sub_federations.len(), 2);
}

#[test]
fn test_spore_info_creation() {
    let info = SporeInfo {
        original_path: Some(PathBuf::from("/media/usb/biomeOS")),
        last_seen: Utc::now(),
        deployment_count: 3,
    };

    assert_eq!(info.deployment_count, 3);
    assert!(info.original_path.is_some());
}

#[test]
fn test_node_config_serialization() {
    let config = NodeConfig {
        node: NodeInfo {
            spore_id: "sp1".to_string(),
            node_id: "n1".to_string(),
            deployed_at: Utc::now(),
            computer_name: "host".to_string(),
            entropy_hash: "h".to_string(),
        },
        lineage: LineageInfo {
            parent_seed_hash: "p".to_string(),
            spore_seed_hash: "s".to_string(),
            deployed_seed_hash: "d".to_string(),
        },
        spore: SporeInfo {
            original_path: None,
            last_seen: Utc::now(),
            deployment_count: 0,
        },
        federation: FederationInfo {
            family_id: "fam".to_string(),
            sub_federations: vec![],
        },
    };

    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: NodeConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.node.node_id, "n1");
    assert_eq!(deserialized.federation.family_id, "fam");
}

#[test]
fn test_spore_incubator_new_missing_seed() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let spore_path = temp_dir.path();
    std::fs::create_dir_all(spore_path).unwrap();
    let result = SporeIncubator::new(spore_path);
    assert!(result.is_err());
}

#[test]
fn test_spore_incubator_new_success() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let spore_path = temp_dir.path();
    std::fs::create_dir_all(spore_path).unwrap();
    std::fs::write(spore_path.join(".family.seed"), [0u8; 32]).unwrap();
    std::fs::write(
        spore_path.join("tower.toml"),
        r#"[meta]
node_id = "incubator-test-spore"
[tower]
family = "test-family"
"#,
    )
    .unwrap();

    let result = SporeIncubator::new(spore_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_local_nodes_empty() {
    let result = list_local_nodes().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_incubate_end_to_end_and_list_local_nodes() {
    let temp_home = tempfile::tempdir().expect("temp home");

    let spore_root = tempfile::tempdir().expect("spore root");
    std::fs::write(spore_root.path().join(".family.seed"), [11u8; 32]).expect("seed");
    std::fs::write(
        spore_root.path().join("tower.toml"),
        r#"[meta]
node_id = "e2e-incubate-spore"
[tower]
family = "fam-e2e-incubate"
"#,
    )
    .expect("tower");

    let incubator = SporeIncubator::new(spore_root.path()).expect("incubator");
    let node = incubator
        .incubate_with_home(Some("e2e-test-host"), false, Some(temp_home.path()))
        .await
        .expect("incubate");

    assert!(node.node_id.contains("e2e-incubate-spore"));
    assert!(node.local_config_path.join("node.toml").exists());
    assert!(node.local_config_path.join(".deployed.seed").exists());
    assert!(node.local_config_path.join("entropy.json").exists());

    let listed = list_local_nodes_in(temp_home.path()).await.expect("list");
    assert!(
        listed.iter().any(|c| c.node.node_id == node.node_id),
        "deployed-nodes scan should find incubated node"
    );
}

#[tokio::test]
async fn test_list_local_nodes_skips_invalid_node_toml() {
    let temp_home = tempfile::tempdir().expect("temp home");

    let nodes_dir = temp_home
        .path()
        .join(".config")
        .join("biomeos")
        .join("deployed-nodes");
    let bad = nodes_dir.join("bad-entry");
    std::fs::create_dir_all(&bad).expect("mkdir");
    std::fs::write(bad.join("node.toml"), "not valid toml {{{").expect("bad toml");

    let listed = list_local_nodes_in(temp_home.path()).await.expect("list");
    assert!(
        listed.is_empty(),
        "invalid node.toml should be skipped, got {} entries",
        listed.len()
    );
}
