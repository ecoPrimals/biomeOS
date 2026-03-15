// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! LiveSpore Deployment Tool
//!
//! Deploy NUCLEUS to USB drive using Neural API graphs

use anyhow::{Context, Result};
use biomeos_spore::{DeploymentMetrics, NeuralSpore};
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(
    name = "livespore-deploy",
    about = "Deploy Neural LiveSpore to USB drive",
    version
)]
struct Args {
    /// USB mount point (e.g., /media/eastgate/biomeOS1)
    #[clap(short, long)]
    usb: PathBuf,

    /// Source graphs directory
    #[clap(short, long, default_value = "graphs")]
    graphs: PathBuf,

    /// Source binaries directory
    #[clap(short, long, default_value = "plasmidBin/primals")]
    binaries: PathBuf,

    /// Nucleus binary path
    #[clap(short, long, default_value = "target/release/nucleus")]
    nucleus: PathBuf,

    /// Skip existing structure (update only)
    #[clap(long)]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    run(Args::parse()).await
}

async fn run(args: Args) -> Result<()> {
    // Initialize logging (try_init ignores if already set, e.g. in tests)
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,biomeos_spore=debug")
        .with_target(false)
        .try_init();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🌱 Neural LiveSpore Deployment");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    info!("USB Target: {}", args.usb.display());
    info!("Graphs: {}", args.graphs.display());
    info!("Binaries: {}", args.binaries.display());
    info!("Nucleus: {}", args.nucleus.display());
    println!();

    // Verify USB is mounted
    if !args.usb.exists() {
        anyhow::bail!("USB mount point does not exist: {}", args.usb.display());
    }

    // Create Neural Spore
    let spore = NeuralSpore::new(args.usb.clone()).context("Failed to create Neural Spore")?;

    // Prepare directory structure
    if !args.update {
        spore
            .prepare()
            .await
            .context("Failed to prepare LiveSpore structure")?;
    } else {
        info!("🔄 Update mode: skipping directory creation");
    }

    // Install graphs
    spore
        .install_graphs(&args.graphs)
        .await
        .context("Failed to install Neural API graphs")?;

    // Install primal binaries
    spore
        .install_binaries(&args.binaries)
        .await
        .context("Failed to install primal binaries")?;

    // Install nucleus orchestrator
    spore
        .install_nucleus(&args.nucleus)
        .await
        .context("Failed to install nucleus binary")?;

    // Create README
    spore
        .create_readme()
        .await
        .context("Failed to create README")?;

    // Create deployment metrics (empty for now)
    let metrics = DeploymentMetrics {
        total_duration_ms: 0,
        primals_deployed: 0,
        primals_failed: 0,
        phase_metrics: Vec::new(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    spore
        .save_metrics(&metrics)
        .await
        .context("Failed to save metrics")?;

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Neural LiveSpore Deployment Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    info!("LiveSpore root: {}", spore.root_path.display());
    info!("Graphs: {}", spore.graphs_dir.display());
    info!("Binaries: {}", spore.binaries_dir.display());
    println!();
    info!("🚀 Next Steps:");
    info!("  1. cd {}", spore.root_path.display());
    info!("  2. ./primals/nucleus deploy --family nat0 --graph graphs/nucleus_simple.toml");
    info!("  3. ./primals/nucleus status");
    println!();

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_fails_when_usb_does_not_exist() {
        let args = Args {
            usb: PathBuf::from("/nonexistent/usb/mount"),
            graphs: PathBuf::from("graphs"),
            binaries: PathBuf::from("plasmidBin/primals"),
            nucleus: PathBuf::from("target/release/nucleus"),
            update: false,
        };
        let result = run(args).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("USB mount point does not exist"));
    }

    #[tokio::test]
    async fn test_run_succeeds_with_temp_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let usb = dir.path().to_path_buf();
        let graphs_dir = dir.path().join("graphs");
        let binaries_dir = dir.path().join("binaries");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        std::fs::create_dir_all(&binaries_dir).unwrap();

        let nucleus_file = dir.path().join("nucleus_bin");
        std::fs::write(&nucleus_file, b"#!/bin/sh\nexit 0").unwrap();

        let args = Args {
            usb,
            graphs: graphs_dir.clone(),
            binaries: binaries_dir.clone(),
            nucleus: nucleus_file,
            update: false,
        };
        let result = run(args).await;
        assert!(result.is_ok(), "run should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_update_mode_skips_prepare() {
        let dir = tempfile::tempdir().unwrap();
        let usb = dir.path().to_path_buf();
        let graphs_dir = dir.path().join("graphs");
        let binaries_dir = dir.path().join("binaries");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        std::fs::create_dir_all(&binaries_dir).unwrap();

        // Update mode skips prepare(), so we must create the spore structure manually
        // (biomeOS/graphs, biomeOS/primals, biomeOS/metrics, biomeOS/logs)
        let spore_root = dir.path().join("biomeOS");
        std::fs::create_dir_all(spore_root.join("graphs")).unwrap();
        std::fs::create_dir_all(spore_root.join("primals")).unwrap();
        std::fs::create_dir_all(spore_root.join("metrics")).unwrap();
        std::fs::create_dir_all(spore_root.join("logs")).unwrap();

        let nucleus_file = dir.path().join("nucleus_bin");
        std::fs::write(&nucleus_file, b"#!/bin/sh\nexit 0").unwrap();

        let args = Args {
            usb,
            graphs: graphs_dir,
            binaries: binaries_dir,
            nucleus: nucleus_file,
            update: true,
        };
        let result = run(args).await;
        assert!(
            result.is_ok(),
            "run with update mode should succeed: {:?}",
            result.err()
        );
    }
}
