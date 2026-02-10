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
use std::path::PathBuf;

mod modes;

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level, cli.verbose)?;

    // Dispatch to mode handler
    match cli.mode {
        Mode::Cli {} => modes::cli::run(modes::cli::CliCommand).await,
        Mode::Genome { command } => handle_genome_command(command).await,
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
        } => {
            // Use family discovery if not explicitly specified
            let resolved_family_id =
                family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
            modes::neural_api::run(graphs_dir, resolved_family_id, socket).await
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
        Mode::Nucleus {
            mode: nucleus_mode,
            node_id,
            family_id,
        } => modes::nucleus::run(nucleus_mode, node_id, family_id).await,
    }
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
                    .map_err(|e| anyhow::anyhow!("Failed to add x86_64 binary: {}", e))?;
                info!("   Added x86_64 binary: {}", path.display());
            }

            // Add aarch64 binary if provided
            if let Some(ref path) = args.binary_aarch64 {
                genome
                    .add_binary(Arch::Aarch64, path)
                    .map_err(|e| anyhow::anyhow!("Failed to add aarch64 binary: {}", e))?;
                info!("   Added aarch64 binary: {}", path.display());
            }

            // Save as JSON manifest
            let json = genome
                .to_json()
                .map_err(|e| anyhow::anyhow!("Failed to serialize: {}", e))?;
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
                .map_err(|e| anyhow::anyhow!("Failed to parse: {}", e))?;

            info!("   Name: {}", genome.manifest.name);
            info!("   Version: {}", genome.manifest.version);
            info!("   Architectures: {:?}", genome.manifest.architectures);
            info!("✅ GenomeBin valid");
            Ok(())
        }

        GenomeCommand::Info(args) => {
            let content = std::fs::read_to_string(&args.path)?;
            let genome = GenomeBin::from_json(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse: {}", e))?;

            println!("GenomeBin: {}", genome.manifest.name);
            println!("  Version: {}", genome.manifest.version);
            println!("  Description: {}", genome.manifest.description);
            println!("  Architectures:");
            for arch in &genome.manifest.architectures {
                println!("    - {:?}", arch);
            }
            Ok(())
        }

        GenomeCommand::List(args) => {
            info!("📋 Listing genomeBins in: {}", args.directory.display());

            if !args.directory.exists() {
                println!("Directory not found: {}", args.directory.display());
                return Ok(());
            }

            for entry in std::fs::read_dir(&args.directory)? {
                let entry = entry?;
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e == "genome" || e == "json")
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(genome) = GenomeBin::from_json(&content) {
                            println!(
                                "  {} v{} ({:?})",
                                genome.manifest.name,
                                genome.manifest.version,
                                genome.manifest.architectures
                            );
                        }
                    }
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
