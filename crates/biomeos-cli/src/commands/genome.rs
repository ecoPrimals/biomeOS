// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Genome CLI Commands
//!
//! CLI interface for genomeBin operations.
//!
//! AGPL-3.0-only License

use anyhow::{Context, Result};
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeBinBuilder, GenomeBinComposer};

/// Extract genome name from binary path (defaults to "genome" if unparseable)
pub fn extract_genome_name_from_path(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("genome")
        .to_string()
}

/// Parse architecture string to Arch enum (testable pure function)
pub fn parse_arch(arch: &str) -> Result<Arch> {
    match arch {
        "x86_64" => Ok(Arch::X86_64),
        "aarch64" => Ok(Arch::Aarch64),
        "arm" => Ok(Arch::Arm),
        "riscv64" => Ok(Arch::Riscv64),
        _ => Err(anyhow::anyhow!(
            "Invalid architecture: {arch}. Supported: x86_64, aarch64, arm, riscv64"
        )),
    }
}
use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing::info;

// ============================================================================
// Args structs for bin/main.rs integration
// ============================================================================

/// Create genomeBin arguments
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Path to primal binary
    #[arg(short, long)]
    pub binary: PathBuf,

    /// Output genomeBin path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Target architecture (x86_64, aarch64, arm, riscv64)
    #[arg(short, long, default_value = "x86_64")]
    pub arch: String,

    /// Genome name (defaults to binary filename)
    #[arg(long)]
    pub name: Option<String>,

    /// Version
    #[arg(long)]
    pub version: Option<String>,

    /// Description
    #[arg(long)]
    pub description: Option<String>,
}

/// Compose genomeBin arguments
#[derive(Debug, Args)]
pub struct ComposeArgs {
    /// Atomic name (e.g., "tower", "node", "nest")
    #[arg(short, long)]
    pub name: String,

    /// NUCLEUS type (TOWER, NODE, NEST, NUCLEUS)
    #[arg(long, default_value = "TOWER")]
    pub nucleus_type: String,

    /// Genome paths to compose
    #[arg(short, long)]
    pub genomes: Vec<PathBuf>,

    /// Output genomeBin path
    #[arg(short, long)]
    pub output: PathBuf,
}

/// Verify genomeBin arguments
#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Path to genomeBin
    pub path: PathBuf,
}

/// Genome-related commands
#[derive(Debug, Args)]
pub struct GenomeArgs {
    /// Genome subcommand to execute
    #[command(subcommand)]
    pub command: GenomeCommand,
}

/// Available genome commands
#[derive(Debug, Subcommand)]
pub enum GenomeCommand {
    /// Build a new genomeBin from primal binary
    Build {
        /// Path to primal binary
        #[arg(short, long)]
        binary: std::path::PathBuf,

        /// Output genomeBin path (JSON format)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Target architecture (x86_64, aarch64, arm, riscv64)
        #[arg(short, long, default_value = "x86_64")]
        arch: String,

        /// Optional genome name (defaults to binary filename)
        #[arg(long)]
        name: Option<String>,

        /// Optional version
        #[arg(long)]
        version: Option<String>,

        /// Optional description
        #[arg(long)]
        description: Option<String>,
    },

    /// Verify genomeBin integrity
    Verify {
        /// Path to genomeBin (JSON format)
        path: std::path::PathBuf,
    },

    /// Extract genomeBin to directory
    Extract {
        /// Path to genomeBin (JSON format)
        genome: std::path::PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: std::path::PathBuf,
    },

    /// Show genomeBin info
    Info {
        /// Path to genomeBin (JSON format)
        path: std::path::PathBuf,
    },
}

// ============================================================================
// Standalone handler functions for CLI integration
// ============================================================================

/// Handle genome create command
pub fn handle_genome_create(args: CreateArgs) -> Result<()> {
    info!("Creating genome from: {}", args.binary.display());

    if !args.binary.exists() {
        anyhow::bail!("Binary not found: {}", args.binary.display());
    }

    let arch = parse_arch(&args.arch)?;

    // Determine genome name
    let genome_name = args
        .name
        .unwrap_or_else(|| extract_genome_name_from_path(&args.binary));

    let mut builder = GenomeBinBuilder::new(&genome_name);

    if let Some(v) = args.version {
        builder = builder.version(v);
    }
    if let Some(d) = args.description {
        builder = builder.description(d);
    }

    builder = builder.add_binary(arch, &args.binary);

    let genome = builder.build().context("Failed to build genomeBin")?;

    genome
        .save(&args.output)
        .with_context(|| format!("Failed to save genomeBin to {}", args.output.display()))?;

    println!("✅ Created genomeBin: {}", args.output.display());
    println!("   Name: {}", genome.manifest.name);
    println!("   Version: {}", genome.manifest.version);
    println!("   Arch: {arch:?}");

    Ok(())
}

/// Handle genome compose command
pub fn handle_genome_compose(args: ComposeArgs) -> Result<()> {
    info!(
        "Composing {} atomic from {} genomes",
        args.nucleus_type,
        args.genomes.len()
    );

    if args.genomes.is_empty() {
        anyhow::bail!("No genomes provided for composition");
    }

    // Create composer
    let mut composer = GenomeBinComposer::new(&args.name).nucleus_type(&args.nucleus_type);

    // Load and add each genome
    for genome_path in &args.genomes {
        if !genome_path.exists() {
            anyhow::bail!("GenomeBin not found: {}", genome_path.display());
        }

        let genome = GenomeBin::load(genome_path)
            .with_context(|| format!("Failed to load genome from {}", genome_path.display()))?;

        println!(
            "  Adding: {} ({})",
            genome.manifest.name,
            genome_path.display()
        );
        composer = composer.add_genome(genome);
    }

    // Build the composed genome
    let composed = composer.build().context("Failed to compose genomes")?;

    // Save to output path
    composed.save(&args.output).with_context(|| {
        format!(
            "Failed to save composed genome to {}",
            args.output.display()
        )
    })?;

    println!("✅ Genome composition complete");
    println!("   Name: {}", composed.manifest.name);
    println!("   Type: {}", args.nucleus_type);
    println!("   Components: {}", args.genomes.len());
    println!("   Output: {}", args.output.display());

    Ok(())
}

/// Handle genome self-replicate command
pub fn handle_genome_self_replicate() -> Result<()> {
    info!("Self-replication initiated");

    let self_binary = std::env::current_exe().context("Failed to get current executable")?;

    let arch = Arch::detect();
    let binary_data = fs::read(&self_binary).context("Failed to read self binary")?;

    let manifest = biomeos_genomebin_v3::GenomeManifest::new("biomeos")
        .version(env!("CARGO_PKG_VERSION"))
        .description("biomeOS CLI - Self-Replicated");

    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(arch, &binary_data);

    println!("✅ Self-replicated biomeOS CLI");
    println!("   Size: {} bytes", binary_data.len());
    println!("   Arch: {arch:?}");

    Ok(())
}

/// Get the XDG-compliant genome storage directory
///
/// DEEP DEBT EVOLUTION: Uses $HOME env instead of `dirs` (C-based)
fn get_genome_storage_dir() -> PathBuf {
    // Check XDG data directory first
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(data_home).join("biomeos/genomes");
    }

    // Fall back to $HOME/.local/share/biomeos/genomes
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/biomeos/genomes");
    }

    // Last resort: XDG-compliant data dir
    biomeos_types::SystemPaths::new_lazy()
        .data_dir()
        .join("genomes")
}

/// Handle genome list command
pub fn handle_genome_list() -> Result<()> {
    info!("Listing genomes");

    let storage_dir = get_genome_storage_dir();

    if !storage_dir.exists() {
        println!("📂 Genome storage: {}", storage_dir.display());
        println!("   No genomes stored locally");
        println!();
        println!(
            "💡 Create genomes with: biomeos genome build --binary <path> --output <name>.json"
        );
        return Ok(());
    }

    println!("📂 Genome storage: {}", storage_dir.display());
    println!();

    let mut genomes_found = 0;

    // Scan for .json genome files
    if let Ok(entries) = fs::read_dir(&storage_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(genome) = GenomeBin::load(&path) {
                    genomes_found += 1;
                    let version = if genome.manifest.version.is_empty() {
                        "?".to_string()
                    } else {
                        genome.manifest.version.clone()
                    };
                    println!(
                        "  {} {} (v{}) - {:?}",
                        if genomes_found == 1 { "📦" } else { "  " },
                        genome.manifest.name,
                        version,
                        genome.binaries.keys().collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    if genomes_found == 0 {
        println!("   No genomes found");
        println!();
        println!(
            "💡 Create genomes with: biomeos genome build --binary <path> --output <name>.json"
        );
    } else {
        println!();
        println!("Total: {genomes_found} genome(s)");
    }

    Ok(())
}

/// Handle genome verify command
pub fn handle_genome_verify(args: VerifyArgs) -> Result<()> {
    if !args.path.exists() {
        anyhow::bail!("GenomeBin not found: {}", args.path.display());
    }

    let genome = GenomeBin::load(&args.path)
        .with_context(|| format!("Failed to load genomeBin from {}", args.path.display()))?;

    println!("Verifying: {}", args.path.display());
    println!("Name: {}", genome.manifest.name);
    println!("Version: {}", genome.manifest.version);

    match genome.is_valid() {
        Ok(valid) => {
            if valid {
                println!("✅ All checksums valid");
                Ok(())
            } else {
                anyhow::bail!("❌ Checksum verification failed");
            }
        }
        Err(e) => {
            anyhow::bail!("Verification error: {e}");
        }
    }
}

// ============================================================================
// Original execute function for subcommand processing
// ============================================================================

/// Execute genome command
pub async fn execute(args: GenomeArgs) -> Result<()> {
    match args.command {
        GenomeCommand::Build {
            binary,
            output,
            arch,
            name,
            version,
            description,
        } => {
            info!("Building genomeBin from {} for {}", binary.display(), arch);

            if !binary.exists() {
                anyhow::bail!("Binary not found: {}", binary.display());
            }

            let arch_enum = parse_arch(&arch)?;

            // Determine genome name
            let genome_name = name.unwrap_or_else(|| extract_genome_name_from_path(&binary));

            // Build genomeBin
            let mut builder = GenomeBinBuilder::new(&genome_name);

            if let Some(v) = version {
                builder = builder.version(v);
            }

            if let Some(d) = description {
                builder = builder.description(d);
            }

            builder = builder.add_binary(arch_enum, binary);

            let genome = builder.build().context("Failed to build genomeBin")?;

            // Save to output path
            genome
                .save(&output)
                .with_context(|| format!("Failed to save genomeBin to {}", output.display()))?;

            println!("✅ Built genomeBin: {}", output.display());
            println!("   Name: {}", genome.manifest.name);
            println!("   Version: {}", genome.manifest.version);
            println!("   Architectures: {}", genome.binaries.len());

            Ok(())
        }
        GenomeCommand::Verify { path } => {
            info!("Verifying genomeBin: {}", path.display());

            if !path.exists() {
                anyhow::bail!("GenomeBin not found: {}", path.display());
            }

            let genome = GenomeBin::load(&path)
                .with_context(|| format!("Failed to load genomeBin from {}", path.display()))?;

            println!("Verifying: {}", path.display());
            println!("Name: {}", genome.manifest.name);
            println!("Version: {}", genome.manifest.version);

            match genome.is_valid() {
                Ok(valid) => {
                    if valid {
                        println!("✅ All checksums valid");
                        Ok(())
                    } else {
                        anyhow::bail!("❌ Checksum verification failed");
                    }
                }
                Err(e) => {
                    anyhow::bail!("Verification error: {e}");
                }
            }
        }
        GenomeCommand::Extract { genome, output } => {
            info!("Extracting {} to {}", genome.display(), output.display());

            if !genome.exists() {
                anyhow::bail!("GenomeBin not found: {}", genome.display());
            }

            let genome_bin = GenomeBin::load(&genome)
                .with_context(|| format!("Failed to load genomeBin from {}", genome.display()))?;

            // Create output directory
            fs::create_dir_all(&output).with_context(|| {
                format!("Failed to create output directory: {}", output.display())
            })?;

            // Extract binaries for current architecture
            let current_arch = Arch::detect();
            if let Some(compressed) = genome_bin.get_binary(current_arch) {
                let decompressed = compressed
                    .decompress()
                    .context("Failed to decompress binary")?;

                let output_name = format!(
                    "{}-{}",
                    genome_bin.manifest.name,
                    current_arch.target_triple()
                );
                let output_path = output.join(&output_name);

                fs::write(&output_path, &decompressed).with_context(|| {
                    format!("Failed to write binary to {}", output_path.display())
                })?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&output_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&output_path, perms)?;
                }

                println!(
                    "✅ Extracted {} binary: {}",
                    current_arch.target_triple(),
                    output_path.display()
                );
            } else {
                anyhow::bail!("No binary available for current architecture: {current_arch:?}");
            }

            Ok(())
        }
        GenomeCommand::Info { path } => {
            info!("Getting info for: {}", path.display());

            if !path.exists() {
                anyhow::bail!("GenomeBin not found: {}", path.display());
            }

            let genome = GenomeBin::load(&path)
                .with_context(|| format!("Failed to load genomeBin from {}", path.display()))?;

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🧬 genomeBin Information");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("Name:        {}", genome.manifest.name);
            println!("Version:     {}", genome.manifest.version);

            if !genome.manifest.description.is_empty() {
                println!("Description: {}", genome.manifest.description);
            }

            println!();
            println!("Architectures:");
            for arch in &genome.manifest.architectures {
                println!("  - {arch}");
            }

            println!();
            println!("Binaries: {}", genome.binaries.len());
            for (arch, compressed) in &genome.binaries {
                println!(
                    "  {:?}: {} bytes (compressed), {} bytes (uncompressed)",
                    arch,
                    compressed.data.len(),
                    compressed.original_size
                );
            }

            if !genome.manifest.capabilities.is_empty() {
                println!();
                println!("Capabilities:");
                for cap in &genome.manifest.capabilities {
                    println!("  - {cap}");
                }
            }

            if let Some(nucleus) = &genome.manifest.nucleus_atomic {
                println!();
                println!("NUCLEUS Atomic: {nucleus}");
            }

            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            Ok(())
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_extract_genome_name_from_path() {
        assert_eq!(
            extract_genome_name_from_path(Path::new("/usr/bin/beardog")),
            "beardog"
        );
        assert_eq!(
            extract_genome_name_from_path(Path::new("tower-x86_64")),
            "tower-x86_64"
        );
        assert_eq!(extract_genome_name_from_path(Path::new("nest")), "nest");
    }

    #[test]
    fn test_extract_genome_name_from_path_empty() {
        assert_eq!(extract_genome_name_from_path(Path::new("")), "genome");
    }

    #[test]
    fn test_extract_genome_name_from_path_dotfile() {
        assert_eq!(
            extract_genome_name_from_path(Path::new("/tmp/.hidden")),
            ".hidden"
        );
    }

    #[test]
    fn test_parse_arch_valid() {
        assert!(matches!(parse_arch("x86_64").unwrap(), Arch::X86_64));
        assert!(matches!(parse_arch("aarch64").unwrap(), Arch::Aarch64));
        assert!(matches!(parse_arch("arm").unwrap(), Arch::Arm));
        assert!(matches!(parse_arch("riscv64").unwrap(), Arch::Riscv64));
    }

    #[test]
    fn test_parse_arch_invalid() {
        let result = parse_arch("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid architecture"));
        assert!(err.to_string().contains("x86_64"));
    }

    #[test]
    fn test_parse_arch_empty() {
        let result = parse_arch("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_arch_case_sensitive() {
        assert!(parse_arch("X86_64").is_err());
        assert!(parse_arch("AARCH64").is_err());
    }

    #[test]
    fn test_get_genome_storage_dir_with_xdg() {
        use biomeos_test_utils::TestEnvGuard;
        let _guard = TestEnvGuard::new("XDG_DATA_HOME", Some("/tmp/xdg_test"));
        let dir = get_genome_storage_dir();
        assert_eq!(dir, PathBuf::from("/tmp/xdg_test/biomeos/genomes"));
    }

    #[test]
    #[ignore = "modifies env vars; run with --ignored"]
    fn test_get_genome_storage_dir_home_fallback() {
        use biomeos_test_utils::remove_test_env;
        let _ = std::env::var("XDG_DATA_HOME").ok();
        remove_test_env("XDG_DATA_HOME");
        let dir = get_genome_storage_dir();
        assert!(dir.to_string_lossy().contains("genomes"));
    }
}
