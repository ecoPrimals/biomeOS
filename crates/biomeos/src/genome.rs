// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Genome mode - Build and manage genomeBins (self-extracting multi-arch binaries).
//!
//! Provides CLI subcommands for genomeBin operations: build, compose, verify,
//! info, and list. Delegates actual implementation to biomeos_genomebin_v3.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

/// Genome info for list output (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct GenomeInfo {
    pub name: String,
    pub version: String,
    pub architectures: Vec<String>,
}

/// Genome subcommands for genomeBin operations
#[derive(Debug, Subcommand)]
pub(crate) enum GenomeCommand {
    /// Build a new genomeBin from primal binary
    Build(GenomeBuildArgs),

    /// Compose multiple genomes into an atomic
    Compose(GenomeComposeArgs),

    /// Verify a genomeBin
    Verify(GenomeVerifyArgs),

    /// Show genomeBin info
    Info(GenomeInfoArgs),

    /// List genomeBins in a directory
    List(GenomeListArgs),
}

/// Arguments for genome build
#[derive(Debug, Args)]
pub(crate) struct GenomeBuildArgs {
    /// Path to primal binary (x86_64)
    #[arg(long)]
    pub binary_x86_64: Option<PathBuf>,

    /// Path to primal binary (aarch64)
    #[arg(long)]
    pub binary_aarch64: Option<PathBuf>,

    /// Output genomeBin path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Genome name (defaults to binary filename)
    #[arg(long)]
    pub name: Option<String>,

    /// Version
    #[arg(long, default_value = "1.0.0")]
    pub version: String,

    /// Description
    #[arg(long)]
    pub description: Option<String>,
}

/// Arguments for genome compose
#[derive(Debug, Args)]
pub(crate) struct GenomeComposeArgs {
    /// Atomic name (e.g., "tower", "node", "nest")
    #[arg(short, long)]
    pub name: String,

    /// Genome paths to compose
    #[arg(short, long)]
    pub genomes: Vec<PathBuf>,

    /// Output genomeBin path
    #[arg(short, long)]
    pub output: PathBuf,
}

/// Arguments for genome verify
#[derive(Debug, Args)]
pub(crate) struct GenomeVerifyArgs {
    /// Path to genomeBin
    pub path: PathBuf,
}

/// Arguments for genome info
#[derive(Debug, Args)]
pub(crate) struct GenomeInfoArgs {
    /// Path to genomeBin
    pub path: PathBuf,
}

/// Arguments for genome list
#[derive(Debug, Args)]
pub(crate) struct GenomeListArgs {
    /// Directory to list (defaults to plasmidBin/)
    #[arg(default_value = "plasmidBin")]
    pub directory: PathBuf,
}

/// List genome bins in directory (pure filesystem scan + parse)
pub(crate) fn list_genome_bins(dir: &Path) -> Result<Vec<GenomeInfo>> {
    use biomeos_genomebin_v3::GenomeBin;

    let mut infos = Vec::new();
    if !dir.exists() {
        return Ok(infos);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|e| e == "genome" || e == "json")
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(genome) = GenomeBin::from_json(&content) {
                    infos.push(GenomeInfo {
                        name: genome.manifest.name,
                        version: genome.manifest.version,
                        architectures: genome
                            .manifest
                            .architectures
                            .iter()
                            .map(|a| format!("{a:?}"))
                            .collect(),
                    });
                }
            }
        }
    }
    Ok(infos)
}

/// Format genome info as display lines (pure, testable)
pub(crate) fn format_genome_info(info: &GenomeInfo) -> Vec<String> {
    vec![format!(
        "  {} v{} ({:?})",
        info.name, info.version, info.architectures
    )]
}

/// Handle genome subcommands
///
/// Delegates to biomeos_genomebin_v3 for actual implementation.
pub(crate) async fn handle_genome_command(command: GenomeCommand) -> Result<()> {
    use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeManifest};
    use tracing::info;

    match command {
        GenomeCommand::Build(args) => {
            info!("🧬 Building genomeBin: {:?}", args.output);

            // Create manifest
            let manifest = GenomeManifest::new(args.name.unwrap_or_else(|| "primal".to_string()))
                .version(&args.version)
                .description(args.description.unwrap_or_default());

            let mut genome = GenomeBin::with_manifest(manifest);

            // Add x86_64 binary if provided
            if let Some(ref path) = args.binary_x86_64 {
                genome
                    .add_binary(Arch::X86_64, path)
                    .map_err(|e| anyhow::anyhow!("Failed to add x86_64 binary: {e}"))?;
                info!("   Added x86_64 binary: {}", path.display());
            }

            // Add aarch64 binary if provided
            if let Some(ref path) = args.binary_aarch64 {
                genome
                    .add_binary(Arch::Aarch64, path)
                    .map_err(|e| anyhow::anyhow!("Failed to add aarch64 binary: {e}"))?;
                info!("   Added aarch64 binary: {}", path.display());
            }

            // Save as JSON manifest
            let json = genome
                .to_json()
                .map_err(|e| anyhow::anyhow!("Failed to serialize: {e}"))?;
            std::fs::write(&args.output, json)?;

            info!("✅ GenomeBin created: {}", args.output.display());
            Ok(())
        }

        GenomeCommand::Compose(args) => {
            info!("🧬 Composing atomic genomeBin: {}", args.name);

            // Load all genomes
            let mut genomes = Vec::new();
            for path in &args.genomes {
                let content = std::fs::read_to_string(path)?;
                let genome = GenomeBin::from_json(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
                genomes.push(genome);
                info!("   Loaded: {}", path.display());
            }

            // Create composed manifest
            let composed = serde_json::json!({
                "name": args.name,
                "type": "atomic",
                "genomes": genomes,
                "created_at": chrono::Utc::now().to_rfc3339(),
            });

            std::fs::write(&args.output, serde_json::to_string_pretty(&composed)?)?;
            info!("✅ Atomic composed: {}", args.output.display());
            Ok(())
        }

        GenomeCommand::Verify(args) => {
            info!("🔍 Verifying genomeBin: {}", args.path.display());

            let content = std::fs::read_to_string(&args.path)?;
            let genome = GenomeBin::from_json(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse: {e}"))?;

            info!("   Name: {}", genome.manifest.name);
            info!("   Version: {}", genome.manifest.version);
            info!("   Architectures: {:?}", genome.manifest.architectures);
            info!("✅ GenomeBin valid");
            Ok(())
        }

        GenomeCommand::Info(args) => {
            let content = std::fs::read_to_string(&args.path)?;
            let genome = GenomeBin::from_json(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse: {e}"))?;

            println!("GenomeBin: {}", genome.manifest.name);
            println!("  Version: {}", genome.manifest.version);
            println!("  Description: {}", genome.manifest.description);
            println!("  Architectures:");
            for arch in &genome.manifest.architectures {
                println!("    - {arch:?}");
            }
            Ok(())
        }

        GenomeCommand::List(args) => {
            info!("📋 Listing genomeBins in: {}", args.directory.display());

            let infos = list_genome_bins(&args.directory)?;
            if infos.is_empty() && !args.directory.exists() {
                println!("Directory not found: {}", args.directory.display());
                return Ok(());
            }
            for info in &infos {
                for line in format_genome_info(info) {
                    println!("{line}");
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_format_genome_info() {
        let info = GenomeInfo {
            name: "beardog".to_string(),
            version: "1.0.0".to_string(),
            architectures: vec!["X86_64".to_string(), "Aarch64".to_string()],
        };
        let lines = format_genome_info(&info);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("beardog"));
        assert!(lines[0].contains("1.0.0"));
    }

    #[test]
    fn test_format_genome_info_empty_architectures() {
        let info = GenomeInfo {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            architectures: vec![],
        };
        let lines = format_genome_info(&info);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("minimal"));
        assert!(lines[0].contains("0.1.0"));
    }

    #[test]
    fn test_list_genome_bins_nonexistent_dir() {
        let infos = list_genome_bins(std::path::Path::new("/nonexistent-path-xyz-12345")).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn test_list_genome_bins_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let infos = list_genome_bins(temp.path()).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn test_list_genome_bins_with_valid_genome() {
        use biomeos_genomebin_v3::{GenomeBin, GenomeManifest};

        let temp = tempfile::tempdir().expect("temp dir");
        let genome = GenomeBin::with_manifest(GenomeManifest::new("test-primal").version("2.0.0"));
        let json = genome.to_json().expect("serialize");
        let path = temp.path().join("test.genome");
        std::fs::write(&path, json).expect("write genome");

        let infos = list_genome_bins(temp.path()).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "test-primal");
        assert_eq!(infos[0].version, "2.0.0");
    }

    #[test]
    fn test_list_genome_bins_skips_invalid_json() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("invalid.genome"), "not valid json").expect("write");
        let infos = list_genome_bins(temp.path()).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn test_list_genome_bins_skips_non_genome_extensions() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("other.txt"), "content").expect("write");
        let infos = list_genome_bins(temp.path()).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn test_list_genome_bins_json_extension() {
        use biomeos_genomebin_v3::{GenomeBin, GenomeManifest};

        let temp = tempfile::tempdir().expect("temp dir");
        let genome = GenomeBin::with_manifest(GenomeManifest::new("json-primal").version("1.0"));
        let json = genome.to_json().expect("serialize");
        let path = temp.path().join("test.json");
        std::fs::write(&path, json).expect("write genome");

        let infos = list_genome_bins(temp.path()).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "json-primal");
    }
}
