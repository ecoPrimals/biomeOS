// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS Federation CLI
//!
//! Command-line interface for deploying and managing federation BYOB manifests
//! Pure Rust implementation for self-contained federation deployment

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};

mod modules;
use modules::{deploy_manifest, list_manifests, load_config, show_status, validate_config};

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("🏰 BiomeOS Federation CLI v1.0.0");

    // Load configuration
    let config_path = Some(cli.config.to_str().unwrap());
    let config = load_config(config_path).await?;

    // Validate configuration - convert to string for validation
    let config_str = serde_yaml::to_string(&config)?;
    let temp_file = std::env::temp_dir().join("temp_config.yaml");
    std::fs::write(&temp_file, config_str)?;
    validate_config(temp_file.to_str().unwrap()).await?;
    std::fs::remove_file(&temp_file)?;

    // Execute command
    match cli.command {
        Commands::Deploy {
            manifest,
            dry_run: _,
            force: _,
        } => deploy_manifest(&manifest).await,
        Commands::List { detailed: _ } => list_manifests().await,
        Commands::Status {
            deployment: _,
            watch: _,
        } => show_status().await,
        Commands::Validate => {
            info!("✅ Configuration validation passed");
            Ok(())
        }
    }
}
