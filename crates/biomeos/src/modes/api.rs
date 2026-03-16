// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! API mode - HTTP/WebSocket API server (UniBin integration)
//!
//! Wires the biomeos-api library into the UniBin `biomeos api` subcommand.
//! This is the production API server — no separate binary needed.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

/// Resolved API configuration (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct ApiConfig {
    pub socket_path: PathBuf,
    pub port_deprecated: Option<u16>,
}

/// Resolve API configuration from CLI overrides and defaults.
/// socket takes priority over default_socket_path.
pub(crate) fn resolve_api_config(
    port: Option<u16>,
    socket: Option<PathBuf>,
    default_socket_path: PathBuf,
) -> ApiConfig {
    let socket_path = socket.unwrap_or(default_socket_path);
    ApiConfig {
        socket_path,
        port_deprecated: port,
    }
}

/// Run the biomeOS API server
///
/// Starts the Unix-socket-only JSON-RPC API server using the biomeos-api library.
/// HTTP bridge is removed — all communication is via Unix socket (TRUE PRIMAL).
pub async fn run(port: Option<u16>, socket: Option<PathBuf>, unix_only: bool) -> Result<()> {
    info!("🌐 biomeOS API Server (UniBin mode)");

    let state = biomeos_api::AppState::builder()
        .config_from_env()
        .build_with_defaults()?;

    let config = state.config().clone();

    if config.standalone_mode {
        warn!("Running in STANDALONE MODE - graceful degradation without primals");
    } else {
        info!("Running in LIVE MODE - discovering real primals");
    }

    let api_config = resolve_api_config(port, socket, config.socket_path.clone());
    let socket_path = api_config.socket_path;

    if !unix_only {
        if let Some(p) = api_config.port_deprecated {
            warn!("HTTP mode (port {p}) is deprecated — using Unix socket only (TRUE PRIMAL)");
        }
    }

    let app = biomeos_api::create_app(state);

    info!("biomeOS API Server starting");
    info!("  Socket: {}", socket_path.display());
    info!("  Security: Owner-only (0600) + Dark Forest gate");
    info!("  Protocol: JSON-RPC 2.0");
    info!("  Port-free: TRUE PRIMAL architecture");

    biomeos_api::serve_unix_socket(&socket_path, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_resolve_api_config_socket_override() {
        let config = resolve_api_config(
            Some(8080),
            Some(PathBuf::from("/tmp/custom.sock")),
            PathBuf::from("/tmp/default.sock"),
        );
        assert_eq!(config.socket_path, PathBuf::from("/tmp/custom.sock"));
        assert_eq!(config.port_deprecated, Some(8080));
    }

    #[test]
    fn test_resolve_api_config_default_socket() {
        let config =
            resolve_api_config(None, None, PathBuf::from("/run/user/1000/biomeos-api.sock"));
        assert_eq!(
            config.socket_path,
            PathBuf::from("/run/user/1000/biomeos-api.sock")
        );
        assert_eq!(config.port_deprecated, None);
    }

    #[test]
    fn test_resolve_api_config_port_only_no_socket() {
        let config =
            resolve_api_config(Some(3000), None, PathBuf::from("/run/biomeos/default.sock"));
        assert_eq!(
            config.socket_path,
            PathBuf::from("/run/biomeos/default.sock")
        );
        assert_eq!(config.port_deprecated, Some(3000));
    }

    #[test]
    fn test_resolve_api_config_socket_overrides_port() {
        let config = resolve_api_config(
            Some(9999),
            Some(PathBuf::from("/var/run/api.sock")),
            PathBuf::from("/default.sock"),
        );
        assert_eq!(config.socket_path, PathBuf::from("/var/run/api.sock"));
        assert_eq!(config.port_deprecated, Some(9999));
    }

    #[test]
    fn test_api_config_debug() {
        let config = ApiConfig {
            socket_path: PathBuf::from("/tmp/sock"),
            port_deprecated: Some(80),
        };
        let s = format!("{config:?}");
        assert!(s.contains("socket_path"));
        assert!(s.contains("port_deprecated"));
    }
}
