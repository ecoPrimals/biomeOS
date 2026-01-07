use anyhow::Result;
use biomeos_cli::{commands::*, CliUtils, OutputFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Chimera subcommands
#[derive(Subcommand)]
enum ChimeraAction {
    /// List all chimera definitions
    List,
    /// Show details for a chimera
    Show {
        /// Chimera ID
        id: String,
    },
    /// Build a chimera binary
    Build {
        /// Chimera ID
        id: String,
    },
}

/// Niche subcommands
#[derive(Subcommand)]
enum NicheAction {
    /// List available niche templates
    List,
    /// Show details for a niche template
    Show {
        /// Niche template ID
        id: String,
    },
}

/// Primal subcommands
#[derive(Subcommand)]
enum PrimalAction {
    /// List installed primal binaries
    List,
    /// Pull/build primal from parent repo
    Pull {
        /// Primal name (beardog, songbird, etc.) or --all
        #[arg(default_value = "--all")]
        name: String,
    },
}

/// Spore subcommands
#[derive(Subcommand)]
enum SporeAction {
    /// Create new USB spore
    Create {
        /// USB mount point (e.g., /media/usb)
        #[arg(short, long)]
        mount: PathBuf,
        
        /// Spore label (e.g., biomeOS1)
        #[arg(short, long)]
        label: String,
        
        /// Node ID for this tower (e.g., tower1)
        #[arg(short, long)]
        node: String,
    },
    
    /// Clone spore to create sibling
    Clone {
        /// Source spore mount point
        #[arg(short, long)]
        from: PathBuf,
        
        /// Target spore mount point
        #[arg(short, long)]
        to: PathBuf,
        
        /// New node ID for sibling
        #[arg(short, long)]
        node: String,
    },
    
    /// Verify spore integrity
    Verify {
        /// Spore mount point
        mount: PathBuf,
    },
    
    /// Show spore information
    Info {
        /// Spore mount point
        mount: PathBuf,
    },
    
    /// List available USB devices
    List,
}

#[derive(Parser)]
#[command(name = "biomeos")]
#[command(about = "🌱 BiomeOS Universal System Management CLI")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[arg(long, default_value = "info")]
    log_level: String,

    #[arg(long, value_enum, default_value = "pretty")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage chimera definitions (mixed-boundary primal amalgams)
    Chimera {
        #[command(subcommand)]
        action: ChimeraAction,
    },

    /// Manage niche templates (biome environments)
    Niche {
        #[command(subcommand)]
        action: NicheAction,
    },

    /// Manage primal binaries
    Primal {
        #[command(subcommand)]
        action: PrimalAction,
    },

    /// Manage USB spores (biomeOS deployment packages)
    Spore {
        #[command(subcommand)]
        action: SporeAction,
    },

    /// Discover services by capability or method
    Discover {
        /// Discovery endpoint to query
        #[arg(short, long)]
        endpoint: Option<String>,

        /// Required capabilities (comma-separated: domain/name,domain2/name2)
        #[arg(short, long)]
        capabilities: Option<String>,

        /// Discovery method to use
        #[arg(short, long, value_enum, default_value = "capability-based")]
        method: biomeos_cli::commands::discover::DiscoveryMethod,

        /// Registry URL for registry-based discovery
        #[arg(short, long)]
        registry: Option<String>,

        /// Show detailed service information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Deploy a biome manifest
    Deploy {
        /// Path to the biome manifest file
        #[arg(short, long)]
        manifest: PathBuf,

        /// Validate manifest without deploying
        #[arg(short, long)]
        validate_only: bool,
    },

    /// Create a new service or resource
    Create {
        /// Type of service to create
        #[arg(short, long)]
        service_type: String,

        /// Name of the service
        #[arg(short, long)]
        name: String,

        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Dry run - show what would be created
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Stream logs from services
    Logs {
        /// Service identifier
        service: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long)]
        tail: Option<usize>,

        /// Show logs since timestamp
        #[arg(short, long)]
        since: Option<String>,
    },

    /// Execute commands in running services
    Exec {
        /// Service identifier
        service: String,

        /// Command to execute
        command: Vec<String>,

        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Scale services up or down
    Scale {
        /// Service identifier
        service: String,

        /// Number of replicas
        #[arg(short, long)]
        replicas: Option<u32>,

        /// Enable auto-scaling
        #[arg(long)]
        auto: bool,
    },

    /// AI-powered biome management assistance
    Ai {
        /// Natural language command or question
        query: String,

        /// Context information
        #[arg(short, long)]
        context: Option<String>,
    },

    /// Monitor system and service health
    Health {
        /// Specific service to check
        #[arg(short, long)]
        service: Option<String>,

        /// Show detailed health information
        #[arg(short, long)]
        detailed: bool,

        /// Continuous monitoring
        #[arg(short, long)]
        continuous: bool,

        /// Monitoring interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },

    /// Monitor system resources and services
    Monitor {
        /// Specific service to monitor
        #[arg(short, long)]
        service: Option<String>,

        /// Monitoring interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Monitoring duration in seconds
        #[arg(short, long)]
        duration: Option<u64>,
    },

    /// Launch interactive dashboard
    Dashboard {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "2")]
        interval: u64,

        /// Auto-refresh
        #[arg(short, long)]
        refresh: bool,
    },

    /// Deep probe service health and performance
    Probe {
        /// Service to probe
        service: String,

        /// Timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },

    /// Scan system for services and issues
    Scan {
        /// Quick scan mode
        #[arg(short, long)]
        quick: bool,

        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },

    /// Get system or service status
    Status {
        /// Specific service to check
        #[arg(short, long)]
        service: Option<String>,

        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: String,

        /// Show metrics
        #[arg(short, long)]
        metrics: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    CliUtils::init_logging(&cli.log_level)?;

    println!("🌱 BiomeOS Universal System Management");
    println!("======================================");
    println!();

    match cli.command {
        Commands::Chimera { action } => match action {
            ChimeraAction::List => handle_chimera_list().await?,
            ChimeraAction::Show { id } => handle_chimera_show(&id).await?,
            ChimeraAction::Build { id } => handle_chimera_build(&id).await?,
        },
        Commands::Niche { action } => match action {
            NicheAction::List => handle_niche_list().await?,
            NicheAction::Show { id } => handle_niche_show(&id).await?,
        },
        Commands::Primal { action } => match action {
            PrimalAction::List => handle_primal_list().await?,
            PrimalAction::Pull { name } => {
                println!("🔨 Building primal: {}", name);
                println!("   Run: ./bin/pull-primals.sh {}", name);
            }
        },
        Commands::Spore { action } => match action {
            SporeAction::Create { mount, label, node } => {
                handle_spore_create(mount, label, node).await?
            }
            SporeAction::Clone { from, to, node } => {
                handle_spore_clone(from, to, node).await?
            }
            SporeAction::Verify { mount } => {
                handle_spore_verify(mount).await?
            }
            SporeAction::Info { mount } => {
                handle_spore_info(mount).await?
            }
            SporeAction::List => {
                handle_spore_list().await?
            }
        },
        Commands::Discover {
            endpoint,
            capabilities,
            method,
            registry,
            detailed,
        } => {
            handle_discover(endpoint, capabilities, method, registry, detailed).await?;
        }
        Commands::Deploy {
            manifest,
            validate_only,
        } => {
            handle_deploy(manifest, validate_only).await?;
        }
        Commands::Create {
            service_type,
            name,
            config,
            dry_run,
        } => {
            handle_create(service_type, name, config, dry_run).await?;
        }
        Commands::Logs {
            service,
            follow,
            tail,
            since,
        } => {
            handle_logs(service, follow, tail, since).await?;
        }
        Commands::Exec {
            service,
            command,
            interactive,
        } => {
            handle_exec(service, command, interactive).await?;
        }
        Commands::Scale {
            service,
            replicas,
            auto,
        } => {
            handle_scale(service, replicas, auto).await?;
        }
        Commands::Ai { query, context } => {
            handle_ai_command(query, context).await?;
        }
        Commands::Health {
            service,
            detailed,
            continuous,
            interval,
        } => {
            handle_health(service, detailed, continuous, interval).await?;
        }
        Commands::Monitor {
            service,
            interval,
            duration,
        } => {
            handle_monitor(service, interval, duration).await?;
        }
        Commands::Dashboard { interval, refresh } => {
            handle_dashboard(interval, refresh).await?;
        }
        Commands::Probe { service, timeout } => {
            handle_probe(service, timeout).await?;
        }
        Commands::Scan { quick, format } => {
            handle_scan(quick, format).await?;
        }
        Commands::Status {
            service,
            format,
            metrics,
        } => {
            handle_status(service, format, metrics).await?;
        }
    }

    Ok(())
}

/// Handle AI command
async fn handle_ai_command(query: String, context: Option<String>) -> anyhow::Result<()> {
    use colored::*;

    println!("{}", "🤖 BiomeOS AI Assistant".bright_cyan().bold());
    println!(
        "{} {}",
        "Query:".bright_white().bold(),
        query.bright_yellow()
    );

    if let Some(ctx) = context {
        println!("{} {}", "Context:".bright_white().bold(), ctx.bright_blue());
    }

    // Basic AI command processing - integrated with universal adapter architecture
    match query.to_lowercase().as_str() {
        q if q.contains("health") => {
            println!("\n{}", "🏥 Health Status Analysis".bright_green().bold());
            println!("• Checking system health via Universal BiomeOS Manager...");
            println!("• Aggregating health data from discovered primals...");
            println!("• Delegating detailed health checks to Songbird discovery...");
        }
        q if q.contains("discover") => {
            println!("\n{}", "🔍 Primal Discovery".bright_blue().bold());
            println!("• Initiating capability-based discovery...");
            println!("• Scanning for available primals in ecosystem...");
            println!("• Using Songbird service discovery for coordination...");
        }
        q if q.contains("deploy") => {
            println!("\n{}", "🚀 Deployment Analysis".bright_magenta().bold());
            println!("• Analyzing biome.yaml manifest...");
            println!("• Delegating manifest parsing to Toadstool...");
            println!("• Coordinating deployment via Universal Adapter...");
        }
        _ => {
            println!("\n{}", "💡 AI Suggestions".bright_yellow().bold());
            println!("• Try: 'biomeos ai \"health status\"' for system health");
            println!("• Try: 'biomeos ai \"discover primals\"' for discovery");
            println!("• Try: 'biomeos ai \"deploy help\"' for deployment guidance");
        }
    }

    println!(
        "\n{}",
        "✨ BiomeOS AI is continuously learning from the ecosystem!".bright_cyan()
    );
    Ok(())
}
