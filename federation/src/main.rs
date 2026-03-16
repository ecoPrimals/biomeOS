// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![forbid(unsafe_code)]

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
    run(Cli::parse()).await
}

async fn run(cli: Cli) -> Result<()> {
    // Initialize logging (try_init ignores if already set, e.g. in tests)
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let _ = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init();

    info!("🏰 BiomeOS Federation CLI v1.0.0");

    // Load configuration
    let config_path = Some(
        cli.config
            .to_str()
            .expect("config path must be valid UTF-8"),
    );
    let config = load_config(config_path).await?;

    let config_str = serde_yaml::to_string(&config)?;
    let temp_file = std::env::temp_dir().join(format!(
        "biome-federation-validate-{}-{:?}.yaml",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&temp_file, &config_str)?;
    let validate_result =
        validate_config(temp_file.to_str().expect("temp path must be valid UTF-8")).await;
    let _ = std::fs::remove_file(&temp_file);
    validate_result?;

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn valid_federation_config() -> String {
        r#"
federation:
  discovery:
    method: network_scan
    timeout: 30
  coordination:
    enabled: true
"#
        .to_string()
    }

    #[tokio::test]
    async fn test_run_list_command() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("federation.toml");
        std::fs::write(&config_path, valid_federation_config()).unwrap();

        let cli = Cli {
            command: Commands::List { detailed: false },
            verbose: false,
            config: config_path,
        };
        let result = run(cli).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_validate_command() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("federation.toml");
        std::fs::write(&config_path, valid_federation_config()).unwrap();

        let cli = Cli {
            command: Commands::Validate,
            verbose: false,
            config: config_path,
        };
        let result = run(cli).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_status_command() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("federation.toml");
        std::fs::write(&config_path, valid_federation_config()).unwrap();

        let cli = Cli {
            command: Commands::Status {
                deployment: None,
                watch: false,
            },
            verbose: false,
            config: config_path,
        };
        let result = run(cli).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_deploy_command() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("federation.toml");
        std::fs::write(&config_path, valid_federation_config()).unwrap();
        let manifest_path = dir.path().join("manifest.yaml");
        std::fs::write(
            &manifest_path,
            r#"
metadata:
  name: test-manifest
  version: "1.0.0"
"#,
        )
        .unwrap();

        let cli = Cli {
            command: Commands::Deploy {
                manifest: manifest_path.to_string_lossy().to_string(),
                dry_run: false,
                force: false,
            },
            verbose: false,
            config: config_path,
        };
        let result = run(cli).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fails_with_invalid_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("bad.toml");
        std::fs::write(&config_path, "other: value").unwrap();

        let cli = Cli {
            command: Commands::Validate,
            verbose: false,
            config: config_path,
        };
        let result = run(cli).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_fails_with_nonexistent_config() {
        let cli = Cli {
            command: Commands::List { detailed: false },
            verbose: false,
            config: PathBuf::from("/nonexistent/federation.toml"),
        };
        let result = run(cli).await;
        assert!(result.is_err());
    }
}
