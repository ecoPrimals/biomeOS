// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests use unwrap/expect for concise assertions"
    )
)]

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

#[cfg(test)]
pub(crate) static CWD_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod genome;
mod modes;
mod proc_metrics;

use genome::{GenomeCommand, handle_genome_command};

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
    Cli {},

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

        /// TCP port for mobile/cross-gate orchestration (alongside UDS by default)
        #[arg(long)]
        port: Option<u16>,

        /// TCP-only mode: skip Unix socket, bind TCP only (mobile substrates)
        #[arg(long, requires = "port")]
        tcp_only: bool,

        /// Disable BTSP enforcement for unauthenticated JSON-RPC clients.
        /// Equivalent to BIOMEOS_BTSP_ENFORCE=0. Use during development or
        /// when downstream springs have not yet implemented the BTSP handshake.
        #[arg(long)]
        btsp_optional: bool,
    },

    /// Deploy mode - Execute deployment graph
    #[command(name = "deploy")]
    Deploy {
        /// Graph file path
        graph: PathBuf,

        /// Validate only (don't execute)
        #[arg(long)]
        validate_only: bool,

        /// Dry run (show what would happen)
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip signature verification (development only)
        #[arg(long)]
        skip_signature_check: bool,
    },

    /// Graph operations - Sign and verify deployment graphs
    #[command(name = "graph")]
    Graph {
        #[command(subcommand)]
        command: GraphCommand,
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

        /// TCP port for Neural API (mobile/cross-gate orchestration, alongside UDS)
        #[arg(long)]
        port: Option<u16>,

        /// TCP-only mode: skip Unix socket for Neural API (mobile substrates)
        #[arg(long, requires = "port")]
        tcp_only: bool,
    },
}

/// Graph subcommands — sign and verify deployment graphs
#[derive(Debug, Subcommand)]
enum GraphCommand {
    /// Sign a graph TOML via BearDog delegation
    #[command(name = "sign")]
    Sign {
        /// Path to the graph TOML file
        path: PathBuf,
    },

    /// Verify a graph's integrity (content hash + signature)
    #[command(name = "verify")]
    Verify {
        /// Path to the graph TOML file
        path: PathBuf,
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
pub(crate) async fn dispatch_mode(cli: Cli) -> Result<()> {
    match cli.mode {
        Mode::Cli {} => modes::cli::run(modes::cli::CliCommand).await,
        Mode::Genome { command } => handle_genome_command(command).await,
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
            port,
            tcp_only,
            btsp_optional,
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
                port,
                tcp_only,
                btsp_optional,
            )
            .await
        }
        Mode::Deploy {
            graph,
            validate_only,
            dry_run,
            skip_signature_check,
        } => modes::deploy::run(graph, validate_only, dry_run, skip_signature_check).await,
        Mode::Graph { command } => match command {
            GraphCommand::Sign { path } => modes::graph_ops::sign(path).await,
            GraphCommand::Verify { path } => modes::graph_ops::verify(path).await,
        },
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
            port,
            tcp_only,
        } => modes::nucleus::run(nucleus_mode, node_id, family_id, port, tcp_only).await,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level, cli.verbose)?;

    if let Err(msg) = biomeos_core::btsp_client::validate_insecure_guard() {
        anyhow::bail!(msg);
    }
    biomeos_core::btsp_client::log_security_posture();

    dispatch_mode(cli).await
}

/// Initialize logging based on configuration
fn init_logging(log_level: &str, verbose: bool) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};

    let level = if verbose { "debug" } else { log_level };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .try_init()
        .ok();

    Ok(())
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
