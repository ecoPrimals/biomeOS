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
use tracing::{info, warn};

/// Builder for creating initramfs (initial RAM filesystem) images
pub struct InitramfsBuilder {
    root: PathBuf,
    binaries: Vec<BinarySpec>,
    directories: Vec<String>,
}

/// Specification for a binary to include in the initramfs
#[derive(Debug, Clone)]
pub struct BinarySpec {
    /// Source path on the host system
    pub source: PathBuf,
    /// Destination path in the initramfs (e.g., "/bin/busybox")
    pub dest: String,
    /// Unix permissions (e.g., 0o755 for executable)
    pub permissions: u32,
}

impl InitramfsBuilder {
    /// Creates a new initramfs builder
    ///
    /// # Arguments
    ///
    /// * `work_dir` - Working directory for building the initramfs
    ///
    /// # Errors
    ///
    /// Returns an error if the working directory cannot be created.
    pub fn new<P: AsRef<Path>>(work_dir: P) -> Result<Self> {
        let root = work_dir.as_ref().join("initramfs-root");
        fs::create_dir_all(&root).context("Failed to create initramfs work directory")?;

        Ok(Self {
            root,
            binaries: Vec::new(),
            directories: Vec::new(),
        })
    }

    /// Add BiomeOS core binaries
    pub fn add_biomeos_binaries(&mut self, project_root: &Path) -> Result<()> {
        info!("📦 Adding BiomeOS binaries...");

        // BiomeOS init (PID 1) - this is the actual init binary
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

    /// Copy required dynamic libraries for binaries
    /// This is needed when binaries are dynamically linked
    pub fn add_required_libraries(&self, binary_path: &Path) -> Result<()> {
        use std::process::Command;

        info!(
            "📚 Adding required libraries for {}...",
            binary_path.display()
        );

        // Use ldd to find required libraries
        let output = Command::new("ldd")
            .arg(binary_path)
            .output()
            .context("Failed to run ldd")?;

        if !output.status.success() {
            // Binary might be statically linked or ldd not available
            warn!(
                "ldd failed for {} - assuming statically linked",
                binary_path.display()
            );
            return Ok(());
        }

        let ldd_output = String::from_utf8_lossy(&output.stdout);

        for line in ldd_output.lines() {
            // Parse lines like: "libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x...)"
            if let Some(lib_path) = line.split("=>").nth(1) {
                let lib_path = lib_path.split_whitespace().next();
                if let Some(lib_path_str) = lib_path {
                    let lib_src = Path::new(lib_path_str);
                    if lib_src.exists() && lib_src.is_absolute() {
                        // Preserve the full directory structure (e.g., /lib/x86_64-linux-gnu/)
                        let relative = lib_src.strip_prefix("/").unwrap_or(lib_src);
                        let dest_full = self.root.join(relative);

                        // Create parent directories
                        if let Some(parent) = dest_full.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        // Copy library
                        if !dest_full.exists() {
                            fs::copy(lib_src, &dest_full).with_context(|| {
                                format!("Failed to copy library: {}", lib_src.display())
                            })?;
                            info!("  ✓ Copied: {}", lib_src.display());
                        }
                    }
                }
            }

            // Also handle dynamic linker (e.g., /lib64/ld-linux-x86-64.so.2)
            if line.contains("ld-linux") && line.starts_with("\t/") {
                if let Some(ld_path_str) = line.split_whitespace().next() {
                    let ld_src = Path::new(ld_path_str);
                    if ld_src.exists() {
                        let relative = ld_src.strip_prefix("/").unwrap_or(ld_src);
                        let dest_full = self.root.join(relative);

                        if let Some(parent) = dest_full.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        if !dest_full.exists() {
                            fs::copy(ld_src, &dest_full).with_context(|| {
                                format!("Failed to copy dynamic linker: {}", ld_src.display())
                            })?;
                            info!("  ✓ Copied dynamic linker: {}", ld_src.display());
                        }
                    }
                }
            }
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
            "bin",
            "sbin",
            "usr/bin",
            "usr/sbin",
            "proc",
            "sys",
            "dev",
            "dev/pts",
            "tmp",
            "run",
            "var",
            "var/log",
            "biomeos",
            "biomeos/primals",
            "biomeos/configs",
            "etc",
            "root",
            "home",
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
            fs::copy(&binary.source, &dest_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    binary.source.display(),
                    dest_path.display()
                )
            })?;

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

        // Create CPIO archive (newc format) compressed with gzip
        // The kernel expects a gzipped CPIO archive, not tar.gz
        let output_file = File::create(output)
            .with_context(|| format!("Failed to create output file: {}", output.display()))?;

        let mut encoder = GzEncoder::new(output_file, Compression::best());

        // Use find + cpio to create proper CPIO archive
        // This is more reliable than trying to create CPIO in pure Rust
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && find . | cpio -o -H newc 2>/dev/null",
                self.root.display()
            ))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn cpio command")?;

        // Pipe cpio output to gzip encoder
        if let Some(mut stdout) = status.stdout {
            std::io::copy(&mut stdout, &mut encoder).context("Failed to compress CPIO archive")?;
        }

        encoder
            .finish()
            .context("Failed to finish gzip compression")?;

        let size = fs::metadata(output)?.len();
        info!(
            "✅ Initramfs built: {} bytes ({:.2} MB)",
            size,
            size as f64 / 1024.0 / 1024.0
        );

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

        // Check environment variable first
        if let Ok(env_kernel) = std::env::var("BIOMEOS_KERNEL") {
            let kernel_path = PathBuf::from(&env_kernel);
            if kernel_path.exists() {
                info!(
                    "Using kernel from BIOMEOS_KERNEL: {}",
                    kernel_path.display()
                );
                let initramfs = kernel_path.with_file_name("biomeos-initramfs.img");

                return Ok(Self {
                    kernel_path,
                    initramfs_path: initramfs,
                });
            } else {
                warn!("BIOMEOS_KERNEL set but file not found: {}", env_kernel);
            }
        }

        // Try to find system kernel
        let kernel_paths = vec!["/boot/vmlinuz", "/boot/vmlinuz-linux", "/vmlinuz"];

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

        anyhow::bail!(
            "No kernel found. Please specify a kernel:\n\
             1. Set BIOMEOS_KERNEL environment variable: export BIOMEOS_KERNEL=/path/to/vmlinuz\n\
             2. Or run scripts/prepare-kernel.sh to copy system kernel to accessible location"
        )
    }

    fn find_matching_initramfs(kernel: &Path) -> Result<PathBuf> {
        // Try to find initramfs matching kernel
        let kernel_dir = kernel.parent().context("Kernel has no parent directory")?;

        let initramfs_paths = vec!["initramfs.img", "initrd.img", "initramfs-linux.img"];

        for name in initramfs_paths {
            let path = kernel_dir.join(name);
            if path.exists() {
                return Ok(path);
            }
        }

        // If not found, we'll create our own
        Ok(kernel_dir.join("biomeos-initramfs.img"))
    }

    /// Get the kernel image path
    pub fn kernel_path(&self) -> &Path {
        &self.kernel_path
    }

    /// Get the initramfs image path
    pub fn initramfs_path(&self) -> &Path {
        &self.initramfs_path
    }
}
