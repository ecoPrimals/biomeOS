// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Root filesystem builder

use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use super::config::RootFsConfig;
use super::dns::parse_resolv_conf;
use super::nbd::NbdGuard;

/// Builder for BiomeOS root filesystems
pub struct RootFsBuilder {
    /// Configuration
    pub(crate) config: RootFsConfig,
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

        self.create_image()?;
        self.format_filesystem()?;

        let mount_point = self.mount_image()?;

        self.install_base_system(&mount_point)?;
        self.install_biomeos(&mount_point).await?;

        if let Some(primals_dir) = &self.config.primals_dir {
            self.install_primals(&mount_point, primals_dir).await?;
        }

        if let Some(services_dir) = &self.config.services_dir {
            self.install_services(&mount_point, services_dir)?;
        }

        self.configure_system(&mount_point)?;
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

        let mut nbd = NbdGuard::attach(&self.config.output)?;

        let status = Command::new(format!("mkfs.{}", self.config.fs_type))
            .arg(nbd.device())
            .arg("-L")
            .arg("BIOMEOS")
            .status()
            .context("Failed to format filesystem")?;

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

        let mount_point = if let Some(ref mp) = self.config.mount_point {
            mp.clone()
        } else {
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
            anyhow::bail!("Failed to mount filesystem");
        }

        info!("  ✓ Mounted at {}", mount_point.display());

        std::mem::forget(nbd);

        Ok(mount_point)
    }

    /// Unmount the filesystem and detach NBD
    fn unmount_image(&self, mount_point: &Path) -> Result<()> {
        info!("📤 Unmounting filesystem...");

        let status = Command::new("umount")
            .arg(mount_point)
            .status()
            .context("Failed to unmount filesystem")?;

        if !status.success() {
            warn!("  ⚠️  Unmount returned error");
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        for i in 0..16 {
            let device = format!("/dev/nbd{}", i);
            let size_path = format!("/sys/block/nbd{}/size", i);

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

        self.install_busybox(root)?;

        info!("  ✓ Base system installed");
        Ok(())
    }

    /// Install BusyBox for minimal Linux userspace
    fn install_busybox(&self, root: &Path) -> Result<()> {
        let busybox_path = which::which("busybox")
            .context("BusyBox not found - install with: apt install busybox-static")?;

        let dest = root.join("bin/busybox");
        std::fs::copy(&busybox_path, &dest).context("Failed to copy busybox")?;

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

        self.install_biomeos_init(root).await?;

        info!("  ✓ BiomeOS init installed");
        Ok(())
    }

    /// Install biomeos-init as PID 1
    async fn install_biomeos_init(&self, root: &Path) -> Result<()> {
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

        let dest = root.join("sbin/init");
        std::fs::copy(&init_binary, &dest).context("Failed to copy biomeos-init")?;

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
    pub(crate) async fn install_primals(&self, root: &Path, primals_dir: &Path) -> Result<()> {
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

    pub(crate) fn install_services(&self, root: &Path, services_dir: &Path) -> Result<()> {
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

                let service_name = filename.to_string_lossy();
                let wants_dir = systemd_dir.join("multi-user.target.wants");
                std::fs::create_dir_all(&wants_dir)?;
                let symlink_dest = wants_dir.join(&*service_name);

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
    pub(crate) fn configure_system(&self, root: &Path) -> Result<()> {
        info!("⚙️  Configuring system...");

        self.configure_dns(root)?;

        let hostname_path = root.join("etc/hostname");
        std::fs::write(&hostname_path, format!("{}\n", self.config.hostname))?;

        info!("  ✓ System configured (hostname: {})", self.config.hostname);
        Ok(())
    }

    /// Configure DNS resolvers
    pub(super) fn configure_dns(&self, root: &Path) -> Result<()> {
        let resolv_conf = root.join("etc/resolv.conf");

        let dns_servers = if let Some(ref servers) = self.config.dns_servers {
            servers.clone()
        } else {
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
    pub(super) fn discover_system_dns(&self) -> Result<Vec<String>> {
        let system_resolv = std::fs::read_to_string("/etc/resolv.conf")
            .context("Failed to read /etc/resolv.conf")?;

        Ok(parse_resolv_conf(&system_resolv))
    }
}
