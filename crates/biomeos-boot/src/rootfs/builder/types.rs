// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::rootfs::config::RootFsConfig;
use crate::rootfs::nbd::NbdGuard;

/// Builder for `BiomeOS` root filesystems
pub struct RootFsBuilder {
    /// Configuration
    pub(crate) config: RootFsConfig,
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

    fn unmount_image(&mut self, mount_point: &Path) -> Result<()> {
        info!("📤 Unmounting filesystem...");

        let status = Command::new("umount")
            .arg(mount_point)
            .status()
            .context("Failed to unmount filesystem")?;

        if !status.success() {
            warn!("  ⚠️  Unmount returned error");
        }

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
}
