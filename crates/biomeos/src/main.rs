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
use tracing::warn;

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

    /// Neural API server mode (DEPRECATED: use `biomeos api` or `biomeos nucleus` instead)
    ///
    /// Both `api` and `nucleus` modes now include the Neural API automatically (G22).
    /// This standalone mode is kept for backward compatibility but will be removed.
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

        /// Transport bind mode: uds_only | dual.
        /// Reads PRIMAL_BIND_MODE env if not specified.
        /// dual: UDS primary + TCP alongside (recommended for cross-gate access).
        /// uds_only: UDS only (default).
        /// tcp_only: DEPRECATED — use dual instead. Transport resolved by composition.
        #[arg(long, value_name = "MODE")]
        bind_mode: Option<String>,

        /// DEPRECATED: use --bind-mode dual. Transport resolved by atomic composition.
        #[arg(long, requires = "port", hide = true)]
        tcp_only: bool,

        /// TCP bind address (default: 127.0.0.1). Use 0.0.0.0 for all interfaces.
        #[arg(long)]
        bind: Option<String>,

        /// Disable BTSP enforcement for unauthenticated JSON-RPC clients.
        /// Auto-inferred when bind-mode is tcp_only (legacy) or dual with external peers.
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

        /// TCP bind address (default: 127.0.0.1). Use 0.0.0.0 for all interfaces.
        #[arg(long)]
        bind: Option<String>,

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

    /// NUCLEUS - Start, ingest, and emit spores
    #[command(name = "nucleus")]
    Nucleus {
        #[command(subcommand)]
        command: NucleusCommand,
    },
}

/// NUCLEUS subcommands — start, ingest, and emit spores
#[derive(Debug, Subcommand)]
pub(crate) enum NucleusCommand {
    /// Start a NUCLEUS (pure Rust primal orchestrator)
    #[command(name = "start")]
    Start {
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

        /// Transport bind mode: uds_only | dual.
        /// Reads PRIMAL_BIND_MODE env if not specified.
        /// tcp_only: DEPRECATED — use dual instead. Transport resolved by composition.
        #[arg(long, value_name = "MODE")]
        bind_mode: Option<String>,

        /// DEPRECATED: use --bind-mode dual. Transport resolved by atomic composition.
        #[arg(long, requires = "port", hide = true)]
        tcp_only: bool,

        /// TCP bind address (default: 127.0.0.1). Use 0.0.0.0 for all interfaces.
        #[arg(long)]
        bind: Option<String>,
    },

    /// Ingest a pseudoSpore into NUCLEUS via provenance trio
    #[command(name = "ingest")]
    Ingest {
        /// Path to the pseudoSpore directory
        pseudospore_dir: PathBuf,

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

    /// Emit a spore from NUCLEUS (retrieve, package, sign)
    #[command(name = "emit")]
    Emit {
        /// Spore ID to emit
        spore_id: String,

        /// Output directory for the emitted spore package
        #[arg(short, long)]
        output: Option<PathBuf>,

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

    /// Attach a cell (garden/protist) to a running NUCLEUS composition
    ///
    /// Validates that the NUCLEUS is healthy, then executes the cell deploy
    /// graph which starts the cell primals and registers them in the
    /// composition's capability registry. The cell graph handles health
    /// verification, primal startup, and capability wiring.
    ///
    /// Example:
    ///   biomeos nucleus attach graphs/esotericwebb_cell.toml
    ///   biomeos nucleus attach graphs/footprint_cell.toml --dry-run
    #[command(name = "attach")]
    Attach {
        /// Path to the cell deploy graph TOML
        cell_graph: PathBuf,

        /// Neural API Unix socket path (auto-discovered if not specified)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,

        /// Dry run (validate and show plan without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Stop a running NUCLEUS via remote lifecycle RPC
    #[command(name = "stop")]
    Stop {
        /// Neural API Unix socket path (auto-discovered if not specified)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
    },

    /// Query NUCLEUS status via remote lifecycle RPC
    #[command(name = "status")]
    Status {
        /// Neural API Unix socket path (auto-discovered if not specified)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
    },

    /// Deploy a spore manifest to a running NUCLEUS
    #[command(name = "deploy")]
    Deploy {
        /// Path to the spore manifest file
        spore_file: PathBuf,

        /// Neural API Unix socket path (auto-discovered if not specified)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
    },

    /// Undeploy a primal from a running NUCLEUS (apoptosis)
    #[command(name = "undeploy")]
    Undeploy {
        /// Primal name to undeploy
        #[arg(long)]
        primal: Option<String>,

        /// Neural API Unix socket path (auto-discovered if not specified)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID (auto-derived from .family.seed if not specified)
        #[arg(long)]
        family_id: Option<String>,
    },
}

/// Graph subcommands — sign, verify, and execute deployment graphs
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

    /// Execute a graph TOML against the live Neural API server
    #[command(name = "execute")]
    Execute {
        /// Path to the graph TOML file (or graph ID from the graphs directory)
        graph: String,

        /// Parameters as key=value pairs (e.g. --param SESSION_ID=abc123)
        #[arg(long = "param", value_name = "KEY=VALUE")]
        params: Vec<String>,

        /// Neural API socket path (auto-discovered from family if not set)
        #[arg(long)]
        socket: Option<PathBuf>,

        /// Family ID
        #[arg(long)]
        family_id: Option<String>,

        /// Dry run — show what would be sent without executing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
}

/// RootPulse subcommands — emergent version control via provenance trio
#[derive(Debug, Subcommand)]
enum RootPulseCommand {
    /// Commit a session (dehydrate + sign + store + commit + attribute)
    #[command(name = "commit")]
    Commit {
        /// Provenance session ID to commit
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

/// Resolve bind mode from `--bind-mode` CLI flag, `--tcp-only` legacy flag,
/// and `PRIMAL_BIND_MODE` env var.
fn resolve_bind_mode(
    cli_bind_mode: Option<&str>,
    tcp_only_flag: bool,
) -> biomeos_types::env_config::BindMode {
    use biomeos_types::env_config::BindMode;

    if let Some(mode_str) = cli_bind_mode {
        if let Some(mode) = BindMode::from_str_flexible(mode_str) {
            if mode == BindMode::TcpOnly {
                tracing::warn!(
                    "DEPRECATED: --bind-mode tcp_only is a legacy pattern. \
                     Use --bind-mode dual — transport strategy is resolved by \
                     the gate's atomic composition profile."
                );
            }
            return mode;
        }
        tracing::warn!("Unknown --bind-mode '{mode_str}', falling back to env/default");
    }

    if tcp_only_flag {
        tracing::warn!(
            "DEPRECATED: --tcp-only is a legacy flag. \
             Use --bind-mode dual — transport strategy is resolved by \
             the gate's atomic composition profile."
        );
        return BindMode::TcpOnly;
    }

    BindMode::from_env_or(BindMode::UdsOnly)
}

/// Dispatch to mode handler based on CLI (thin orchestration)
#[expect(
    clippy::too_many_lines,
    reason = "flat dispatch table — complexity is O(1) per arm"
)]
pub(crate) async fn dispatch_mode(cli: Cli) -> Result<()> {
    match cli.mode {
        Mode::Cli {} => modes::cli::run(modes::cli::CliCommand).await,
        Mode::Genome { command } => handle_genome_command(command).await,
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
            port,
            bind_mode,
            tcp_only,
            bind,
            btsp_optional,
        } => {
            warn!(
                "⚠️  `biomeos neural-api` is deprecated. Use `biomeos api` or `biomeos nucleus` \
                 instead — both include the Neural API automatically (G22 convergence)."
            );
            let bind_mode = resolve_bind_mode(bind_mode.as_deref(), tcp_only);
            let effective_btsp_optional = btsp_optional || bind_mode.is_tcp_only();
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
                bind_mode.is_tcp_only(),
                bind,
                effective_btsp_optional,
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
            GraphCommand::Execute {
                graph,
                params,
                socket,
                family_id,
                dry_run,
            } => modes::graph_ops::execute(graph, params, socket, family_id, dry_run).await,
        },
        Mode::Api {
            port,
            socket,
            bind,
            unix_only,
        } => modes::api::run(port, socket, unix_only, bind).await,
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
        Mode::Nucleus { command } => match command {
            NucleusCommand::Start {
                mode: nucleus_mode,
                node_id,
                family_id,
                port,
                bind_mode,
                tcp_only,
                bind,
            } => {
                let bind_mode = resolve_bind_mode(bind_mode.as_deref(), tcp_only);
                modes::nucleus::run(modes::nucleus::NucleusRunConfig {
                    mode: nucleus_mode,
                    node_id,
                    family_id,
                    tcp_port: port,
                    tcp_only: bind_mode.is_tcp_only(),
                    bind,
                })
                .await
            }
            NucleusCommand::Ingest {
                pseudospore_dir,
                socket,
                family_id,
                dry_run,
            } => {
                modes::nucleus_ingest::run_ingest(pseudospore_dir, socket, family_id, dry_run).await
            }
            NucleusCommand::Emit {
                spore_id,
                output,
                socket,
                family_id,
                dry_run,
            } => {
                modes::nucleus_ingest::run_emit(spore_id, output, socket, family_id, dry_run).await
            }
            NucleusCommand::Attach {
                cell_graph,
                socket,
                family_id,
                dry_run,
            } => modes::nucleus_attach::run(cell_graph, socket, family_id, dry_run).await,
            NucleusCommand::Stop { socket, family_id } => {
                modes::nucleus::run_stop(socket, family_id).await
            }
            NucleusCommand::Status { socket, family_id } => {
                let summary = modes::nucleus::run_status(socket, family_id).await?;
                println!("{summary:?}");
                Ok(())
            }
            NucleusCommand::Deploy {
                spore_file,
                socket,
                family_id,
            } => modes::nucleus::run_deploy(spore_file, socket, family_id).await,
            NucleusCommand::Undeploy {
                primal,
                socket,
                family_id,
            } => modes::nucleus::run_undeploy(primal, socket, family_id).await,
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level, cli.verbose)?;

    raise_fd_limit();

    if let Err(e) = biomeos_core::btsp_client::validate_insecure_guard() {
        anyhow::bail!(e);
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

/// Raise the process soft FD limit to 65536 (or hard limit if lower).
///
/// P1 fix: several gates don't have `LimitNOFILE=65536` in their systemd units.
/// Rather than depending on external service configuration, biomeOS raises its own
/// limit at startup. This is safe: we only raise the soft limit up to the hard limit.
#[cfg(unix)]
fn raise_fd_limit() {
    use rustix::process::{Resource, getrlimit, setrlimit, Rlimit};
    use tracing::{debug, info, warn};

    const TARGET_NOFILE: u64 = 65536;

    let current = getrlimit(Resource::Nofile);
    let soft = current.current.unwrap_or(1024);
    let hard = current.maximum.unwrap_or(TARGET_NOFILE);

    if soft >= TARGET_NOFILE {
        debug!("FD soft limit already adequate: {soft}");
        return;
    }

    let new_soft = TARGET_NOFILE.min(hard);
    let new_limit = Rlimit {
        current: Some(new_soft),
        maximum: current.maximum,
    };

    match setrlimit(Resource::Nofile, new_limit) {
        Ok(()) => info!("Raised FD soft limit: {soft} → {new_soft} (hard={hard})"),
        Err(e) => warn!("Failed to raise FD limit from {soft} to {new_soft}: {e}"),
    }
}

#[cfg(not(unix))]
fn raise_fd_limit() {
    // Windows/WASM: no-op (FD limits are not a concern on these platforms)
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
