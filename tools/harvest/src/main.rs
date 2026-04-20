// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]

//! # biomeOS Primal Harvest System
//!
//! Modern Rust-based primal binary harvesting system.
//! Evolves toward NUCLEUS (automated primal management).
//!
//! Features:
//! - Harvest from local phase1/ directories
//! - Pull from GitHub releases
//! - Clean old versions
//! - Store in plasmidBin/
//! - Verify binary integrity
//! - Track versions and provenance
//!
//! Usage:
//!   biomeos-harvest local --primal songbird
//!   biomeos-harvest github --repo ecoPrimals/songbird --version v3.20.0
//!   biomeos-harvest clean --keep-latest 2
//!   biomeos-harvest list

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Bootstrap-time primal roster for harvest operations.
///
/// Canonical source of truth: `biomeos-types::primal_names::{CORE_PRIMALS, PROVENANCE_PRIMALS}`.
/// This standalone tool does not depend on `biomeos-types` to keep compile times fast.
/// Keep in sync manually; all names must be lowercase (filesystem convention).
/// Last synced: 2026-04-20.
const KNOWN_PRIMALS: &[&str] = &[
    // Tower atomic (CORE_PRIMALS)
    "beardog",
    "songbird",
    "toadstool",
    "barracuda",
    "coralreef",
    "nestgate",
    "squirrel",
    // Provenance trio
    "loamspine",
    "rhizocrypt",
    "sweetgrass",
    // UI
    "petaltongue",
];

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

    /// Harvest primal binary from a GitHub release.
    ///
    /// Uses `curl` for HTTPS (zero C-dep: subprocess, not linked) and the GitHub
    /// Releases API. Supports `GITHUB_TOKEN` for authenticated requests.
    fn harvest_github(&self, repo: &str, version: &str, arch: &str) -> Result<()> {
        println!("🌐 Harvesting from GitHub: {} @ {} ({})", repo, version, arch);

        let primal = repo
            .rsplit('/')
            .next()
            .unwrap_or(repo)
            .trim_start_matches("eco-")
            .to_lowercase();
        let binary_name = self.get_binary_name(&primal);

        let tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{version}")
        };

        let api_url = format!(
            "https://api.github.com/repos/{repo}/releases/tags/{tag}"
        );
        let release_json = self.curl_fetch_json(&api_url)?;

        let assets = release_json
            .get("assets")
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow::anyhow!("No assets array in GitHub release response"))?;

        let matching_asset = assets.iter().find(|a| {
            let name = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
            name.contains(arch) && (name.contains(&binary_name) || name.contains(&primal))
                && !name.ends_with(".sha256")
                && !name.ends_with(".sig")
                && !name.ends_with(".toml")
        });

        let asset = matching_asset.ok_or_else(|| {
            let available: Vec<&str> = assets
                .iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                .collect();
            anyhow::anyhow!(
                "No asset matching arch '{}' and primal '{}' in release {}. Available: {:?}",
                arch,
                primal,
                tag,
                available
            )
        })?;

        let download_url = asset
            .get("browser_download_url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow::anyhow!("Asset missing browser_download_url"))?;

        let asset_name = asset
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or(&binary_name);

        println!("  📥 Downloading: {}", asset_name);
        let dest_binary = self.plasmid_bin.join(&binary_name);
        self.curl_download(&download_url, &dest_binary)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_binary, perms)?;
        }

        let sha256 = self.calculate_sha256(&dest_binary)?;

        let manifest = PrimalManifest {
            name: primal.clone(),
            version: tag.clone(),
            source: HarvestSource::GitHub {
                repo: repo.to_string(),
                tag,
            },
            binary_path: dest_binary.clone(),
            sha256,
            harvested_at: chrono::Utc::now().to_rfc3339(),
        };
        self.save_manifest(&manifest)?;

        let size_kb = fs::metadata(&dest_binary)?.len() / 1024;
        println!("  ✅ Harvested {} from GitHub to plasmidBin/", primal);
        println!("     Binary: {}", binary_name);
        println!("     Size: {} KB", size_kb);
        Ok(())
    }

    /// Fetch JSON from HTTPS URL via curl subprocess.
    fn curl_fetch_json(&self, url: &str) -> Result<serde_json::Value> {
        let mut cmd = Command::new("curl");
        cmd.args(["-sSfL", "-H", "Accept: application/vnd.github+json", url]);
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            cmd.args(["-H", &format!("Authorization: Bearer {token}")]);
        }
        cmd.arg("-H").arg("User-Agent: biomeos-harvest/0.1");
        let output = cmd
            .output()
            .context("curl not found — install curl for GitHub harvesting")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("curl failed for {url}: {stderr}");
        }
        serde_json::from_slice(&output.stdout)
            .context("Invalid JSON from GitHub API")
    }

    /// Download a file via curl subprocess.
    fn curl_download(&self, url: &str, dest: &Path) -> Result<()> {
        let mut cmd = Command::new("curl");
        cmd.args(["-sSfL", "-o"]);
        cmd.arg(dest);
        cmd.arg(url);
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            cmd.args(["-H", &format!("Authorization: Bearer {token}")]);
        }
        cmd.arg("-H").arg("User-Agent: biomeos-harvest/0.1");
        let output = cmd
            .output()
            .context("curl not found — install curl for GitHub harvesting")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Download failed for {url}: {stderr}");
        }
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
        let binary_name = self.get_binary_name(primal);
        println!("  🧹 Cleaning old versions of {}...", primal);

        let mut entries = self.collect_primal_version_artifacts(&binary_name)?;
        if entries.is_empty() {
            return Ok(());
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));

        if keep_latest >= entries.len() {
            return Ok(());
        }

        for (path, _) in entries.into_iter().skip(keep_latest) {
            self.remove_primal_version_artifact(&path)?;
            println!("     Removed {}", path.display());
        }

        Ok(())
    }

    /// True if `file_name` in plasmidBin (or primals/) is a version artifact for this binary
    /// (exact name, or a prefixed variant like `songbird-1.0.0`, excluding `.toml` manifests).
    fn is_version_artifact_name(&self, binary_name: &str, file_name: &str) -> bool {
        if file_name.ends_with(".toml") {
            return false;
        }
        if file_name == binary_name {
            return true;
        }
        for sep in [".", "-", "@"] {
            let prefix = format!("{binary_name}{sep}");
            if file_name.starts_with(&prefix) {
                return true;
            }
        }
        false
    }

    /// Collect (path, modified time) for every on-disk version of this primal.
    ///
    /// Covers: flat `plasmidBin/`, `plasmidBin/primals/` (files and `primals/{binary}/` children),
    /// and `plasmidBin/archive/{binary}/` version slots. Uses [`std::fs::metadata`] mtime for ordering.
    fn collect_primal_version_artifacts(&self, binary_name: &str) -> Result<Vec<(PathBuf, SystemTime)>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();

        let mut insert = |path: PathBuf| -> Result<()> {
            if !seen.insert(path.clone()) {
                return Ok(());
            }
            let mtime = fs::metadata(&path)
                .with_context(|| format!("stat {}", path.display()))?
                .modified()
                .unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
            out.push((path, mtime));
            Ok(())
        };

        self.push_version_files_in_dir(&self.plasmid_bin, binary_name, &mut insert)?;

        let primals = self.plasmid_bin.join("primals");
        if primals.is_dir() {
            self.push_version_files_in_dir(&primals, binary_name, &mut insert)?;

            let nested = primals.join(binary_name);
            if nested.is_dir() {
                for entry in fs::read_dir(&nested)
                    .with_context(|| format!("read_dir {}", nested.display()))?
                {
                    let entry = entry.with_context(|| format!("read_dir entry {}", nested.display()))?;
                    insert(entry.path())?;
                }
            }
        }

        let archive_slot = self.plasmid_bin.join("archive").join(binary_name);
        if archive_slot.is_dir() {
            for entry in fs::read_dir(&archive_slot)
                .with_context(|| format!("read_dir {}", archive_slot.display()))?
            {
                let entry =
                    entry.with_context(|| format!("read_dir entry {}", archive_slot.display()))?;
                insert(entry.path())?;
            }
        }

        Ok(out)
    }

    fn push_version_files_in_dir(
        &self,
        dir: &Path,
        binary_name: &str,
        insert: &mut impl FnMut(PathBuf) -> Result<()>,
    ) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
            let entry = entry.with_context(|| format!("read_dir entry {}", dir.display()))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let Some(fname) = file_name.to_str() else {
                continue;
            };
            let meta = entry
                .metadata()
                .with_context(|| format!("metadata {}", path.display()))?;
            if meta.is_file() && self.is_version_artifact_name(binary_name, fname) {
                insert(path)?;
            }
        }
        Ok(())
    }

    fn remove_primal_version_artifact(&self, path: &Path) -> Result<()> {
        let meta = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        if meta.is_dir() {
            fs::remove_dir_all(path).with_context(|| format!("remove_dir_all {}", path.display()))?;
        } else {
            fs::remove_file(path).with_context(|| format!("remove_file {}", path.display()))?;
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let manifest = path.with_file_name(format!("{name}.manifest.toml"));
                if manifest.exists() {
                    fs::remove_file(&manifest).with_context(|| {
                        format!("remove_file manifest {}", manifest.display())
                    })?;
                }
            }
        }
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
            let Some(file_name) = path.file_name() else {
                continue;
            };
            let name = file_name.to_string_lossy();
            
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

        let primals = KNOWN_PRIMALS;

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
            system.harvest_github(&repo, &version, &arch)?;
        }
        Commands::Clean { keep_latest, primal } => {
            if let Some(p) = primal {
                system.clean_old_versions(&p, keep_latest)?;
            } else {
                println!("🧹 Cleaning all primals (keep {} latest)", keep_latest);
                for primal in KNOWN_PRIMALS {
                    if let Err(e) = system.clean_old_versions(primal, keep_latest) {
                        eprintln!("  ⚠️  Failed to clean {}: {}", primal, e);
                    }
                }
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

