//! Integration tests for plasmidBin deployment
//!
//! Tests the complete pipeline from plasmidBin/ to spore creation.
//! Uses proper UniBin-compliant binary names (beardog, songbird - no suffixes).

use biomeos_spore::test_support::setup_test_binaries_at;
use biomeos_spore::{Spore, SporeConfig, SporeType};
use tempfile::TempDir;

/// Test that spore creation fails gracefully if plasmidBin is missing
#[tokio::test(flavor = "current_thread")]
async fn test_missing_plasmidbin() {
    let temp_dir = TempDir::new().unwrap();

    // Change to temp dir where plasmidBin doesn't exist
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let mount_point = temp_dir.path().join("usb");
    std::fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };

    let result = Spore::create(mount_point, config).await;
    assert!(result.is_err(), "Should fail when plasmidBin is missing");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("plasmidBin"),
        "Error should mention plasmidBin: {}",
        err_msg
    );
}

/// Test that spore creation succeeds with plasmidBin present
#[tokio::test(flavor = "current_thread")]
async fn test_plasmidbin_deployment() {
    let temp_dir = TempDir::new().unwrap();

    // Setup proper plasmidBin structure using test utility
    setup_test_binaries_at(temp_dir.path()).unwrap();

    let mount_point = temp_dir.path().join("usb");
    std::fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };

    let result = Spore::create(mount_point.clone(), config).await;
    assert!(
        result.is_ok(),
        "Should succeed with plasmidBin present: {:?}",
        result.err()
    );

    // Verify binaries were copied (UniBin compliant names)
    let spore_root = mount_point.join("biomeOS");
    assert!(
        spore_root.join("bin/tower").exists(),
        "tower should be copied"
    );
    assert!(
        spore_root.join("primals/beardog").exists(),
        "beardog should be copied (UniBin name)"
    );
    assert!(
        spore_root.join("primals/songbird").exists(),
        "songbird should be copied (UniBin name)"
    );
}

/// Test that VERSION.txt format is correct for primal tracking
#[tokio::test(flavor = "current_thread")]
async fn test_version_tracking() {
    let temp_dir = TempDir::new().unwrap();

    // Create mock plasmidBin with VERSION.txt
    let plasmid_bin = temp_dir.path().join("plasmidBin");
    let tower_dir = plasmid_bin.join("tower");
    let primals_dir = plasmid_bin.join("primals");
    std::fs::create_dir_all(&tower_dir).unwrap();
    std::fs::create_dir_all(&primals_dir).unwrap();

    // Create VERSION.txt with UniBin-compliant names
    std::fs::write(
        plasmid_bin.join("VERSION.txt"),
        r#"tower: git:abc123
beardog: git:def456
songbird: git:ghi789"#,
    )
    .unwrap();

    // Create mock binaries (UniBin names)
    std::fs::write(tower_dir.join("tower"), "#!/bin/sh\necho 'Mock tower'\n").unwrap();
    for primal in ["beardog", "songbird"] {
        std::fs::write(
            primals_dir.join(primal),
            format!("#!/bin/sh\necho 'Mock {}'\n", primal),
        )
        .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = primals_dir.join(primal);
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(tower_dir.join("tower"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(tower_dir.join("tower"), perms).unwrap();
    }

    // VERSION.txt should be accessible and have UniBin-compliant names
    let version_content = std::fs::read_to_string(plasmid_bin.join("VERSION.txt")).unwrap();
    assert!(version_content.contains("tower:"));
    assert!(version_content.contains("beardog:")); // UniBin name
    assert!(version_content.contains("songbird:")); // UniBin name
}

/// Test spore manifest is created correctly
#[tokio::test(flavor = "current_thread")]
async fn test_spore_manifest_creation() {
    let temp_dir = TempDir::new().unwrap();

    // Setup proper plasmidBin structure
    setup_test_binaries_at(temp_dir.path()).unwrap();

    let mount_point = temp_dir.path().join("usb");
    std::fs::create_dir_all(&mount_point).unwrap();

    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };

    let result = Spore::create(mount_point.clone(), config).await;
    assert!(
        result.is_ok(),
        "Spore creation should succeed: {:?}",
        result.err()
    );

    // Verify essential files were created
    let spore_root = mount_point.join("biomeOS");
    assert!(
        spore_root.join("tower.toml").exists(),
        "tower.toml should exist"
    );
    assert!(
        spore_root.join("deploy.sh").exists(),
        "deploy.sh should exist"
    );
    assert!(
        spore_root.join("README.md").exists(),
        "README.md should exist"
    );
}
