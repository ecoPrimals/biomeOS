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
    tracing_subscriber::fmt()
        .with_env_filter("info,biomeos_spore=debug")
        .with_target(false)
        .init();

    let args = Args::parse();

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
