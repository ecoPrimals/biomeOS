//! BiomeOS Federation CLI
//!
//! Command-line interface for deploying and managing federation BYOB manifests
//! Pure Rust implementation for self-contained federation deployment

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};

mod modules;
use modules::{load_config, validate_config, deploy_manifest, list_manifests, show_status};

#[derive(Parser)]
#[command(name = "biome-federation")]
#[command(about = "BiomeOS Federation CLI for basement tower deployment")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file
    #[arg(short, long, default_value = "/etc/ecoprimal/federation.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy federation manifests
    Deploy {
        /// Manifest file or template name
        manifest: String,

        /// Dry run (validate only)
        #[arg(long)]
        dry_run: bool,

        /// Force deployment
        #[arg(long)]
        force: bool,
    },

    /// List available manifests
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Show deployment or federation status
    Status {
        /// Specific deployment to check
        deployment: Option<String>,

        /// Watch for changes
        #[arg(short, long)]
        watch: bool,
    },

    /// Validate configuration
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("🏰 BiomeOS Federation CLI v1.0.0");

    // Load configuration
    let config = load_config(&cli.config)?;

    // Validate configuration
    validate_config(&config)?;

    // Execute command
    match cli.command {
        Commands::Deploy { manifest, dry_run, force } => {
            deploy_manifest(&config, &manifest, dry_run, force)
        }
        Commands::List { detailed } => {
            list_manifests(&config, detailed)
        }
        Commands::Status { deployment, watch } => {
            show_status(&config, deployment, watch)
        }
        Commands::Validate => {
            info!("✅ Configuration validation passed");
            Ok(())
        }
    }
} 