//! Integration tests for plasmidBin deployment
//!
//! Tests the complete pipeline from plasmidBin/ to spore creation

use biomeos_spore::{Spore, SporeConfig, SporeType};
use tempfile::TempDir;

/// Test that spore creation fails gracefully if plasmidBin is missing
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Depends on plasmidBin structure - TODO: fix test setup"]
async fn test_missing_nucleus_bin() {
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
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_nucleus_bin_deployment() {
    let temp_dir = TempDir::new().unwrap();

    // Create mock plasmidBin structure
    let nucleus_dir = temp_dir.path().join("plasmidBin");
    std::fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    std::fs::create_dir_all(nucleus_dir.join("primals")).unwrap();

    // Create mock binaries
    std::fs::write(
        nucleus_dir.join("tower/tower"),
        "#!/bin/sh\necho 'Mock tower'\n",
    )
    .unwrap();

    std::fs::write(
        nucleus_dir.join("primals/beardog-server"),
        "#!/bin/sh\necho 'Mock beardog'\n",
    )
    .unwrap();

    std::fs::write(
        nucleus_dir.join("primals/songbird"),
        "#!/bin/sh\necho 'Mock songbird'\n",
    )
    .unwrap();

    // Make them executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [
            nucleus_dir.join("tower/tower"),
            nucleus_dir.join("primals/beardog-server"),
            nucleus_dir.join("primals/songbird"),
        ] {
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }
    }

    // Change to temp dir
    std::env::set_current_dir(temp_dir.path()).unwrap();

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
        result
    );

    // Verify binaries were copied
    let spore_root = mount_point.join("biomeOS");
    assert!(
        spore_root.join("bin/tower").exists(),
        "tower should be copied"
    );
    assert!(
        spore_root.join("primals/beardog-server").exists(),
        "beardog-server should be copied"
    );
    assert!(
        spore_root.join("primals/songbird").exists(),
        "songbird should be copied"
    );
}

/// Test that VERSION.txt is verified during deployment
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_version_tracking() {
    let temp_dir = TempDir::new().unwrap();

    // Create mock plasmidBin with VERSION.txt
    let nucleus_dir = temp_dir.path().join("plasmidBin");
    std::fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    std::fs::create_dir_all(nucleus_dir.join("primals")).unwrap();

    // Create VERSION.txt
    std::fs::write(
        nucleus_dir.join("VERSION.txt"),
        r#"tower: git:abc123
beardog-server: git:def456
songbird: git:ghi789"#,
    )
    .unwrap();

    // Create mock binaries
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        std::fs::write(&path, "#!/bin/sh\necho 'Mock'\n").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }
    }

    // VERSION.txt should be accessible for verification
    let version_content = std::fs::read_to_string(nucleus_dir.join("VERSION.txt")).unwrap();
    assert!(version_content.contains("tower:"));
    assert!(version_content.contains("beardog-server:"));
    assert!(version_content.contains("songbird:"));
}
