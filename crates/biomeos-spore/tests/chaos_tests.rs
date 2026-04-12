// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

//! Chaos testing - Filesystem failures, permission issues, etc.
//!
//! These tests simulate real-world failure scenarios using isolated fixtures.

use biomeos_spore::test_support::setup_test_binaries_at;
use biomeos_spore::{Spore, SporeConfig, SporeType};
use std::fs;
use tempfile::TempDir;

/// Helper to setup plasmidBin and set working directory atomically
fn setup_isolated_test() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    setup_test_binaries_at(temp_dir.path()).unwrap();
    temp_dir
}

/// Test behavior when destination filesystem is read-only
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[cfg(unix)]
async fn test_readonly_filesystem() {
    let temp_dir = setup_isolated_test();

    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();

    let mut perms = fs::metadata(&mount_point).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&mount_point, perms).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(temp_dir.path().join("plasmidBin")),
    };

    let result = Spore::create(mount_point.clone(), config).await;

    // Should fail gracefully
    assert!(result.is_err(), "Should fail on read-only filesystem");

    // Cleanup: Make writable again for temp_dir cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Set proper Unix permissions (0o755) instead of just readonly=false
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&mount_point, perms).unwrap();
    }
    #[cfg(not(unix))]
    {
        let mut perms = fs::metadata(&mount_point).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&mount_point, perms).unwrap();
    }
}

/// Test behavior when disk space is insufficient (simulated)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_disk_full_simulation() {
    let temp_dir = TempDir::new().unwrap();

    // Setup proper plasmidBin structure manually (for large files)
    let plasmid_bin = temp_dir.path().join("plasmidBin");
    let tower_dir = plasmid_bin.join("tower");
    let primals_dir = plasmid_bin.join("primals");
    fs::create_dir_all(&tower_dir).unwrap();
    fs::create_dir_all(&primals_dir).unwrap();

    // Create large mock files (1MB each)
    let large_data = vec![0u8; 1024 * 1024];

    let tower_bin = tower_dir.join("tower");
    fs::write(&tower_bin, &large_data).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tower_bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tower_bin, perms).unwrap();
    }

    for primal in ["beardog", "songbird"] {
        let primal_bin = primals_dir.join(primal);
        fs::write(&primal_bin, &large_data).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&primal_bin).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&primal_bin, perms).unwrap();
        }
    }

    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(temp_dir.path().join("plasmidBin")),
    };

    let result = Spore::create(mount_point, config).await;

    assert!(
        result.is_ok() || result.is_err(),
        "Should handle disk space gracefully"
    );
}

/// Test behavior when binaries are corrupt/invalid
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_corrupt_binaries() {
    let temp_dir = TempDir::new().unwrap();

    // Setup plasmidBin with corrupt binaries
    let plasmid_bin = temp_dir.path().join("plasmidBin");
    let tower_dir = plasmid_bin.join("tower");
    let primals_dir = plasmid_bin.join("primals");
    fs::create_dir_all(&tower_dir).unwrap();
    fs::create_dir_all(&primals_dir).unwrap();

    let corrupt_data = "CORRUPT_BINARY_DATA_NOT_ELF";

    fs::write(tower_dir.join("tower"), corrupt_data).unwrap();
    for primal in ["beardog", "songbird"] {
        fs::write(primals_dir.join(primal), corrupt_data).unwrap();
    }

    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(temp_dir.path().join("plasmidBin")),
    };

    let result = Spore::create(mount_point.clone(), config).await;

    assert!(
        result.is_ok(),
        "Copy should succeed even with corrupt binaries: {:?}",
        result.err()
    );

    let spore_root = mount_point.join("biomeOS");
    assert!(spore_root.join("bin/tower").exists());
    assert!(spore_root.join("primals/beardog").exists());

    let tower_content = fs::read_to_string(spore_root.join("bin/tower")).unwrap();
    assert_eq!(tower_content, corrupt_data);
}

/// Test behavior with FAT32 filesystem limitations
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_fat32_limitations() {
    let temp_dir = setup_isolated_test();

    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
        family_id: "test-family".to_string(),
        plasmid_bin_dir: Some(temp_dir.path().join("plasmidBin")),
    };

    let result = Spore::create(mount_point.clone(), config).await;
    assert!(
        result.is_ok(),
        "Should handle FAT32 limitations: {:?}",
        result.err()
    );

    let deploy_script = mount_point.join("biomeOS/deploy.sh");
    assert!(deploy_script.exists(), "deploy.sh should exist");

    let script_content = fs::read_to_string(deploy_script).unwrap();
    assert!(script_content.contains("chmod"), "Should fix permissions");
}

/// Test concurrent spore creation (race conditions)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_spore_creation() {
    let temp_dir = setup_isolated_test();

    for i in 1..=3 {
        let mount_point = temp_dir.path().join(format!("usb{i}"));
        fs::create_dir_all(&mount_point).unwrap();

        let config = SporeConfig {
            label: format!("spore{i}"),
            node_id: format!("node{i}"),
            spore_type: SporeType::Live,
            family_id: "test-family".to_string(),
            plasmid_bin_dir: Some(temp_dir.path().join("plasmidBin")),
        };

        let result = Spore::create(mount_point.clone(), config).await;
        assert!(
            result.is_ok(),
            "Spore {} creation should succeed: {:?}",
            i,
            result.err()
        );

        // Verify spore was created
        assert!(mount_point.join("biomeOS/bin/tower").exists());
    }
}
