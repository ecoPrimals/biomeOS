//! Neural API mode - Graph-based orchestration server
//!
//! Starts the Neural API JSON-RPC server for graph execution

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use std::path::PathBuf;
use tracing::info;

/// Resolve socket path: use explicit path or default /tmp/neural-api-{family_id}.sock
pub(crate) fn resolve_socket_path(socket: Option<PathBuf>, family_id: &str) -> PathBuf {
    socket.unwrap_or_else(|| PathBuf::from(format!("/tmp/neural-api-{}.sock", family_id)))
}

pub async fn run(graphs_dir: PathBuf, family_id: String, socket: Option<PathBuf>) -> Result<()> {
    let socket_path = resolve_socket_path(socket, &family_id);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_socket_path_default() {
        let path = resolve_socket_path(None, "my-family");
        assert_eq!(path, PathBuf::from("/tmp/neural-api-my-family.sock"));
    }

    #[test]
    fn test_resolve_socket_path_explicit() {
        let custom = PathBuf::from("/var/run/neural.sock");
        let path = resolve_socket_path(Some(custom.clone()), "any-family");
        assert_eq!(path, custom);
    }

    #[test]
    fn test_resolve_socket_path_family_id_in_default() {
        let path = resolve_socket_path(None, "family-with-special_chars");
        assert!(path.to_string_lossy().contains("family-with-special_chars"));
        assert!(path.to_string_lossy().ends_with(".sock"));
    }
}
