//! BiomeOS Initramfs Builder
//! 
//! Pure Rust initramfs generation. No shell scripts, no external dependencies.
//! Creates a minimal boot environment with BiomeOS binaries.

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Builder;
use tracing::{info, warn};

pub struct InitramfsBuilder {
    root: PathBuf,
    binaries: Vec<BinarySpec>,
    directories: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BinarySpec {
    pub source: PathBuf,
    pub dest: String,
    pub permissions: u32,
}

impl InitramfsBuilder {
    pub fn new<P: AsRef<Path>>(work_dir: P) -> Result<Self> {
        let root = work_dir.as_ref().join("initramfs-root");
        fs::create_dir_all(&root)
            .context("Failed to create initramfs work directory")?;

        Ok(Self {
            root,
            binaries: Vec::new(),
            directories: Vec::new(),
        })
    }

    /// Add BiomeOS core binaries
    pub fn add_biomeos_binaries(&mut self, project_root: &Path) -> Result<()> {
        info!("📦 Adding BiomeOS binaries...");

        // BiomeOS init (PID 1)
        self.add_binary(BinarySpec {
            source: project_root.join("target/release/biomeos-init"),
            dest: "/init".to_string(),
            permissions: 0o755,
        })?;

        // BiomeOS CLI
        self.add_binary(BinarySpec {
            source: project_root.join("target/release/biome"),
            dest: "/bin/biome".to_string(),
            permissions: 0o755,
        })?;

        // Busybox for basic utilities (if available)
        if let Ok(busybox) = which::which("busybox") {
            self.add_binary(BinarySpec {
                source: busybox,
                dest: "/bin/busybox".to_string(),
                permissions: 0o755,
            })?;
        } else {
            warn!("busybox not found - some utilities may not be available");
        }

        Ok(())
    }

    /// Add a binary to the initramfs
    pub fn add_binary(&mut self, spec: BinarySpec) -> Result<()> {
        if !spec.source.exists() {
            warn!("Binary not found: {} (skipping)", spec.source.display());
            return Ok(());
        }

        info!("  • {} → {}", spec.source.display(), spec.dest);
        self.binaries.push(spec);
        Ok(())
    }

    /// Create essential directory structure
    pub fn create_directory_structure(&mut self) -> Result<()> {
        info!("📁 Creating directory structure...");

        let dirs = vec![
            "bin", "sbin", "usr/bin", "usr/sbin",
            "proc", "sys", "dev", "dev/pts",
            "tmp", "run", "var", "var/log",
            "biomeos", "biomeos/primals", "biomeos/configs",
            "etc", "root", "home",
        ];

        for dir in dirs {
            let path = self.root.join(dir);
            fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create directory: {}", dir))?;
            self.directories.push(dir.to_string());
        }

        info!("✅ Directory structure created");
        Ok(())
    }

    /// Copy binaries into initramfs root
    pub fn install_binaries(&self) -> Result<()> {
        info!("🔧 Installing binaries...");

        for binary in &self.binaries {
            let dest_path = self.root.join(binary.dest.trim_start_matches('/'));
            
            // Create parent directory
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy binary
            fs::copy(&binary.source, &dest_path)
                .with_context(|| format!(
                    "Failed to copy {} to {}",
                    binary.source.display(),
                    dest_path.display()
                ))?;

            // Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(binary.permissions);
                fs::set_permissions(&dest_path, perms)?;
            }
        }

        info!("✅ Binaries installed");
        Ok(())
    }

    /// Create init script that calls biomeos-init
    pub fn create_init_script(&self) -> Result<()> {
        info!("📝 Creating init script...");

        let init_path = self.root.join("init");
        let mut file = File::create(&init_path)?;

        writeln!(file, "#!/bin/sh")?;
        writeln!(file, "# BiomeOS Init Launcher")?;
        writeln!(file, "exec /init \"$@\"")?;

        // Set executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&init_path, perms)?;
        }

        info!("✅ Init script created");
        Ok(())
    }

    /// Build the initramfs archive
    pub fn build(&self, output: &Path) -> Result<()> {
        info!("🏗️  Building initramfs archive...");
        info!("Output: {}", output.display());

        // Create parent directory
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create tar.gz archive
        let tar_gz = File::create(output)
            .with_context(|| format!("Failed to create output file: {}", output.display()))?;
        let enc = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = Builder::new(enc);

        // Add all files from initramfs root
        tar.append_dir_all(".", &self.root)
            .context("Failed to add files to archive")?;

        tar.finish()
            .context("Failed to finish tar archive")?;

        let size = fs::metadata(output)?.len();
        info!("✅ Initramfs built: {} bytes ({:.2} MB)", 
              size, size as f64 / 1024.0 / 1024.0);

        Ok(())
    }
}

/// Kernel manager for selecting/downloading kernel
pub struct KernelManager {
    kernel_path: PathBuf,
    initramfs_path: PathBuf,
}

impl KernelManager {
    /// Detect system kernel or use custom
    pub fn detect_or_custom(custom: Option<PathBuf>) -> Result<Self> {
        if let Some(kernel) = custom {
            info!("Using custom kernel: {}", kernel.display());
            let initramfs = kernel.with_file_name("initramfs.img");
            
            return Ok(Self {
                kernel_path: kernel,
                initramfs_path: initramfs,
            });
        }

        // Try to find system kernel
        let kernel_paths = vec![
            "/boot/vmlinuz",
            "/boot/vmlinuz-linux",
            "/vmlinuz",
        ];

        for path in kernel_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                info!("Found system kernel: {}", path);
                
                let initramfs = Self::find_matching_initramfs(&path_buf)?;
                
                return Ok(Self {
                    kernel_path: path_buf,
                    initramfs_path: initramfs,
                });
            }
        }

        anyhow::bail!("No kernel found. Please specify a custom kernel path.")
    }

    fn find_matching_initramfs(kernel: &Path) -> Result<PathBuf> {
        // Try to find initramfs matching kernel
        let kernel_dir = kernel.parent()
            .context("Kernel has no parent directory")?;
        
        let initramfs_paths = vec![
            "initramfs.img",
            "initrd.img",
            "initramfs-linux.img",
        ];

        for name in initramfs_paths {
            let path = kernel_dir.join(name);
            if path.exists() {
                return Ok(path);
            }
        }

        // If not found, we'll create our own
        Ok(kernel_dir.join("biomeos-initramfs.img"))
    }

    pub fn kernel_path(&self) -> &Path {
        &self.kernel_path
    }

    pub fn initramfs_path(&self) -> &Path {
        &self.initramfs_path
    }
}

