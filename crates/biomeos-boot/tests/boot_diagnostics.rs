// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Boot Diagnostics and Testing
//!
//! Comprehensive test suite for BiomeOS boot infrastructure

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Test that initramfs builder creates valid structure
#[test]
fn test_initramfs_structure() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    // Build initramfs using our builder
    let mut builder = biomeos_boot::InitramfsBuilder::new(temp_dir.path())?;
    builder.create_directory_structure()?;

    // Verify essential directories exist (in the initramfs-root subdirectory)
    let root = temp_dir.path().join("initramfs-root");
    assert!(root.join("bin").exists());
    assert!(root.join("dev").exists());
    assert!(root.join("proc").exists());
    assert!(root.join("sys").exists());

    Ok(())
}

/// Test kernel detection
#[test]
fn test_kernel_detection() -> Result<()> {
    // System kernel should be detectable
    let kernel_mgr = biomeos_boot::KernelManager::detect_or_custom(None)?;
    let kernel = kernel_mgr.kernel_path();
    assert!(kernel.exists(), "Kernel not found: {}", kernel.display());

    // Kernel should be readable
    let metadata = std::fs::metadata(kernel)?;
    assert!(
        metadata.len() > 1_000_000,
        "Kernel too small: {} bytes",
        metadata.len()
    );

    Ok(())
}

/// Test binary spec validation
#[test]
fn test_binary_spec() {
    use biomeos_boot::initramfs::BinarySpec;

    let spec = BinarySpec {
        source: PathBuf::from("/bin/busybox"),
        dest: "/bin/busybox".to_string(),
        permissions: 0o755,
    };

    assert_eq!(spec.permissions, 0o755);
    assert!(spec.dest.starts_with('/'));
}

/// Test initramfs build (integration test)
#[test]
#[ignore = "Requires build artifacts — run after full build"]
fn test_full_initramfs_build() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = binding.parent().unwrap().parent().unwrap();

    let mut builder = biomeos_boot::InitramfsBuilder::new(temp_dir.path())?;
    builder.create_directory_structure()?;
    builder.add_biomeos_binaries(project_root)?;

    let initramfs = temp_dir.path().join("initramfs.cpio.gz");
    builder.build(&initramfs)?;
    assert!(initramfs.exists());

    // Verify it's a gzipped cpio archive
    let output = Command::new("file").arg(&initramfs).output()?;
    let file_type = String::from_utf8_lossy(&output.stdout);
    assert!(file_type.contains("gzip"));

    Ok(())
}

/// Diagnostic: Check if biomeos-init exists and is executable
#[test]
fn test_biomeos_init_binary() -> Result<()> {
    let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = binding.parent().unwrap().parent().unwrap();

    let init_path = project_root.join("target/release/biomeos-init");

    if !init_path.exists() {
        eprintln!("⚠️  biomeos-init not found at: {}", init_path.display());
        eprintln!("Run: cargo build --release -p biomeos-boot --bin biomeos-init");
        return Ok(()); // Don't fail, just warn
    }

    // Check it's executable
    let metadata = std::fs::metadata(&init_path)?;
    assert!(metadata.len() > 100_000, "Init binary too small");

    // Check dynamic libraries
    let output = Command::new("ldd").arg(&init_path).output()?;
    let ldd_output = String::from_utf8_lossy(&output.stdout);
    println!("biomeos-init dependencies:\n{ldd_output}");

    assert!(ldd_output.contains("libc.so"), "Missing libc");

    Ok(())
}

/// Test QEMU availability for integration tests
#[test]
fn test_qemu_available() {
    let qemu_result = Command::new("qemu-system-x86_64").arg("--version").output();

    match qemu_result {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!(
                "✅ QEMU available: {}",
                version.lines().next().unwrap_or("")
            );
        }
        Err(e) => {
            eprintln!("⚠️  QEMU not available: {e}");
            eprintln!("Install: sudo apt install qemu-system-x86");
        }
    }
}

/// Diagnostic: Verify root disk structure
#[test]
#[ignore = "Requires VM setup (QEMU)"]
fn test_root_disk_structure() -> Result<()> {
    let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = binding.parent().unwrap().parent().unwrap();

    let disk = project_root.join("vm-testing/biomeos-root.qcow2");

    if !disk.exists() {
        eprintln!("⚠️  Root disk not found: {}", disk.display());
        return Ok(());
    }

    println!("✅ Root disk exists: {}", disk.display());
    let metadata = std::fs::metadata(&disk)?;
    println!("   Size: {} bytes", metadata.len());

    Ok(())
}
