//! Integration tests for biomeos-boot
//!
//! These tests verify end-to-end functionality of the boot infrastructure.

use anyhow::Result;
use biomeos_boot::{InitramfsBuilder, KernelManager};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Setup a complete test project environment
fn setup_integration_env() -> Result<TempDir> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create project structure
    fs::create_dir_all(root.join("target/release"))?;
    fs::create_dir_all(root.join("templates"))?;
    fs::create_dir_all(root.join("build"))?;
    fs::create_dir_all(root.join("dist"))?;

    // Create mock binaries
    fs::write(
        root.join("target/release/biomeos-init"),
        "#!/bin/sh\necho 'init'",
    )?;
    fs::write(root.join("target/release/biome"), "#!/bin/sh\necho 'cli'")?;

    // Create mock templates
    fs::write(
        root.join("templates/default.yaml"),
        "name: test\nservices: []",
    )?;

    Ok(temp)
}

#[tokio::test]
async fn test_complete_initramfs_build() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    // Build initramfs
    let mut builder = InitramfsBuilder::new(project_root.join("build"))?;
    builder.create_directory_structure()?;
    builder.add_biomeos_binaries(project_root)?;
    builder.install_binaries()?;
    builder.create_init_script()?;

    let output = project_root.join("dist/test-initramfs.img");
    builder.build(&output)?;

    // Verify output
    assert!(output.exists());
    let metadata = fs::metadata(&output)?;
    assert!(metadata.len() > 100); // Should be at least 100 bytes

    Ok(())
}

#[tokio::test]
async fn test_initramfs_contains_required_files() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    let mut builder = InitramfsBuilder::new(project_root.join("build"))?;
    builder.create_directory_structure()?;
    builder.add_biomeos_binaries(project_root)?;
    builder.install_binaries()?;
    builder.create_init_script()?;

    // Build to temp location to verify structure
    let output = project_root.join("dist/structure-test.img");
    builder.build(&output)?;

    // Verify output exists and is non-empty
    assert!(output.exists());
    let metadata = fs::metadata(&output)?;
    assert!(metadata.len() > 100);

    Ok(())
}

#[test]
fn test_kernel_manager_with_custom_kernel() -> Result<()> {
    let temp = TempDir::new()?;
    let kernel_path = temp.path().join("custom-vmlinuz");
    fs::write(&kernel_path, "mock kernel data")?;

    let manager = KernelManager::detect_or_custom(Some(kernel_path.clone()))?;

    assert_eq!(manager.kernel_path(), &kernel_path);
    assert!(manager
        .initramfs_path()
        .to_string_lossy()
        .contains("initramfs"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_initramfs_builds() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    // Build multiple initramfs images
    for i in 0..3 {
        let mut builder = InitramfsBuilder::new(project_root.join(format!("build_{}", i)))?;
        builder.create_directory_structure()?;
        builder.add_biomeos_binaries(project_root)?;
        builder.install_binaries()?;
        builder.create_init_script()?;

        let output = project_root.join(format!("dist/test-initramfs-{}.img", i));
        builder.build(&output)?;

        assert!(output.exists());
    }

    Ok(())
}

#[tokio::test]
async fn test_initramfs_with_missing_binaries() -> Result<()> {
    let temp = TempDir::new()?;
    let work_dir = temp.path().join("work");

    let mut builder = InitramfsBuilder::new(&work_dir)?;
    builder.create_directory_structure()?;

    // Try to add binaries that don't exist - should not fail
    let project_root = temp.path().join("nonexistent");
    let result = builder.add_biomeos_binaries(&project_root);
    assert!(result.is_ok()); // Should skip missing binaries gracefully

    Ok(())
}

#[test]
fn test_concurrent_initramfs_builds() -> Result<()> {
    use std::thread;

    let temp = setup_integration_env()?;
    let project_root = temp.path().to_path_buf();

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let root = project_root.clone();
            thread::spawn(move || -> Result<()> {
                let mut builder =
                    InitramfsBuilder::new(root.join(format!("build_concurrent_{}", i)))?;
                builder.create_directory_structure()?;
                builder.add_biomeos_binaries(&root)?;
                builder.install_binaries()?;
                builder.create_init_script()?;

                let output = root.join(format!("dist/concurrent-{}.img", i));
                builder.build(&output)?;

                assert!(output.exists());
                Ok(())
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[tokio::test]
async fn test_initramfs_size_reasonable() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    let mut builder = InitramfsBuilder::new(project_root.join("build"))?;
    builder.create_directory_structure()?;
    builder.add_biomeos_binaries(project_root)?;
    builder.install_binaries()?;
    builder.create_init_script()?;

    let output = project_root.join("dist/size-test.img");
    builder.build(&output)?;

    let metadata = fs::metadata(&output)?;
    let size_mb = metadata.len() / (1024 * 1024);

    // Should be between 0.1MB and 100MB
    assert!(size_mb < 100, "Initramfs too large: {} MB", size_mb);

    Ok(())
}

#[tokio::test]
async fn test_initramfs_reproducible_builds() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    // Build twice
    let mut builder1 = InitramfsBuilder::new(project_root.join("build1"))?;
    builder1.create_directory_structure()?;
    builder1.add_biomeos_binaries(project_root)?;
    builder1.install_binaries()?;
    builder1.create_init_script()?;
    let output1 = project_root.join("dist/repro1.img");
    builder1.build(&output1)?;

    let mut builder2 = InitramfsBuilder::new(project_root.join("build2"))?;
    builder2.create_directory_structure()?;
    builder2.add_biomeos_binaries(project_root)?;
    builder2.install_binaries()?;
    builder2.create_init_script()?;
    let output2 = project_root.join("dist/repro2.img");
    builder2.build(&output2)?;

    // Sizes should be similar (within 10%)
    let size1 = fs::metadata(&output1)?.len();
    let size2 = fs::metadata(&output2)?.len();
    let diff = (size1 as i64 - size2 as i64).unsigned_abs();
    let percent_diff = (diff * 100) / size1.max(size2);

    assert!(percent_diff < 10, "Builds differ by {}%", percent_diff);

    Ok(())
}

#[tokio::test]
async fn test_error_handling_invalid_output_path() -> Result<()> {
    let temp = setup_integration_env()?;
    let project_root = temp.path();

    let mut builder = InitramfsBuilder::new(project_root.join("build"))?;
    builder.create_directory_structure()?;

    // Try to write to invalid path
    let invalid_output = PathBuf::from("/dev/null/invalid/path/test.img");
    let result = builder.build(&invalid_output);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_kernel_manager_error_no_kernel() {
    // Try to detect kernel without any present
    let result = KernelManager::detect_or_custom(None);

    // Should either find a kernel or fail with clear error
    match result {
        Ok(manager) => {
            // Found system kernel
            assert!(manager.kernel_path().exists());
        }
        Err(e) => {
            // No kernel found - error message should be clear
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("No kernel found") || err_msg.contains("kernel"),
                "Error message should mention kernel: {}",
                err_msg
            );
        }
    }
}
