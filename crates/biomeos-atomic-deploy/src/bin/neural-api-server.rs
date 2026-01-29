//! Neural API Server Binary
//!
//! Starts the Neural API JSON-RPC server for graph execution.
//!
//! Usage:
//!   neural-api-server [OPTIONS]
//!
//! Options:
//!   --graphs-dir <PATH>    Directory containing graph TOML files (default: ./graphs)
//!   --family-id <ID>       Family ID for this instance (auto-discovered from .family.seed or env)
//!   --socket <PATH>        Unix socket path (default: /run/user/{uid}/biomeos/neural-api-{family}.sock)

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use biomeos_core::family_discovery;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
        )
        .init();

    // Parse command line args
    let graphs_dir = std::env::args()
        .position(|arg| arg == "--graphs-dir")
        .and_then(|i| std::env::args().nth(i + 1))
        .unwrap_or_else(|| "graphs".to_string());

    // Family ID: CLI arg > env var > .family.seed file > "default"
    // NOTE: Removed "nat0" hardcoding - use proper Dark Forest family discovery
    let family_id = std::env::args()
        .position(|arg| arg == "--family-id")
        .and_then(|i| std::env::args().nth(i + 1))
        .unwrap_or_else(|| family_discovery::get_family_id());

    let socket_path = std::env::args()
        .position(|arg| arg == "--socket")
        .and_then(|i| std::env::args().nth(i + 1))
        .unwrap_or_else(|| format!("/tmp/neural-api-{}.sock", family_id));

    info!("╔══════════════════════════════════════════════════════════════════════════╗");
    info!("║                                                                          ║");
    info!("║                  🧠 Neural API Server Starting 🧠                        ║");
    info!("║                                                                          ║");
    info!("╚══════════════════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Configuration:");
    info!("  Graphs Directory: {}", graphs_dir);
    info!("  Family ID: {}", family_id);
    info!("  Socket Path: {}", socket_path);
    info!("");

    // Create Neural API server
    let server = NeuralApiServer::new(
        PathBuf::from(&graphs_dir),
        family_id,
        PathBuf::from(&socket_path),
    );

    // Start server
    info!("🚀 Starting Neural API server...");
    server.serve().await.context("Neural API server failed")?;

    Ok(())
}
