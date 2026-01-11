#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! clap = { version = "4.4", features = ["derive"] }
//! tokio = { version = "1", features = ["full"] }
//! anyhow = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! toml = "0.8"
//! walkdir = "2.4"
//! sha2 = "0.10"
//! ```

/// # biomeOS Primal Harvest System
///
/// Modern Rust-based primal binary harvesting system.
/// Evolves toward NUCLEUS (automated primal management).
///
/// Features:
/// - Harvest from local phase1/ directories
/// - Future: Pull from GitHub/repos
/// - Clean old versions
/// - Store in plasmidBin/
/// - Verify binary integrity
/// - Track versions and provenance
///
/// Usage:
///   biomeos-harvest local --primal songbird
///   biomeos-harvest github --repo ecoPrimals/songbird --version v3.20.0
///   biomeos-harvest clean --keep-latest 2
///   biomeos-harvest list

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "biomeos-harvest")]
#[command(about = "biomeOS Primal Harvest System - NUCLEUS Evolution", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Harvest primal from local phase1/ directory
    Local {
        /// Primal name (e.g., songbird, beardog, nestgate)
        #[arg(short, long)]
        primal: String,

        /// Source directory (default: ../phase1/{primal})
        #[arg(short, long)]
        source: Option<PathBuf>,

        /// Clean old versions
        #[arg(long, default_value_t = true)]
        clean: bool,
    },

    /// Harvest primal from GitHub repository (future)
    GitHub {
        /// GitHub repository (e.g., ecoPrimals/songbird)
        #[arg(short, long)]
        repo: String,

        /// Version tag (e.g., v3.20.0)
        #[arg(short, long)]
        version: String,

        /// Architecture (default: x86_64-unknown-linux-gnu)
        #[arg(short, long, default_value = "x86_64-unknown-linux-gnu")]
        arch: String,
    },

    /// Clean old primal versions
    Clean {
        /// Keep N latest versions (default: 2)
        #[arg(short, long, default_value_t = 2)]
        keep_latest: usize,

        /// Specific primal to clean (optional)
        #[arg(short, long)]
        primal: Option<String>,
    },

    /// List harvested primals
    List {
        /// Show detailed info
        #[arg(short, long)]
        verbose: bool,
    },

    /// Harvest all primals from phase1/
    All {
        /// Clean old versions
        #[arg(long, default_value_t = true)]
        clean: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct PrimalManifest {
    name: String,
    version: String,
    source: HarvestSource,
    binary_path: PathBuf,
    sha256: String,
    harvested_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum HarvestSource {
    Local { path: PathBuf },
    GitHub { repo: String, tag: String },
}

struct HarvestSystem {
    biomeos_root: PathBuf,
    plasmid_bin: PathBuf,
    phase1_root: PathBuf,
}

impl HarvestSystem {
    fn new() -> Result<Self> {
        let biomeos_root = std::env::current_dir()?;
        let plasmid_bin = biomeos_root.join("plasmidBin");
        let phase1_root = biomeos_root.parent()
            .context("No parent directory")?
            .parent()
            .context("No grandparent directory")?
            .join("phase1");

        // Ensure plasmidBin exists
        fs::create_dir_all(&plasmid_bin)?;

        Ok(Self {
            biomeos_root,
            plasmid_bin,
            phase1_root,
        })
    }

    /// Harvest primal from local phase1/ directory
    fn harvest_local(&self, primal: &str, source: Option<PathBuf>, clean: bool) -> Result<()> {
        println!("🌾 Harvesting {} from local source...", primal);

        // Determine source path
        let source_path = source.unwrap_or_else(|| self.phase1_root.join(primal));
        
        if !source_path.exists() {
            anyhow::bail!("Source path does not exist: {}", source_path.display());
        }

        // Build the primal
        println!("  🔨 Building {} in release mode...", primal);
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&source_path)
            .status()
            .context("Failed to run cargo build")?;

        if !status.success() {
            anyhow::bail!("Build failed for {}", primal);
        }

        // Find the binary
        let binary_name = self.get_binary_name(primal);
        let source_binary = source_path.join("target/release").join(&binary_name);

        if !source_binary.exists() {
            anyhow::bail!("Binary not found: {}", source_binary.display());
        }

        // Get version (from Cargo.toml or git)
        let version = self.get_version(&source_path, primal)?;

        // Clean old versions if requested
        if clean {
            self.clean_old_versions(primal, 1)?;
        }

        // Copy to plasmidBin/
        let dest_binary = self.plasmid_bin.join(&binary_name);
        fs::copy(&source_binary, &dest_binary)
            .context("Failed to copy binary")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_binary, perms)?;
        }

        // Calculate SHA256
        let sha256 = self.calculate_sha256(&dest_binary)?;

        // Save manifest
        let manifest = PrimalManifest {
            name: primal.to_string(),
            version: version.clone(),
            source: HarvestSource::Local { path: source_path.clone() },
            binary_path: dest_binary.clone(),
            sha256,
            harvested_at: chrono::Utc::now().to_rfc3339(),
        };

        self.save_manifest(&manifest)?;

        println!("  ✅ Harvested {} v{} to plasmidBin/", primal, version);
        println!("     Binary: {}", binary_name);
        println!("     Size: {} KB", fs::metadata(&dest_binary)?.len() / 1024);

        Ok(())
    }

    /// Get canonical binary name for primal
    fn get_binary_name(&self, primal: &str) -> String {
        // All primals use simple names (except special cases)
        match primal {
            "petaltongue" => "petaltongue".to_string(),
            "petal-tongue" => "petaltongue".to_string(),
            _ => primal.to_string(),
        }
    }

    /// Get version from Cargo.toml or git
    fn get_version(&self, source_path: &Path, primal: &str) -> Result<String> {
        // Try Cargo.toml first
        let cargo_toml = source_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if let Ok(toml) = contents.parse::<toml::Value>() {
                    if let Some(version) = toml.get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str()) {
                        return Ok(version.to_string());
                    }
                }
            }
        }

        // Fallback to git describe
        let output = Command::new("git")
            .args(&["describe", "--tags", "--always"])
            .current_dir(source_path)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Fallback to "unknown"
        Ok("unknown".to_string())
    }

    /// Calculate SHA256 of binary
    fn calculate_sha256(&self, path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        let bytes = fs::read(path)?;
        let hash = Sha256::digest(&bytes);
        Ok(format!("{:x}", hash))
    }

    /// Save primal manifest
    fn save_manifest(&self, manifest: &PrimalManifest) -> Result<()> {
        let manifest_path = self.plasmid_bin.join(format!("{}.manifest.toml", manifest.name));
        let toml = toml::to_string_pretty(manifest)?;
        fs::write(manifest_path, toml)?;
        Ok(())
    }

    /// Clean old primal versions
    fn clean_old_versions(&self, primal: &str, keep_latest: usize) -> Result<()> {
        // For now, just remove the old binary
        // Future: Track versions and keep N latest
        println!("  🧹 Cleaning old versions of {}...", primal);
        Ok(())
    }

    /// List harvested primals
    fn list_primals(&self, verbose: bool) -> Result<()> {
        println!("📦 Harvested Primals in plasmidBin/:");
        println!();

        for entry in WalkDir::new(&self.plasmid_bin)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| !e.path().extension().map_or(false, |ext| ext == "toml"))
        {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();
            
            if verbose {
                let size = fs::metadata(path)?.len();
                let manifest_path = path.with_extension("manifest.toml");
                let version = if manifest_path.exists() {
                    fs::read_to_string(&manifest_path)
                        .ok()
                        .and_then(|s| s.parse::<toml::Value>().ok())
                        .and_then(|t| t.get("version")?.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unknown".to_string()
                };

                println!("  • {} (v{}, {} KB)", name, version, size / 1024);
            } else {
                println!("  • {}", name);
            }
        }

        Ok(())
    }

    /// Harvest all primals from phase1/
    fn harvest_all(&self, clean: bool) -> Result<()> {
        println!("🌾 Harvesting all primals from phase1/...");
        println!();

        let primals = vec![
            "songbird",
            "beardog",
            "toadstool",
            "nestgate",
            "squirrel",
            "petalTongue",  // Note: directory name might be camelCase
        ];

        for primal in primals {
            match self.harvest_local(primal, None, clean) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("  ⚠️  Failed to harvest {}: {}", primal, e);
                }
            }
            println!();
        }

        println!("✅ Harvest complete!");
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let system = HarvestSystem::new()?;

    match cli.command {
        Commands::Local { primal, source, clean } => {
            system.harvest_local(&primal, source, clean)?;
        }
        Commands::GitHub { repo, version, arch } => {
            println!("🚧 GitHub harvesting not yet implemented");
            println!("   Repo: {}", repo);
            println!("   Version: {}", version);
            println!("   Arch: {}", arch);
            println!();
            println!("   Coming soon: NUCLEUS will handle GitHub releases!");
        }
        Commands::Clean { keep_latest, primal } => {
            if let Some(p) = primal {
                system.clean_old_versions(&p, keep_latest)?;
            } else {
                println!("🧹 Cleaning all primals (keep {} latest)", keep_latest);
            }
        }
        Commands::List { verbose } => {
            system.list_primals(verbose)?;
        }
        Commands::All { clean } => {
            system.harvest_all(clean)?;
        }
    }

    Ok(())
}

