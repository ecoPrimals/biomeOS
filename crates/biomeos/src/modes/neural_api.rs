//! Neural API mode - Graph-based orchestration server
//!
//! Starts the Neural API JSON-RPC server for graph execution

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use std::path::PathBuf;
use tracing::info;

pub async fn run(graphs_dir: PathBuf, family_id: String, socket: Option<PathBuf>) -> Result<()> {
    let socket_path =
        socket.unwrap_or_else(|| PathBuf::from(format!("/tmp/neural-api-{}.sock", family_id)));

    info!("╔══════════════════════════════════════════════════════════════════════════╗");
    info!("║                                                                          ║");
    info!("║                  🧠 Neural API Server Starting 🧠                        ║");
    info!("║                                                                          ║");
    info!("╚══════════════════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Configuration:");
    info!("  Graphs Directory: {}", graphs_dir.display());
    info!("  Family ID: {}", family_id);
    info!("  Socket Path: {}", socket_path.display());
    info!("");

    // Create Neural API server
    let server = NeuralApiServer::new(graphs_dir, family_id, socket_path);

    // Start server
    info!("🚀 Starting Neural API server...");
    server.serve().await.context("Neural API server failed")
}
