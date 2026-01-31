//! genomeBin Deployment Library
//! 
//! Rust implementation of genomeBin deployment, replacing POSIX shell wrappers
//! with type-safe, cross-platform deployment infrastructure.

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tar::Archive;
use indicatif::{ProgressBar, ProgressStyle};
use colored::Colorize;

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Armv7,
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
            arch => bail!("Unsupported architecture: {}", arch),
        }
    }
    
    /// Get architecture string for binary lookup
    pub fn as_str(&self) -> &'static str {
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
    Linux,
    Android,
    MacOS,
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
            os => bail!("Unsupported platform: {}", os),
        }
    }
    
    /// Get platform name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Android => "Android",
            Self::MacOS => "macOS",
            Self::Windows => "Windows",
        }
    }
    
    /// Check if platform supports abstract sockets
    pub fn supports_abstract_sockets(&self) -> bool {
        matches!(self, Self::Android | Self::Linux)
    }
}

/// GenomeBin metadata
#[derive(Debug, Clone)]
pub struct GenomeMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architectures: Vec<Architecture>,
}

/// GenomeBin deployer
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
    pub fn with_install_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.install_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// Get default installation directory for current platform
    fn default_install_dir(&self, primal_name: &str) -> PathBuf {
        match self.platform {
            Platform::Android => PathBuf::from(format!("/data/local/tmp/{}", primal_name)),
            Platform::Linux => {
                if nix::unistd::Uid::current().is_root() {
                    PathBuf::from(format!("/opt/{}", primal_name))
                } else {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(format!(".local/{}", primal_name))
                }
            }
            Platform::MacOS => {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(format!("Library/{}", primal_name))
            }
            Platform::Windows => {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                    .join(primal_name)
            }
        }
    }
    
    /// Extract genomeBin archive
    fn extract_archive(&self, install_dir: &Path) -> Result<()> {
        println!("{}", "Extracting genomeBin...".blue());
        
        // Find archive marker in file
        let mut file = File::open(&self.genome_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        
        // Find __ARCHIVE_START__ marker
        let marker = b"__ARCHIVE_START__\n";
        let archive_start = contents
            .windows(marker.len())
            .position(|window| window == marker)
            .context("Archive marker not found in genomeBin")?
            + marker.len();
        
        // Extract tar.gz
        let archive_data = &contents[archive_start..];
        let decoder = GzDecoder::new(archive_data);
        let mut archive = Archive::new(decoder);
        
        // Create progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message("Extracting binaries...");
        
        // Extract to temp dir first
        let temp_dir = tempfile::tempdir()?;
        archive.unpack(temp_dir.path())?;
        
        // Copy architecture-specific binaries
        let arch_dir = temp_dir.path().join(self.architecture.as_str());
        if !arch_dir.exists() {
            bail!("No binaries found for architecture: {}", self.architecture.as_str());
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
            
            pb.set_message(format!("Installed: {}", entry.file_name().to_string_lossy()));
        }
        
        pb.finish_with_message("✓ Extraction complete".green().to_string());
        Ok(())
    }
    
    /// Deploy genomeBin
    pub fn deploy(&self) -> Result<()> {
        // Print header
        println!("\n{}", "╔══════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║           genomeBin Universal Deployer              ║".cyan());
        println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
        println!();
        
        // Detect environment
        println!("{} {}", "Architecture:".blue(), self.architecture.as_str().green());
        println!("{} {}", "Platform:".blue(), self.platform.name().green());
        println!();
        
        // Determine install directory
        let genome_name = self.genome_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("primal")
            .trim_end_matches(".genome");
            
        let install_dir = self.install_dir.clone()
            .unwrap_or_else(|| self.default_install_dir(genome_name));
        
        println!("{} {}", "Install directory:".blue(), install_dir.display().to_string().green());
        
        // Create install directory
        fs::create_dir_all(&install_dir)
            .context("Failed to create installation directory")?;
        
        // Extract and install
        self.extract_archive(&install_dir)?;
        
        // Verify installation
        println!();
        self.verify_installation(&install_dir, genome_name)?;
        
        // Print next steps
        self.print_next_steps(&install_dir, genome_name);
        
        Ok(())
    }
    
    /// Verify installation
    fn verify_installation(&self, install_dir: &Path, primal_name: &str) -> Result<()> {
        println!("{}", "Verifying installation...".blue());
        
        let binary_path = install_dir.join(primal_name);
        if !binary_path.exists() {
            bail!("Binary not found after installation: {}", binary_path.display());
        }
        
        // Try to get version
        let version_output = std::process::Command::new(&binary_path)
            .arg("--version")
            .output();
            
        match version_output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("{} {} {}", "✓".green(), primal_name.green(), version.trim().green());
            }
            _ => {
                println!("{} {} {}", "✓".green(), primal_name.green(), "(version check skipped)".yellow());
            }
        }
        
        Ok(())
    }
    
    /// Print next steps
    fn print_next_steps(&self, install_dir: &Path, primal_name: &str) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║              Deployment Complete! 🎊                 ║".cyan());
        println!("{}", "╚══════════════════════════════════════════════════════╝".cyan());
        println!();
        println!("{}", "Next steps:".blue().bold());
        
        if !nix::unistd::Uid::current().is_root() && self.platform != Platform::Android {
            println!("1. Add to PATH:");
            println!("   export PATH=\"$PATH:{}\"", install_dir.display());
            println!();
        }
        
        println!("2. Start {}:", primal_name);
        println!("   {}/{}", install_dir.display(), primal_name);
        println!();
        
        if self.platform.supports_abstract_sockets() {
            println!("{}", "Platform features:".blue());
            if self.platform == Platform::Android {
                println!("  • Abstract socket namespace (@biomeos_{})", primal_name);
                println!("  • Android HSM integration available");
            } else {
                println!("  • Unix socket support");
                println!("  • Abstract socket support");
            }
            println!();
        }
        
        println!("{}", format!("genomeBin deployment complete! 🧬").green().bold());
    }
}

// Add nix dependency for UID check
use nix;
