// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use super::types::RootFsBuilder;

impl RootFsBuilder {
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

    pub(in crate::rootfs::builder) fn install_biomeos(&self, root: &Path) -> Result<()> {
        info!("🦀 Installing BiomeOS components...");

        self.install_biomeos_init(root)?;

        info!("  ✓ BiomeOS init installed");
        Ok(())
    }

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
}
