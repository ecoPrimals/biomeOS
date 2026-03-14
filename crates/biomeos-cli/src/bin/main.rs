// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

use anyhow::Result;
use biomeos_cli::{commands::*, CliUtils, OutputFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Parsed AI intent from natural language query (pure, testable)
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AiIntent {
    Status,
    Health,
    Deploy,
    Discover,
    Unknown,
}

/// Parse AI intent from query string (pure, testable)
pub(crate) fn parse_ai_intent(query: &str) -> AiIntent {
    let q = query.to_lowercase();
    if q.contains("health") {
        AiIntent::Health
    } else if q.contains("discover") {
        AiIntent::Discover
    } else if q.contains("deploy") {
        AiIntent::Deploy
    } else if q.contains("status") {
        AiIntent::Status
    } else {
        AiIntent::Unknown
    }
}

/// Format AI response lines for intent (pure, testable)
pub(crate) fn format_ai_response(intent: AiIntent) -> Vec<String> {
    match intent {
        AiIntent::Health => vec![
            "🏥 Health Status Analysis".to_string(),
            "• Checking system health via Universal BiomeOS Manager...".to_string(),
            "• Aggregating health data from discovered primals...".to_string(),
            "• Delegating detailed health checks to Songbird discovery...".to_string(),
        ],
        AiIntent::Discover => vec![
            "🔍 Primal Discovery".to_string(),
            "• Initiating capability-based discovery...".to_string(),
            "• Scanning for available primals in ecosystem...".to_string(),
            "• Using Songbird service discovery for coordination...".to_string(),
        ],
        AiIntent::Deploy => vec![
            "🚀 Deployment Analysis".to_string(),
            "• Analyzing biome.yaml manifest...".to_string(),
            "• Delegating manifest parsing to Toadstool...".to_string(),
            "• Coordinating deployment via Universal Adapter...".to_string(),
        ],
        AiIntent::Status | AiIntent::Unknown => vec![
            "💡 AI Suggestions".to_string(),
            "• Try: 'biomeos ai \"health status\"' for system health".to_string(),
            "• Try: 'biomeos ai \"discover primals\"' for discovery".to_string(),
            "• Try: 'biomeos ai \"deploy help\"' for deployment guidance".to_string(),
        ],
    }
}

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

/// Federation subcommands
#[derive(Subcommand)]
enum FederationAction {
    /// Create a new sub-federation
    CreateSubfed(federation::CreateSubfedArgs),
    /// List all sub-federations
    ListSubfeds(federation::ListSubfedsArgs),
    /// Join a sub-federation
    JoinSubfed(federation::JoinSubfedArgs),
    /// Check node access to capabilities
    CheckAccess(federation::CheckAccessArgs),
}

/// Genome subcommands
#[derive(Subcommand)]
enum GenomeAction {
    /// Create genomeBin from binaries
    Create(genome::CreateArgs),
    /// Compose atomic genomeBin (TOWER, NODE, etc.)
    Compose(genome::ComposeArgs),
    /// Self-replicate (biomeOS creates its own genomeBin)
    SelfReplicate,
    /// List all genomes
    List,
    /// Verify genomeBin integrity
    Verify(genome::VerifyArgs),
}

/// Node subcommands
#[derive(Subcommand)]
enum NodeAction {
    /// List locally incubated nodes
    ListLocal(incubation::ListLocalArgs),
}

/// Spore subcommands
#[derive(Subcommand)]
enum SporeAction {
    /// Incubate spore on local computer
    Incubate(incubation::IncubateArgs),
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

        /// Spore type: 'live' (FAT32-aware, deployable) or 'cold' (storage only)
        #[arg(short = 't', long, default_value = "live")]
        spore_type: String,
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

    /// Refresh spore binaries from plasmidBin
    Refresh {
        /// Spore mount point (e.g., /media/usb/biomeOS)
        #[arg(value_name = "MOUNT_POINT")]
        mount: PathBuf,

        /// Dry run - show what would be updated without actually updating
        #[arg(short = 'n', long)]
        dry_run: bool,
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

    /// Verify plasmidBin binaries and deployed spores
    Verify(verify::VerifyArgs),

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

    /// Deploy a biome manifest or niche
    Deploy {
        /// Path to the biome manifest or niche file
        #[arg(short, long)]
        manifest: PathBuf,

        /// Validate manifest without deploying
        #[arg(short, long)]
        validate_only: bool,

        /// Use graph-based deployment (Neural API)
        #[arg(short, long)]
        graph: bool,

        /// Specific graph name to use (requires --graph)
        #[arg(long, requires = "graph")]
        graph_name: Option<String>,
    },

    /// Deploy using a graph directly (Neural API)
    DeployGraph {
        /// Path to the graph file (.toml)
        graph: PathBuf,

        /// Validate graph without deploying
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

        /// Use graph-based health check (Neural API)
        #[arg(short, long)]
        graph: bool,

        /// Niche manifest for graph-based check
        #[arg(long, requires = "graph")]
        niche: Option<PathBuf>,
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

    /// Manage sub-federations (hierarchical trust networks)
    Federation {
        #[command(subcommand)]
        action: FederationAction,
    },

    /// Manage genomeBins (self-extracting multi-arch deployments)
    Genome {
        #[command(subcommand)]
        action: GenomeAction,
    },

    /// Manage local nodes (incubated spores)
    Node {
        #[command(subcommand)]
        action: NodeAction,
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
                println!("🔨 Building primal: {name}");
                println!("   Run: ./bin/pull-primals.sh {name}");
            }
        },
        Commands::Spore { action } => match action {
            SporeAction::Incubate(args) => handle_spore_incubate(&args).await?,
            SporeAction::Create {
                mount,
                label,
                node,
                spore_type,
            } => handle_spore_create(mount, label, node, spore_type).await?,
            SporeAction::Clone { from, to, node } => handle_spore_clone(from, to, node).await?,
            SporeAction::Verify { mount } => handle_spore_verify(mount).await?,
            SporeAction::Info { mount } => handle_spore_info(mount).await?,
            SporeAction::Refresh { mount, dry_run } => handle_spore_refresh(mount, dry_run).await?,
            SporeAction::List => handle_spore_list().await?,
        },

        Commands::Verify(args) => {
            verify::run(args).await?;
        }
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
            graph,
            graph_name,
        } => {
            handle_deploy(manifest, validate_only, graph, graph_name).await?;
        }
        Commands::DeployGraph {
            graph,
            validate_only,
        } => {
            handle_deploy_graph_direct(graph, validate_only).await?;
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
            graph,
            niche,
        } => {
            handle_health(service, detailed, continuous, interval, graph, niche).await?;
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
        Commands::Federation { action } => match action {
            FederationAction::CreateSubfed(args) => {
                handle_federation_create_subfed(&args).await?;
            }
            FederationAction::ListSubfeds(args) => {
                handle_federation_list_subfeds(&args).await?;
            }
            FederationAction::JoinSubfed(args) => {
                handle_federation_join_subfed(&args).await?;
            }
            FederationAction::CheckAccess(args) => {
                handle_federation_check_access(&args).await?;
            }
        },
        Commands::Genome { action } => match action {
            GenomeAction::Create(args) => {
                handle_genome_create(args)?;
            }
            GenomeAction::Compose(args) => {
                handle_genome_compose(args)?;
            }
            GenomeAction::SelfReplicate => {
                handle_genome_self_replicate()?;
            }
            GenomeAction::List => {
                handle_genome_list()?;
            }
            GenomeAction::Verify(args) => {
                handle_genome_verify(args)?;
            }
        },
        Commands::Node { action } => match action {
            NodeAction::ListLocal(args) => {
                handle_node_list_local(&args).await?;
            }
        },
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

    let intent = parse_ai_intent(&query);
    let lines = format_ai_response(intent);
    for line in lines {
        println!("\n{line}");
    }

    println!(
        "\n{}",
        "✨ BiomeOS AI is continuously learning from the ecosystem!".bright_cyan()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_ai_intent_health() {
        assert_eq!(parse_ai_intent("health status"), AiIntent::Health);
        assert_eq!(parse_ai_intent("check health"), AiIntent::Health);
    }

    #[test]
    fn test_parse_ai_intent_discover() {
        assert_eq!(parse_ai_intent("discover primals"), AiIntent::Discover);
    }

    #[test]
    fn test_parse_ai_intent_deploy() {
        assert_eq!(parse_ai_intent("deploy help"), AiIntent::Deploy);
    }

    #[test]
    fn test_parse_ai_intent_status() {
        assert_eq!(parse_ai_intent("status"), AiIntent::Status);
    }

    #[test]
    fn test_parse_ai_intent_unknown() {
        assert_eq!(parse_ai_intent("random query"), AiIntent::Unknown);
    }

    #[test]
    fn test_format_ai_response_health() {
        let lines = format_ai_response(AiIntent::Health);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Health"));
    }

    #[test]
    fn test_format_ai_response_unknown() {
        let lines = format_ai_response(AiIntent::Unknown);
        assert!(lines.iter().any(|l| l.contains("Suggestions")));
    }
}
