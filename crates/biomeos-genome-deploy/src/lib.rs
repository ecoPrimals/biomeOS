// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! genomeBin Deployment Library
//!
//! Rust implementation of genomeBin deployment, replacing POSIX shell wrappers
//! with type-safe, cross-platform deployment infrastructure.
//!
//! ## Safe I/O Architecture
//!
//! Uses `std::fs::read()` for genomeBin extraction — 100% safe Rust with
//! zero `unsafe` blocks. For a one-shot deployment tool, the allocation
//! cost of reading the file is negligible compared to the disk I/O and
//! extraction time. This allows `#![forbid(unsafe_code)]` across the entire crate.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use colored::Colorize;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86-64 (AMD64)
    X86_64,
    /// ARM 64-bit (`AArch64`)
    Aarch64,
    /// ARM 32-bit (`ARMv7`)
    Armv7,
    /// RISC-V 64-bit
    Riscv64,
}

impl Architecture {
    /// Detect host architecture
    pub fn detect() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Self::X86_64),
            "aarch64" | "arm64" => Ok(Self::Aarch64),
            "armv7" => Ok(Self::Armv7),
            "riscv64" => Ok(Self::Riscv64),
            arch => bail!("Unsupported architecture: {arch}"),
        }
    }

    /// Get architecture string for binary lookup
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Armv7 => "armv7",
            Self::Riscv64 => "riscv64",
        }
    }
}

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux
    Linux,
    /// Android (Linux-family)
    Android,
    /// macOS
    MacOS,
    /// Windows
    Windows,
}

impl Platform {
    /// Detect host platform
    pub fn detect() -> Result<Self> {
        // Check for Android first (it reports as Linux)
        if Path::new("/system/build.prop").exists() {
            return Ok(Self::Android);
        }

        match std::env::consts::OS {
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::MacOS),
            "windows" => Ok(Self::Windows),
            os => bail!("Unsupported platform: {os}"),
        }
    }

    /// Get platform name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Android => "Android",
            Self::MacOS => "macOS",
            Self::Windows => "Windows",
        }
    }

    /// Check if platform supports abstract sockets
    #[must_use]
    pub const fn supports_abstract_sockets(&self) -> bool {
        matches!(self, Self::Android | Self::Linux)
    }
}

/// `GenomeBin` metadata
#[derive(Debug, Clone)]
pub struct GenomeMetadata {
    /// Genome name
    pub name: String,
    /// Genome version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Supported architectures
    pub architectures: Vec<Architecture>,
}

/// `GenomeBin` deployer
#[derive(Debug)]
pub struct GenomeDeployer {
    genome_path: PathBuf,
    architecture: Architecture,
    platform: Platform,
    install_dir: Option<PathBuf>,
}

impl GenomeDeployer {
    /// Create new deployer for genomeBin file
    pub fn new<P: AsRef<Path>>(genome_path: P) -> Result<Self> {
        let genome_path = genome_path.as_ref().to_path_buf();

        if !genome_path.exists() {
            bail!("genomeBin not found: {}", genome_path.display());
        }

        let architecture = Architecture::detect()?;
        let platform = Platform::detect()?;

        Ok(Self {
            genome_path,
            architecture,
            platform,
            install_dir: None,
        })
    }

    /// Set custom installation directory
    #[must_use]
    pub fn with_install_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.install_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Get default installation directory for current platform
    ///
    /// Uses `$HOME` env instead of `dirs` crate (ecoBin: no C deps).
    /// Root detection via $EUID/$USER instead of `nix::Uid`.
    pub(crate) fn default_install_dir(&self, primal_name: &str) -> PathBuf {
        let home_dir = || -> PathBuf {
            std::env::var("HOME").map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
        };

        let is_root = std::env::var("EUID")
            .or_else(|_| std::env::var("UID"))
            .map_or_else(
                |_| std::env::var("USER").map(|u| u == "root").unwrap_or(false),
                |uid| uid == "0",
            );

        match self.platform {
            Platform::Android => PathBuf::from(format!("/data/local/tmp/{primal_name}")),
            Platform::Linux => {
                if is_root {
                    PathBuf::from(format!("/opt/{primal_name}"))
                } else {
                    home_dir().join(format!(".local/{primal_name}"))
                }
            }
            Platform::MacOS => home_dir().join(format!("Library/{primal_name}")),
            Platform::Windows => std::env::var("LOCALAPPDATA")
                .map_or_else(|_| PathBuf::from("C:\\ProgramData"), PathBuf::from)
                .join(primal_name),
        }
    }

    /// Extract genomeBin archive using safe buffered I/O
    ///
    /// Reads the genomeBin file into memory and extracts the embedded tar.gz archive.
    /// For a one-shot deployment operation, the read allocation is negligible
    /// compared to disk I/O and decompression time.
    fn extract_archive(&self, install_dir: &Path) -> Result<()> {
        println!("{}", "Extracting genomeBin...".blue());

        let file = File::open(&self.genome_path)?;
        let file_size = file.metadata()?.len();

        // Truncation acceptable: f64 mantissa is 52 bits; display-only for human-readable MB
        #[expect(
            clippy::cast_precision_loss,
            reason = "truncation acceptable: f64 mantissa is 52 bits; display-only human-readable MB"
        )]
        let size_mb = file_size as f64 / 1_048_576.0;
        println!(
            "{}",
            format!("  Reading {size_mb:.1}MB genomeBin...").dimmed()
        );

        // Safe read — no unsafe, no mmap, no SIGBUS risk
        let mut file = file;
        let capacity = usize::try_from(file_size).unwrap_or(0);
        let mut data = Vec::with_capacity(capacity);
        file.read_to_end(&mut data)?;

        // Find __ARCHIVE_START__ marker
        let marker = b"__ARCHIVE_START__\n";
        let archive_start = data
            .windows(marker.len())
            .position(|window| window == marker)
            .context("Archive marker not found in genomeBin")?
            + marker.len();

        // Extract tar.gz from the read data
        let archive_slice = &data[archive_start..];
        let decoder = GzDecoder::new(archive_slice);
        let mut archive = Archive::new(decoder);

        // Create progress bar
        let pb = ProgressBar::new_spinner();
        let style = ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .context("valid progress bar template")?;
        pb.set_style(style);
        pb.set_message("Extracting binaries...");

        // Extract to temp dir first
        let temp_dir = tempfile::tempdir()?;
        archive.unpack(temp_dir.path())?;

        // Copy architecture-specific binaries
        let arch_dir = temp_dir.path().join(self.architecture.as_str());
        if !arch_dir.exists() {
            bail!(
                "No binaries found for architecture: {}",
                self.architecture.as_str()
            );
        }

        // Copy all files from arch dir to install dir
        for entry in fs::read_dir(&arch_dir)? {
            let entry = entry?;
            let dest = install_dir.join(entry.file_name());
            fs::copy(entry.path(), &dest)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                let mut perms = fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest, perms)?;
            }

            pb.set_message(format!(
                "Installed: {}",
                entry.file_name().to_string_lossy()
            ));
        }

        pb.finish_with_message("✓ Extraction complete".green().to_string());
        Ok(())
    }

    /// Deploy genomeBin
    pub fn deploy(&self) -> Result<()> {
        // Print header
        println!(
            "\n{}",
            "╔══════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║           genomeBin Universal Deployer              ║".cyan()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════╝".cyan()
        );
        println!();

        // Detect environment
        println!(
            "{} {}",
            "Architecture:".blue(),
            self.architecture.as_str().green()
        );
        println!("{} {}", "Platform:".blue(), self.platform.name().green());
        println!();

        // Determine install directory
        let genome_name = self
            .genome_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("primal")
            .trim_end_matches(".genome");

        let install_dir = self
            .install_dir
            .clone()
            .unwrap_or_else(|| self.default_install_dir(genome_name));

        println!(
            "{} {}",
            "Install directory:".blue(),
            install_dir.display().to_string().green()
        );

        // Create install directory
        fs::create_dir_all(&install_dir).context("Failed to create installation directory")?;

        // Extract and install
        self.extract_archive(&install_dir)?;

        // Verify installation
        println!();
        Self::verify_installation(&install_dir, genome_name)?;

        // Print next steps
        self.print_next_steps(&install_dir, genome_name);

        Ok(())
    }

    /// Verify installation
    fn verify_installation(install_dir: &Path, primal_name: &str) -> Result<()> {
        println!("{}", "Verifying installation...".blue());

        let binary_path = install_dir.join(primal_name);
        if !binary_path.exists() {
            bail!(
                "Binary not found after installation: {}",
                binary_path.display()
            );
        }

        // Try to get version
        let version_output = std::process::Command::new(&binary_path)
            .arg("--version")
            .output();

        match version_output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!(
                    "{} {} {}",
                    "✓".green(),
                    primal_name.green(),
                    version.trim().green()
                );
            }
            _ => {
                println!(
                    "{} {} {}",
                    "✓".green(),
                    primal_name.green(),
                    "(version check skipped)".yellow()
                );
            }
        }

        Ok(())
    }

    /// Print next steps
    fn print_next_steps(&self, install_dir: &Path, primal_name: &str) {
        println!();
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║              Deployment Complete! 🎊                 ║".cyan()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════╝".cyan()
        );
        println!();
        println!("{}", "Next steps:".blue().bold());

        let is_root = std::env::var("EUID")
            .or_else(|_| std::env::var("UID"))
            .map(|uid| uid == "0")
            .unwrap_or(false);
        if !is_root && self.platform != Platform::Android {
            println!("1. Add to PATH:");
            println!("   export PATH=\"$PATH:{}\"", install_dir.display());
            println!();
        }

        println!("2. Start {primal_name}:");
        println!("   {}/{}", install_dir.display(), primal_name);
        println!();

        if self.platform.supports_abstract_sockets() {
            println!("{}", "Platform features:".blue());
            if self.platform == Platform::Android {
                println!("  • Abstract socket namespace (@biomeos_{primal_name})");
                println!("  • Android HSM integration available");
            } else {
                println!("  • Unix socket support");
                println!("  • Abstract socket support");
            }
            println!();
        }

        println!(
            "{}",
            "genomeBin deployment complete! 🧬"
                .to_string()
                .green()
                .bold()
        );
    }
}

// Add nix dependency for UID check

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
    use std::io::Write;
    use tempfile::TempDir;

    // ============================================================================
    // Architecture Tests
    // ============================================================================

    #[test]
    fn test_architecture_as_str() {
        assert_eq!(Architecture::X86_64.as_str(), "x86_64");
        assert_eq!(Architecture::Aarch64.as_str(), "aarch64");
        assert_eq!(Architecture::Armv7.as_str(), "armv7");
        assert_eq!(Architecture::Riscv64.as_str(), "riscv64");
    }

    #[test]
    fn test_architecture_detect_returns_known_arch() {
        // detect() should succeed on any supported platform
        let result = Architecture::detect();
        // On test systems, this should succeed (we're running on a known arch)
        assert!(
            result.is_ok(),
            "Should detect architecture on supported system"
        );
        let arch = result.unwrap();
        // Verify it returns a valid architecture string
        let arch_str = arch.as_str();
        assert!(
            ["x86_64", "aarch64", "armv7", "riscv64"].contains(&arch_str),
            "Should be a known architecture"
        );
    }

    #[test]
    fn test_architecture_equality() {
        assert_eq!(Architecture::X86_64, Architecture::X86_64);
        assert_ne!(Architecture::X86_64, Architecture::Aarch64);
    }

    #[test]
    fn test_architecture_clone() {
        let arch = Architecture::Aarch64;
        let cloned = arch;
        assert_eq!(arch, cloned);
    }

    #[test]
    fn test_architecture_debug() {
        let arch = Architecture::X86_64;
        let debug_str = format!("{arch:?}");
        assert!(debug_str.contains("X86_64"));
    }

    // ============================================================================
    // Platform Tests
    // ============================================================================

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::Linux.name(), "Linux");
        assert_eq!(Platform::Android.name(), "Android");
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Windows.name(), "Windows");
    }

    #[test]
    fn test_platform_supports_abstract_sockets() {
        assert!(Platform::Android.supports_abstract_sockets());
        assert!(Platform::Linux.supports_abstract_sockets());
        assert!(!Platform::MacOS.supports_abstract_sockets());
        assert!(!Platform::Windows.supports_abstract_sockets());
    }

    #[test]
    fn test_platform_detect_returns_known_platform() {
        let result = Platform::detect();
        assert!(result.is_ok(), "Should detect platform on supported system");
        let platform = result.unwrap();
        let name = platform.name();
        assert!(
            ["Linux", "Android", "macOS", "Windows"].contains(&name),
            "Should be a known platform"
        );
    }

    #[test]
    fn test_platform_equality() {
        assert_eq!(Platform::Linux, Platform::Linux);
        assert_ne!(Platform::Linux, Platform::Android);
    }

    #[test]
    fn test_platform_clone() {
        let platform = Platform::Linux;
        let cloned = platform;
        assert_eq!(platform, cloned);
    }

    #[test]
    fn test_platform_debug() {
        let platform = Platform::Linux;
        let debug_str = format!("{platform:?}");
        assert!(debug_str.contains("Linux"));
    }

    // ============================================================================
    // GenomeMetadata Tests
    // ============================================================================

    #[test]
    fn test_genome_metadata_creation() {
        let metadata = GenomeMetadata {
            name: "test-primal".to_string(),
            version: "1.0.0".to_string(),
            description: "Test primal for testing".to_string(),
            architectures: vec![Architecture::X86_64, Architecture::Aarch64],
        };

        assert_eq!(metadata.name, "test-primal");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "Test primal for testing");
        assert_eq!(metadata.architectures.len(), 2);
    }

    #[test]
    fn test_genome_metadata_clone() {
        let metadata = GenomeMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "desc".to_string(),
            architectures: vec![Architecture::X86_64],
        };
        let cloned = metadata.clone();
        assert_eq!(metadata.name, cloned.name);
        assert_eq!(metadata.version, cloned.version);
    }

    #[test]
    fn test_genome_metadata_debug() {
        let metadata = GenomeMetadata {
            name: "beardog".to_string(),
            version: "0.9.0".to_string(),
            description: "Security primal".to_string(),
            architectures: vec![Architecture::X86_64],
        };
        let debug_str = format!("{metadata:?}");
        assert!(debug_str.contains("beardog"));
        assert!(debug_str.contains("0.9.0"));
    }

    // ============================================================================
    // GenomeDeployer Tests
    // ============================================================================

    #[test]
    fn test_genome_deployer_new_file_not_found() {
        let result = GenomeDeployer::new("/nonexistent/path/to/genome.genome");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_genome_deployer_new_with_existing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        // Create a dummy genome file
        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let result = GenomeDeployer::new(&genome_path);
        assert!(result.is_ok(), "Should create deployer for existing file");
    }

    #[test]
    fn test_genome_deployer_with_install_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let install_path = temp_dir.path().join("custom_install");
        std::fs::create_dir_all(&install_path).expect("create");

        let deployer = GenomeDeployer::new(&genome_path)
            .expect("Should create deployer")
            .with_install_dir(&install_path);

        let result = deployer.deploy();
        assert!(result.is_err());
    }

    #[test]
    fn test_genome_deployer_deploy_with_custom_dir_fails_without_archive() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

        let custom_dir = temp_dir.path().join("install");
        std::fs::create_dir_all(&custom_dir).expect("create");
        let deployer_with_dir = deployer.with_install_dir(&custom_dir);
        let result = deployer_with_dir.deploy();
        assert!(result.is_err());
    }

    #[test]
    fn test_genome_deployer_architecture_field() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

        // Verify architecture was detected
        let arch_str = deployer.architecture.as_str();
        assert!(
            ["x86_64", "aarch64", "armv7", "riscv64"].contains(&arch_str),
            "Should have valid architecture"
        );
    }

    #[test]
    fn test_genome_deployer_platform_field() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

        // Verify platform was detected
        let platform_name = deployer.platform.name();
        assert!(
            ["Linux", "Android", "macOS", "Windows"].contains(&platform_name),
            "Should have valid platform"
        );
    }

    // ============================================================================
    // Integration-style Tests (without actual deployment)
    // ============================================================================

    #[test]
    fn test_deploy_fails_without_archive_marker() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("invalid.genome");

        // Create file without archive marker
        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"not a valid genome file")
            .expect("Failed to write");

        let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");
        let result = deployer.deploy();

        // Should fail because no archive marker
        assert!(result.is_err(), "Should fail without archive marker");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("marker") || err.contains("Archive"),
            "Error should mention missing marker"
        );
    }

    #[test]
    fn test_all_platform_install_dirs_are_unique() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let genome_path = temp_dir.path().join("test.genome");

        let mut file = File::create(&genome_path).expect("Failed to create file");
        file.write_all(b"dummy content").expect("Failed to write");

        let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

        // Generate paths for different primals
        let path1 = deployer.default_install_dir("primal1");
        let path2 = deployer.default_install_dir("primal2");

        // They should be different
        assert_ne!(
            path1, path2,
            "Different primals should have different install dirs"
        );
    }

    #[test]
    fn test_platform_abstract_socket_consistency() {
        // Platforms that support abstract sockets should be Linux-family
        let linux_supports = Platform::Linux.supports_abstract_sockets();
        let android_supports = Platform::Android.supports_abstract_sockets();
        let macos_supports = Platform::MacOS.supports_abstract_sockets();
        let windows_supports = Platform::Windows.supports_abstract_sockets();

        // Linux and Android are Linux-family, should support abstract sockets
        assert!(linux_supports);
        assert!(android_supports);

        // macOS and Windows should not
        assert!(!macos_supports);
        assert!(!windows_supports);
    }

    #[test]
    fn test_default_install_dir_linux_user_uses_dot_local() {
        let temp = TempDir::new().expect("temp dir");
        let genome_path = temp.path().join("primal.genome");
        let mut f = File::create(&genome_path).expect("create");
        f.write_all(b"x").expect("write");
        let deployer = GenomeDeployer::new(&genome_path).expect("deployer");
        if deployer.platform != Platform::Linux {
            return;
        }
        let p = deployer.default_install_dir("myprimal");
        let s = p.to_string_lossy();
        assert!(
            s.contains(".local/myprimal") || s.contains("/opt/myprimal"),
            "unexpected install dir: {s}"
        );
    }

    #[test]
    fn test_extract_archive_wrong_arch_fails() {
        let temp = TempDir::new().expect("temp dir");
        let genome_path = temp.path().join("wrongarch.genome");
        write_genome_bin_with_arch_dir(&genome_path, "definitely_not_a_real_arch_triple");
        let install = temp.path().join("out");
        std::fs::create_dir_all(&install).expect("install");
        let deployer = GenomeDeployer::new(&genome_path)
            .expect("deployer")
            .with_install_dir(&install);
        let err = deployer
            .deploy()
            .expect_err("deploy should fail when arch dir missing");
        let msg = err.to_string();
        assert!(
            msg.contains("No binaries") || msg.contains("architecture"),
            "unexpected: {msg}"
        );
    }

    #[test]
    fn test_deploy_full_success_with_stub_binary() {
        let temp = TempDir::new().expect("temp dir");
        let genome_path = temp.path().join("stub.genome");
        let arch = Architecture::detect().expect("arch").as_str();
        write_genome_bin_with_arch_dir(&genome_path, arch);
        let install = temp.path().join("install");
        std::fs::create_dir_all(&install).expect("install");
        let deployer = GenomeDeployer::new(&genome_path)
            .expect("deployer")
            .with_install_dir(&install);
        deployer.deploy().expect("deploy");
        let binary = install.join("stub");
        assert!(binary.exists(), "stub binary should be installed");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&binary)
                .expect("meta")
                .permissions()
                .mode();
            assert!(mode & 0o111 != 0, "binary should be executable");
        }
    }

    /// Minimal genomeBin: marker + gzipped tar with `{arch}/stub` executable script.
    fn write_genome_bin_with_arch_dir(path: &std::path::Path, arch_dir: &str) {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        use tar::{Builder, Header};

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut tar = Builder::new(&mut encoder);
            let script =
                b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo \"stub 1.0.0\"; fi\nexit 0\n";
            let mut header = Header::new_gnu();
            let inner_path = format!("{arch_dir}/stub");
            header.set_path(&inner_path).expect("path");
            header.set_size(script.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append(&header, &script[..]).expect("append");
            tar.finish().expect("finish");
        }
        let compressed = encoder.finish().expect("gz finish");
        let mut f = File::create(path).expect("create genome");
        f.write_all(b"stub-genome-header\n__ARCHIVE_START__\n")
            .expect("marker");
        f.write_all(&compressed).expect("gz");
    }

    #[test]
    fn test_verify_installation_skips_version_when_binary_nonzero() {
        let temp = TempDir::new().expect("temp dir");
        let install = temp.path().join("verify");
        std::fs::create_dir_all(&install).expect("dir");
        let primal = install.join("verifybin");
        std::fs::write(&primal, b"not executable").expect("file");
        let r = GenomeDeployer::verify_installation(&install, "verifybin");
        assert!(r.is_ok());
    }
}
