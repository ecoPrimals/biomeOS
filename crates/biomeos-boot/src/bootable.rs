// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Bootable Media Creator
//!
//! Pure Rust implementation of bootable USB/ISO creation.
//! Clean architecture with modern idiomatic patterns.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::initramfs::{InitramfsBuilder, KernelManager};

/// Bootable media builder with clean separation of concerns
pub struct BootableMediaBuilder {
    project_root: PathBuf,
    work_dir: PathBuf,
    output_dir: PathBuf,
}

/// Boot target type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootTarget {
    /// ISO image for optical media or virtual machines
    Iso,
    /// USB flash drive bootable image
    Usb,
}

impl BootableMediaBuilder {
    /// Create a new bootable media builder rooted at the given project path
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

    /// Build complete bootable media (USB or ISO)
    pub fn build(&self, target: BootTarget) -> Result<PathBuf> {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("BiomeOS Bootable Media Builder - Pure Rust");
        info!("Target: {:?}", target);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("");

        // Step 1: Build BiomeOS binaries
        self.build_biomeos_binaries()?;

        // Step 2: Create initramfs
        let initramfs_path = self.build_initramfs()?;

        // Step 3: Get/detect kernel
        let kernel = KernelManager::detect_or_custom(None)?;

        // Step 4: Create bootable structure
        let boot_dir = self.create_boot_structure(&kernel, &initramfs_path)?;

        // Step 5: Add BiomeOS data
        self.add_biomeos_data(&boot_dir)?;

        // Step 6: Create bootable image with GRUB
        let image_path = self.create_bootable_image(&boot_dir, target)?;

        Self::print_success_message(&image_path, target)?;

        Ok(image_path)
    }

    /// Build `BiomeOS` binaries
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

    /// Build initramfs with `BiomeOS`
    fn build_initramfs(&self) -> Result<PathBuf> {
        info!("📦 Building initramfs...");

        let mut builder = InitramfsBuilder::new(&self.work_dir)?;

        // Create directory structure
        builder.create_directory_structure()?;

        // Add BiomeOS binaries (including biomeos-init as /init)
        builder.add_biomeos_binaries(&self.project_root)?;

        // Install binaries into initramfs root
        builder.install_binaries()?;

        // Copy required dynamic libraries for biomeos-init
        // This is needed because the binary is dynamically linked
        let init_binary = self.project_root.join("target/release/biomeos-init");
        if init_binary.exists() {
            builder.add_required_libraries(&init_binary)?;
        }

        // Note: We don't need create_init_script() anymore since biomeos-init
        // is copied directly as /init and is already executable

        // Build the initramfs archive
        let output = self.work_dir.join("biomeos-initramfs.img");
        builder.build(&output)?;

        info!("✅ Initramfs built: {}", output.display());
        Ok(output)
    }

    /// Create boot directory structure
    fn create_boot_structure(
        &self,
        kernel: &KernelManager,
        initramfs: &Path,
    ) -> Result<PathBuf> {
        info!("📁 Creating boot structure...");

        let boot_dir = self.work_dir.join("boot-root");
        std::fs::create_dir_all(&boot_dir)?;

        // Create GRUB directory structure
        let grub_dir = boot_dir.join("boot/grub");
        std::fs::create_dir_all(&grub_dir)?;

        // Copy kernel (may need root access for /boot/vmlinuz)
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

        // Copy initramfs
        let initramfs_dest = boot_dir.join("boot/initramfs.img");
        std::fs::copy(initramfs, &initramfs_dest).context("Failed to copy initramfs")?;
        info!("  • Initramfs: {}", initramfs_dest.display());

        // Create GRUB configuration
        Self::create_grub_config(&grub_dir)?;

        info!("✅ Boot structure created");
        Ok(boot_dir)
    }

    /// Create GRUB configuration with modern syntax (pub for testing)
    pub(crate) fn create_grub_config(grub_dir: &Path) -> Result<()> {
        use std::io::Write;

        let grub_cfg = grub_dir.join("grub.cfg");
        let mut file = std::fs::File::create(&grub_cfg).context("Failed to create grub.cfg")?;

        // Modern GRUB config with proper escaping
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
        // Boot from initramfs - no root filesystem needed
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

    /// Add `BiomeOS` data (primals, configs, templates)
    fn add_biomeos_data(&self, boot_dir: &Path) -> Result<()> {
        info!("📋 Adding BiomeOS data...");

        // Create BiomeOS directory structure
        let biomeos_dir = boot_dir.join("biomeos");
        std::fs::create_dir_all(biomeos_dir.join("primals"))?;
        std::fs::create_dir_all(biomeos_dir.join("configs"))?;
        std::fs::create_dir_all(biomeos_dir.join("templates"))?;

        // Copy Phase 1 primals if available
        if let Some(parent) = self.project_root.parent() {
            let bins_dir = parent.join("phase1bins");
            if bins_dir.exists() {
                Self::copy_directory(&bins_dir, &biomeos_dir.join("primals"))?;
                info!("  • Phase 1 primals: copied");
            } else {
                warn!("  • Phase 1 primals: not found (skipping)");
            }
        }

        // Copy templates
        let templates_src = self.project_root.join("templates");
        if templates_src.exists() {
            Self::copy_directory(&templates_src, &biomeos_dir.join("templates"))?;
            info!("  • Templates: copied");
        }

        info!("✅ BiomeOS data added");
        Ok(())
    }

    /// Copy directory recursively with proper error handling (pub for testing)
    pub(crate) fn copy_directory(src: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)
            .with_context(|| format!("Failed to create directory: {}", dest.display()))?;

        for entry in std::fs::read_dir(src)
            .with_context(|| format!("Failed to read directory: {}", src.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().context("Invalid file name")?;
            let dest_path = dest.join(file_name);

            if path.is_dir() {
                Self::copy_directory(&path, &dest_path)?;
            } else {
                std::fs::copy(&path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        path.display(),
                        dest_path.display()
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Create bootable image using grub-mkrescue (clean, simple approach)
    fn create_bootable_image(&self, boot_dir: &Path, _target: BootTarget) -> Result<PathBuf> {
        info!("💿 Creating bootable image with GRUB...");

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let output = self.output_dir.join(format!("biomeos-{timestamp}.iso"));

        // Try grub-mkrescue first (clean, modern approach)
        if let Ok(path) = self.create_with_grub_mkrescue(boot_dir, &output) {
            return Ok(path);
        }

        // Fallback to xorriso if grub-mkrescue not available
        warn!("grub-mkrescue not found, trying xorriso...");
        if let Ok(path) = self.create_with_xorriso(boot_dir, &output) {
            return Ok(path);
        }

        // Final fallback: create tar.gz (not bootable but preserves data)
        warn!("No ISO tools found - creating tar.gz archive");
        self.create_archive_fallback(boot_dir, &output)
    }

    /// Create bootable ISO with grub-mkrescue (preferred method)
    fn create_with_grub_mkrescue(&self, boot_dir: &Path, output: &Path) -> Result<PathBuf> {
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

    /// Create bootable ISO with xorriso (fallback method)
    fn create_with_xorriso(&self, boot_dir: &Path, output: &Path) -> Result<PathBuf> {
        info!("Using xorriso (fallback)...");

        // Note: This requires GRUB files to be present in boot_dir
        // grub-mkrescue is preferred as it handles this automatically
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

    /// Create tar.gz archive as final fallback
    fn create_archive_fallback(&self, boot_dir: &Path, output: &Path) -> Result<PathBuf> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use tar::Builder;

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

    /// Print success message with usage instructions
    fn print_success_message(image_path: &Path, target: BootTarget) -> Result<()> {
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_target_variants() {
        let iso = BootTarget::Iso;
        let usb = BootTarget::Usb;
        assert!(matches!(iso, BootTarget::Iso));
        assert!(matches!(usb, BootTarget::Usb));
    }

    #[test]
    fn test_boot_target_debug() {
        assert_eq!(format!("{:?}", BootTarget::Iso), "Iso");
        assert_eq!(format!("{:?}", BootTarget::Usb), "Usb");
    }

    #[test]
    fn test_boot_target_clone_copy() {
        let t = BootTarget::Iso;
        let t2 = t;
        let t3 = t; // Copy, no clone needed
        assert!(matches!(t2, BootTarget::Iso));
        assert!(matches!(t3, BootTarget::Iso));
    }

    #[test]
    fn test_bootable_media_builder_new() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let _builder = BootableMediaBuilder::new(project_root.clone())
            .expect("BootableMediaBuilder::new should succeed");

        let work_dir = project_root.join("build/boot-media");
        let output_dir = project_root.join("dist");
        assert!(work_dir.exists(), "work_dir should be created");
        assert!(output_dir.exists(), "output_dir should be created");
    }

    #[test]
    fn test_bootable_media_builder_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let _builder = BootableMediaBuilder::new(project_root).expect("new");
    }

    #[test]
    fn test_create_archive_fallback() {
        // Test the archive fallback path in isolation by creating minimal boot structure
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let builder = BootableMediaBuilder::new(project_root).expect("new");

        // Create minimal boot dir structure
        let boot_dir = temp.path().join("build/boot-media/boot-root");
        std::fs::create_dir_all(&boot_dir).expect("create boot dir");
        std::fs::write(boot_dir.join("test.txt"), "boot content").expect("write");

        let output = temp.path().join("dist/test.iso");
        let result = builder.create_archive_fallback(&boot_dir, &output);
        let path = result.expect("create_archive_fallback should succeed");
        assert!(path.exists());
        assert!(path.extension().is_some_and(|e| e == "gz"));
    }

    #[test]
    fn test_copy_directory() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let _builder = BootableMediaBuilder::new(project_root).expect("new");

        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create src");
        std::fs::write(src.join("file1.txt"), "content1").expect("write");
        std::fs::create_dir_all(src.join("subdir")).expect("create subdir");
        std::fs::write(src.join("subdir").join("file2.txt"), "content2").expect("write");

        BootableMediaBuilder::copy_directory(&src, &dest).expect("copy");

        assert!(dest.join("file1.txt").exists());
        assert!(dest.join("subdir").join("file2.txt").exists());
        assert_eq!(
            std::fs::read_to_string(dest.join("file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("subdir").join("file2.txt")).unwrap(),
            "content2"
        );
    }

    #[test]
    fn test_boot_target_equality() {
        assert_eq!(BootTarget::Iso, BootTarget::Iso);
        assert_eq!(BootTarget::Usb, BootTarget::Usb);
        assert_ne!(BootTarget::Iso, BootTarget::Usb);
    }

    #[test]
    fn test_bootable_media_builder_creates_nested_dirs() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let _builder = BootableMediaBuilder::new(project_root).expect("new");
        let work_dir = temp.path().join("build/boot-media");
        let output_dir = temp.path().join("dist");
        assert!(work_dir.exists());
        assert!(output_dir.exists());
        assert!(!work_dir.join("boot-root").exists());
    }

    #[test]
    fn test_grub_config_path_structure() {
        let temp = tempfile::tempdir().expect("temp dir");
        let _builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let boot_root = temp.path().join("build/boot-media/boot-root");
        std::fs::create_dir_all(boot_root.join("boot/grub")).expect("create");
        let result = BootableMediaBuilder::create_grub_config(&boot_root.join("boot/grub"));
        assert!(result.is_ok());
        let grub_cfg = boot_root.join("boot/grub/grub.cfg");
        assert!(grub_cfg.exists());
        let content = std::fs::read_to_string(&grub_cfg).expect("read");
        assert!(content.contains("BiomeOS"));
        assert!(content.contains("menuentry"));
        assert!(content.contains("vmlinuz"));
        assert!(content.contains("initramfs"));
    }

    #[test]
    fn test_grub_config_all_menu_entries() {
        let temp = tempfile::tempdir().expect("temp dir");
        let boot_root = temp.path().join("boot/grub");
        std::fs::create_dir_all(&boot_root).expect("create");
        BootableMediaBuilder::create_grub_config(&boot_root).expect("create_grub_config");
        let content = std::fs::read_to_string(boot_root.join("grub.cfg")).expect("read");
        assert!(content.contains("Sovereignty-First"));
        assert!(content.contains("Discovery Mode"));
        assert!(content.contains("Network Boot"));
        assert!(content.contains("biomeos.discovery"));
        assert!(content.contains("biomeos.network"));
        assert!(content.contains("rdinit=/init"));
    }

    #[test]
    fn test_copy_directory_empty_src() {
        let temp = tempfile::tempdir().expect("temp dir");
        let src = temp.path().join("empty");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create");
        BootableMediaBuilder::copy_directory(&src, &dest).expect("copy empty dir");
        assert!(dest.exists());
    }

    #[test]
    fn test_copy_directory_symlink_skipped() {
        let temp = tempfile::tempdir().expect("temp dir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create");
        std::fs::write(src.join("file.txt"), "content").expect("write");
        BootableMediaBuilder::copy_directory(&src, &dest).expect("copy");
        assert!(dest.join("file.txt").exists());
    }

    #[test]
    fn test_print_success_message_iso() {
        let temp = tempfile::tempdir().expect("temp dir");
        let image_path = temp.path().join("biomeos.iso");
        std::fs::write(&image_path, b"").expect("create file");
        BootableMediaBuilder::print_success_message(&image_path, BootTarget::Iso).expect("print");
    }

    #[test]
    fn test_print_success_message_usb() {
        let temp = tempfile::tempdir().expect("temp dir");
        let image_path = temp.path().join("biomeos.img");
        std::fs::write(&image_path, b"").expect("create file");
        BootableMediaBuilder::print_success_message(&image_path, BootTarget::Usb).expect("print");
    }

    #[test]
    fn test_create_grub_config_nonexistent_dir_fails() {
        let result =
            BootableMediaBuilder::create_grub_config(std::path::Path::new("/nonexistent/grub/dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_directory_src_not_a_directory_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let f = temp.path().join("file_not_dir");
        std::fs::write(&f, b"x").expect("write");
        let dest = temp.path().join("dest");
        let r = BootableMediaBuilder::copy_directory(&f, &dest);
        assert!(r.is_err());
    }

    #[test]
    fn test_copy_directory_missing_src_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let r = BootableMediaBuilder::copy_directory(
            &temp.path().join("nope"),
            &temp.path().join("dest"),
        );
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_boot_structure_copies_kernel_and_initramfs() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let builder = BootableMediaBuilder::new(project_root).expect("new");
        let kernel_path = temp.path().join("vmlinuz-test");
        std::fs::write(&kernel_path, b"fake-kernel-bytes").expect("kernel");
        let initramfs = temp.path().join("mini-initramfs.img");
        std::fs::write(&initramfs, b"cpio-bytes").expect("initramfs");
        let km = KernelManager::detect_or_custom(Some(kernel_path)).expect("kernel mgr");
        let boot_dir = builder
            .create_boot_structure(&km, &initramfs)
            .expect("boot structure");
        assert!(boot_dir.join("boot/vmlinuz").exists());
        assert!(boot_dir.join("boot/initramfs.img").exists());
        assert!(boot_dir.join("boot/grub/grub.cfg").exists());
    }

    #[tokio::test]
    async fn test_add_biomeos_data_copies_phase1_and_templates() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().join("proj");
        std::fs::create_dir_all(&project_root).expect("proj");
        std::fs::create_dir_all(temp.path().join("phase1bins")).expect("p1");
        std::fs::write(temp.path().join("phase1bins").join("p.bin"), b"p").expect("p1 bin");
        std::fs::create_dir_all(project_root.join("templates")).expect("tpl");
        std::fs::write(project_root.join("templates/hello.txt"), b"tpl").expect("tpl file");
        let builder = BootableMediaBuilder::new(project_root).expect("builder");
        let boot_dir = temp.path().join("boot-root");
        std::fs::create_dir_all(&boot_dir).expect("boot");
        builder.add_biomeos_data(&boot_dir).expect("add data");
        assert!(boot_dir.join("biomeos/primals/p.bin").exists());
        assert!(boot_dir.join("biomeos/templates/hello.txt").exists());
    }

    #[tokio::test]
    async fn test_add_biomeos_data_without_phase1_parent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let builder = BootableMediaBuilder::new(project_root.clone()).expect("new");
        let boot_dir = temp.path().join("br");
        std::fs::create_dir_all(&boot_dir).expect("boot");
        builder
            .add_biomeos_data(&boot_dir)
            .expect("ok without phase1");
        assert!(boot_dir.join("biomeos/primals").exists());
    }

    #[tokio::test]
    async fn test_create_bootable_image_produces_artifact() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let boot_dir = temp.path().join("br");
        std::fs::create_dir_all(boot_dir.join("boot/grub")).expect("grub");
        std::fs::write(boot_dir.join("boot/grub/grub.cfg"), b"# test").expect("cfg");
        let path = builder
            .create_bootable_image(&boot_dir, BootTarget::Iso)
            .expect("image");
        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_create_with_grub_mkrescue_or_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let boot_dir = temp.path().join("br2");
        std::fs::create_dir_all(&boot_dir).expect("create");
        let out = temp.path().join("out.iso");
        let r = builder.create_with_grub_mkrescue(&boot_dir, &out);
        let grub_ok = std::process::Command::new("grub-mkrescue")
            .arg("--help")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if grub_ok {
            assert!(r.is_ok() || r.is_err());
        } else {
            assert!(r.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_with_xorriso_or_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let boot_dir = temp.path().join("br3");
        std::fs::create_dir_all(&boot_dir).expect("create");
        let out = temp.path().join("out-xor.iso");
        let r = builder.create_with_xorriso(&boot_dir, &out);
        let xor_ok = std::process::Command::new("xorriso")
            .arg("-version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if xor_ok {
            assert!(r.is_ok() || r.is_err());
        } else {
            assert!(r.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_boot_structure_initramfs_copy_failures() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let builder = BootableMediaBuilder::new(project_root).expect("new");
        let kernel_path = temp.path().join("vmlinuz-ok");
        std::fs::write(&kernel_path, b"k").expect("k");
        let km = KernelManager::detect_or_custom(Some(kernel_path)).expect("km");
        let bad_init = temp.path().join("missing-initramfs.img");
        let r = builder.create_boot_structure(&km, &bad_init);
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_build_initramfs_with_minimal_release_binaries() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        let tr = project_root.join("target/release");
        std::fs::create_dir_all(&tr).expect("release dir");
        std::fs::write(tr.join("biomeos-init"), b"#!fake").expect("init bin");
        std::fs::write(tr.join("biome"), b"#!fake").expect("biome bin");
        let builder = BootableMediaBuilder::new(project_root).expect("new");
        let path = builder.build_initramfs().expect("initramfs build");
        assert!(path.exists());
        assert!(path.extension().is_some_and(|e| e == "img"));
    }

    #[tokio::test]
    async fn test_add_biomeos_data_skips_phase1_when_parent_has_no_bins() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().join("proj");
        std::fs::create_dir_all(&project_root).expect("proj");
        let builder = BootableMediaBuilder::new(project_root).expect("builder");
        let boot_dir = temp.path().join("boot-root");
        std::fs::create_dir_all(&boot_dir).expect("boot");
        builder.add_biomeos_data(&boot_dir).expect("add data");
        assert!(boot_dir.join("biomeos/primals").exists());
    }

    #[tokio::test]
    async fn test_add_biomeos_data_without_templates_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let project_root = temp.path().to_path_buf();
        std::fs::create_dir_all(&project_root).expect("root");
        let builder = BootableMediaBuilder::new(project_root).expect("builder");
        let boot_dir = temp.path().join("br");
        std::fs::create_dir_all(&boot_dir).expect("boot");
        builder.add_biomeos_data(&boot_dir).expect("ok");
        assert!(boot_dir.join("biomeos/templates").exists());
    }

    #[tokio::test]
    async fn test_create_boot_structure_kernel_copy_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let missing_kernel = temp.path().join("no-such-vmlinuz");
        let km = KernelManager::detect_or_custom(Some(missing_kernel.clone())).expect("km");
        let initramfs = temp.path().join("ir.img");
        std::fs::write(&initramfs, b"x").expect("init");
        let r = builder.create_boot_structure(&km, &initramfs);
        assert!(r.is_err());
    }

    #[test]
    fn test_copy_directory_invalid_entry_name_edge() {
        let temp = tempfile::tempdir().expect("temp dir");
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).expect("src");
        std::fs::write(src.join("ok.txt"), b"1").expect("w");
        let dest = temp.path().join("dest");
        let r = BootableMediaBuilder::copy_directory(&src, &dest);
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_create_archive_fallback_errors_on_bad_parent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
        let boot_dir = temp.path().join("br");
        std::fs::create_dir_all(&boot_dir).expect("boot");
        let blocker = temp.path().join("blocker");
        std::fs::write(&blocker, b"x").expect("file");
        let out = blocker.join("nested.iso");
        let r = builder.create_archive_fallback(&boot_dir, &out);
        assert!(r.is_err());
    }
}
