// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! genomeBin extraction and installation.

use crate::types::{Architecture, Platform};
use anyhow::{Context, Result, bail};
use colored::Colorize;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Detect whether the current user is root ($EUID/$UID == "0" or $USER == "root").
fn is_root_user() -> bool {
    std::env::var("EUID")
        .or_else(|_| std::env::var("UID"))
        .map_or_else(
            |_| std::env::var("USER").map(|u| u == "root").unwrap_or(false),
            |uid| uid == "0",
        )
}

/// `GenomeBin` deployer
#[derive(Debug)]
pub struct GenomeDeployer {
    genome_path: PathBuf,
    /// Detected host architecture (crate tests).
    pub(crate) architecture: Architecture,
    /// Detected host platform (crate tests).
    pub(crate) platform: Platform,
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
    pub(crate) fn default_install_dir(&self, primal_name: &str) -> PathBuf {
        let home_dir = || -> PathBuf {
            std::env::var("HOME").map_or_else(|_| std::env::temp_dir(), PathBuf::from)
        };

        let is_root = is_root_user();

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
    pub(crate) fn verify_installation(install_dir: &Path, primal_name: &str) -> Result<()> {
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

        if !is_root_user() && self.platform != Platform::Android {
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
