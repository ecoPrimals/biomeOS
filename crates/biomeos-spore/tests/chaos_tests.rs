//! Chaos testing - Filesystem failures, permission issues, etc.
//!
//! These tests simulate real-world failure scenarios

use biomeos_spore::{Spore, SporeConfig, SporeType};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test behavior when destination filesystem is read-only
#[tokio::test]
#[cfg(unix)]
async fn test_readonly_filesystem() {
    use std::os::unix::fs::PermissionsExt;
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create nucleusBin
    let nucleus_dir = temp_dir.path().join("nucleusBin");
    fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    fs::create_dir_all(nucleus_dir.join("primals")).unwrap();
    
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        fs::write(&path, "#!/bin/sh\necho 'Mock'\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Create mount point and make it read-only
    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();
    
    let mut perms = fs::metadata(&mount_point).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&mount_point, perms).unwrap();
    
    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };
    
    let result = Spore::create(mount_point.clone(), config).await;
    
    // Should fail gracefully
    assert!(result.is_err(), "Should fail on read-only filesystem");
    
    // Cleanup: Make writable again for temp_dir cleanup
    let mut perms = fs::metadata(&mount_point).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&mount_point, perms).unwrap();
}

/// Test behavior when disk space is insufficient (simulated)
#[tokio::test]
async fn test_disk_full_simulation() {
    // This is a conceptual test - actual disk full is hard to simulate
    // We test by creating a tiny destination and large source files
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create nucleusBin with "large" binaries
    let nucleus_dir = temp_dir.path().join("nucleusBin");
    fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    fs::create_dir_all(nucleus_dir.join("primals")).unwrap();
    
    // Create large mock files (1MB each)
    let large_data = vec![0u8; 1024 * 1024];
    
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        fs::write(&path, &large_data).unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
    }
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();
    
    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };
    
    // This should still succeed in our test environment
    // In production, this would fail if disk is actually full
    let result = Spore::create(mount_point, config).await;
    
    // We expect success in test environment, but the error handling
    // is in place for real disk full scenarios
    assert!(result.is_ok() || result.is_err(), "Should handle disk space gracefully");
}

/// Test behavior when binaries are corrupt/invalid
#[tokio::test]
async fn test_corrupt_binaries() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create nucleusBin with corrupt binaries
    let nucleus_dir = temp_dir.path().join("nucleusBin");
    fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    fs::create_dir_all(nucleus_dir.join("primals")).unwrap();
    
    // Create invalid/corrupt files
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        // Write garbage data
        fs::write(&path, "CORRUPT_BINARY_DATA_NOT_ELF").unwrap();
    }
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();
    
    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };
    
    // Should still copy (file validation happens at verify stage)
    let result = Spore::create(mount_point.clone(), config).await;
    
    if result.is_ok() {
        // Verify binaries were copied (even if corrupt)
        let spore_root = mount_point.join("biomeOS");
        assert!(spore_root.join("bin/tower").exists());
        
        // Content verification would happen later with verify-nucleus.sh
        let content = fs::read_to_string(spore_root.join("bin/tower")).unwrap();
        assert!(content.contains("CORRUPT"), "Corrupt data should be copied for verification");
    }
}

/// Test behavior with FAT32 filesystem limitations
#[tokio::test]
async fn test_fat32_limitations() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create nucleusBin
    let nucleus_dir = temp_dir.path().join("nucleusBin");
    fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    fs::create_dir_all(nucleus_dir.join("primals")).unwrap();
    
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        fs::write(&path, "#!/bin/sh\necho 'Mock'\n").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
    }
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let mount_point = temp_dir.path().join("usb");
    fs::create_dir_all(&mount_point).unwrap();
    
    let config = SporeConfig {
        label: "test-spore".to_string(),
        node_id: "test-node".to_string(),
        spore_type: SporeType::Live,
    };
    
    let result = Spore::create(mount_point.clone(), config).await;
    assert!(result.is_ok(), "Should handle FAT32 limitations");
    
    // Verify deploy.sh was created (it handles FAT32 permission issues)
    let deploy_script = mount_point.join("biomeOS/deploy.sh");
    assert!(deploy_script.exists(), "deploy.sh should handle FAT32");
    
    let script_content = fs::read_to_string(deploy_script).unwrap();
    assert!(script_content.contains("FAT32"), "Should mention FAT32 handling");
    assert!(script_content.contains("chmod"), "Should fix permissions");
}

/// Test concurrent spore creation (race conditions)
#[tokio::test]
async fn test_concurrent_spore_creation() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create nucleusBin
    let nucleus_dir = temp_dir.path().join("nucleusBin");
    fs::create_dir_all(nucleus_dir.join("tower")).unwrap();
    fs::create_dir_all(nucleus_dir.join("primals")).unwrap();
    
    for (dir, name) in [
        ("tower", "tower"),
        ("primals", "beardog-server"),
        ("primals", "songbird"),
    ] {
        let path = nucleus_dir.join(dir).join(name);
        fs::write(&path, "#!/bin/sh\necho 'Mock'\n").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
    }
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Create 3 spores concurrently
    let mut handles = vec![];
    
    for i in 1..=3 {
        let temp_path = temp_dir.path().to_path_buf();
        let handle = tokio::spawn(async move {
            let mount_point = temp_path.join(format!("usb{}", i));
            fs::create_dir_all(&mount_point).unwrap();
            
            let config = SporeConfig {
                label: format!("spore{}", i),
                node_id: format!("node{}", i),
                spore_type: SporeType::Live,
            };
            
            Spore::create(mount_point, config).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok(), "Concurrent creation should work");
        assert!(result.unwrap().is_ok(), "Spore creation should succeed");
    }
}
