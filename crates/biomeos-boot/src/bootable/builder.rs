// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::initramfs::{InitramfsBuilder, KernelManager};

use super::BootableMediaBuilder;
use super::types::BootTarget;

impl BootableMediaBuilder {
    /// Create a new bootable media builder rooted at the given project path.
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let work_dir = project_root.join("build/boot-media");
        let output_dir = project_root.join("dist");

        std::fs::create_dir_all(&work_dir)?;
        std::fs::create_dir_all(&output_dir)?;

        Ok(Self {
            project_root,
            work_dir,
            output_dir,
        })
    }

    /// Build complete bootable media (USB or ISO).
    pub fn build(&self, target: BootTarget) -> Result<PathBuf> {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("BiomeOS Bootable Media Builder - Pure Rust");
        info!("Target: {:?}", target);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("");

        self.build_biomeos_binaries()?;

        let initramfs_path = self.build_initramfs()?;

        let kernel = KernelManager::detect_or_custom(None)?;

        let boot_dir = self.create_boot_structure(&kernel, &initramfs_path)?;

        self.add_biomeos_data(&boot_dir)?;

        let image_path = self.create_bootable_image(&boot_dir, target)?;

        Self::print_success_message(&image_path, target)?;

        Ok(image_path)
    }

    fn build_biomeos_binaries(&self) -> Result<()> {
        info!("🔨 Building BiomeOS binaries...");

        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--workspace")
            .arg("--bins")
            .current_dir(&self.project_root)
            .status()
            .context("Failed to execute cargo build")?;

        if !status.success() {
            anyhow::bail!("cargo build failed");
        }

        info!("✅ BiomeOS binaries built");
        Ok(())
    }

    pub(crate) fn build_initramfs(&self) -> Result<PathBuf> {
        info!("📦 Building initramfs...");

        let mut builder = InitramfsBuilder::new(&self.work_dir)?;

        builder.create_directory_structure()?;

        builder.add_biomeos_binaries(&self.project_root)?;

        builder.install_binaries()?;

        let init_binary = self.project_root.join("target/release/biomeos-init");
        if init_binary.exists() {
            builder.add_required_libraries(&init_binary)?;
        }

        let output = self.work_dir.join("biomeos-initramfs.img");
        builder.build(&output)?;

        info!("✅ Initramfs built: {}", output.display());
        Ok(output)
    }

    pub(crate) fn create_boot_structure(
        &self,
        kernel: &KernelManager,
        initramfs: &Path,
    ) -> Result<PathBuf> {
        info!("📁 Creating boot structure...");

        let boot_dir = self.work_dir.join("boot-root");
        std::fs::create_dir_all(&boot_dir)?;

        let grub_dir = boot_dir.join("boot/grub");
        std::fs::create_dir_all(&grub_dir)?;

        let kernel_dest = boot_dir.join("boot/vmlinuz");
        std::fs::copy(kernel.kernel_path(), &kernel_dest).with_context(|| {
            format!(
                "Failed to copy kernel from {}\n\
                 Hint: Kernel files in /boot/ typically require root access.\n\
                 Solutions:\n\
                 1. Run with sudo: sudo -E cargo run ...\n\
                 2. Copy kernel manually: sudo cp {} /tmp/vmlinuz && chmod 644 /tmp/vmlinuz\n\
                 3. Use custom kernel: --kernel /path/to/accessible/vmlinuz",
                kernel.kernel_path().display(),
                kernel.kernel_path().display()
            )
        })?;
        info!("  • Kernel: {}", kernel_dest.display());

        let initramfs_dest = boot_dir.join("boot/initramfs.img");
        std::fs::copy(initramfs, &initramfs_dest).context("Failed to copy initramfs")?;
        info!("  • Initramfs: {}", initramfs_dest.display());

        Self::create_grub_config(&grub_dir)?;

        info!("✅ Boot structure created");
        Ok(boot_dir)
    }

    pub(crate) fn add_biomeos_data(&self, boot_dir: &Path) -> Result<()> {
        info!("📋 Adding BiomeOS data...");

        let biomeos_dir = boot_dir.join("biomeos");
        std::fs::create_dir_all(biomeos_dir.join("primals"))?;
        std::fs::create_dir_all(biomeos_dir.join("configs"))?;
        std::fs::create_dir_all(biomeos_dir.join("templates"))?;

        if let Some(parent) = self.project_root.parent() {
            let bins_dir = parent.join("phase1bins");
            if bins_dir.exists() {
                Self::copy_directory(&bins_dir, &biomeos_dir.join("primals"))?;
                info!("  • Phase 1 primals: copied");
            } else {
                warn!("  • Phase 1 primals: not found (skipping)");
            }
        }

        let templates_src = self.project_root.join("templates");
        if templates_src.exists() {
            Self::copy_directory(&templates_src, &biomeos_dir.join("templates"))?;
            info!("  • Templates: copied");
        }

        info!("✅ BiomeOS data added");
        Ok(())
    }

    pub(crate) fn print_success_message(image_path: &Path, target: BootTarget) -> Result<()> {
        info!("");
        info!("✅ Bootable {:?} created!", target);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("Image: {}", image_path.display());
        info!("");

        match target {
            BootTarget::Iso => {
                info!("To test in QEMU:");
                info!("  qemu-system-x86_64 \\");
                info!("    -cdrom {} \\", image_path.display());
                info!("    -m 2048 \\");
                info!("    -enable-kvm");
                info!("");
                info!("To write to USB:");
                info!(
                    "  sudo dd if={} of=/dev/sdX bs=4M status=progress",
                    image_path.display()
                );
            }
            BootTarget::Usb => {
                info!("To write to USB:");
                info!(
                    "  sudo dd if={} of=/dev/sdX bs=4M status=progress",
                    image_path.display()
                );
                info!("");
                info!("To test in QEMU:");
                info!("  qemu-system-x86_64 \\");
                info!("    -drive file={},format=raw \\", image_path.display());
                info!("    -m 2048 \\");
                info!("    -enable-kvm");
            }
        }

        info!("");
        Ok(())
    }
}
