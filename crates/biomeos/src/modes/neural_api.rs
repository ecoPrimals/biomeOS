// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API mode - Graph-based orchestration server
//!
//! Starts the Neural API JSON-RPC server for graph execution

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use std::path::PathBuf;
use tracing::info;

/// Resolved Neural API configuration (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct NeuralApiConfig {
    pub graphs_dir: PathBuf,
    pub family_id: String,
    pub socket_path: PathBuf,
}

/// Resolve Neural API configuration from CLI/env inputs.
pub(crate) fn resolve_neural_api_config(
    graphs_dir: PathBuf,
    socket: Option<PathBuf>,
    family_id: Option<&str>,
) -> NeuralApiConfig {
    let family_id = family_id
        .map(String::from)
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path =
        socket.unwrap_or_else(|| PathBuf::from(format!("/tmp/neural-api-{}.sock", family_id)));
    NeuralApiConfig {
        graphs_dir,
        family_id,
        socket_path,
    }
}

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
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_resolve_neural_api_config() {
        let config = resolve_neural_api_config(PathBuf::from("graphs"), None, Some("my-family"));
        assert_eq!(config.family_id, "my-family");
        assert_eq!(config.graphs_dir, PathBuf::from("graphs"));
        assert!(config.socket_path.to_string_lossy().contains("my-family"));
    }

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

    #[test]
    fn test_resolve_socket_path_empty_family_id() {
        let path = resolve_socket_path(None, "");
        assert_eq!(path, PathBuf::from("/tmp/neural-api-.sock"));
    }

    #[test]
    fn test_resolve_socket_path_explicit_with_components() {
        let custom = PathBuf::from("/var/run/biomeos/neural-api.sock");
        let path = resolve_socket_path(Some(custom.clone()), "ignored");
        assert_eq!(path, custom);
    }

    #[test]
    fn test_resolve_socket_path_default_format() {
        let path = resolve_socket_path(None, "abc123");
        assert!(path.is_absolute());
        assert!(path.parent().unwrap() == std::path::Path::new("/tmp"));
        assert_eq!(path.file_name().unwrap(), "neural-api-abc123.sock");
    }
}
