// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural API mode - Graph-based orchestration server
//!
//! Starts the Neural API JSON-RPC server for graph execution

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use biomeos_types::paths::SystemPaths;
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
    resolve_neural_api_config_with(graphs_dir, socket, family_id, None)
}

pub(crate) fn resolve_neural_api_config_with(
    graphs_dir: PathBuf,
    socket: Option<PathBuf>,
    family_id: Option<&str>,
    family_id_from_discovery: Option<&str>,
) -> NeuralApiConfig {
    let family_id = family_id
        .map(String::from)
        .or_else(|| family_id_from_discovery.map(String::from))
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket.unwrap_or_else(|| {
        SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family_id}"))
    });
    NeuralApiConfig {
        graphs_dir,
        family_id,
        socket_path,
    }
}

/// Resolve socket path: use explicit path or XDG-compliant SystemPaths default
pub(crate) fn resolve_socket_path(socket: Option<PathBuf>, family_id: &str) -> PathBuf {
    socket.unwrap_or_else(|| {
        SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family_id}"))
    })
}

pub async fn run(
    graphs_dir: PathBuf,
    family_id: String,
    socket: Option<PathBuf>,
    tcp_port: Option<u16>,
    tcp_only: bool,
) -> Result<()> {
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
    if tcp_only {
        info!("  Transport: TCP-only (port {})", tcp_port.unwrap_or(0));
    } else if let Some(port) = tcp_port {
        info!("  Socket Path: {}", socket_path.display());
        info!("  TCP Port: {port} (alongside UDS)");
    } else {
        info!("  Socket Path: {}", socket_path.display());
    }
    info!("");

    let mut server = NeuralApiServer::new(graphs_dir, family_id, socket_path);
    if tcp_only {
        if let Some(port) = tcp_port {
            server = server.with_tcp_only(port);
        }
    } else if let Some(port) = tcp_port {
        server = server.with_tcp_port(port);
    }

    info!("🚀 Starting Neural API server...");
    server.serve().await.context("Neural API server failed")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

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
        assert!(path.to_string_lossy().contains("neural-api-my-family.sock"));
        assert!(path.is_absolute());
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
        assert!(path.to_string_lossy().contains("neural-api-.sock"));
        assert!(path.is_absolute());
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
        assert_eq!(path.file_name().unwrap(), "neural-api-abc123.sock");
    }

    #[test]
    fn test_resolve_neural_api_config_with_socket() {
        let socket = PathBuf::from("/custom/neural.sock");
        let config =
            resolve_neural_api_config(PathBuf::from("graphs"), Some(socket.clone()), Some("fam1"));
        assert_eq!(config.socket_path, socket);
        assert_eq!(config.family_id, "fam1");
        assert_eq!(config.graphs_dir, PathBuf::from("graphs"));
    }

    #[test]
    fn test_neural_api_config_debug() {
        let config = NeuralApiConfig {
            graphs_dir: PathBuf::from("g"),
            family_id: "f".to_string(),
            socket_path: PathBuf::from("/tmp/s.sock"),
        };
        let s = format!("{config:?}");
        assert!(s.contains("NeuralApiConfig"));
        assert!(s.contains('g'));
        assert!(s.contains('f'));
    }

    #[test]
    fn test_resolve_neural_api_config_family_from_discovery() {
        let config = resolve_neural_api_config_with(
            PathBuf::from("g"),
            None,
            None,
            Some("discovery-family"),
        );
        assert_eq!(config.family_id, "discovery-family");
        assert_eq!(config.graphs_dir, PathBuf::from("g"));
        assert!(
            config
                .socket_path
                .to_string_lossy()
                .contains("discovery-family")
        );
    }

    #[test]
    fn test_resolve_neural_api_config_family_id_takes_precedence_over_discovery() {
        let config = resolve_neural_api_config_with(
            PathBuf::from("g"),
            None,
            Some("explicit-family"),
            Some("discovery-family"),
        );
        assert_eq!(config.family_id, "explicit-family");
        assert!(
            config
                .socket_path
                .to_string_lossy()
                .contains("explicit-family")
        );
    }

    #[test]
    fn test_resolve_neural_api_config_socket_takes_precedence() {
        let custom_socket = PathBuf::from("/custom/neural-api.sock");
        let config = resolve_neural_api_config_with(
            PathBuf::from("graphs"),
            Some(custom_socket.clone()),
            Some("family"),
            Some("discovery"),
        );
        assert_eq!(config.socket_path, custom_socket);
        assert_eq!(config.family_id, "family");
    }

    #[test]
    fn test_resolve_neural_api_config_all_none_uses_family_discovery() {
        let config = resolve_neural_api_config_with(PathBuf::from("g"), None, None, None);
        assert!(!config.family_id.is_empty());
        assert_eq!(config.graphs_dir, PathBuf::from("g"));
        assert!(config.socket_path.to_string_lossy().contains("neural-api"));
    }

    #[test]
    fn test_resolve_neural_api_config_clone() {
        let original = NeuralApiConfig {
            graphs_dir: PathBuf::from("g"),
            family_id: "f".to_string(),
            socket_path: PathBuf::from("/tmp/s.sock"),
        };
        let cloned = original.clone();
        assert_eq!(cloned.family_id, original.family_id);
        assert_eq!(cloned.graphs_dir, original.graphs_dir);
    }

    #[test]
    fn test_resolve_neural_api_config_discovery_fallback_no_explicit() {
        let config =
            resolve_neural_api_config_with(PathBuf::from("graphs"), None, None, Some("disc-fam"));
        assert_eq!(config.family_id, "disc-fam");
    }

    #[test]
    fn test_resolve_neural_api_config_with_all_some() {
        let sock = PathBuf::from("/custom.sock");
        let config = resolve_neural_api_config_with(
            PathBuf::from("g"),
            Some(sock.clone()),
            Some("explicit"),
            Some("discovery"),
        );
        assert_eq!(config.family_id, "explicit");
        assert_eq!(config.socket_path, sock);
        assert_eq!(config.graphs_dir, PathBuf::from("g"));
    }
}
