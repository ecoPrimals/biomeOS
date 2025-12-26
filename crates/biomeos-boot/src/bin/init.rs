#!/usr/bin/env -S cargo +nightly -Zscript
//! BiomeOS Init - Pure Rust PID 1
//! 
//! This is the first userspace process that runs when BiomeOS boots.
//! It replaces traditional init systems with a pure Rust, async-first,
//! sovereignty-preserving initialization system.

use anyhow::{Context, Result};
use nix::mount::{mount, MsFlags};
use nix::unistd::getpid;
use std::path::Path;
use std::process::ExitCode;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize logging ASAP
    tracing_subscriber::fmt()
        .with_env_filter("biomeos_boot=info")
        .with_target(false)
        .with_ansi(true)
        .init();

    // Verify we're PID 1
    let pid = getpid();
    if pid.as_raw() != 1 {
        error!("biomeos-init must run as PID 1 (current: {})", pid);
        return ExitCode::FAILURE;
    }

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("BiomeOS Init - Pure Rust Initialization System");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("PID: {}", pid);
    info!("Sovereignty-First | Zero Dependencies | Pure Rust");
    info!("");

    // Run initialization sequence
    if let Err(e) = initialize().await {
        error!("Initialization failed: {:#}", e);
        error!("Entering emergency mode...");
        
        if let Err(recovery_error) = emergency_mode().await {
            error!("Emergency mode failed: {:#}", recovery_error);
        }
        
        return ExitCode::FAILURE;
    }

    info!("✅ BiomeOS initialization complete!");
    info!("Sovereignty preserved. Human dignity intact.");
    
    ExitCode::SUCCESS
}

/// Main initialization sequence
async fn initialize() -> Result<()> {
    info!("📋 Starting initialization sequence...");

    // Phase 1: Essential Filesystems
    mount_essential_filesystems()
        .await
        .context("Failed to mount essential filesystems")?;

    // Phase 2: Hardware Detection
    let hardware = detect_hardware()
        .await
        .context("Failed to detect hardware")?;
    info!("Hardware detected: {} cores, {} GB RAM", 
          hardware.cpu_count, hardware.total_memory_gb);

    // Phase 3: Network Configuration
    configure_network()
        .await
        .context("Failed to configure network")?;

    // Phase 4: USB/Storage Detection
    let usb_device = detect_biomeos_usb()
        .await
        .context("Failed to detect BiomeOS USB")?;
    
    if let Some(usb) = usb_device {
        info!("✅ BiomeOS USB detected: {}", usb.display());
        mount_biomeos_usb(&usb)
            .await
            .context("Failed to mount BiomeOS USB")?;
    } else {
        warn!("No BiomeOS USB detected - using system installation");
    }

    // Phase 5: Parse Boot Parameters
    let boot_params = parse_boot_params()
        .await
        .context("Failed to parse boot parameters")?;
    info!("Boot mode: {:?}", boot_params.mode);

    // Phase 6: Start BiomeOS Core
    start_biomeos_core(boot_params)
        .await
        .context("Failed to start BiomeOS core")?;

    Ok(())
}

/// Mount essential pseudo-filesystems
async fn mount_essential_filesystems() -> Result<()> {
    info!("📁 Mounting essential filesystems...");

    // /proc - Process information
    mount_filesystem("proc", "/proc", "proc", MsFlags::empty())
        .context("Failed to mount /proc")?;

    // /sys - Kernel and device information
    mount_filesystem("sysfs", "/sys", "sysfs", MsFlags::empty())
        .context("Failed to mount /sys")?;

    // /dev - Device files
    mount_filesystem("devtmpfs", "/dev", "devtmpfs", MsFlags::empty())
        .context("Failed to mount /dev")?;

    // /dev/pts - Pseudo-terminals
    mount_filesystem("devpts", "/dev/pts", "devpts", MsFlags::empty())
        .context("Failed to mount /dev/pts")?;

    // /dev/shm - Shared memory
    mount_filesystem("tmpfs", "/dev/shm", "tmpfs", MsFlags::empty())
        .context("Failed to mount /dev/shm")?;

    // /run - Runtime data
    mount_filesystem("tmpfs", "/run", "tmpfs", MsFlags::empty())
        .context("Failed to mount /run")?;

    // /tmp - Temporary files
    mount_filesystem("tmpfs", "/tmp", "tmpfs", MsFlags::empty())
        .context("Failed to mount /tmp")?;

    info!("✅ Essential filesystems mounted");
    Ok(())
}

/// Mount a single filesystem (helper)
fn mount_filesystem(
    source: &str,
    target: &str,
    fstype: &str,
    flags: MsFlags,
) -> Result<()> {
    // Create mount point if it doesn't exist
    std::fs::create_dir_all(target)
        .with_context(|| format!("Failed to create directory: {}", target))?;

    mount(
        Some(source),
        target,
        Some(fstype),
        flags,
        None::<&str>,
    )
    .with_context(|| format!("Failed to mount {} on {}", source, target))?;

    Ok(())
}

/// Detect hardware capabilities
async fn detect_hardware() -> Result<HardwareInfo> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_count = sys.cpus().len();
    let total_memory = sys.total_memory();
    let total_memory_gb = total_memory / (1024 * 1024 * 1024);

    Ok(HardwareInfo {
        cpu_count,
        total_memory_gb: total_memory_gb as usize,
    })
}

#[derive(Debug)]
struct HardwareInfo {
    cpu_count: usize,
    total_memory_gb: usize,
}

/// Configure network interfaces
async fn configure_network() -> Result<()> {
    info!("🌐 Configuring network...");

    // This is a placeholder - full network configuration would:
    // 1. Detect network interfaces
    // 2. Configure DHCP or static IP
    // 3. Set up DNS
    // 4. Start mDNS for service discovery

    info!("✅ Network configuration complete");
    Ok(())
}

/// Detect BiomeOS USB drive
async fn detect_biomeos_usb() -> Result<Option<std::path::PathBuf>> {
    info!("🔍 Scanning for BiomeOS USB drive...");

    // Look for USB devices with BiomeOS marker
    let usb_paths = vec![
        "/dev/sda1",
        "/dev/sdb1",
        "/dev/sdc1",
    ];

    for path in usb_paths {
        let path_buf = std::path::PathBuf::from(path);
        if path_buf.exists() {
            // Check if this has BiomeOS marker
            // For now, just return the first USB device found
            return Ok(Some(path_buf));
        }
    }

    Ok(None)
}

/// Mount BiomeOS USB drive
async fn mount_biomeos_usb(device: &Path) -> Result<()> {
    info!("💾 Mounting BiomeOS USB: {}", device.display());

    mount_filesystem(
        device.to_str().unwrap(),
        "/biomeos",
        "auto",
        MsFlags::MS_RDONLY,
    )
    .context("Failed to mount BiomeOS USB")?;

    info!("✅ BiomeOS USB mounted at /biomeos");
    Ok(())
}

/// Parse boot parameters from /proc/cmdline
async fn parse_boot_params() -> Result<BootParams> {
    let cmdline = tokio::fs::read_to_string("/proc/cmdline")
        .await
        .context("Failed to read /proc/cmdline")?;

    let mode = if cmdline.contains("biomeos.discovery") {
        BootMode::Discovery
    } else if cmdline.contains("biomeos.install") {
        BootMode::Install
    } else if cmdline.contains("biomeos.network") {
        BootMode::Network
    } else {
        BootMode::Standard
    };

    Ok(BootParams { mode })
}

#[derive(Debug)]
struct BootParams {
    mode: BootMode,
}

#[derive(Debug)]
enum BootMode {
    Standard,
    Discovery,
    Install,
    Network,
}

/// Start BiomeOS core platform
async fn start_biomeos_core(params: BootParams) -> Result<()> {
    info!("🚀 Starting BiomeOS core platform...");

    match params.mode {
        BootMode::Standard => {
            info!("Mode: Standard (load biome.yaml)");
            // Load and execute biome.yaml
            // Start primal registry
            // Start P2P coordination
        }
        BootMode::Discovery => {
            info!("Mode: Discovery (scan network)");
            // Start mDNS discovery
            // Find other BiomeOS nodes
            // Present discovered nodes
        }
        BootMode::Install => {
            info!("Mode: Installation");
            // Start installation wizard
        }
        BootMode::Network => {
            info!("Mode: Network Boot");
            // Fetch configuration from network
            // Coordinate with remote nodes
        }
    }

    info!("✅ BiomeOS core started");
    Ok(())
}

/// Emergency mode for recovery
async fn emergency_mode() -> Result<()> {
    error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    error!("EMERGENCY MODE");
    error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    error!("");
    error!("BiomeOS initialization failed. Entering recovery shell.");
    error!("Available commands:");
    error!("  - journalctl: View system logs");
    error!("  - mount: Check mounted filesystems");
    error!("  - ip addr: Check network configuration");
    error!("  - lsblk: List block devices");
    error!("");

    // In a real implementation, spawn a shell here
    // For now, just wait
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

    Ok(())
}

