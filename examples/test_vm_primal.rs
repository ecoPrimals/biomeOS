// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS VM Testing Example
//!
//! Demonstrates testing a single BiomeOS VM with primals.
//! Replaces test-primals-vm.sh with pure Rust implementation.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

// Note: This example demonstrates the pattern for VM testing.
// The biomeos-deploy crate provides the full implementation.
// For now, we'll use direct QEMU commands until the crate is fully integrated.

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🧪 BiomeOS VM Test - Single VM with Primals");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Configuration
    let vm_name = "test-vm-primals";
    let disk_image = PathBuf::from("vm-testing/biomeos-with-primals.qcow2");
    let serial_log = PathBuf::from("vm-testing/test-vm.log");

    // Validate disk image exists
    if !disk_image.exists() {
        warn!("⚠️  Disk image not found: {}", disk_image.display());
        warn!("   Build it first with: cargo run --release -p biomeos-boot --bin biomeos-rootfs");
        return Ok(());
    }

    info!("📦 Configuration:");
    info!("   VM: {}", vm_name);
    info!("   Disk: {}", disk_image.display());
    info!("   Log: {}", serial_log.display());
    info!("");

    // Launch VM using QEMU
    info!("🚀 Launching VM with QEMU...");
    info!("   (Press Ctrl+C to stop)");
    info!("");

    let status = tokio::process::Command::new("qemu-system-x86_64")
        .args([
            "-name",
            vm_name,
            "-drive",
            &format!("file={},format=qcow2,if=ide", disk_image.display()),
            "-m",
            "2G",
            "-smp",
            "2",
            "-enable-kvm",
            "-serial",
            &format!("file:{}", serial_log.display()),
            "-nographic",
        ])
        .status()
        .await?;

    if status.success() {
        info!("✅ VM test complete!");
        info!("   Check serial log: {}", serial_log.display());
    } else {
        warn!("⚠️  VM exited with status: {:?}", status.code());
    }

    Ok(())
}
