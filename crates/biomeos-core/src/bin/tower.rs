//! Tower CLI - Capability-Based Primal Orchestration
//!
//! Modern, idiomatic, platform-agnostic primal management.
//! Supports config files, auto-discovery, and concurrent startup!

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use biomeos_core::{
    create_discovery_orchestrator, create_security_provider, discover_primals, start_in_waves,
    Capability, LogSessionTracker, PrimalBuilder, PrimalHealthMonitor, PrimalMetadata,
    PrimalOrchestrator, RetryPolicy, TowerConfig, TowerPrimalConfig,
};
use clap::{Parser, Subcommand};
use tracing::{error, info, warn};
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

    /// Scan directory and list discovered primals
    Discover {
        /// Directory to scan
        #[arg(default_value = "./primals")]
        directory: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            config,
            scan,
            concurrent,
        } => {
            info!("🚀 Starting tower with modern config-driven orchestration");

            // Load config
            let tower_config = if config.exists() {
                info!("📋 Loading configuration from: {}", config.display());
                TowerConfig::from_file(&config).context("Failed to load tower config")?
            } else {
                warn!("⚠️  Config file not found, using defaults");
                TowerConfig::default_config()
            };

            // Create health monitor
            let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());

            // Create retry policy
            let retry_policy = RetryPolicy::exponential(
                tower_config.health.recovery_attempts as usize,
                Duration::from_millis(100),
            );

            // Create orchestrator
            let orchestrator = Arc::new(PrimalOrchestrator::new(
                health_monitor.clone(),
                retry_policy,
            ));

            // Collect all primals
            let mut all_primals: Vec<Arc<dyn biomeos_core::ManagedPrimal>> = Vec::new();

            // Discover from scan directory if specified
            if let Some(scan_dir) = scan {
                info!("🔍 Auto-discovering primals from: {}", scan_dir.display());
                let discovered = discover_primals(&scan_dir).await?;
                info!("✅ Discovered {} primals", discovered.len());

                for metadata in discovered {
                    let primal = metadata_to_primal(metadata)?;
                    all_primals.push(primal);
                }
            }

            // Load primals from config
            for primal_config in &tower_config.primals {
                info!(
                    "📦 Loading primal from config: {}",
                    primal_config.binary.display()
                );
                let primal = config_to_primal(primal_config).await?;
                all_primals.push(primal);
            }

            if all_primals.is_empty() {
                error!("❌ No primals configured or discovered!");
                error!("💡 Either:");
                error!("   1. Add [[primals]] sections to {}", config.display());
                error!("   2. Use --scan ./primals to auto-discover");
                error!("   3. Ensure config file exists with primal definitions");
                return Ok(());
            }

            // Register all primals
            info!(
                "📋 Registering {} primals with orchestrator",
                all_primals.len()
            );
            for primal in &all_primals {
                orchestrator.register(primal.clone()).await;
            }

            // Start primals - concurrent or sequential
            if concurrent && tower_config.tower.concurrent_startup {
                info!("🌊 Starting primals with concurrent wave-based orchestration");
                start_in_waves(&orchestrator, all_primals.clone()).await?;
            } else {
                info!("🔄 Starting primals sequentially");
                orchestrator.start_all().await?;
            }

            info!("✅ Tower started successfully!");
            info!(
                "🌸 {} primals running with modern idiomatic Rust!",
                all_primals.len()
            );

            // Start health monitoring
            tokio::spawn(async move {
                if let Err(e) = health_monitor.start_monitoring().await {
                    error!("Health monitoring failed: {}", e);
                }
            });

            // Create log session tracker
            let node_id = std::env::var("NODE_ID")
                .or_else(|_| std::env::var("BEARDOG_NODE_ID"))
                .unwrap_or_else(|_| "unknown-node".to_string());
            let log_tracker = Arc::new(LogSessionTracker::new(node_id));

            // Wait for interrupt
            tokio::signal::ctrl_c().await?;
            info!("🛑 Received shutdown signal, stopping tower...");

            // Archive logs before stopping
            if let Err(e) = log_tracker.archive_all_sessions("graceful_shutdown").await {
                warn!("Failed to archive log sessions: {}", e);
            }

            orchestrator.stop_all().await?;
            info!("✅ Tower stopped gracefully.");
        }

        Commands::Start {
            security_binary,
            security_port,
            discovery_binary,
            additional,
        } => {
            info!("🚀 Starting tower with capability-based orchestration...");

            // Create health monitor
            let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());

            // Create retry policy
            let retry_policy = RetryPolicy::exponential(3, Duration::from_millis(100));

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
                if let Err(e) = health_monitor.start_monitoring().await {
                    error!("Health monitoring failed: {}", e);
                }
            });

            // Create log session tracker
            let node_id = std::env::var("NODE_ID")
                .or_else(|_| std::env::var("BEARDOG_NODE_ID"))
                .unwrap_or_else(|_| "unknown-node".to_string());
            let log_tracker = Arc::new(LogSessionTracker::new(node_id));

            // Wait for interrupt
            tokio::signal::ctrl_c().await?;
            info!("🛑 Received shutdown signal, stopping tower...");

            // Archive logs before stopping
            if let Err(e) = log_tracker.archive_all_sessions("graceful_shutdown").await {
                warn!("Failed to archive log sessions: {}", e);
            }

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
            let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());
            let retry_policy = RetryPolicy::exponential(3, Duration::from_millis(100));
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
                        if let Err(e) = health_monitor.start_monitoring().await {
                            error!("Health monitoring failed: {}", e);
                        }
                    });

                    // Create log session tracker
                    let node_id = std::env::var("NODE_ID")
                        .or_else(|_| std::env::var("BEARDOG_NODE_ID"))
                        .unwrap_or_else(|_| "unknown-node".to_string());
                    let log_tracker = Arc::new(LogSessionTracker::new(node_id));

                    // Wait for interrupt
                    tokio::signal::ctrl_c().await?;
                    info!("🛑 Stopping tower...");

                    // Archive logs before stopping
                    if let Err(e) = log_tracker.archive_all_sessions("graceful_shutdown").await {
                        warn!("Failed to archive log sessions: {}", e);
                    }

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
            error!("❌ Not implemented yet - requires persistent orchestrator state");
            error!("   Rationale: Stop command needs a shared state mechanism (e.g., Unix socket, Redis, or file-based PID tracking)");
            error!("   Workaround: Use Ctrl+C to stop the tower process, or kill individual primal PIDs");
            std::process::exit(1);
        }

        Commands::Status => {
            info!("📊 Primal Status:");
            error!("❌ Not implemented yet - requires persistent orchestrator state");
            error!("   Rationale: Status command needs to query the running orchestrator instance");
            error!("   Future: Implement via Unix socket or HTTP status endpoint on tower process");
            error!("   Workaround: Check process list (ps aux | grep primal) or use health monitor logs");
            std::process::exit(1);
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

        Commands::Discover { directory } => {
            info!("🔍 Scanning directory for primals: {}", directory.display());

            match discover_primals(&directory).await {
                Ok(primals) => {
                    info!("✅ Discovered {} primal(s)", primals.len());
                    info!("");

                    for (i, primal) in primals.iter().enumerate() {
                        info!("📦 Primal #{}", i + 1);
                        info!("   ID:       {}", primal.id);
                        info!("   Binary:   {}", primal.binary.display());
                        info!("   Provides: {:?}", primal.provides);
                        info!("   Requires: {:?}", primal.requires);
                        if let Some(version) = &primal.version {
                            info!("   Version:  {}", version);
                        }
                        if let Some(name) = &primal.name {
                            info!("   Name:     {}", name);
                        }
                        info!("");
                    }

                    info!("💡 To use these primals:");
                    info!("   tower run --scan {}", directory.display());
                }
                Err(e) => {
                    error!("❌ Failed to discover primals: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

// Helper: Convert PrimalMetadata to ManagedPrimal
fn metadata_to_primal(metadata: PrimalMetadata) -> Result<Arc<dyn biomeos_core::ManagedPrimal>> {
    use biomeos_core::PrimalBuilder;

    let provides: Vec<Capability> = metadata
        .provides
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let requires: Vec<Capability> = metadata
        .requires
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let primal = PrimalBuilder::new()
        .binary_path(metadata.binary.display().to_string())
        .provides(provides)
        .requires(requires)
        .build()?;

    Ok(primal)
}

// Helper: Convert TowerConfig PrimalConfig to ManagedPrimal
async fn config_to_primal(
    config: &TowerPrimalConfig,
) -> Result<Arc<dyn biomeos_core::ManagedPrimal>> {
    use biomeos_core::PrimalBuilder;

    // Auto-discover capabilities if enabled and not specified
    let (provides_str, requires_str) =
        if config.auto_discover && config.provides.is_empty() && config.requires.is_empty() {
            let id = config
                .id
                .clone()
                .or_else(|| {
                    config
                        .binary
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            info!("🔍 Auto-discovering capabilities for {}", id);
            match biomeos_core::query_primal_metadata(&config.binary).await {
                Ok(metadata) => (metadata.provides, metadata.requires),
                Err(e) => {
                    warn!("⚠️  Could not auto-discover capabilities: {}", e);
                    (config.provides.clone(), config.requires.clone())
                }
            }
        } else {
            (config.provides.clone(), config.requires.clone())
        };

    let provides: Vec<Capability> = provides_str
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    let requires: Vec<Capability> = requires_str
        .iter()
        .map(|s| Capability::Custom(s.clone()))
        .collect();

    // CRITICAL FIX: Pass environment variables from config to primal
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary.display().to_string())
        .provides(provides)
        .requires(requires);

    // Add all env vars from tower.toml [primals.env] section
    for (key, value) in &config.env {
        builder = builder.env_var(key.clone(), value.clone());
    }

    // Add protocol if specified (tarpc, jsonrpc, or auto-detect)
    if let Some(protocol) = &config.protocol {
        builder = builder.env_var("IPC_PROTOCOL".to_string(), protocol.clone());
    }

    // Add HTTP port if specified
    if config.http_port > 0 {
        builder = builder.http_port(config.http_port);
    }

    let primal = builder.build()?;

    Ok(primal)
}
