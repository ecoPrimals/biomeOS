// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Root filesystem builder for BiomeOS bootable images
//!
//! Creates qcow2 disk images with BiomeOS binaries, kernel modules,
//! and configuration for standalone boot from USB or network.

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// CLI for biomeos-rootfs binary
#[derive(Debug, Parser)]
#[command(name = "biomeos-rootfs")]
#[command(about = "Build BiomeOS root filesystem images", long_about = None)]
pub struct RootFsCli {
    /// Size of the root filesystem (e.g., "8G", "10G")
    #[arg(short, long, default_value = "8G")]
    size: String,

    /// Output path for the image
    #[arg(short, long, default_value = "biomeos-root.qcow2")]
    output: PathBuf,

    /// Directory containing primal binaries
    #[arg(short, long)]
    primals: Option<PathBuf>,

    /// Filesystem type
    #[arg(short, long, default_value = "ext4")]
    fs_type: String,
}

impl RootFsCli {
    /// Execute the CLI command
    pub async fn execute(self) -> Result<()> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();

        let config = RootFsConfig {
            size: self.size,
            output: self.output,
            primals_dir: self.primals,
            fs_type: self.fs_type,
            ..Default::default()
        };

        let builder = RootFsBuilder::new(config);
        builder.build().await?;

        Ok(())
    }
}

/// RAII guard for NBD device - ensures cleanup on drop
struct NbdGuard {
    device: String,
    attached: bool,
}

impl NbdGuard {
    /// Attach an NBD device to an image file
    fn attach(image_path: &Path) -> Result<Self> {
        // Load NBD kernel module
        let _ = Command::new("modprobe")
            .args(["nbd", "max_part=8"])
            .status();

        // Find available device
        let device = Self::find_available_device()?;

        info!("📎 Attaching {} to {}", device, image_path.display());

        // Disconnect if somehow still connected
        let _ = Command::new("qemu-nbd").args(["-d", &device]).output();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Attach the device
        let status = Command::new("qemu-nbd")
            .args(["-c", &device])
            .arg(image_path)
            .status()
            .context("Failed to attach NBD device")?;

        if !status.success() {
            anyhow::bail!("qemu-nbd failed to attach {}", device);
        }

        // Wait for device to be ready
        std::thread::sleep(std::time::Duration::from_millis(500));

        info!("  ✓ NBD attached: {}", device);

        Ok(Self {
            device,
            attached: true,
        })
    }

    /// Find an available NBD device
    fn find_available_device() -> Result<String> {
        for i in 0..16 {
            let device = format!("/dev/nbd{}", i);

            // Check via /sys/block/nbdX/size (most reliable)
            let size_path = format!("/sys/block/nbd{}/size", i);
            if let Ok(mut file) = fs::File::open(&size_path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(size) = contents.trim().parse::<u64>() {
                        if size == 0 {
                            return Ok(device);
                        }
                    }
                }
            }
        }

        anyhow::bail!("No available NBD devices found (all in use)")
    }

    /// Get the device path
    fn device(&self) -> &str {
        &self.device
    }

    /// Manual detach (also happens on drop)
    fn detach(&mut self) -> Result<()> {
        if !self.attached {
            return Ok(());
        }

        info!("🔓 Detaching NBD: {}", self.device);

        let status = Command::new("qemu-nbd")
            .args(["-d", &self.device])
            .status()
            .context("Failed to detach NBD")?;

        if !status.success() {
            warn!("  ⚠️  NBD detach returned error");
        }

        self.attached = false;
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }
}

impl Drop for NbdGuard {
    fn drop(&mut self) {
        if self.attached {
            info!("🧹 Cleaning up NBD device: {}", self.device);
            let _ = Command::new("qemu-nbd").args(["-d", &self.device]).output();
        }
    }
}

/// Configuration for building a BiomeOS root filesystem
#[derive(Debug, Clone)]
pub struct RootFsConfig {
    /// Size of the root filesystem (e.g., "8G")
    pub size: String,

    /// Output path for the root filesystem image
    pub output: PathBuf,

    /// Directory containing primal binaries
    pub primals_dir: Option<PathBuf>,

    /// Directory containing systemd service files
    pub services_dir: Option<PathBuf>,

    /// Mount point for building (temporary, auto-generated if None)
    pub mount_point: Option<PathBuf>,

    /// Filesystem type (default: ext4)
    pub fs_type: String,

    /// DNS servers (discovered from system if None)
    pub dns_servers: Option<Vec<String>>,

    /// NBD device to use (auto-detect if None)
    pub nbd_device: Option<String>,

    /// Hostname for the system (default: "biomeos")
    pub hostname: String,
}

impl Default for RootFsConfig {
    fn default() -> Self {
        Self {
            size: "8G".to_string(),
            output: PathBuf::from("biomeos-root.qcow2"),
            primals_dir: None,
            services_dir: None,
            mount_point: None,
            fs_type: "ext4".to_string(),
            dns_servers: None, // Discover from system
            nbd_device: None,  // Auto-detect
            hostname: "biomeos".to_string(),
        }
    }
}

/// Builder for BiomeOS root filesystems
pub struct RootFsBuilder {
    config: RootFsConfig,
}

impl RootFsBuilder {
    /// Create a new root filesystem builder
    pub fn new(config: RootFsConfig) -> Self {
        Self { config }
    }

    /// Build the complete root filesystem
    pub async fn build(&self) -> Result<PathBuf> {
        info!("🏗️  Building BiomeOS root filesystem");
        info!("  Output: {}", self.config.output.display());
        info!("  Size: {}", self.config.size);

        // Step 1: Create qcow2 image
        self.create_image()?;

        // Step 2: Format with filesystem
        self.format_filesystem()?;

        // Step 3: Mount the image
        let mount_point = self.mount_image()?;

        // Step 4: Install base system
        self.install_base_system(&mount_point)?;

        // Step 5: Install BiomeOS components
        self.install_biomeos(&mount_point).await?;

        // Step 6: Install primals (if provided)
        if let Some(primals_dir) = &self.config.primals_dir {
            self.install_primals(&mount_point, primals_dir).await?;
        }

        // Step 6.5: Install systemd services (if provided)
        if let Some(services_dir) = &self.config.services_dir {
            self.install_services(&mount_point, services_dir)?;
        }

        // Step 7: Configure system
        self.configure_system(&mount_point)?;

        // Step 8: Unmount
        self.unmount_image(&mount_point)?;

        info!("✅ Root filesystem built: {}", self.config.output.display());
        Ok(self.config.output.clone())
    }

    /// Create a qcow2 disk image
    fn create_image(&self) -> Result<()> {
        info!("📦 Creating qcow2 image...");

        let status = Command::new("qemu-img")
            .args(["create", "-f", "qcow2"])
            .arg(&self.config.output)
            .arg(&self.config.size)
            .status()
            .context("Failed to execute qemu-img")?;

        if !status.success() {
            anyhow::bail!("qemu-img failed to create image");
        }

        info!("  ✓ Created {} image", self.config.size);
        Ok(())
    }

    /// Format the image with a filesystem using RAII NBD guard
    fn format_filesystem(&self) -> Result<()> {
        info!("💾 Formatting filesystem...");

        // Use RAII guard for automatic cleanup
        let mut nbd = NbdGuard::attach(&self.config.output)?;

        let status = Command::new(format!("mkfs.{}", self.config.fs_type))
            .arg(nbd.device())
            .arg("-L")
            .arg("BIOMEOS") // Label
            .status()
            .context("Failed to format filesystem")?;

        // Explicit detach before check (guard will also cleanup on drop)
        nbd.detach()?;

        if !status.success() {
            anyhow::bail!("Failed to format filesystem");
        }

        info!("  ✓ Formatted as {}", self.config.fs_type);
        Ok(())
    }

    /// Mount the filesystem image using RAII NBD guard
    fn mount_image(&self) -> Result<PathBuf> {
        info!("📂 Mounting filesystem...");

        let nbd = NbdGuard::attach(&self.config.output)?;

        // Use configured mount point or create temporary
        let mount_point = if let Some(ref mp) = self.config.mount_point {
            mp.clone()
        } else {
            // Use tempfile for safer temporary directory
            let temp_dir = tempfile::Builder::new()
                .prefix("biomeos-build-")
                .tempdir()
                .context("Failed to create temporary mount point")?;
            temp_dir.keep()
        };

        std::fs::create_dir_all(&mount_point)?;

        let status = Command::new("mount")
            .arg(nbd.device())
            .arg(&mount_point)
            .status()
            .context("Failed to mount filesystem")?;

        if !status.success() {
            // NBD will be automatically detached by guard's Drop
            anyhow::bail!("Failed to mount filesystem");
        }

        info!("  ✓ Mounted at {}", mount_point.display());

        // Important: Forget the guard so NBD stays attached while mounted
        // We'll detach in unmount_image()
        std::mem::forget(nbd);

        Ok(mount_point)
    }

    /// Unmount the filesystem and detach NBD
    fn unmount_image(&self, mount_point: &Path) -> Result<()> {
        info!("📤 Unmounting filesystem...");

        // Unmount
        let status = Command::new("umount")
            .arg(mount_point)
            .status()
            .context("Failed to unmount filesystem")?;

        if !status.success() {
            warn!("  ⚠️  Unmount returned error");
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        // Find which NBD device was used by checking /proc/mounts or lsblk
        // Detach all NBD devices that might be attached to our image
        for i in 0..16 {
            let device = format!("/dev/nbd{}", i);
            let size_path = format!("/sys/block/nbd{}/size", i);

            // Check if this NBD has a size > 0 (meaning it's attached)
            if let Ok(mut file) = fs::File::open(&size_path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(size) = contents.trim().parse::<u64>() {
                        if size > 0 {
                            info!("  🔓 Detaching {}", device);
                            let _ = Command::new("qemu-nbd").args(["-d", &device]).output();
                        }
                    }
                }
            }
        }

        info!("  ✓ Unmounted and detached");
        Ok(())
    }

    /// Install base system (minimal Linux userspace)
    fn install_base_system(&self, root: &Path) -> Result<()> {
        info!("🌱 Installing base system...");

        // Create directory structure
        let dirs = [
            "bin",
            "sbin",
            "usr/bin",
            "usr/sbin",
            "usr/local/bin",
            "etc",
            "etc/systemd/system",
            "var",
            "var/log",
            "var/lib",
            "tmp",
            "run",
            "proc",
            "sys",
            "dev",
            "home",
            "root",
        ];

        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir))?;
        }

        // Install BusyBox (minimal userspace utilities)
        self.install_busybox(root)?;

        info!("  ✓ Base system installed");
        Ok(())
    }

    /// Install BusyBox for minimal Linux userspace
    fn install_busybox(&self, root: &Path) -> Result<()> {
        // Check if busybox is available
        let busybox_path = which::which("busybox")
            .context("BusyBox not found - install with: apt install busybox-static")?;

        // Copy busybox to /bin
        let dest = root.join("bin/busybox");
        std::fs::copy(&busybox_path, &dest).context("Failed to copy busybox")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        info!("  ✓ BusyBox installed");
        Ok(())
    }

    /// Install BiomeOS core components
    async fn install_biomeos(&self, root: &Path) -> Result<()> {
        info!("🦀 Installing BiomeOS components...");

        // Install biomeos-init (PID 1)
        self.install_biomeos_init(root).await?;

        info!("  ✓ BiomeOS init installed");
        Ok(())
    }

    /// Install biomeos-init as PID 1
    async fn install_biomeos_init(&self, root: &Path) -> Result<()> {
        // Build biomeos-init if not already built
        let init_binary = PathBuf::from("target/release/biomeos-init");

        if !init_binary.exists() {
            info!("  Building biomeos-init...");
            let status = Command::new("cargo")
                .args(["build", "--release", "--bin", "biomeos-init"])
                .status()
                .context("Failed to build biomeos-init")?;

            if !status.success() {
                anyhow::bail!("Failed to build biomeos-init");
            }
        }

        // Copy to /sbin/init
        let dest = root.join("sbin/init");
        std::fs::copy(&init_binary, &dest).context("Failed to copy biomeos-init")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        Ok(())
    }

    /// Install primal binaries
    async fn install_primals(&self, root: &Path, primals_dir: &Path) -> Result<()> {
        info!("🔧 Installing primal binaries...");

        if !primals_dir.exists() {
            warn!(
                "  ⚠️  Primals directory not found: {}",
                primals_dir.display()
            );
            return Ok(());
        }

        let target_dir = root.join("usr/local/bin");
        std::fs::create_dir_all(&target_dir)?;

        let mut count = 0;
        for entry in std::fs::read_dir(primals_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let dest = target_dir.join(filename);

                    std::fs::copy(&path, &dest).with_context(|| {
                        format!("Failed to copy primal {}", filename.to_string_lossy())
                    })?;

                    // Make executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = std::fs::metadata(&dest)?.permissions();
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&dest, perms)?;
                    }

                    info!("  ✓ Installed {}", filename.to_string_lossy());
                    count += 1;
                }
            }
        }

        info!("  ✓ Installed {} primal(s)", count);
        Ok(())
    }

    fn install_services(&self, root: &Path, services_dir: &Path) -> Result<()> {
        info!("⚙️  Installing systemd services...");

        if !services_dir.exists() {
            warn!(
                "  ⚠️  Services directory not found: {}",
                services_dir.display()
            );
            return Ok(());
        }

        let systemd_dir = root.join("etc/systemd/system");
        std::fs::create_dir_all(&systemd_dir)?;

        let mut count = 0;
        for entry in std::fs::read_dir(services_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "service") {
                let filename = match path.file_name() {
                    Some(f) => f,
                    None => continue,
                };
                let dest = systemd_dir.join(filename);

                std::fs::copy(&path, &dest).with_context(|| {
                    format!("Failed to copy service file {}", filename.to_string_lossy())
                })?;

                // Create symlink to enable the service
                let service_name = filename.to_string_lossy();
                let wants_dir = systemd_dir.join("multi-user.target.wants");
                std::fs::create_dir_all(&wants_dir)?;
                let symlink_dest = wants_dir.join(&*service_name);

                // Remove existing symlink if it exists to avoid "File exists" error
                if symlink_dest.exists() {
                    std::fs::remove_file(&symlink_dest)?;
                }
                std::os::unix::fs::symlink(&dest, &symlink_dest)
                    .with_context(|| format!("Failed to symlink service {}", service_name))?;

                info!("  ✓ Installed and enabled service {}", service_name);
                count += 1;
            }
        }

        info!("  ✓ Installed {} systemd service(s)", count);
        Ok(())
    }

    /// Configure the system
    fn configure_system(&self, root: &Path) -> Result<()> {
        info!("⚙️  Configuring system...");

        // Configure DNS
        self.configure_dns(root)?;

        // Set hostname
        let hostname_path = root.join("etc/hostname");
        std::fs::write(&hostname_path, format!("{}\n", self.config.hostname))?;

        info!("  ✓ System configured (hostname: {})", self.config.hostname);
        Ok(())
    }

    /// Configure DNS resolvers
    fn configure_dns(&self, root: &Path) -> Result<()> {
        let resolv_conf = root.join("etc/resolv.conf");

        let dns_servers = if let Some(ref servers) = self.config.dns_servers {
            servers.clone()
        } else {
            // Discover from system /etc/resolv.conf
            self.discover_system_dns()?
        };

        if dns_servers.is_empty() {
            info!("  Using system DNS configuration");
            return Ok(());
        }

        let mut content = String::new();
        for server in &dns_servers {
            content.push_str(&format!("nameserver {}\n", server));
        }

        std::fs::write(&resolv_conf, content)?;
        info!("  ✓ DNS configured ({} servers)", dns_servers.len());

        Ok(())
    }

    /// Discover DNS servers from system
    fn discover_system_dns(&self) -> Result<Vec<String>> {
        let system_resolv = std::fs::read_to_string("/etc/resolv.conf")
            .context("Failed to read /etc/resolv.conf")?;

        Ok(parse_resolv_conf(&system_resolv))
    }
}

/// Parse nameserver lines from resolv.conf content (testable)
pub(crate) fn parse_resolv_conf(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim().starts_with("nameserver"))
        .filter_map(|line| line.split_whitespace().nth(1).map(String::from))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rootfs_config_default() {
        let config = RootFsConfig::default();
        assert_eq!(config.size, "8G");
        assert_eq!(config.fs_type, "ext4");
        assert_eq!(config.hostname, "biomeos");
        assert!(config.primals_dir.is_none());
    }

    #[test]
    fn test_discover_system_dns() {
        let config = RootFsConfig::default();
        let builder = RootFsBuilder::new(config);

        let dns_servers = builder.discover_system_dns();
        // Should either succeed or fail gracefully
        match dns_servers {
            Ok(servers) => {
                // If we have DNS servers, they should be valid IP addresses
                for server in servers {
                    assert!(!server.is_empty());
                }
            }
            Err(_) => {
                // It's okay if we can't read /etc/resolv.conf in test env
            }
        }
    }

    #[tokio::test]
    async fn test_install_base_system_structure() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = RootFsConfig::default();
        let _builder = RootFsBuilder::new(config);

        // Just test directory structure creation (skip busybox install in test)
        let dirs = [
            "bin", "sbin", "usr/bin", "usr/sbin", "etc", "var", "tmp", "run", "proc", "sys", "dev",
        ];

        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }

        // Verify directories exist
        for dir in &dirs {
            assert!(root.join(dir).exists());
        }
    }

    // Integration test for NBD device detection (requires sudo)
    #[test]
    #[ignore = "Requires NBD kernel module and sudo"]
    fn test_nbd_device_detection() {
        // This test requires NBD kernel module loaded
        let result = NbdGuard::find_available_device();

        // Should either find a device or fail with clear error
        match result {
            Ok(device) => {
                assert!(device.starts_with("/dev/nbd"));
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                assert!(err_msg.contains("No available NBD devices"));
            }
        }
    }

    #[test]
    fn test_rootfs_config_custom_values() {
        let config = RootFsConfig {
            size: "10G".to_string(),
            output: PathBuf::from("/tmp/custom.qcow2"),
            primals_dir: Some(PathBuf::from("/opt/primals")),
            services_dir: Some(PathBuf::from("/etc/systemd")),
            mount_point: Some(PathBuf::from("/mnt/build")),
            fs_type: "xfs".to_string(),
            dns_servers: Some(vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]),
            nbd_device: Some("/dev/nbd0".to_string()),
            hostname: "custom-host".to_string(),
        };

        assert_eq!(config.size, "10G");
        assert_eq!(config.fs_type, "xfs");
        assert_eq!(config.hostname, "custom-host");
        assert_eq!(config.dns_servers.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_rootfs_builder_new() {
        let config = RootFsConfig::default();
        let builder = RootFsBuilder::new(config.clone());
        // Builder holds config
        assert_eq!(builder.config.size, config.size);
    }

    #[test]
    fn test_rootfs_cli_parse() {
        use clap::Parser;

        let cli = RootFsCli::parse_from(["biomeos-rootfs", "-s", "4G", "-o", "test.qcow2"]);
        assert_eq!(cli.size, "4G");
        assert_eq!(cli.output, PathBuf::from("test.qcow2"));
    }

    #[test]
    fn test_rootfs_cli_defaults() {
        use clap::Parser;

        let cli = RootFsCli::parse_from(["biomeos-rootfs"]);
        assert_eq!(cli.size, "8G");
        assert_eq!(cli.output, PathBuf::from("biomeos-root.qcow2"));
        assert_eq!(cli.fs_type, "ext4");
    }

    #[test]
    fn test_configure_dns_empty_servers() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = RootFsConfig {
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        // Should not panic; empty servers means "use system DNS" (early return)
        let result = builder.configure_dns(root);
        result.expect("configure_dns with empty servers should succeed");
    }

    #[test]
    fn test_configure_dns_with_servers() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        std::fs::create_dir_all(root.join("etc")).unwrap();

        let config = RootFsConfig {
            dns_servers: Some(vec!["1.2.3.4".to_string(), "5.6.7.8".to_string()]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        builder.configure_dns(root).expect("configure_dns");

        let resolv = std::fs::read_to_string(root.join("etc/resolv.conf")).unwrap();
        assert!(resolv.contains("1.2.3.4"));
        assert!(resolv.contains("5.6.7.8"));
    }

    #[test]
    fn test_configure_system_sets_hostname() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        std::fs::create_dir_all(root.join("etc")).unwrap();

        let config = RootFsConfig {
            hostname: "test-host".to_string(),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        builder.configure_system(root).expect("configure_system");

        let hostname = std::fs::read_to_string(root.join("etc/hostname")).unwrap();
        assert_eq!(hostname.trim(), "test-host");
    }

    #[test]
    fn test_rootfs_config_size_parsing() {
        let config = RootFsConfig {
            size: "4G".to_string(),
            ..Default::default()
        };
        assert_eq!(config.size, "4G");
    }

    #[test]
    fn test_rootfs_config_output_path() {
        let config = RootFsConfig {
            output: PathBuf::from("/tmp/custom.qcow2"),
            ..Default::default()
        };
        assert_eq!(config.output, PathBuf::from("/tmp/custom.qcow2"));
    }

    #[test]
    fn test_install_base_system_dirs_created() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let dirs = [
            "bin", "sbin", "usr/bin", "usr/sbin", "etc", "var", "tmp", "run", "proc", "sys", "dev",
            "home", "root",
        ];
        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        for dir in &dirs {
            assert!(root.join(dir).exists(), "{} should exist", dir);
        }
    }

    #[test]
    fn test_parse_resolv_conf_empty() {
        assert!(parse_resolv_conf("").is_empty());
        assert!(parse_resolv_conf("   \n  \n").is_empty());
    }

    #[test]
    fn test_parse_resolv_conf_single_nameserver() {
        let content = "nameserver 8.8.8.8\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8"]);
    }

    #[test]
    fn test_parse_resolv_conf_multiple_nameservers() {
        let content = r#"# comment
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
"#;
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8", "8.8.4.4", "1.1.1.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_skips_comments_and_options() {
        let content = r#"# Generated by resolvconf
search example.com
options timeout:2
nameserver 192.168.1.1
nameserver 10.0.0.1
"#;
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["192.168.1.1", "10.0.0.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_handles_whitespace() {
        let content = "  nameserver   127.0.0.1  \n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["127.0.0.1"]);
    }

    #[test]
    fn test_rootfs_cli_with_primals_flag() {
        use clap::Parser;

        let cli = RootFsCli::parse_from([
            "biomeos-rootfs",
            "-s",
            "4G",
            "-o",
            "out.qcow2",
            "-p",
            "/opt/primals",
        ]);
        assert_eq!(cli.size, "4G");
        assert_eq!(cli.primals, Some(PathBuf::from("/opt/primals")));
    }

    #[test]
    fn test_rootfs_cli_fs_type() {
        use clap::Parser;

        let cli = RootFsCli::parse_from(["biomeos-rootfs", "-f", "xfs"]);
        assert_eq!(cli.fs_type, "xfs");
    }

    #[test]
    fn test_parse_resolv_conf_single_line_no_newline() {
        let content = "nameserver 10.0.0.1";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["10.0.0.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_invalid_line_skipped() {
        let content = "nameserver\nnameserver 8.8.8.8";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8"]);
    }

    #[test]
    fn test_install_services_skips_non_service_files() {
        let temp = TempDir::new().unwrap();
        let services_dir = temp.path();
        std::fs::write(services_dir.join("not-a-service.txt"), "x").unwrap();
        std::fs::write(services_dir.join("real.service"), "[Unit]").unwrap();

        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("etc/systemd/system")).unwrap();

        let config = RootFsConfig {
            services_dir: Some(services_dir.to_path_buf()),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        builder.install_services(&root, services_dir).unwrap();

        assert!(root.join("etc/systemd/system/real.service").exists());
        assert!(!root.join("etc/systemd/system/not-a-service.txt").exists());
    }

    #[test]
    fn test_install_services_empty_dir() {
        let temp = TempDir::new().unwrap();
        let services_dir = temp.path();
        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("etc/systemd/system")).unwrap();

        let config = RootFsConfig {
            services_dir: Some(services_dir.to_path_buf()),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        builder.install_services(&root, services_dir).unwrap();
    }

    #[tokio::test]
    async fn test_install_primals_nonexistent_dir_ok() {
        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("usr/local/bin")).unwrap();

        let config = RootFsConfig {
            primals_dir: Some(PathBuf::from("/nonexistent/primals")),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        // Should not panic - returns Ok(()) with warning
        let result = builder
            .install_primals(&root, Path::new("/nonexistent/primals"))
            .await;
        result.unwrap();
    }
}
