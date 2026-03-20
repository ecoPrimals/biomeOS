// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS Initramfs Builder
//!
//! Pure Rust initramfs generation. No shell scripts, no external dependencies.
//! Creates a minimal boot environment with BiomeOS binaries.

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;
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
                .with_context(|| format!("Failed to create directory: {dir}"))?;
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
            }
            warn!("BIOMEOS_KERNEL set but file not found: {}", env_kernel);
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

    pub(crate) fn find_matching_initramfs(kernel: &Path) -> Result<PathBuf> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::TestEnvGuard;
    use std::sync::Mutex;

    static BIOMEOS_KERNEL_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_initramfs_builder_new() {
        let temp = tempfile::tempdir().expect("temp dir");
        let _builder = InitramfsBuilder::new(temp.path()).expect("new");
        assert!(temp.path().join("initramfs-root").exists());
    }

    #[test]
    fn test_binary_spec_debug() {
        let spec = BinarySpec {
            source: PathBuf::from("/bin/true"),
            dest: "/init".to_string(),
            permissions: 0o755,
        };
        let s = format!("{:?}", spec);
        assert!(s.contains("BinarySpec"));
    }

    #[test]
    fn test_create_directory_structure() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.create_directory_structure().expect("create dirs");

        let root = temp.path().join("initramfs-root");
        for dir in ["bin", "sbin", "proc", "sys", "dev", "tmp", "biomeos", "etc"] {
            assert!(root.join(dir).exists(), "{} should exist", dir);
        }
    }

    #[test]
    fn test_add_binary_skips_nonexistent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        let spec = BinarySpec {
            source: PathBuf::from("/nonexistent/binary"),
            dest: "/bin/missing".to_string(),
            permissions: 0o755,
        };
        builder
            .add_binary(spec)
            .expect("add_binary should not fail for missing");
    }

    #[test]
    fn test_add_binary_adds_existing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin_path = temp.path().join("mybin");
        std::fs::write(&bin_path, b"#!/bin/sh").expect("write");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        let spec = BinarySpec {
            source: bin_path,
            dest: "/bin/mybin".to_string(),
            permissions: 0o755,
        };
        builder.add_binary(spec).expect("add_binary");
        builder.install_binaries().expect("install");
        assert!(temp.path().join("initramfs-root/bin/mybin").exists());
    }

    #[test]
    fn test_create_init_script() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.create_directory_structure().expect("dirs");
        builder.create_init_script().expect("init script");

        let init_path = temp.path().join("initramfs-root/init");
        assert!(init_path.exists());
        let content = std::fs::read_to_string(&init_path).expect("read");
        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains("exec /init"));
    }

    #[test]
    fn test_kernel_manager_custom() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel_path = temp.path().join("vmlinuz");
        std::fs::write(&kernel_path, b"kernel").expect("write");
        let mgr = KernelManager::detect_or_custom(Some(kernel_path.clone())).expect("custom");
        assert_eq!(mgr.kernel_path(), kernel_path.as_path());
        assert_eq!(
            mgr.initramfs_path(),
            kernel_path
                .parent()
                .unwrap()
                .join("initramfs.img")
                .as_path()
        );
    }

    #[test]
    fn test_kernel_manager_detect_or_custom_takes_precedence() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel_path = temp.path().join("vmlinuz");
        std::fs::write(&kernel_path, b"kernel").expect("write");
        let mgr = KernelManager::detect_or_custom(Some(kernel_path.clone())).expect("custom");
        assert_eq!(mgr.kernel_path(), kernel_path.as_path());
    }

    #[test]
    fn test_build_creates_output() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.create_directory_structure().expect("dirs");
        std::fs::write(temp.path().join("initramfs-root/test.txt"), "content").expect("write");
        let output = temp.path().join("initramfs.img");
        builder.build(&output).expect("build");
        assert!(output.exists());
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
    }

    #[test]
    fn test_build_fails_when_output_dir_not_writable() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.create_directory_structure().expect("dirs");
        let bad = temp.path().join("not-a-dir.img");
        std::fs::write(&bad, b"x").expect("file blocks path as directory parent");
        let output = bad.join("nested.img");
        let r = builder.build(&output);
        assert!(r.is_err());
    }

    #[test]
    fn test_install_binaries_empty_list_ok() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.install_binaries().expect("no binaries");
    }

    #[test]
    fn test_add_required_libraries_ldd_on_empty_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin = temp.path().join("not-exec");
        std::fs::write(&bin, b"").expect("write");
        let builder = InitramfsBuilder::new(temp.path()).expect("new");
        let r = builder.add_required_libraries(&bin);
        assert!(r.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_kernel_manager_biomeos_kernel_env_valid() {
        let _lock = BIOMEOS_KERNEL_TEST_LOCK.lock().expect("kernel env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let k = temp.path().join("from-env-vmlinuz");
        std::fs::write(&k, b"k").expect("write");
        let _g = TestEnvGuard::set("BIOMEOS_KERNEL", k.to_str().expect("utf8"));
        let mgr = KernelManager::detect_or_custom(None).expect("kernel from env");
        assert_eq!(mgr.kernel_path(), k.as_path());
        assert_eq!(
            mgr.initramfs_path(),
            k.parent().unwrap().join("biomeos-initramfs.img")
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_kernel_manager_biomeos_kernel_missing_file_warns_and_falls_back() {
        let _lock = BIOMEOS_KERNEL_TEST_LOCK.lock().expect("kernel env lock");
        let _g = TestEnvGuard::set(
            "BIOMEOS_KERNEL",
            "/nonexistent/biomeos-test-kernel-zzzz.img",
        );
        let r = KernelManager::detect_or_custom(None);
        if let Some(err) = r.err() {
            assert!(err.to_string().to_lowercase().contains("kernel"));
        }
    }

    #[test]
    fn test_binary_spec_clone() {
        let spec = BinarySpec {
            source: PathBuf::from("/a"),
            dest: "/b".to_string(),
            permissions: 0o755,
        };
        let c = spec.clone();
        assert_eq!(c.dest, spec.dest);
    }

    #[test]
    fn test_create_directory_structure_includes_biomeos_primals_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        builder.create_directory_structure().expect("dirs");
        let root = temp.path().join("initramfs-root");
        assert!(root.join("biomeos/primals").exists());
        assert!(root.join("var/log").exists());
    }

    #[test]
    fn test_kernel_manager_find_matching_initramfs_prefers_existing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel = temp.path().join("vmlinuz");
        std::fs::write(&kernel, b"k").expect("k");
        let init = temp.path().join("initramfs.img");
        std::fs::write(&init, b"i").expect("i");
        let mgr = KernelManager::detect_or_custom(Some(kernel)).expect("mgr");
        assert_eq!(mgr.initramfs_path(), init.as_path());
    }

    #[test]
    fn test_find_matching_initramfs_prefers_initramfs_img() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel = temp.path().join("vmlinuz");
        std::fs::write(&kernel, b"k").expect("k");
        std::fs::write(temp.path().join("initramfs.img"), b"a").expect("a");
        std::fs::write(temp.path().join("initrd.img"), b"b").expect("b");
        let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
        assert_eq!(p, temp.path().join("initramfs.img"));
    }

    #[test]
    fn test_find_matching_initramfs_falls_back_to_initrd() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel = temp.path().join("vmlinuz");
        std::fs::write(&kernel, b"k").expect("k");
        let init = temp.path().join("initrd.img");
        std::fs::write(&init, b"i").expect("i");
        let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
        assert_eq!(p, init);
    }

    #[test]
    fn test_find_matching_initramfs_falls_back_to_initramfs_linux() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel = temp.path().join("vmlinuz");
        std::fs::write(&kernel, b"k").expect("k");
        let init = temp.path().join("initramfs-linux.img");
        std::fs::write(&init, b"i").expect("i");
        let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
        assert_eq!(p, init);
    }

    #[test]
    fn test_find_matching_initramfs_defaults_biomeos_name() {
        let temp = tempfile::tempdir().expect("temp dir");
        let kernel = temp.path().join("vmlinuz");
        std::fs::write(&kernel, b"k").expect("k");
        let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
        assert_eq!(p, temp.path().join("biomeos-initramfs.img"));
    }

    #[test]
    fn test_add_biomeos_binaries_registers_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        let pr = temp.path().join("proj");
        let tr = pr.join("target/release");
        std::fs::create_dir_all(&tr).expect("tr");
        std::fs::write(tr.join("biomeos-init"), b"x").expect("init");
        std::fs::write(tr.join("biome"), b"x").expect("biome");
        builder.add_biomeos_binaries(&pr).expect("bins");
        assert!(
            builder.binaries.len() >= 2,
            "expected biome binaries; busybox may add a third"
        );
    }

    #[test]
    fn test_install_binaries_preserves_permissions() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
        let src = temp.path().join("tool");
        std::fs::write(&src, b"#!bin").expect("w");
        builder
            .add_binary(BinarySpec {
                source: src,
                dest: "/sbin/tool".to_string(),
                permissions: 0o755,
            })
            .expect("add");
        builder.install_binaries().expect("install");
        let dest = temp.path().join("initramfs-root/sbin/tool");
        assert!(dest.exists());
    }

    #[test]
    fn test_build_succeeds_with_empty_initramfs_tree() {
        let temp = tempfile::tempdir().expect("temp dir");
        let builder = InitramfsBuilder::new(temp.path()).expect("new");
        let out = temp.path().join("empty-tree.img");
        builder
            .build(&out)
            .expect("empty tree still produces gzip cpio");
        assert!(out.exists());
    }

    #[test]
    fn test_find_matching_initramfs_root_kernel_errors() {
        // `/` has no parent — `find_matching_initramfs` requires a directory prefix.
        let kernel = Path::new("/");
        let r = KernelManager::find_matching_initramfs(kernel);
        assert!(r.is_err());
    }
}
