// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! NBD device management for rootfs image mounting

use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use tracing::info;

/// RAII guard for NBD device - ensures cleanup on drop
pub(super) struct NbdGuard {
    device: String,
    attached: bool,
}

impl NbdGuard {
    /// Attach an NBD device to an image file
    pub fn attach(image_path: &Path) -> Result<Self> {
        let _ = Command::new("modprobe")
            .args(["nbd", "max_part=8"])
            .status();

        let device = Self::find_available_device()?;

        info!("📎 Attaching {} to {}", device, image_path.display());

        let _ = Command::new("qemu-nbd").args(["-d", &device]).output();
        std::thread::sleep(std::time::Duration::from_millis(100));

        let status = Command::new("qemu-nbd")
            .args(["-c", &device])
            .arg(image_path)
            .status()
            .context("Failed to attach NBD device")?;

        if !status.success() {
            anyhow::bail!("qemu-nbd failed to attach {}", device);
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        info!("  ✓ NBD attached: {}", device);

        Ok(Self {
            device,
            attached: true,
        })
    }

    /// Find an available NBD device
    pub fn find_available_device() -> Result<String> {
        for i in 0..16 {
            let device = format!("/dev/nbd{}", i);

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
    pub fn device(&self) -> &str {
        &self.device
    }

    /// Manual detach (also happens on drop)
    pub fn detach(&mut self) -> Result<()> {
        if !self.attached {
            return Ok(());
        }

        info!("🔓 Detaching NBD: {}", self.device);

        let status = Command::new("qemu-nbd")
            .args(["-d", &self.device])
            .status()
            .context("Failed to detach NBD")?;

        if !status.success() {
            tracing::warn!("  ⚠️  NBD detach returned error");
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
