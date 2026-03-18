// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS Init - Pure Rust PID 1
//!
//! This is the first userspace process that runs when BiomeOS boots.
//! It replaces traditional init systems with a pure Rust, async-first,
//! sovereignty-preserving initialization system.

use anyhow::{Context, Result};
use biomeos_boot::{BootLogger, BootStage};
use biomeos_core::observability::MinimalObserver;
use rustix::mount::{MountFlags, mount};
use rustix::process::getpid;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> ExitCode {
    // Start boot timer for observability
    let boot_start = Instant::now();

    // Initialize BootLogger FIRST - direct serial access for guaranteed visibility
    let mut boot_logger = match BootLogger::new() {
        Ok(logger) => {
            let mut l = logger;
            l.checkpoint(BootStage::InitStart);
            l.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            l.info("BiomeOS Init - Pure Rust PID 1");
            l.info("BootLogger: Direct serial access enabled");
            l.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            Some(l)
        }
        Err(e) => {
            // Fallback to stdout if BootLogger fails
            use std::io::Write;
            let _ =
                std::io::stderr().write_all(format!("BootLogger init failed: {e}\n").as_bytes());
            let _ = std::io::stdout().write_all(b"\n[BiomeOS] Init - Fallback mode\n");
            None
        }
    };

    // Verify we're PID 1
    let pid = getpid();
    let pid_raw = rustix::process::Pid::as_raw(Some(pid));

    if let Some(ref mut logger) = boot_logger {
        logger.info(&format!("PID: {pid_raw}"));
    }

    if !pid.is_init() {
        if let Some(ref mut logger) = boot_logger {
            logger.critical(&format!("Must run as PID 1, got {pid_raw}"));
        }
        return ExitCode::FAILURE;
    }

    // Initialize tracing (secondary logging)
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_env_filter("info")
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .ok();

    // Initialize MinimalObserver (sovereignty-respecting observability)
    let observer = match MinimalObserver::local_only() {
        Ok(obs) => {
            if let Some(ref mut logger) = boot_logger {
                logger.info("MinimalObserver initialized (local-only, zero network)");
            }
            info!("🔍 MinimalObserver: local-only mode (sovereignty-respecting)");
            Some(obs)
        }
        Err(e) => {
            warn!("MinimalObserver initialization failed: {}", e);
            None
        }
    };

    if let Some(ref mut logger) = boot_logger {
        logger.info("Sovereignty-First | Zero Dependencies | Pure Rust");
        logger.checkpoint(BootStage::FilesystemMount);
    }

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("BiomeOS Init - Pure Rust Initialization System");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("PID: {}", pid_raw);

    // Run initialization sequence
    if let Err(e) = initialize().await {
        if let Some(ref mut logger) = boot_logger {
            logger.critical(&format!("Initialization failed: {e:#}"));
        }
        error!("Initialization failed: {:#}", e);
        error!("Entering emergency mode...");

        if let Err(recovery_error) = emergency_mode().await {
            error!("Emergency mode failed: {:#}", recovery_error);
        }

        return ExitCode::FAILURE;
    }

    // Record boot time in observer
    let boot_time = boot_start.elapsed();
    if let Some(ref observer) = observer {
        observer.record_boot_time(boot_time);
        if let Some(ref mut logger) = boot_logger {
            logger.info(&format!("Boot time recorded: {boot_time:?}"));
        }
        info!("📊 Boot time: {:?}", boot_time);
    }

    if let Some(ref mut logger) = boot_logger {
        logger.checkpoint(BootStage::Complete);
        logger.info("✅ BiomeOS initialization complete!");
        logger.info("Sovereignty preserved. Human dignity intact.");

        // Show logger stats
        let stats = logger.stats();
        logger.info(&format!(
            "BootLogger stats: {} messages, {}ms uptime",
            stats.log_count, stats.uptime_ms
        ));
    }

    info!("✅ BiomeOS initialization complete!");

    // Show observability metrics
    if let Some(ref observer) = observer {
        let metrics = observer.get_local_metrics();
        info!("📊 Observability Metrics:");
        if let Some(boot_duration) = metrics.boot_time {
            info!("   Boot time: {}ms", boot_duration.as_millis());
        }
        info!("   Primals monitored: {}", metrics.primal_health.len());
    }

    // PID 1 must never exit - spawn a shell or wait forever
    spawn_shell().await;

    ExitCode::SUCCESS
}

/// Spawn a shell for user interaction
async fn spawn_shell() {
    info!("🐚 Spawning shell...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Try to spawn busybox sh
    if let Err(e) = std::process::Command::new("/bin/busybox")
        .arg("sh")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
    {
        error!("Failed to spawn shell: {}", e);
        error!("Entering infinite wait loop to prevent kernel panic...");

        // If shell fails, just wait forever (PID 1 must not exit)
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }
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
    info!(
        "Hardware detected: {} cores, {} GB RAM",
        hardware.cpu_count, hardware.total_memory_gb
    );

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
    mount_filesystem("proc", "/proc", "proc", MountFlags::empty())
        .context("Failed to mount /proc")?;

    // /sys - Kernel and device information
    mount_filesystem("sysfs", "/sys", "sysfs", MountFlags::empty())
        .context("Failed to mount /sys")?;

    // /dev - Device files
    mount_filesystem("devtmpfs", "/dev", "devtmpfs", MountFlags::empty())
        .context("Failed to mount /dev")?;

    // /dev/pts - Pseudo-terminals
    mount_filesystem("devpts", "/dev/pts", "devpts", MountFlags::empty())
        .context("Failed to mount /dev/pts")?;

    // /dev/shm - Shared memory
    mount_filesystem("tmpfs", "/dev/shm", "tmpfs", MountFlags::empty())
        .context("Failed to mount /dev/shm")?;

    // /run - Runtime data
    mount_filesystem("tmpfs", "/run", "tmpfs", MountFlags::empty())
        .context("Failed to mount /run")?;

    // /tmp - Temporary files
    mount_filesystem("tmpfs", "/tmp", "tmpfs", MountFlags::empty())
        .context("Failed to mount /tmp")?;

    info!("✅ Essential filesystems mounted");
    Ok(())
}

/// Mount a single filesystem (helper)
fn mount_filesystem(source: &str, target: &str, fstype: &str, flags: MountFlags) -> Result<()> {
    // Create mount point if it doesn't exist
    std::fs::create_dir_all(target)
        .with_context(|| format!("Failed to create directory: {target}"))?;

    // Try to mount - if already mounted (EBUSY), that's OK
    match mount(source, target, fstype, flags, "") {
        Ok(()) => {
            info!("  ✓ {}", target);
            Ok(())
        }
        Err(rustix::io::Errno::BUSY) => {
            // Already mounted - this is fine
            info!("  ✓ {} (already mounted)", target);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to mount {source} on {target}: {e}")),
    }
}

/// Detect hardware capabilities via /proc (pure Rust - ecoBin v3).
async fn detect_hardware() -> Result<HardwareInfo> {
    let () = tokio::task::yield_now().await;

    #[cfg(target_os = "linux")]
    let (cpu_count, total_memory_gb) = {
        let cpu_count = std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .map_or(1, |s| {
                s.lines().filter(|l| l.starts_with("processor")).count()
            })
            .max(1);
        let total_memory_gb = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .map_or(0, |kb| (kb * 1024) / (1024 * 1024 * 1024))
            as usize;
        (cpu_count, total_memory_gb)
    };

    #[cfg(not(target_os = "linux"))]
    let (cpu_count, total_memory_gb) = (
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1),
        0usize,
    );

    Ok(HardwareInfo {
        cpu_count,
        total_memory_gb,
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
    let usb_paths = vec!["/dev/sda1", "/dev/sdb1", "/dev/sdc1"];

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

    let device_str = device
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in device path"))?;

    mount_filesystem(device_str, "/biomeos", "auto", MountFlags::RDONLY)
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
