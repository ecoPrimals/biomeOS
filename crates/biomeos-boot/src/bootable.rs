//! BiomeOS Bootable Media Creator
//! 
//! Pure Rust implementation of bootable USB/ISO creation.
//! No bash scripts, all idiomatic Rust.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::initramfs::{InitramfsBuilder, KernelManager};

pub struct BootableMediaBuilder {
    project_root: PathBuf,
    work_dir: PathBuf,
    output_dir: PathBuf,
}

impl BootableMediaBuilder {
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

    /// Build complete bootable USB image
    pub async fn build_usb_image(&self) -> Result<PathBuf> {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("BiomeOS Bootable USB Builder - Pure Rust");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("");

        // Step 1: Build BiomeOS binaries
        self.build_biomeos_binaries().await?;

        // Step 2: Create initramfs
        let initramfs_path = self.build_initramfs().await?;

        // Step 3: Get/detect kernel
        let kernel = KernelManager::detect_or_custom(None)?;

        // Step 4: Create bootable structure
        let boot_dir = self.create_boot_structure(&kernel, &initramfs_path).await?;

        // Step 5: Add BiomeOS data
        self.add_biomeos_data(&boot_dir).await?;

        // Step 6: Create bootable image
        let image_path = self.create_bootable_image(&boot_dir).await?;

        info!("");
        info!("✅ Bootable USB image created!");
        info!("Image: {}", image_path.display());
        info!("");
        info!("To write to USB:");
        info!("  sudo dd if={} of=/dev/sdX bs=4M status=progress", 
              image_path.display());
        info!("");
        info!("To test in QEMU:");
        info!("  qemu-system-x86_64 -m 2048 -enable-kvm -drive file={},format=raw", 
              image_path.display());

        Ok(image_path)
    }

    /// Build BiomeOS binaries
    async fn build_biomeos_binaries(&self) -> Result<()> {
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

    /// Build initramfs with BiomeOS
    async fn build_initramfs(&self) -> Result<PathBuf> {
        info!("📦 Building initramfs...");

        let mut builder = InitramfsBuilder::new(&self.work_dir)?;
        
        // Create directory structure
        builder.create_directory_structure()?;
        
        // Add BiomeOS binaries
        builder.add_biomeos_binaries(&self.project_root)?;
        
        // Install binaries
        builder.install_binaries()?;
        
        // Create init script
        builder.create_init_script()?;
        
        // Build archive
        let output = self.work_dir.join("biomeos-initramfs.img");
        builder.build(&output)?;

        info!("✅ Initramfs built: {}", output.display());
        Ok(output)
    }

    /// Create boot directory structure
    async fn create_boot_structure(
        &self,
        kernel: &KernelManager,
        initramfs: &Path,
    ) -> Result<PathBuf> {
        info!("📁 Creating boot structure...");

        let boot_dir = self.work_dir.join("boot-root");
        std::fs::create_dir_all(&boot_dir)?;

        // Create directory structure
        let dirs = vec!["boot", "boot/grub", "biomeos", "biomeos/primals"];
        for dir in dirs {
            std::fs::create_dir_all(boot_dir.join(dir))?;
        }

        // Copy kernel
        let kernel_dest = boot_dir.join("boot/vmlinuz");
        std::fs::copy(kernel.kernel_path(), &kernel_dest)
            .context("Failed to copy kernel")?;
        info!("  • Kernel: {}", kernel_dest.display());

        // Copy initramfs
        let initramfs_dest = boot_dir.join("boot/initramfs.img");
        std::fs::copy(initramfs, &initramfs_dest)
            .context("Failed to copy initramfs")?;
        info!("  • Initramfs: {}", initramfs_dest.display());

        // Create GRUB configuration
        self.create_grub_config(&boot_dir)?;

        info!("✅ Boot structure created");
        Ok(boot_dir)
    }

    /// Create GRUB configuration
    fn create_grub_config(&self, boot_dir: &Path) -> Result<()> {
        use std::io::Write;

        let grub_cfg = boot_dir.join("boot/grub/grub.cfg");
        std::fs::create_dir_all(grub_cfg.parent().unwrap())?;

        let mut file = std::fs::File::create(&grub_cfg)?;
        writeln!(file, "set timeout=10")?;
        writeln!(file, "set default=0")?;
        writeln!(file, "")?;
        writeln!(file, "menuentry \"BiomeOS - Sovereignty-First Operating System\" {{")?;
        writeln!(file, "    echo 'BiomeOS - Loading Pure Rust Platform...'")?;
        writeln!(file, "    echo ''")?;
        writeln!(file, "    linux /boot/vmlinuz")?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;
        writeln!(file, "")?;
        writeln!(file, "menuentry \"BiomeOS - Discovery Mode\" {{")?;
        writeln!(file, "    echo 'BiomeOS - Network Discovery Mode'")?;
        writeln!(file, "    linux /boot/vmlinuz biomeos.discovery")?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;
        writeln!(file, "")?;
        writeln!(file, "menuentry \"BiomeOS - Network Boot\" {{")?;
        writeln!(file, "    echo 'BiomeOS - Network Coordination'")?;
        writeln!(file, "    linux /boot/vmlinuz biomeos.network")?;
        writeln!(file, "    initrd /boot/initramfs.img")?;
        writeln!(file, "}}")?;

        info!("  • GRUB config: {}", grub_cfg.display());
        Ok(())
    }

    /// Add BiomeOS data (primals, configs, templates)
    async fn add_biomeos_data(&self, boot_dir: &Path) -> Result<()> {
        info!("📋 Adding BiomeOS data...");

        // Copy Phase 1 primals if available
        let phase1bins = self.project_root.parent()
            .and_then(|p| Some(p.join("phase1bins")));

        if let Some(bins_dir) = phase1bins {
            if bins_dir.exists() {
                let dest = boot_dir.join("biomeos/primals");
                self.copy_directory(&bins_dir, &dest)?;
                info!("  • Phase 1 primals: copied");
            } else {
                warn!("  • Phase 1 primals: not found (skipping)");
            }
        }

        // Copy templates
        let templates_src = self.project_root.join("templates");
        if templates_src.exists() {
            let dest = boot_dir.join("biomeos/templates");
            self.copy_directory(&templates_src, &dest)?;
            info!("  • Templates: copied");
        }

        info!("✅ BiomeOS data added");
        Ok(())
    }

    /// Copy directory recursively
    fn copy_directory(&self, src: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dest.join(file_name);

            if path.is_dir() {
                self.copy_directory(&path, &dest_path)?;
            } else {
                std::fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Create bootable image
    async fn create_bootable_image(&self, boot_dir: &Path) -> Result<PathBuf> {
        info!("💿 Creating bootable image...");

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let output = self.output_dir.join(format!("biomeos-{}.img", timestamp));

        // Use xorriso to create hybrid ISO that works as both ISO and USB
        let status = Command::new("xorriso")
            .args([
                "-as", "mkisofs",
                "-o", output.to_str().unwrap(),
                "-b", "boot/grub/i386-pc/eltorito.img",
                "-no-emul-boot",
                "-boot-load-size", "4",
                "-boot-info-table",
                "--grub2-boot-info",
                "--grub2-mbr", "/usr/lib/grub/i386-pc/boot_hybrid.img",
                "-eltorito-alt-boot",
                "-e", "boot/grub/efiboot.img",
                "-no-emul-boot",
                "-isohybrid-gpt-basdat",
                "-V", "BIOMEOS",
                boot_dir.to_str().unwrap(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                info!("✅ Bootable image created");
                Ok(output)
            }
            Ok(_) => anyhow::bail!("xorriso failed"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                warn!("xorriso not found - creating simple image");
                self.create_simple_image(boot_dir, &output).await
            }
            Err(e) => Err(e).context("Failed to execute xorriso"),
        }
    }

    /// Create simple bootable image (fallback if xorriso not available)
    async fn create_simple_image(&self, boot_dir: &Path, output: &Path) -> Result<PathBuf> {
        warn!("Creating simple tar.gz image (not bootable without extraction)");
        
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let tar_gz = std::fs::File::create(output)?;
        let enc = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = Builder::new(enc);

        tar.append_dir_all(".", boot_dir)?;
        tar.finish()?;

        Ok(output.to_owned())
    }
}

