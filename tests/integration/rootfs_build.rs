// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Integration test for rootfs building
//!
//! These tests require sudo and external tools (qemu-img, qemu-nbd, mkfs.ext4)
//! Run with: cargo test --package biomeos-boot --test rootfs_integration -- --ignored

use anyhow::Result;
use biomeos_boot::rootfs::{RootFsBuilder, RootFsConfig};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires sudo
async fn test_minimal_rootfs_build() -> Result<()> {
    // Setup
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("test-minimal.qcow2");
    
    let config = RootFsConfig {
        output: output_path.clone(),
        size: "1G".to_string(),
        primals_dir: None,  // No primals for minimal test
        services_dir: None,
        fs_type: "ext4".to_string(),
        ..Default::default()
    };
    
    // Execute
    let mut builder = RootFsBuilder::new(config);
    let result = builder.build().await;
    
    // Verify
    assert!(result.is_ok(), "Build should succeed");
    assert!(output_path.exists(), "Output file should exist");
    
    // Verify it's a valid qcow2 image
    let output = std::process::Command::new("qemu-img")
        .args(["info", output_path.to_str().unwrap()])
        .output()?;
    
    assert!(output.status.success(), "qemu-img info should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("qcow2"), "Should be qcow2 format");
    assert!(stdout.contains("1 GiB"), "Should be 1GB size");
    
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires sudo
async fn test_rootfs_with_primals() -> Result<()> {
    // Skip if plasmidBin doesn't exist
    let primals_dir = PathBuf::from("../../plasmidBin");
    if !primals_dir.exists() {
        eprintln!("Skipping: ../../plasmidBin not found");
        return Ok(());
    }
    
    // Setup
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("test-with-primals.qcow2");
    
    let config = RootFsConfig {
        output: output_path.clone(),
        size: "2G".to_string(),
        primals_dir: Some(primals_dir),
        services_dir: None,
        fs_type: "ext4".to_string(),
        ..Default::default()
    };
    
    // Execute
    let mut builder = RootFsBuilder::new(config);
    let result = builder.build().await;
    
    // Verify
    assert!(result.is_ok(), "Build with primals should succeed");
    assert!(output_path.exists(), "Output file should exist");
    
    // Verify it's larger than minimal (has primals)
    let metadata = std::fs::metadata(&output_path)?;
    assert!(metadata.len() > 100 * 1024 * 1024, "Should be > 100MB with primals");
    
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires sudo
async fn test_rootfs_concurrent_builds() -> Result<()> {
    // Test that multiple builds can run concurrently
    // Each should get its own NBD device
    
    let temp_dir = TempDir::new()?;
    
    let mut handles = vec![];
    
    for i in 0..3 {
        let output_path = temp_dir.path().join(format!("test-concurrent-{}.qcow2", i));
        
        let handle = tokio::spawn(async move {
            let config = RootFsConfig {
                output: output_path.clone(),
                size: "512M".to_string(),
                primals_dir: None,
                services_dir: None,
                fs_type: "ext4".to_string(),
                ..Default::default()
            };
            
            let mut builder = RootFsBuilder::new(config);
            builder.build().await
        });
        
        handles.push(handle);
    }
    
    // Wait for all builds
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent build should succeed");
    }
    
    // Verify all files exist
    for i in 0..3 {
        let path = temp_dir.path().join(format!("test-concurrent-{}.qcow2", i));
        assert!(path.exists(), "Concurrent build {} should create file", i);
    }
    
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires sudo
async fn test_rootfs_error_handling() -> Result<()> {
    // Test that errors are handled gracefully and cleanup happens
    
    // Test 1: Invalid size
    let temp_dir = TempDir::new()?;
    let config = RootFsConfig {
        output: temp_dir.path().join("test-invalid.qcow2"),
        size: "not-a-size".to_string(),
        ..Default::default()
    };
    
    let mut builder = RootFsBuilder::new(config);
    let result = builder.build().await;
    assert!(result.is_err(), "Invalid size should fail");
    
    // Test 2: Invalid output path (directory doesn't exist)
    let config = RootFsConfig {
        output: PathBuf::from("/nonexistent/directory/test.qcow2"),
        size: "1G".to_string(),
        ..Default::default()
    };
    
    let mut builder = RootFsBuilder::new(config);
    let result = builder.build().await;
    assert!(result.is_err(), "Invalid path should fail");
    
    Ok(())
}

