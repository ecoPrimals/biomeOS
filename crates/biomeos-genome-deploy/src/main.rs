// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! genomeBin Deployment CLI
//!
//! Command-line interface for deploying genomeBins

#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use genome_deploy::GenomeDeployer;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "genome-deploy")]
#[command(about = "Universal genomeBin deployment tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy a genomeBin
    Deploy {
        /// Path to genomeBin file
        #[arg(value_name = "GENOME")]
        genome_path: PathBuf,

        /// Custom installation directory
        #[arg(short, long, value_name = "DIR")]
        install_dir: Option<PathBuf>,

        /// Skip verification
        #[arg(short, long)]
        skip_verify: bool,
    },

    /// Show system information
    Info,

    /// Validate genomeBin format
    Validate {
        /// Path to genomeBin file
        #[arg(value_name = "GENOME")]
        genome_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy {
            genome_path,
            install_dir,
            skip_verify: _,
        } => {
            let mut deployer = GenomeDeployer::new(&genome_path)?;

            if let Some(dir) = install_dir {
                deployer = deployer.with_install_dir(dir);
            }

            deployer.deploy()?;
        }

        Commands::Info => {
            use genome_deploy::{Architecture, Platform};

            println!("System Information:");
            println!("  Architecture: {}", Architecture::detect()?.as_str());
            println!("  Platform: {}", Platform::detect()?.name());
            println!(
                "  Supports abstract sockets: {}",
                Platform::detect()?.supports_abstract_sockets()
            );
        }

        Commands::Validate { genome_path } => {
            use std::fs::File;
            use std::io::Read;

            println!("Validating genomeBin: {}", genome_path.display());

            let mut file = File::open(&genome_path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            // Check for archive marker
            let marker = b"__ARCHIVE_START__";
            if contents
                .windows(marker.len())
                .any(|window| window == marker)
            {
                println!("✓ Archive marker found");
            } else {
                anyhow::bail!("✗ Archive marker not found");
            }

            println!("✓ genomeBin format valid");
        }
    }

    Ok(())
}
