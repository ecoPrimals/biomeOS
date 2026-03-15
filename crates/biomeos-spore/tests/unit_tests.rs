// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for spore system components

use biomeos_spore::{setup_test_binaries, FamilySeed, Spore, SporeConfig, SporeType};
use tempfile::TempDir;

#[test]
fn test_spore_type_properties() {
    // Live spore requires execution environment
    assert!(SporeType::Live.requires_execution_env());
    assert!(!SporeType::Cold.requires_execution_env());

    // Cold spore is archival
    assert!(SporeType::Cold.is_archival());
    assert!(!SporeType::Live.is_archival());

    // Default is live
    assert_eq!(SporeType::default(), SporeType::Live);
}

#[test]
fn test_spore_type_display() {
    assert_eq!(SporeType::Cold.to_string(), "ColdSpore");
    assert_eq!(SporeType::Live.to_string(), "LiveSpore");

    assert_eq!(SporeType::Cold.emoji(), "❄️");
    assert_eq!(SporeType::Live.emoji(), "🌱");
}

#[test]
fn test_spore_type_description() {
    assert!(SporeType::Cold.description().contains("storage"));
    assert!(SporeType::Live.description().contains("executable"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_spore_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config = SporeConfig {
        label: "test_spore".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: None,
    };

    // Note: This will fail without actual binaries, but tests the structure
    let _result = Spore::create(temp_dir.path().to_path_buf(), config).await;

    // Should create directory structure even if binary copy fails
    let root_path = temp_dir.path().join("biomeOS");
    assert!(root_path.exists());
    assert!(root_path.join("bin").exists());
    assert!(root_path.join("primals").exists());
    assert!(root_path.join("primals/certs").exists());
    assert!(root_path.join("secrets").exists());
    assert!(root_path.join("logs").exists());
    assert!(root_path.join("config").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cold_spore_no_deploy_script() {
    let temp_dir = TempDir::new().unwrap();
    let config = SporeConfig {
        label: "cold_test".to_string(),
        node_id: "tower_cold".to_string(),
        spore_type: SporeType::Cold,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: None,
    };

    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;

    let root_path = temp_dir.path().join("biomeOS");

    // ColdSpore should NOT have deploy.sh
    assert!(!root_path.join("deploy.sh").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_live_spore_has_deploy_script() {
    let plasmid_bin = setup_test_binaries().expect("Failed to setup test binaries");
    let temp_dir = TempDir::new().unwrap();
    let config = SporeConfig {
        label: "live_test".to_string(),
        node_id: "tower_live".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(plasmid_bin),
    };

    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;

    let root_path = temp_dir.path().join("biomeOS");

    // LiveSpore should have deploy.sh
    assert!(root_path.join("deploy.sh").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_spore_manifest_creation() {
    let plasmid_bin = setup_test_binaries().expect("Failed to setup test binaries");
    let temp_dir = TempDir::new().unwrap();
    let config = SporeConfig {
        label: "manifest_test".to_string(),
        node_id: "tower_manifest".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(plasmid_bin),
    };

    let result = Spore::create(temp_dir.path().to_path_buf(), config).await;
    assert!(
        result.is_ok(),
        "Spore creation should succeed: {:?}",
        result.err()
    );

    let root_path = temp_dir.path().join("biomeOS");

    // Should have .manifest.toml (TOML format manifest)
    let manifest_path = root_path.join(".manifest.toml");
    assert!(manifest_path.exists(), ".manifest.toml should exist");

    // Validate manifest content (TOML format)
    let manifest_content = std::fs::read_to_string(manifest_path).unwrap();
    assert!(
        manifest_content.contains("node_id"),
        "Should contain node_id"
    );
    assert!(
        manifest_content.contains("spore_type"),
        "Should contain spore_type"
    );
    assert!(manifest_content.contains("family"), "Should contain family");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_family_seed_generation() {
    let temp_dir = TempDir::new().unwrap();
    let seed_path = temp_dir.path().join("test.seed");

    let seed = FamilySeed::generate_and_write(&seed_path).unwrap();

    // Seed file should exist
    assert!(seed_path.exists());

    // Seed file should be 32 bytes
    let metadata = std::fs::metadata(&seed_path).unwrap();
    assert_eq!(metadata.len(), 32);

    // Should be able to read it back
    let loaded_seed = FamilySeed::from_file(&seed_path).unwrap();

    // Both should reference same file
    assert_eq!(seed.file_path(), loaded_seed.file_path());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_spore_readme_differentiation() {
    let plasmid_bin = setup_test_binaries().expect("Failed to setup test binaries");
    let temp_dir = TempDir::new().unwrap();

    let cold_config = SporeConfig {
        label: "cold_readme".to_string(),
        node_id: "tower_cold".to_string(),
        spore_type: SporeType::Cold,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(plasmid_bin.clone()),
    };
    let _ = Spore::create(temp_dir.path().to_path_buf(), cold_config).await;
    let cold_readme = std::fs::read_to_string(temp_dir.path().join("biomeOS/README.md")).unwrap();

    // Clean up
    std::fs::remove_dir_all(temp_dir.path().join("biomeOS")).unwrap();

    let live_config = SporeConfig {
        label: "live_readme".to_string(),
        node_id: "tower_live".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(plasmid_bin),
    };
    let _ = Spore::create(temp_dir.path().to_path_buf(), live_config).await;
    let live_readme = std::fs::read_to_string(temp_dir.path().join("biomeOS/README.md")).unwrap();

    // READMEs should be different
    assert_ne!(cold_readme, live_readme);

    // Cold README should mention storage
    assert!(cold_readme.contains("ColdSpore"));
    assert!(cold_readme.contains("storage"));

    // Live README should mention deployment
    assert!(live_readme.contains("LiveSpore"));
    assert!(live_readme.contains("deploy.sh"));
}

#[test]
fn test_spore_config_serialization() {
    let config = SporeConfig {
        label: "serialize_test".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Cold,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: None,
    };

    // Should be able to serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"label\":"));
    assert!(json.contains("\"node_id\":"));
    assert!(json.contains("\"spore_type\":"));

    // Should be able to deserialize
    let deserialized: SporeConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.label, config.label);
    assert_eq!(deserialized.node_id, config.node_id);
    assert_eq!(deserialized.spore_type, config.spore_type);
}

#[test]
fn test_spore_type_equality() {
    assert_eq!(SporeType::Cold, SporeType::Cold);
    assert_eq!(SporeType::Live, SporeType::Live);
    assert_ne!(SporeType::Cold, SporeType::Live);
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_secrets_directory_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let config = SporeConfig {
        label: "permissions_test".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: None,
    };

    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;

    let secrets_dir = temp_dir.path().join("biomeOS/secrets");

    // Check permissions (should be 700)
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(secrets_dir).unwrap();
    let mode = metadata.permissions().mode();

    // Last 3 octal digits should be 700
    assert_eq!(mode & 0o777, 0o700);
}
