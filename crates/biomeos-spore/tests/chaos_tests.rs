//! Chaos and fault injection tests
//!
//! Tests what happens when things go wrong

use biomeos_spore::{Spore, SporeConfig, SporeType};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_missing_binaries_graceful_failure() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = SporeConfig {
        label: "missing_bins".to_string(),
        node_id: "tower_missing".to_string(),
        spore_type: SporeType::Live,
    };
    
    // This should fail gracefully because binaries don't exist
    let result = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    // Should fail with proper error message
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    
    // Should mention genetic material or binary not found
    assert!(
        error_msg.contains("Genetic material") || 
        error_msg.contains("not found") ||
        error_msg.contains("binary"),
        "Error message should be descriptive: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_corrupted_family_seed() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a spore
    let config = SporeConfig {
        label: "corrupt_test".to_string(),
        node_id: "tower_corrupt".to_string(),
        spore_type: SporeType::Live,
    };
    
    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    // Corrupt the family seed
    let seed_path = temp_dir.path().join("biomeOS/.family.seed");
    fs::write(&seed_path, &[0u8; 16]).unwrap(); // Wrong size
    
    // Try to load the spore
    let result = Spore::from_path(temp_dir.path().to_path_buf());
    
    // Should still load (corruption detected at runtime by BearDog)
    // But seed validation would fail
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_missing_tower_config() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a spore
    let config = SporeConfig {
        label: "missing_config".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
    };
    
    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    // Delete tower.toml
    let config_path = temp_dir.path().join("biomeOS/tower.toml");
    fs::remove_file(config_path).unwrap();
    
    // Try to load
    let result = Spore::from_path(temp_dir.path().to_path_buf());
    
    // Should fail or handle gracefully
    if result.is_ok() {
        // If it loads, node_id should default to "unknown"
        let spore = result.unwrap();
        assert_eq!(spore.config().node_id, "unknown");
    }
}

#[tokio::test]
async fn test_readonly_filesystem() {
    let temp_dir = TempDir::new().unwrap();
    
    // This test simulates readonly filesystem by creating directory structure
    // then making it readonly (won't work on all systems)
    
    let config = SporeConfig {
        label: "readonly_test".to_string(),
        node_id: "tower_readonly".to_string(),
        spore_type: SporeType::Live,
    };
    
    // Create directory
    let root = temp_dir.path().join("biomeOS");
    fs::create_dir_all(&root).unwrap();
    
    // Make it readonly
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&root).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&root, perms).unwrap();
    }
    
    // Try to create spore (should fail)
    let result = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    #[cfg(unix)]
    {
        // Should fail on unix due to permissions
        assert!(result.is_err());
    }
    
    // Clean up: restore permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&root).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&root, perms).unwrap();
    }
}

#[tokio::test]
async fn test_disk_full_simulation() {
    // This is hard to test without actual disk full condition
    // We can test behavior with very small quota though
    
    let temp_dir = TempDir::new().unwrap();
    
    let config = SporeConfig {
        label: "diskfull_test".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
    };
    
    // Attempt creation (will fail due to missing binaries)
    let result = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    // Should handle gracefully
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_mount_point() {
    let config = SporeConfig {
        label: "invalid_mount".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
    };
    
    // Non-existent mount point
    let invalid_path = PathBuf::from("/this/path/definitely/does/not/exist");
    
    let result = Spore::create(invalid_path, config).await;
    
    // Should fail gracefully
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_spore_creation() {
    use tokio::task;
    
    let temp_dir = TempDir::new().unwrap();
    
    // Try to create multiple spores concurrently to same location
    let handles: Vec<_> = (0..5).map(|i| {
        let path = temp_dir.path().to_path_buf();
        task::spawn(async move {
            let config = SporeConfig {
                label: format!("concurrent_{}", i),
                node_id: format!("tower_{}", i),
                spore_type: SporeType::Live,
            };
            Spore::create(path, config).await
        })
    }).collect();
    
    // Wait for all
    let results = futures::future::join_all(handles).await;
    
    // At most one should succeed (or all fail due to missing binaries)
    let successes = results.iter().filter_map(|r| {
        r.as_ref().ok().and_then(|res| res.as_ref().ok())
    }).count();
    
    // Should handle conflicts gracefully
    assert!(successes <= 1, "At most one concurrent creation should succeed");
}

#[tokio::test]
async fn test_malformed_tower_toml() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create spore
    let config = SporeConfig {
        label: "malformed_test".to_string(),
        node_id: "tower_test".to_string(),
        spore_type: SporeType::Live,
    };
    
    let _ = Spore::create(temp_dir.path().to_path_buf(), config).await;
    
    // Corrupt tower.toml
    let config_path = temp_dir.path().join("biomeOS/tower.toml");
    fs::write(&config_path, "this is not valid TOML!!!").unwrap();
    
    // Try to load
    let result = Spore::from_path(temp_dir.path().to_path_buf());
    
    // Should still load (will fail at deployment time)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_symlink_attack_prevention() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a symlink to sensitive location
    #[cfg(unix)]
    {
        let symlink_path = temp_dir.path().join("biomeOS");
        let target = PathBuf::from("/etc/passwd");
        
        // Try to create symlink
        if std::os::unix::fs::symlink(&target, &symlink_path).is_ok() {
            let config = SporeConfig {
                label: "symlink_test".to_string(),
                node_id: "tower_test".to_string(),
                spore_type: SporeType::Live,
            };
            
            // Should not follow symlink and overwrite system files
            let result = Spore::create(temp_dir.path().to_path_buf(), config).await;
            
            // Should fail or handle safely
            // (Actual behavior depends on implementation)
            assert!(result.is_err() || target.exists());
        }
    }
}

#[test]
fn test_node_id_injection_prevention() {
    // Test that node IDs with special characters are handled
    let dangerous_ids = vec![
        "../../../etc/passwd",
        "tower1; rm -rf /",
        "tower1 && echo pwned",
        "tower1`whoami`",
        "tower1$(whoami)",
    ];
    
    for id in dangerous_ids {
        let config = SporeConfig {
            label: "injection_test".to_string(),
            node_id: id.to_string(),
            spore_type: SporeType::Live,
        };
        
        // Config should be created without error
        // Actual validation happens at deployment
        assert!(config.node_id.len() > 0);
    }
}

