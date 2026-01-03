//! Tower CLI - Capability-Based Primal Orchestration
//!
//! Zero-hardcoding, environment-driven primal management.
//! Uses the "Infant Model" - discovers everything at runtime!

use std::sync::Arc;

use anyhow::Result;
use biomeos_core::{
    create_discovery_orchestrator, create_security_provider, Capability, PrimalBuilder,
    PrimalHealthMonitor, PrimalOrchestrator, RetryPolicy,
};
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "tower")]
#[command(about = "Capability-based primal orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a tower with primals discovered from environment
    Start {
        /// Security provider binary path (e.g., /path/to/beardog-server)
        #[arg(long, env = "SECURITY_PROVIDER_BINARY")]
        security_binary: Option<String>,

        /// Security provider HTTP port (0 = auto)
        #[arg(long, env = "SECURITY_PROVIDER_PORT", default_value = "0")]
        security_port: u16,

        /// Discovery orchestrator binary path (e.g., /path/to/songbird-orchestrator)
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
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            security_binary,
            security_port,
            discovery_binary,
            additional,
        } => {
            info!("🚀 Starting tower with capability-based orchestration...");

            // Create health monitor
            let health_monitor = Arc::new(
                PrimalHealthMonitor::builder()
                    .health_checker(Arc::new(biomeos_core::HttpHealthChecker))
                    .build()?,
            );

            // Create retry policy
            let retry_policy = RetryPolicy::builder()
                .max_attempts(3)
                .build();

            // Create orchestrator
            let orchestrator = PrimalOrchestrator::new(health_monitor.clone(), retry_policy);

            // Register security provider if specified
            if let Some(security_bin) = security_binary {
                info!("📦 Registering security provider: {}", security_bin);
                let security = create_security_provider(security_bin, security_port)?;
                orchestrator.register(security).await;
            }

            // Register discovery orchestrator if specified
            if let Some(discovery_bin) = discovery_binary {
                info!("📦 Registering discovery orchestrator: {}", discovery_bin);
                let discovery = create_discovery_orchestrator(discovery_bin)?;
                orchestrator.register(discovery).await;
            }

            // Register additional primals
            if let Some(additional_bins) = additional {
                for bin_path in additional_bins.split(',') {
                    let bin_path = bin_path.trim();
                    if !bin_path.is_empty() {
                        info!("📦 Registering additional primal: {}", bin_path);
                        // Auto-discover capabilities from environment
                        let primal = PrimalBuilder::new()
                            .binary_path(bin_path.to_string())
                            .provides(Capability::from_env("PRIMAL_PROVIDES"))
                            .requires(Capability::from_env("PRIMAL_REQUIRES"))
                            .build()?;
                        orchestrator.register(primal).await;
                    }
                }
            }

            // Start all with automatic dependency resolution!
            info!("🔄 Starting all primals with capability-based resolution...");
            orchestrator.start_all().await?;

            info!("✅ Tower started successfully!");
            info!("🌸 All primals running with zero-hardcoded configuration!");

            // Start health monitoring
            tokio::spawn(async move {
                health_monitor.start_monitoring().await;
            });

            // Wait for interrupt
            tokio::signal::ctrl_c().await?;
            info!("🛑 Received shutdown signal, stopping tower...");

            orchestrator.stop_all().await?;
            info!("✅ Tower stopped gracefully.");
        }

        Commands::StartFromEnv => {
            info!("🌱 Starting tower from PURE ENVIRONMENT (Infant Model)...");
            info!("📖 Reading configuration from:");
            info!("   - PRIMAL_PROVIDES (comma-separated capabilities)");
            info!("   - PRIMAL_REQUIRES (comma-separated capabilities)");
            info!("   - PRIMAL_BINARY (path to executable)");
            info!("   - HTTP_PORT (0 = auto-select)");

            // Create orchestrator
            let health_monitor = Arc::new(
                PrimalHealthMonitor::builder()
                    .health_checker(Arc::new(biomeos_core::HttpHealthChecker))
                    .build()?,
            );
            let retry_policy = RetryPolicy::builder().max_attempts(3).build();
            let orchestrator = PrimalOrchestrator::new(health_monitor.clone(), retry_policy);

            // Discover and register primal from environment
            match biomeos_core::GenericManagedPrimal::from_env() {
                Ok(primal) => {
                    info!("✅ Discovered primal from environment!");
                    orchestrator.register(Arc::new(primal)).await;

                    // Start it
                    orchestrator.start_all().await?;
                    info!("✅ Tower started from environment!");

                    // Start health monitoring
                    tokio::spawn(async move {
                        health_monitor.start_monitoring().await;
                    });

                    // Wait for interrupt
                    tokio::signal::ctrl_c().await?;
                    info!("🛑 Stopping tower...");
                    orchestrator.stop_all().await?;
                }
                Err(e) => {
                    error!("❌ Failed to discover primal from environment: {}", e);
                    error!("💡 Make sure these are set:");
                    error!("   export PRIMAL_PROVIDES=security,crypto");
                    error!("   export PRIMAL_BINARY=/path/to/binary");
                    error!("   export HTTP_PORT=9000  # or 0 for auto");
                    return Err(e.into());
                }
            }
        }

        Commands::Stop => {
            info!("🛑 Stopping all primals...");
            // TODO: Implement persistent orchestrator state
            error!("❌ Not implemented yet - need persistent orchestrator");
        }

        Commands::Status => {
            info!("📊 Primal Status:");
            // TODO: Implement persistent orchestrator state
            error!("❌ Not implemented yet - need persistent orchestrator");
        }

        Commands::Capabilities => {
            info!("📋 Available Capabilities:");
            info!("");
            info!("  🔐 Security     - Crypto, signing, encryption, key management");
            info!("  🔍 Discovery    - Service discovery, orchestration");
            info!("  💻 Compute      - Execution, processing, containers");
            info!("  🧠 AI           - ML inference, neural networks");
            info!("  💾 Storage      - Content-addressed, distributed storage");
            info!("  📊 Observability - Metrics, logging, tracing");
            info!("  🌐 Federation   - Multi-org coordination");
            info!("  🌍 Network      - NAT traversal, routing, mesh");
            info!("");
            info!("💡 Use PRIMAL_PROVIDES and PRIMAL_REQUIRES to declare capabilities!");
            info!("   Example: export PRIMAL_PROVIDES=security,crypto");
            info!("   Example: export PRIMAL_REQUIRES=storage");
        }
    }

    Ok(())
}
