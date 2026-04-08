// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Builder;
use tracing::{info, warn};

use super::BootableMediaBuilder;
use super::types::BootTarget;

impl BootableMediaBuilder {
    pub(crate) fn create_bootable_image(
        &self,
        boot_dir: &Path,
        _target: BootTarget,
    ) -> Result<PathBuf> {
        info!("💿 Creating bootable image with GRUB...");

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let output = self.output_dir.join(format!("biomeos-{timestamp}.iso"));

        if let Ok(path) = self.create_with_grub_mkrescue(boot_dir, &output) {
            return Ok(path);
        }

        warn!("grub-mkrescue not found, trying xorriso...");
        if let Ok(path) = self.create_with_xorriso(boot_dir, &output) {
            return Ok(path);
        }

        warn!("No ISO tools found - creating tar.gz archive");
        self.create_archive_fallback(boot_dir, &output)
    }

    #[expect(
        clippy::unused_self,
        reason = "method for API consistency with fallback chain"
    )]
    pub(crate) fn create_with_grub_mkrescue(
        &self,
        boot_dir: &Path,
        output: &Path,
    ) -> Result<PathBuf> {
        info!("Using grub-mkrescue (GRUB built-in)...");

        let status = Command::new("grub-mkrescue")
            .arg("-o")
            .arg(output)
            .arg(boot_dir)
            .status()
            .context("Failed to execute grub-mkrescue")?;

        if !status.success() {
            anyhow::bail!("grub-mkrescue failed with exit code: {:?}", status.code());
        }

        info!("✅ Bootable image created with grub-mkrescue");
        Ok(output.to_owned())
    }

    #[expect(
        clippy::unused_self,
        reason = "method for API consistency with fallback chain"
    )]
    pub(crate) fn create_with_xorriso(&self, boot_dir: &Path, output: &Path) -> Result<PathBuf> {
        info!("Using xorriso (fallback)...");

        let status = Command::new("xorriso")
            .args([
                "-as",
                "mkisofs",
                "-o",
                output.to_str().context("Invalid output path")?,
                "-r",
                "-J",
                "-V",
                "BIOMEOS",
                boot_dir.to_str().context("Invalid boot directory path")?,
            ])
            .status()
            .context("Failed to execute xorriso")?;

        if !status.success() {
            anyhow::bail!("xorriso failed with exit code: {:?}", status.code());
        }

        warn!("⚠️  Created with xorriso - may not be bootable without GRUB installation");
        Ok(output.to_owned())
    }

    #[expect(
        clippy::unused_self,
        reason = "method for API consistency with fallback chain"
    )]
    pub(crate) fn create_archive_fallback(
        &self,
        boot_dir: &Path,
        output: &Path,
    ) -> Result<PathBuf> {
        let output_tar = output.with_extension("tar.gz");

        let tar_gz = std::fs::File::create(&output_tar).context("Failed to create tar.gz file")?;
        let enc = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = Builder::new(enc);

        tar.append_dir_all(".", boot_dir)
            .context("Failed to add files to archive")?;
        tar.finish().context("Failed to finish archive")?;

        warn!("⚠️  Created tar.gz archive (not bootable)");
        warn!("   Extract and use grub-mkrescue manually to create bootable media");

        Ok(output_tar)
    }
}
