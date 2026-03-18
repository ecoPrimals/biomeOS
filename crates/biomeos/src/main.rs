// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![cfg_attr(not(test), forbid(unsafe_code))]

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
        #[arg(long)]
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
pub(crate) async fn dispatch_mode(cli: Cli) -> Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level, cli.verbose)?;
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
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_cli_parse_version() {
        let cli = Cli::parse_from(["biomeos", "version"]);
        match &cli.mode {
            Mode::Version { detailed } => assert!(!detailed),
            _ => panic!("expected Version mode"),
        }
    }

    #[test]
    fn test_cli_parse_version_detailed() {
        let cli = Cli::parse_from(["biomeos", "version", "--detailed"]);
        match &cli.mode {
            Mode::Version { detailed } => assert!(*detailed),
            _ => panic!("expected Version mode"),
        }
    }

    #[test]
    fn test_cli_parse_doctor() {
        let cli = Cli::parse_from(["biomeos", "doctor"]);
        match &cli.mode {
            Mode::Doctor { format, .. } => assert_eq!(format, "text"),
            _ => panic!("expected Doctor mode"),
        }
    }

    #[test]
    fn test_cli_parse_doctor_json_format() {
        let cli = Cli::parse_from(["biomeos", "doctor", "-f", "json"]);
        match &cli.mode {
            Mode::Doctor { format, .. } => assert_eq!(format, "json"),
            _ => panic!("expected Doctor mode"),
        }
    }

    #[test]
    fn test_cli_parse_model_cache_list() {
        let cli = Cli::parse_from(["biomeos", "model-cache", "list"]);
        match &cli.mode {
            Mode::ModelCache { command } => match command {
                ModelCacheCommand::List => {}
                _ => panic!("expected List subcommand"),
            },
            _ => panic!("expected ModelCache mode"),
        }
    }

    #[test]
    fn test_cli_parse_rootpulse_commit() {
        let cli = Cli::parse_from([
            "biomeos",
            "rootpulse",
            "commit",
            "--session-id",
            "sess-1",
            "--agent-did",
            "did:key:z6Mk",
        ]);
        match &cli.mode {
            Mode::RootPulse { command } => match command {
                RootPulseCommand::Commit {
                    session_id,
                    agent_did,
                    dry_run,
                    ..
                } => {
                    assert_eq!(session_id, "sess-1");
                    assert_eq!(agent_did, "did:key:z6Mk");
                    assert!(!*dry_run);
                }
                _ => panic!("expected Commit subcommand"),
            },
            _ => panic!("expected RootPulse mode"),
        }
    }

    #[test]
    fn test_cli_parse_neural_api() {
        let cli = Cli::parse_from(["biomeos", "neural-api"]);
        match &cli.mode {
            Mode::NeuralApi {
                graphs_dir,
                family_id,
                socket,
            } => {
                assert_eq!(graphs_dir, &PathBuf::from("graphs"));
                assert!(family_id.is_none());
                assert!(socket.is_none());
            }
            _ => panic!("expected NeuralApi mode"),
        }
    }

    #[test]
    fn test_cli_parse_neural_api_with_opts() {
        let cli = Cli::parse_from([
            "biomeos",
            "neural-api",
            "--graphs-dir",
            "/tmp/graphs",
            "--family-id",
            "fam1",
            "--socket",
            "/tmp/api.sock",
        ]);
        match &cli.mode {
            Mode::NeuralApi {
                graphs_dir,
                family_id,
                socket,
            } => {
                assert_eq!(graphs_dir, &PathBuf::from("/tmp/graphs"));
                assert_eq!(family_id.as_deref(), Some("fam1"));
                assert_eq!(
                    socket.as_ref().map(|p| p.as_path()),
                    Some(std::path::Path::new("/tmp/api.sock"))
                );
            }
            _ => panic!("expected NeuralApi mode"),
        }
    }

    #[test]
    fn test_cli_parse_deploy() {
        let cli = Cli::parse_from(["biomeos", "deploy", "graph.json"]);
        match &cli.mode {
            Mode::Deploy {
                graph,
                validate_only,
                dry_run,
            } => {
                assert_eq!(graph, &PathBuf::from("graph.json"));
                assert!(!*validate_only);
                assert!(!*dry_run);
            }
            _ => panic!("expected Deploy mode"),
        }
    }

    #[test]
    fn test_cli_parse_deploy_validate_dry_run() {
        let cli = Cli::parse_from([
            "biomeos",
            "deploy",
            "g.json",
            "--validate-only",
            "--dry-run",
        ]);
        match &cli.mode {
            Mode::Deploy {
                validate_only,
                dry_run,
                ..
            } => {
                assert!(*validate_only);
                assert!(*dry_run);
            }
            _ => panic!("expected Deploy mode"),
        }
    }

    #[test]
    fn test_cli_parse_api() {
        let cli = Cli::parse_from(["biomeos", "api"]);
        match &cli.mode {
            Mode::Api {
                port,
                socket,
                unix_only,
            } => {
                assert!(port.is_none());
                assert!(socket.is_none());
                assert!(!*unix_only);
            }
            _ => panic!("expected Api mode"),
        }
    }

    #[test]
    fn test_cli_parse_api_with_port_and_socket() {
        let cli = Cli::parse_from(["biomeos", "api", "-p", "8080", "--socket", "/tmp/api.sock"]);
        match &cli.mode {
            Mode::Api {
                port,
                socket,
                unix_only,
            } => {
                assert_eq!(*port, Some(8080));
                assert_eq!(
                    socket.as_ref().map(|p| p.as_path()),
                    Some(std::path::Path::new("/tmp/api.sock"))
                );
                assert!(!*unix_only);
            }
            _ => panic!("expected Api mode"),
        }
    }

    #[test]
    fn test_cli_parse_verify_lineage() {
        let cli = Cli::parse_from(["biomeos", "verify-lineage", "/path/to/spore"]);
        match &cli.mode {
            Mode::VerifyLineage { path, detailed } => {
                assert_eq!(path, &PathBuf::from("/path/to/spore"));
                assert!(!*detailed);
            }
            _ => panic!("expected VerifyLineage mode"),
        }
    }

    #[test]
    fn test_cli_parse_verify_lineage_detailed() {
        let cli = Cli::parse_from(["biomeos", "verify-lineage", "/p", "--detailed"]);
        match &cli.mode {
            Mode::VerifyLineage { detailed, .. } => assert!(*detailed),
            _ => panic!("expected VerifyLineage mode"),
        }
    }

    #[test]
    fn test_cli_parse_nucleus() {
        let cli = Cli::parse_from(["biomeos", "nucleus", "--node-id", "node1"]);
        match &cli.mode {
            Mode::Nucleus {
                mode,
                node_id,
                family_id,
            } => {
                assert_eq!(mode, "full");
                assert_eq!(node_id, "node1");
                assert!(family_id.is_none());
            }
            _ => panic!("expected Nucleus mode"),
        }
    }

    #[test]
    fn test_cli_parse_nucleus_with_mode_and_family() {
        let cli = Cli::parse_from([
            "biomeos",
            "nucleus",
            "--mode",
            "tower",
            "--node-id",
            "n1",
            "--family-id",
            "fam1",
        ]);
        match &cli.mode {
            Mode::Nucleus {
                mode,
                node_id,
                family_id,
            } => {
                assert_eq!(mode, "tower");
                assert_eq!(node_id, "n1");
                assert_eq!(family_id.as_deref(), Some("fam1"));
            }
            _ => panic!("expected Nucleus mode"),
        }
    }

    #[test]
    fn test_cli_parse_continuous() {
        let cli = Cli::parse_from(["biomeos", "continuous", "graph.json"]);
        match &cli.mode {
            Mode::Continuous { graph, dry_run } => {
                assert_eq!(graph, &PathBuf::from("graph.json"));
                assert!(!*dry_run);
            }
            _ => panic!("expected Continuous mode"),
        }
    }

    #[test]
    fn test_cli_parse_plasmodium_status() {
        let cli = Cli::parse_from(["biomeos", "plasmodium", "status"]);
        match &cli.mode {
            Mode::Plasmodium { command } => match command {
                PlasmodiumCommand::Status => {}
                _ => panic!("expected Status subcommand"),
            },
            _ => panic!("expected Plasmodium mode"),
        }
    }

    #[test]
    fn test_cli_parse_verbose_and_log_level() {
        let cli = Cli::parse_from(["biomeos", "--verbose", "--log-level", "debug", "version"]);
        assert!(cli.verbose);
        assert_eq!(cli.log_level, "debug");
    }

    #[test]
    fn test_cli_parse_genome_list_default_dir() {
        let cli = Cli::parse_from(["biomeos", "genome", "list"]);
        match &cli.mode {
            Mode::Genome { command } => match command {
                GenomeCommand::List(args) => {
                    assert_eq!(args.directory, PathBuf::from("plasmidBin"));
                }
                _ => panic!("expected List subcommand"),
            },
            _ => panic!("expected Genome mode"),
        }
    }

    #[test]
    fn test_cli_parse_cli_mode() {
        let cli = Cli::parse_from(["biomeos", "cli"]);
        match &cli.mode {
            Mode::Cli {} => {}
            _ => panic!("expected Cli mode"),
        }
    }

    #[test]
    fn test_cli_parse_genome_build() {
        let cli = Cli::parse_from([
            "biomeos",
            "genome",
            "build",
            "--output",
            "/tmp/out.genome",
            "--name",
            "test",
        ]);
        match &cli.mode {
            Mode::Genome { command } => match command {
                GenomeCommand::Build(args) => {
                    assert_eq!(args.output, PathBuf::from("/tmp/out.genome"));
                    assert_eq!(args.name.as_deref(), Some("test"));
                }
                _ => panic!("expected Build subcommand"),
            },
            _ => panic!("expected Genome mode"),
        }
    }

    #[test]
    fn test_cli_parse_genome_verify() {
        let cli = Cli::parse_from(["biomeos", "genome", "verify", "/path/to/genome.genome"]);
        match &cli.mode {
            Mode::Genome { command } => match command {
                GenomeCommand::Verify(args) => {
                    assert_eq!(args.path, PathBuf::from("/path/to/genome.genome"))
                }
                _ => panic!("expected Verify subcommand"),
            },
            _ => panic!("expected Genome mode"),
        }
    }

    #[test]
    fn test_cli_parse_model_cache_resolve() {
        let cli = Cli::parse_from([
            "biomeos",
            "model-cache",
            "resolve",
            "TinyLlama/TinyLlama-1.1B",
        ]);
        match &cli.mode {
            Mode::ModelCache { command } => match command {
                ModelCacheCommand::Resolve { model_id } => {
                    assert_eq!(model_id, "TinyLlama/TinyLlama-1.1B")
                }
                _ => panic!("expected Resolve subcommand"),
            },
            _ => panic!("expected ModelCache mode"),
        }
    }

    #[test]
    fn test_cli_parse_model_cache_register() {
        let cli = Cli::parse_from([
            "biomeos",
            "model-cache",
            "register",
            "test/model",
            "/path/to/model",
        ]);
        match &cli.mode {
            Mode::ModelCache { command } => match command {
                ModelCacheCommand::Register { model_id, path } => {
                    assert_eq!(model_id, "test/model");
                    assert_eq!(path, &PathBuf::from("/path/to/model"));
                }
                _ => panic!("expected Register subcommand"),
            },
            _ => panic!("expected ModelCache mode"),
        }
    }

    #[test]
    fn test_cli_parse_model_cache_status() {
        let cli = Cli::parse_from(["biomeos", "model-cache", "status"]);
        match &cli.mode {
            Mode::ModelCache { command } => match command {
                ModelCacheCommand::Status => {}
                _ => panic!("expected Status subcommand"),
            },
            _ => panic!("expected ModelCache mode"),
        }
    }

    #[test]
    fn test_cli_parse_model_cache_import_hf() {
        let cli = Cli::parse_from(["biomeos", "model-cache", "import-hf"]);
        match &cli.mode {
            Mode::ModelCache { command } => match command {
                ModelCacheCommand::ImportHf => {}
                _ => panic!("expected ImportHf subcommand"),
            },
            _ => panic!("expected ModelCache mode"),
        }
    }

    #[test]
    fn test_cli_parse_plasmodium_gates() {
        let cli = Cli::parse_from(["biomeos", "plasmodium", "gates"]);
        match &cli.mode {
            Mode::Plasmodium { command } => match command {
                PlasmodiumCommand::Gates => {}
                _ => panic!("expected Gates subcommand"),
            },
            _ => panic!("expected Plasmodium mode"),
        }
    }

    #[test]
    fn test_cli_parse_plasmodium_models() {
        let cli = Cli::parse_from(["biomeos", "plasmodium", "models"]);
        match &cli.mode {
            Mode::Plasmodium { command } => match command {
                PlasmodiumCommand::Models => {}
                _ => panic!("expected Models subcommand"),
            },
            _ => panic!("expected Plasmodium mode"),
        }
    }

    #[test]
    fn test_cli_parse_rootpulse_branch() {
        let cli = Cli::parse_from([
            "biomeos",
            "rootpulse",
            "branch",
            "--session-id",
            "s1",
            "--branch-name",
            "feature",
            "--agent-did",
            "did:key:z6Mk",
        ]);
        match &cli.mode {
            Mode::RootPulse { command } => match command {
                RootPulseCommand::Branch {
                    session_id,
                    branch_name,
                    agent_did,
                    ..
                } => {
                    assert_eq!(session_id, "s1");
                    assert_eq!(branch_name, "feature");
                    assert_eq!(agent_did, "did:key:z6Mk");
                }
                _ => panic!("expected Branch subcommand"),
            },
            _ => panic!("expected RootPulse mode"),
        }
    }

    #[test]
    fn test_cli_parse_rootpulse_status() {
        let cli = Cli::parse_from(["biomeos", "rootpulse", "status"]);
        match &cli.mode {
            Mode::RootPulse { command } => match command {
                RootPulseCommand::Status { .. } => {}
                _ => panic!("expected Status subcommand"),
            },
            _ => panic!("expected RootPulse mode"),
        }
    }

    #[test]
    fn test_cli_parse_continuous_dry_run() {
        let cli = Cli::parse_from(["biomeos", "continuous", "graph.json", "--dry-run"]);
        match &cli.mode {
            Mode::Continuous { graph, dry_run } => {
                assert_eq!(graph, &PathBuf::from("graph.json"));
                assert!(*dry_run);
            }
            _ => panic!("expected Continuous mode"),
        }
    }

    #[test]
    fn test_cli_parse_api_unix_only() {
        let cli = Cli::parse_from(["biomeos", "api", "--unix-only"]);
        match &cli.mode {
            Mode::Api { unix_only, .. } => assert!(*unix_only),
            _ => panic!("expected Api mode"),
        }
    }

    #[test]
    #[ignore = "init_logging sets global subscriber; run with --test-threads=1"]
    fn test_init_logging_verbose_overrides_level() {
        let result = init_logging("warn", true);
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "init_logging sets global subscriber; run with --test-threads=1"]
    async fn test_dispatch_mode_version() {
        let cli = Cli::parse_from(["biomeos", "version"]);
        let result = dispatch_mode(cli).await;
        result.expect("dispatch version should succeed");
    }

    #[test]
    #[ignore = "init_logging sets global subscriber; run with --test-threads=1"]
    fn test_init_logging() {
        let result = init_logging("warn", false);
        assert!(result.is_ok());
    }
}
