// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Root filesystem builder

use anyhow::{Context, Result};
use std::fmt::Write;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use super::config::RootFsConfig;
use super::dns::parse_resolv_conf;
use super::nbd::NbdGuard;

/// Builder for `BiomeOS` root filesystems
pub struct RootFsBuilder {
    /// Configuration
    pub(crate) config: RootFsConfig,
    /// NBD guard for the mounted image (held until [`Self::unmount_image`]).
    nbd_mount_guard: Option<NbdGuard>,
}

impl RootFsBuilder {
    /// Create a new root filesystem builder
    #[must_use]
    pub fn new(config: RootFsConfig) -> Self {
        Self {
            config,
            nbd_mount_guard: None,
        }
    }

    /// Build the complete root filesystem
    pub async fn build(&mut self) -> Result<PathBuf> {
        info!("🏗️  Building BiomeOS root filesystem");
        info!("  Output: {}", self.config.output.display());
        info!("  Size: {}", self.config.size);

        self.create_image()?;
        self.format_filesystem()?;

        let mount_point = self.mount_image()?;

        Self::install_base_system(&mount_point)?;
        self.install_biomeos(&mount_point)?;

        if let Some(primals_dir) = &self.config.primals_dir {
            self.install_primals(&mount_point, primals_dir)?;
        }

        if let Some(services_dir) = &self.config.services_dir {
            Self::install_services(&mount_point, services_dir)?;
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
    fn mount_image(&mut self) -> Result<PathBuf> {
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

        self.nbd_mount_guard = Some(nbd);

        Ok(mount_point)
    }

    /// Unmount the filesystem and detach NBD
    fn unmount_image(&mut self, mount_point: &Path) -> Result<()> {
        info!("📤 Unmounting filesystem...");

        let status = Command::new("umount")
            .arg(mount_point)
            .status()
            .context("Failed to unmount filesystem")?;

        if !status.success() {
            warn!("  ⚠️  Unmount returned error");
        }

        // Sync path: let the kernel release the mount/NBD device before detach (not async).
        std::thread::sleep(std::time::Duration::from_millis(100));

        if let Some(mut guard) = self.nbd_mount_guard.take() {
            guard.detach()?;
        } else {
            for i in 0..16 {
                let device = format!("/dev/nbd{i}");
                let size_path = format!("/sys/block/nbd{i}/size");

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
        }

        info!("  ✓ Unmounted and detached");
        Ok(())
    }

    /// Install base system (minimal Linux userspace)
    pub(crate) fn install_base_system(root: &Path) -> Result<()> {
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

        Self::install_busybox(root)?;

        info!("  ✓ Base system installed");
        Ok(())
    }

    /// Install `BusyBox` for minimal Linux userspace
    fn install_busybox(root: &Path) -> Result<()> {
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

    /// Install `BiomeOS` core components
    fn install_biomeos(&self, root: &Path) -> Result<()> {
        info!("🦀 Installing BiomeOS components...");

        self.install_biomeos_init(root)?;

        info!("  ✓ BiomeOS init installed");
        Ok(())
    }

    /// Install biomeos-init as PID 1
    #[expect(clippy::unused_self, reason = "method for builder API consistency")]
    fn install_biomeos_init(&self, root: &Path) -> Result<()> {
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
    #[expect(clippy::unused_self, reason = "method for builder API consistency")]
    pub(crate) fn install_primals(&self, root: &Path, primals_dir: &Path) -> Result<()> {
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

    pub(crate) fn install_services(root: &Path, services_dir: &Path) -> Result<()> {
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
                let Some(filename) = path.file_name() else {
                    continue;
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
                    .with_context(|| format!("Failed to symlink service {service_name}"))?;

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
            Self::discover_system_dns()?
        };

        if dns_servers.is_empty() {
            info!("  Using system DNS configuration");
            return Ok(());
        }

        let mut content = String::new();
        for server in &dns_servers {
            writeln!(content, "nameserver {server}")
                .map_err(|e| anyhow::anyhow!("Failed to write DNS config: {e}"))?;
        }

        std::fs::write(&resolv_conf, content)?;
        info!("  ✓ DNS configured ({} servers)", dns_servers.len());

        Ok(())
    }

    /// Discover DNS servers from system
    pub(super) fn discover_system_dns() -> Result<Vec<String>> {
        let system_resolv = std::fs::read_to_string("/etc/resolv.conf")
            .context("Failed to read /etc/resolv.conf")?;

        Ok(parse_resolv_conf(&system_resolv))
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config() -> RootFsConfig {
        RootFsConfig {
            size: "1G".to_string(),
            output: PathBuf::from("/tmp/test-rootfs.qcow2"),
            primals_dir: None,
            services_dir: None,
            mount_point: None,
            fs_type: "ext4".to_string(),
            dns_servers: Some(vec![]),
            nbd_device: None,
            hostname: "test-biomeos".to_string(),
        }
    }

    #[test]
    fn test_builder_new() {
        let config = test_config();
        let builder = RootFsBuilder::new(config);
        assert_eq!(builder.config.size, "1G");
        assert_eq!(builder.config.hostname, "test-biomeos");
    }

    #[test]
    fn test_configure_dns_empty_servers_early_return() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");

        let config = RootFsConfig {
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_dns(root).expect("configure_dns");
        // resolv.conf should not be written when empty
        assert!(!root.join("etc/resolv.conf").exists());
    }

    #[test]
    fn test_configure_dns_with_servers() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");

        let config = RootFsConfig {
            dns_servers: Some(vec!["10.0.0.1".to_string(), "10.0.0.2".to_string()]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_dns(root).expect("configure_dns");

        let content = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
        assert!(content.contains("nameserver 10.0.0.1"));
        assert!(content.contains("nameserver 10.0.0.2"));
    }

    #[test]
    fn test_configure_system_writes_hostname() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");

        let config = RootFsConfig {
            hostname: "my-custom-host".to_string(),
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_system(root).expect("configure_system");

        let hostname = std::fs::read_to_string(root.join("etc/hostname")).expect("read");
        assert_eq!(hostname.trim(), "my-custom-host");
    }

    #[tokio::test]
    async fn test_install_primals_with_files() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        let primals_dir = temp.path().join("primals");
        std::fs::create_dir_all(&primals_dir).expect("create primals");
        std::fs::write(primals_dir.join("beardog"), b"#!/bin/sh\necho beardog").expect("write");
        std::fs::write(primals_dir.join("songbird"), b"#!/bin/sh\necho songbird").expect("write");

        let config = RootFsConfig {
            primals_dir: Some(primals_dir.clone()),
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder
            .install_primals(root, &primals_dir)
            .expect("install_primals");

        let target = root.join("usr/local/bin");
        assert!(target.join("beardog").exists());
        assert!(target.join("songbird").exists());
    }

    #[tokio::test]
    async fn test_install_primals_nonexistent_dir_returns_ok() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("usr/local/bin")).expect("create target");

        let config = RootFsConfig {
            primals_dir: Some(PathBuf::from("/nonexistent/path")),
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder
            .install_primals(root, Path::new("/nonexistent/path"))
            .expect("install_primals");
    }

    #[test]
    fn test_install_services_nonexistent_dir_returns_ok() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

        let services_dir = Path::new("/nonexistent/services");
        RootFsBuilder::install_services(root, services_dir).expect("install_services");
    }

    #[test]
    #[cfg(unix)]
    fn test_install_services_with_service_file() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let services_dir = temp.path().join("services");
        std::fs::create_dir_all(&services_dir).expect("create services");
        std::fs::write(
            services_dir.join("test.service"),
            "[Unit]\nDescription=Test\n[Service]\nExecStart=/bin/true\n",
        )
        .expect("write");

        let root = temp.path().join("root");
        std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

        RootFsBuilder::install_services(&root, &services_dir).expect("install_services");

        assert!(root.join("etc/systemd/system/test.service").exists());
        assert!(
            root.join("etc/systemd/system/multi-user.target.wants/test.service")
                .exists()
        );
    }

    #[test]
    fn test_install_services_skips_non_service_files() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let services_dir = temp.path().join("services");
        std::fs::create_dir_all(&services_dir).expect("create services");
        std::fs::write(services_dir.join("not-a-service.txt"), "content").expect("write");
        std::fs::write(services_dir.join("config.conf"), "config").expect("write");

        let root = temp.path().join("root");
        std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

        RootFsBuilder::install_services(&root, &services_dir).expect("install_services");

        assert!(!root.join("etc/systemd/system/not-a-service.txt").exists());
        assert!(!root.join("etc/systemd/system/config.conf").exists());
    }

    #[test]
    fn test_install_services_with_existing_symlink() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let services_dir = temp.path().join("services");
        std::fs::create_dir_all(&services_dir).expect("create services");
        std::fs::write(
            services_dir.join("dup.service"),
            "[Unit]\nDescription=Dup\n[Service]\nExecStart=/bin/true\n",
        )
        .expect("write");

        let root = temp.path().join("root");
        let systemd_dir = root.join("etc/systemd/system");
        let wants_dir = systemd_dir.join("multi-user.target.wants");
        std::fs::create_dir_all(&wants_dir).expect("create");
        std::fs::write(systemd_dir.join("dup.service"), "old").expect("write");
        std::os::unix::fs::symlink(
            systemd_dir.join("dup.service"),
            wants_dir.join("dup.service"),
        )
        .expect("symlink");

        RootFsBuilder::install_services(&root, &services_dir).expect("install_services");
        assert!(wants_dir.join("dup.service").exists());
    }

    #[test]
    fn test_configure_dns_uses_system_when_none() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");

        let config = RootFsConfig {
            dns_servers: None,
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        let result = builder.configure_dns(root);
        if result.is_ok() {
            if root.join("etc/resolv.conf").exists() {
                let content = std::fs::read_to_string(root.join("etc/resolv.conf")).unwrap();
                assert!(content.contains("nameserver"));
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_install_base_system_creates_dirs() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        if RootFsBuilder::install_base_system(root).is_ok() {
            for dir in [
                "bin", "sbin", "usr/bin", "etc", "var/log", "proc", "sys", "dev", "tmp", "run",
            ] {
                assert!(root.join(dir).exists(), "{} should exist", dir);
            }
        }
    }

    #[test]
    fn test_root_fs_config_fields() {
        let c = RootFsConfig {
            size: "2G".to_string(),
            output: PathBuf::from("/tmp/out.qcow2"),
            primals_dir: Some(PathBuf::from("/p")),
            services_dir: Some(PathBuf::from("/s")),
            mount_point: Some(PathBuf::from("/mnt")),
            fs_type: "xfs".to_string(),
            dns_servers: Some(vec!["8.8.8.8".to_string()]),
            nbd_device: None,
            hostname: "h".to_string(),
        };
        assert_eq!(c.fs_type, "xfs");
        assert_eq!(c.size, "2G");
    }

    #[test]
    fn test_configure_system_empty_hostname_still_writes() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");
        let config = RootFsConfig {
            hostname: String::new(),
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_system(root).expect("configure_system");
        let h = std::fs::read_to_string(root.join("etc/hostname")).expect("read");
        assert_eq!(h.trim(), "");
    }

    #[test]
    fn test_install_primals_skips_subdirectories() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        let primals_dir = temp.path().join("primals");
        std::fs::create_dir_all(&primals_dir).expect("create primals");
        std::fs::create_dir_all(primals_dir.join("subdir")).expect("subdir");
        std::fs::write(primals_dir.join("exec-only"), b"x").expect("file");

        let config = RootFsConfig {
            primals_dir: Some(primals_dir.clone()),
            dns_servers: Some(vec![]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder
            .install_primals(root, &primals_dir)
            .expect("install_primals");
        assert!(root.join("usr/local/bin/exec-only").exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_install_services_replaces_stale_symlink_target() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let services_dir = temp.path().join("services");
        std::fs::create_dir_all(&services_dir).expect("create services");
        std::fs::write(
            services_dir.join("replace.service"),
            "[Unit]\nDescription=R\n[Service]\nExecStart=/bin/true\n",
        )
        .expect("write");

        let root = temp.path().join("root");
        let systemd_dir = root.join("etc/systemd/system");
        let wants = systemd_dir.join("multi-user.target.wants");
        std::fs::create_dir_all(&wants).expect("create");
        let unit = systemd_dir.join("replace.service");
        std::fs::write(&unit, "old content").expect("old unit");
        let wrong = wants.join("replace.service");
        std::os::unix::fs::symlink("/dev/null", &wrong).expect("wrong link");

        RootFsBuilder::install_services(&root, &services_dir).expect("install_services");
        assert!(wrong.exists());
        let new_content = std::fs::read_to_string(&unit).expect("read unit");
        assert!(new_content.contains("Description=R"));
    }

    #[test]
    fn test_builder_exposes_config() {
        let b = RootFsBuilder::new(test_config());
        assert_eq!(b.config.hostname, "test-biomeos");
    }

    #[test]
    fn test_discover_system_dns_reads_resolv_conf() {
        let servers = RootFsBuilder::discover_system_dns();
        match servers {
            Ok(v) => {
                if std::path::Path::new("/etc/resolv.conf").exists() {
                    assert!(
                        v.iter().all(|s| !s.is_empty()),
                        "nameservers should be non-empty strings when parsed"
                    );
                }
            }
            Err(e) => {
                assert!(
                    e.to_string().contains("resolv") || e.to_string().contains("Failed"),
                    "unexpected error: {e}"
                );
            }
        }
    }

    #[test]
    fn test_configure_dns_system_branch_via_none_config() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");
        let config = RootFsConfig {
            dns_servers: None,
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        let r = builder.configure_dns(root);
        assert!(r.is_ok());
    }

    #[test]
    fn test_configure_dns_single_nameserver() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");
        let config = RootFsConfig {
            dns_servers: Some(vec!["1.1.1.1".to_string()]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_dns(root).expect("dns");
        let c = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
        assert_eq!(c.lines().count(), 1);
        assert!(c.contains("1.1.1.1"));
    }

    #[test]
    fn test_root_fs_config_mount_point_optional() {
        let c = RootFsConfig {
            mount_point: Some(PathBuf::from("/mnt/rootfs")),
            ..test_config()
        };
        assert_eq!(
            c.mount_point
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            Some("/mnt/rootfs".to_string())
        );
    }

    #[test]
    fn test_install_services_path_is_file_errors() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let not_a_dir = temp.path().join("file-not-dir");
        std::fs::write(&not_a_dir, b"x").expect("write");
        let root = temp.path().join("root");
        std::fs::create_dir_all(&root).expect("root");
        let result = RootFsBuilder::install_services(&root, &not_a_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_dns_three_nameservers_order_preserved() {
        let temp = tempfile::Builder::new().tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("etc")).expect("create etc");
        let config = RootFsConfig {
            dns_servers: Some(vec![
                "9.9.9.9".to_string(),
                "149.112.112.112".to_string(),
                "1.0.0.1".to_string(),
            ]),
            ..test_config()
        };
        let builder = RootFsBuilder::new(config);
        builder.configure_dns(root).expect("configure_dns");
        let content = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
        assert_eq!(content.lines().count(), 3);
        assert!(content.contains("9.9.9.9"));
        assert!(content.contains("149.112.112.112"));
        assert!(content.contains("1.0.0.1"));
    }
}
