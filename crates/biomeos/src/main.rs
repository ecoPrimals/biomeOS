// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! biomeOS Universal Nucleus & Orchestrator
//!
//! UniBin architecture with mode-based execution.
//!
//! Modes:
//! - cli: System management commands (default)
//! - genome: Build and manage genomeBins (self-extracting multi-arch binaries)
//! - neural-api: Graph-based orchestration server
//! - deploy: Deployment executor
//! - api: HTTP/WebSocket API server
//! - verify-lineage: Lineage verification
//! - doctor: Health diagnostics
//! - version: Version information

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};

mod modes;
mod proc_metrics;

/// Genome info for list output (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct GenomeInfo {
    pub name: String,
    pub version: String,
    pub architectures: Vec<String>,
}

// ============================================================================
// GENOME COMMAND DEFINITIONS
// ============================================================================

/// Genome subcommands for genomeBin operations
#[derive(Debug, Subcommand)]
enum GenomeCommand {
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
struct GenomeBuildArgs {
    /// Path to primal binary (x86_64)
    #[arg(long)]
    binary_x86_64: Option<PathBuf>,

    /// Path to primal binary (aarch64)
    #[arg(long)]
    binary_aarch64: Option<PathBuf>,

    /// Output genomeBin path
    #[arg(short, long)]
    output: PathBuf,

    /// Genome name (defaults to binary filename)
    #[arg(long)]
    name: Option<String>,

    /// Version
    #[arg(long, default_value = "1.0.0")]
    version: String,

    /// Description
    #[arg(long)]
    description: Option<String>,
}

/// Arguments for genome compose
#[derive(Debug, Args)]
struct GenomeComposeArgs {
    /// Atomic name (e.g., "tower", "node", "nest")
    #[arg(short, long)]
    name: String,

    /// Genome paths to compose
    #[arg(short, long)]
    genomes: Vec<PathBuf>,

    /// Output genomeBin path
    #[arg(short, long)]
    output: PathBuf,
}

/// Arguments for genome verify
#[derive(Debug, Args)]
struct GenomeVerifyArgs {
    /// Path to genomeBin
    path: PathBuf,
}

/// Arguments for genome info
#[derive(Debug, Args)]
struct GenomeInfoArgs {
    /// Path to genomeBin
    path: PathBuf,
}

/// Arguments for genome list
#[derive(Debug, Args)]
struct GenomeListArgs {
    /// Directory to list (defaults to plasmidBin/)
    #[arg(default_value = "plasmidBin")]
    directory: PathBuf,
}

#[derive(Parser)]
#[command(name = "biomeos")]
#[command(about = "🧠 biomeOS Universal Nucleus & Orchestrator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = "biomeOS: Self-evolving ecosystem coordinator with UniBin architecture")]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// CLI mode - System management commands (default)
    #[command(name = "cli")]
    Cli {
        // EVOLVED (Jan 27, 2026): Bridge to full CLI functionality
        // Shows system status and available commands
    },

    /// Genome mode - Build and manage genomeBins (self-extracting multi-arch binaries)
    #[command(name = "genome")]
    Genome {
        #[command(subcommand)]
        command: GenomeCommand,
    },

    /// Neural API server mode - Graph-based orchestration
    #[command(name = "neural-api")]
    NeuralApi {
        /// Graphs directory
        #[arg(long, default_value = "graphs")]
        graphs_dir: PathBuf,

        /// Family ID (auto-discovered from .family.seed or env if not specified)
        #[arg(long)]
        family_id: Option<String>,

        /// Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
    },

    /// Deploy mode - Execute deployment graph
    #[command(name = "deploy")]
    Deploy {
        /// Graph file path
        graph: PathBuf,

        /// Validate only (don't execute)
        #[arg(short, long)]
        validate_only: bool,

        /// Dry run (show what would happen)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// API server mode - HTTP/WebSocket API
    #[command(name = "api")]
    Api {
        /// Port to bind (HTTP mode)
        #[arg(short, long)]
        port: Option<u16>,

        /// Unix socket path (Unix socket mode, preferred)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Disable HTTP, Unix socket only
        #[arg(long)]
        unix_only: bool,
    },

    /// Verify lineage - Validate genetic lineage
    #[command(name = "verify-lineage")]
    VerifyLineage {
        /// Path to verify
        path: PathBuf,

        /// Detailed output
        #[arg(short, long)]
        detailed: bool,
    },

    /// Doctor mode - Health diagnostics
    #[command(name = "doctor")]
    Doctor {
        /// Detailed diagnostics
        #[arg(short, long)]
        detailed: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Check specific subsystem
        #[arg(short, long)]
        subsystem: Option<String>,
    },

    /// Version information
    #[command(name = "version")]
    Version {
        /// Show detailed version info
        #[arg(short, long)]
        detailed: bool,
    },

    /// Enroll device - Derive unique lineage seed from family root
    #[command(name = "enroll")]
    Enroll(modes::enroll::EnrollArgs),

    /// Model cache - Manage cached AI models across the mesh
    #[command(name = "model-cache")]
    ModelCache {
        #[command(subcommand)]
        command: ModelCacheCommand,
    },

    /// Plasmodium - Over-NUCLEUS collective view (slime mold coordination)
    #[command(name = "plasmodium")]
    Plasmodium {
        #[command(subcommand)]
        command: PlasmodiumCommand,
    },

    /// RootPulse - Emergent version control via provenance trio coordination
    #[command(name = "rootpulse")]
    RootPulse {
        #[command(subcommand)]
        command: RootPulseCommand,
    },

    /// Continuous mode - Run a continuous coordination graph (game loops, dashboards)
    #[command(name = "continuous")]
    Continuous {
        /// Graph file path (must use coordination = "continuous")
        graph: PathBuf,

        /// Dry run (show pipeline without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// NUCLEUS - Start a NUCLEUS (pure Rust replacement for start_nucleus.sh)
    #[command(name = "nucleus")]
    Nucleus {
        /// Deployment mode: tower|node|nest|full
        #[arg(long, default_value = "full")]
        mode: String,

        /// Node ID (required)
        #[arg(long)]
        node_id: String,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
    },
}

/// RootPulse subcommands — emergent version control via provenance trio
#[derive(Debug, Subcommand)]
enum RootPulseCommand {
    /// Commit a session (dehydrate + sign + store + commit + attribute)
    #[command(name = "commit")]
    Commit {
        /// rhizoCrypt session ID to commit
        #[arg(long)]
        session_id: String,
        /// Agent DID performing the commit
        #[arg(long)]
        agent_did: String,
        /// Neural API Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Family ID (auto-discovered from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
        /// Dry run (show what would happen)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Create a branch from a session
    #[command(name = "branch")]
    Branch {
        /// Source session ID to branch from
        #[arg(long)]
        session_id: String,
        /// Name for the new branch
        #[arg(long)]
        branch_name: String,
        /// Agent DID performing the branch
        #[arg(long)]
        agent_did: String,
        /// Neural API Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Family ID
        #[arg(long)]
        family_id: Option<String>,
        /// Dry run
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Merge two sessions
    #[command(name = "merge")]
    Merge {
        /// Source session ID (branch to merge from)
        #[arg(long)]
        source_session: String,
        /// Target session ID (branch to merge into)
        #[arg(long)]
        target_session: String,
        /// Agent DID performing the merge
        #[arg(long)]
        agent_did: String,
        /// Neural API Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Family ID
        #[arg(long)]
        family_id: Option<String>,
        /// Dry run
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Diff between two commits
    #[command(name = "diff")]
    Diff {
        /// First commit reference
        #[arg(long)]
        from: String,
        /// Second commit reference
        #[arg(long)]
        to: String,
        /// Neural API Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Family ID
        #[arg(long)]
        family_id: Option<String>,
    },

    /// Show status of the provenance trio
    #[command(name = "status")]
    Status {
        /// Neural API Unix socket path
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Family ID
        #[arg(long)]
        family_id: Option<String>,
    },
}

/// Plasmodium subcommands - Over-NUCLEUS collective coordination
#[derive(Debug, Subcommand)]
enum PlasmodiumCommand {
    /// Show collective status of all bonded gates
    #[command(name = "status")]
    Status,

    /// List all gates with hardware details
    #[command(name = "gates")]
    Gates,

    /// Aggregate model caches across all gates
    #[command(name = "models")]
    Models,
}

/// Model cache subcommands
#[derive(Debug, Subcommand)]
enum ModelCacheCommand {
    /// Import all models from HuggingFace cache
    #[command(name = "import-hf")]
    ImportHf,

    /// List all cached models
    #[command(name = "list")]
    List,

    /// Resolve a model (check local, then mesh)
    #[command(name = "resolve")]
    Resolve {
        /// Model ID (e.g., "TinyLlama/TinyLlama-1.1B-Chat-v1.0")
        model_id: String,
    },

    /// Register a model from a local path
    #[command(name = "register")]
    Register {
        /// Model ID
        model_id: String,

        /// Path to model directory
        path: PathBuf,
    },

    /// Show model cache status
    #[command(name = "status")]
    Status,
}

/// Dispatch to mode handler based on CLI (thin orchestration)
async fn dispatch_mode(cli: Cli) -> Result<()> {
    match cli.mode {
        Mode::Cli {} => modes::cli::run(modes::cli::CliCommand).await,
        Mode::Genome { command } => handle_genome_command(command).await,
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
        } => {
            let config = modes::neural_api::resolve_neural_api_config(
                graphs_dir,
                socket,
                family_id.as_deref(),
            );
            modes::neural_api::run(
                config.graphs_dir,
                config.family_id,
                Some(config.socket_path),
            )
            .await
        }
        Mode::Deploy {
            graph,
            validate_only,
            dry_run,
        } => modes::deploy::run(graph, validate_only, dry_run).await,
        Mode::Api {
            port,
            socket,
            unix_only,
        } => modes::api::run(port, socket, unix_only).await,
        Mode::VerifyLineage { path, detailed } => modes::verify_lineage::run(path, detailed).await,
        Mode::Doctor {
            detailed,
            format,
            subsystem,
        } => modes::doctor::run(detailed, format, subsystem).await,
        Mode::Version { detailed } => modes::version::run(detailed).await,
        Mode::Enroll(args) => modes::enroll::run(args).await,
        Mode::ModelCache { command } => modes::model_cache::run(command).await,
        Mode::Plasmodium { command } => modes::plasmodium::run(command).await,
        Mode::RootPulse { command } => modes::rootpulse::dispatch(command).await,
        Mode::Continuous { graph, dry_run } => modes::continuous::run(graph, dry_run).await,
        Mode::Nucleus {
            mode: nucleus_mode,
            node_id,
            family_id,
        } => modes::nucleus::run(nucleus_mode, node_id, family_id).await,
    }
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level, cli.verbose)?;
    dispatch_mode(cli).await
}

/// Handle genome subcommands
///
/// Delegates to biomeos_genomebin_v3 for actual implementation.
async fn handle_genome_command(command: GenomeCommand) -> Result<()> {
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

/// Initialize logging based on configuration
fn init_logging(log_level: &str, verbose: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = if verbose { "debug" } else { log_level };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    Ok(())
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
    #[ignore = "init_logging sets global subscriber; run with --test-threads=1"]
    fn test_init_logging() {
        let result = init_logging("warn", false);
        assert!(result.is_ok());
    }
}
