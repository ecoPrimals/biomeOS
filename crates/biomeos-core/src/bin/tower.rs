// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]

//! Tower CLI - Thin binary wrapper over `biomeos_core::tower_orchestration`.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

use biomeos_core::tower_orchestration::{self, std_env_lookup};

#[derive(Parser)]
#[command(name = "tower")]
#[command(about = "Capability-based primal orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start tower from config file (modern, idiomatic way!)
    Run {
        /// Config file path (default: tower.toml)
        #[arg(short, long, default_value = "tower.toml")]
        config: PathBuf,

        /// Override: scan directory for primals
        #[arg(long)]
        scan: Option<PathBuf>,

        /// Enable concurrent wave-based startup
        #[arg(long, default_value_t = true)]
        concurrent: bool,
    },

    /// Start a tower with primals discovered from environment (legacy)
    Start {
        /// Security provider binary path
        #[arg(long, env = "SECURITY_PROVIDER_BINARY")]
        security_binary: Option<String>,

        /// Security provider HTTP port (0 = auto)
        #[arg(long, env = "SECURITY_PROVIDER_PORT", default_value = "0")]
        security_port: u16,

        /// Discovery orchestrator binary path
        #[arg(long, env = "DISCOVERY_ORCHESTRATOR_BINARY")]
        discovery_binary: Option<String>,

        /// Additional primal binaries (comma-separated)
        #[arg(long, env = "ADDITIONAL_PRIMALS")]
        additional: Option<String>,
    },

    /// Start from pure environment (Infant Model - ZERO flags!)
    StartFromEnv,

    /// Stop all managed primals
    Stop,

    /// Show primal status
    Status,

    /// List available capabilities
    Capabilities,

    /// Scan directory and list discovered primals
    Discover {
        /// Directory to scan
        #[arg(default_value = "./primals")]
        directory: PathBuf,
    },
}

#[tokio::main]
#[expect(
    clippy::too_many_lines,
    reason = "CLI dispatch — each arm is thin delegation"
)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    if let Err(msg) = biomeos_core::btsp_client::validate_insecure_guard() {
        anyhow::bail!(msg);
    }
    biomeos_core::btsp_client::log_security_posture();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            config,
            scan,
            concurrent,
        } => tower_orchestration::run_tower(&config, scan, concurrent, &std_env_lookup).await,

        Commands::Start {
            security_binary,
            security_port,
            discovery_binary,
            additional,
        } => {
            tower_orchestration::start_tower_legacy(
                security_binary,
                security_port,
                discovery_binary,
                additional,
                &std_env_lookup,
            )
            .await
        }

        Commands::StartFromEnv => {
            info!("Starting tower from PURE ENVIRONMENT (Infant Model)...");
            match biomeos_core::GenericManagedPrimal::from_env() {
                Ok(primal) => {
                    info!("Discovered primal from environment");
                    let health_monitor =
                        std::sync::Arc::new(biomeos_core::PrimalHealthMonitor::builder().build());
                    let retry_policy = biomeos_core::RetryPolicy::exponential(
                        3,
                        std::time::Duration::from_millis(100),
                    );
                    let orchestrator =
                        biomeos_core::PrimalOrchestrator::new(health_monitor.clone(), retry_policy);
                    orchestrator.register(std::sync::Arc::new(primal)).await;
                    orchestrator.start_all().await?;
                    info!("Tower started from environment");

                    let pid_file = tower_orchestration::pid_file_path(&std_env_lookup);
                    if let Err(e) = tower_orchestration::write_pid_file(&pid_file) {
                        tracing::warn!("Failed to write PID file: {}", e);
                    }

                    tokio::signal::ctrl_c().await?;
                    info!("Stopping tower...");
                    tower_orchestration::cleanup_pid_file(&pid_file);
                    orchestrator.stop_all().await?;
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to discover primal from environment: {}", e);
                    error!("Set PRIMAL_PROVIDES, PRIMAL_BINARY, HTTP_PORT");
                    Err(e.into())
                }
            }
        }

        Commands::Stop => {
            let pid_file = tower_orchestration::pid_file_path(&std_env_lookup);
            tower_orchestration::stop_tower(&pid_file)
        }

        Commands::Status => {
            let pid_file = tower_orchestration::pid_file_path(&std_env_lookup);
            match tower_orchestration::tower_status(&pid_file, &std_env_lookup)? {
                tower_orchestration::TowerStatusReport::NotRunning => {
                    info!("No running tower found");
                    info!("Start with: tower run --config tower.toml");
                }
                tower_orchestration::TowerStatusReport::InvalidPid => {
                    error!("Invalid PID in file");
                }
                tower_orchestration::TowerStatusReport::Running {
                    pid,
                    socket_dir,
                    sockets,
                    family_id,
                } => {
                    info!("Tower is RUNNING (PID: {})", pid);
                    info!("Socket Directory: {}", socket_dir.display());
                    for s in &sockets {
                        info!("  Socket: {}", s);
                    }
                    if let Some(fam) = family_id {
                        info!("Family ID: {}", fam);
                    }
                }
                tower_orchestration::TowerStatusReport::Stale { pid } => {
                    info!("Tower process (PID: {}) is not running, cleaned up", pid);
                }
            }
            Ok(())
        }

        Commands::Capabilities => {
            info!("Available Capabilities:");
            for (name, desc) in tower_orchestration::format_capabilities() {
                info!("  {name} - {desc}");
            }
            info!("Use PRIMAL_PROVIDES and PRIMAL_REQUIRES to declare capabilities");
            Ok(())
        }

        Commands::Discover { directory } => {
            info!("Scanning directory for primals: {}", directory.display());
            match biomeos_core::discover_primals(&directory).await {
                Ok(primals) => {
                    info!("Discovered {} primal(s)", primals.len());
                    for (i, primal) in primals.iter().enumerate() {
                        info!("Primal #{}: {} ({:?})", i + 1, primal.id, primal.binary);
                        info!("  Provides: {:?}", primal.provides);
                        info!("  Requires: {:?}", primal.requires);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to discover primals: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
