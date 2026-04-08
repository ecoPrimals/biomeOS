// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

use crate::rootfs::nbd::NbdGuard;

use super::types::RootFsBuilder;

impl RootFsBuilder {
    pub(in crate::rootfs::builder) fn create_image(&self) -> Result<()> {
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

    pub(in crate::rootfs::builder) fn format_filesystem(&self) -> Result<()> {
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
}
