// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use tracing::info;

use super::BootableMediaBuilder;

impl BootableMediaBuilder {
    pub(crate) fn create_grub_config(grub_dir: &Path) -> Result<()> {
        let grub_cfg = grub_dir.join("grub.cfg");
        let mut file = std::fs::File::create(&grub_cfg).context("Failed to create grub.cfg")?;

        writeln!(file, "set timeout=10")?;
        writeln!(file, "set default=0")?;
        writeln!(file, "serial --unit=0 --speed=115200")?;
        writeln!(file, "terminal_input console serial")?;
        writeln!(file, "terminal_output console serial")?;
        writeln!(file)?;
        writeln!(
            file,
            "menuentry 'BiomeOS - Sovereignty-First Operating System' {{"
        )?;
        writeln!(file, "    echo 'BiomeOS - Loading Pure Rust Platform...'")?;
        writeln!(file, "    echo ''")?;
        writeln!(
            file,
            "    linux /boot/vmlinuz rdinit=/init rootfstype=rootfs rw console=tty0 console=ttyS0,115200"
        )?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        writeln!(file, "menuentry 'BiomeOS - Discovery Mode' {{")?;
        writeln!(file, "    echo 'BiomeOS - Network Discovery Mode'")?;
        writeln!(
            file,
            "    linux /boot/vmlinuz rootfstype=rootfs rdinit=/init rw biomeos.discovery"
        )?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;
        writeln!(file)?;
        writeln!(file, "menuentry 'BiomeOS - Network Boot' {{")?;
        writeln!(file, "    echo 'BiomeOS - Network Coordination'")?;
        writeln!(
            file,
            "    linux /boot/vmlinuz rootfstype=rootfs rdinit=/init rw biomeos.network"
        )?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;

        info!("  • GRUB config: {}", grub_cfg.display());
        Ok(())
    }
}
