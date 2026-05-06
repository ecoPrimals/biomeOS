// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
}

/// Resolve API configuration from CLI overrides and defaults.
pub(crate) fn resolve_api_config(
    socket: Option<PathBuf>,
    default_socket_path: PathBuf,
) -> ApiConfig {
    ApiConfig {
        socket_path: socket.unwrap_or(default_socket_path),
    }
}

/// Run the biomeOS API server
///
/// Starts the JSON-RPC API server. Default transport is Unix socket (TRUE
/// PRIMAL). When `--port` is provided, a TCP listener is bound alongside UDS
/// for mobile/Android substrates where Unix sockets are unavailable.
pub async fn run(
    port: Option<u16>,
    socket: Option<PathBuf>,
    _unix_only: bool,
    bind: Option<String>,
) -> Result<()> {
    info!("biomeOS API Server (UniBin mode)");

    let state = biomeos_api::AppState::builder()
        .config_from_env()
        .build_with_defaults()?;

    let config = state.config().clone();

    if config.standalone_mode {
        warn!("Running in STANDALONE MODE - graceful degradation without primals");
    } else {
        info!("Running in LIVE MODE - discovering real primals");
    }

    let api_config = resolve_api_config(socket, config.socket_path.clone());
    let socket_path = api_config.socket_path;

    let app = biomeos_api::create_app(state);

    info!("biomeOS API Server starting");
    info!("  Socket: {}", socket_path.display());
    if let Some(p) = port {
        info!("  TCP Port: {p} (alongside UDS for mobile/cross-gate)");
    }
    if let Some(ref addr) = bind {
        info!("  Bind Address: {addr}");
    }
    info!("  Protocol: JSON-RPC 2.0");

    if let Some(tcp_port) = port {
        let tcp_app = app.clone();
        let bind_host = bind.clone();
        tokio::spawn(async move {
            if let Err(e) = biomeos_api::serve_tcp(tcp_port, tcp_app, bind_host.as_deref()).await {
                tracing::error!("API TCP server error: {e}");
            }
        });
    }

    biomeos_api::serve_unix_socket(&socket_path, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_run_fails_when_socket_path_is_directory() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_dir = temp.path().to_path_buf();

        let result = run(None, Some(socket_dir), true).await;
        assert!(
            result.is_err(),
            "run with directory as socket path should fail: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("socket") || err.to_string().contains("remove"),
            "Expected socket-related error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_run_fails_when_socket_path_parent_nonexistent() {
        let socket_path = PathBuf::from("/nonexistent-parent-xyz-12345/biomeos.sock");

        let result = run(None, Some(socket_path), true).await;
        assert!(
            result.is_err(),
            "run with nonexistent parent should fail: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_run_succeeds_with_temp_socket() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("api.sock");
        let path_for_spawn = socket_path.clone();

        let run_handle = tokio::spawn(async move { run(None, Some(path_for_spawn), true).await });

        // Wait for server to bind (socket file appears)
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !socket_path.exists() && std::time::Instant::now() < deadline {
            tokio::task::yield_now().await;
        }
        assert!(socket_path.exists(), "Server should create socket");

        run_handle.abort();
        let _ = run_handle.await;
    }

    #[test]
    fn test_resolve_api_config_socket_override() {
        let config = resolve_api_config(
            Some(PathBuf::from("/tmp/custom.sock")),
            PathBuf::from("/tmp/default.sock"),
        );
        assert_eq!(config.socket_path, PathBuf::from("/tmp/custom.sock"));
    }

    #[test]
    fn test_resolve_api_config_default_socket() {
        let config = resolve_api_config(None, PathBuf::from("/run/user/1000/biomeos-api.sock"));
        assert_eq!(
            config.socket_path,
            PathBuf::from("/run/user/1000/biomeos-api.sock")
        );
    }

    #[test]
    fn test_resolve_api_config_socket_overrides_default() {
        let config = resolve_api_config(
            Some(PathBuf::from("/var/run/api.sock")),
            PathBuf::from("/default.sock"),
        );
        assert_eq!(config.socket_path, PathBuf::from("/var/run/api.sock"));
    }

    #[test]
    fn test_api_config_debug() {
        let config = ApiConfig {
            socket_path: PathBuf::from("/tmp/sock"),
        };
        let s = format!("{config:?}");
        assert!(s.contains("socket_path"));
    }

    #[test]
    fn test_api_config_clone() {
        let config = ApiConfig {
            socket_path: PathBuf::from("/tmp/clone-test.sock"),
        };
        let cloned = config.clone();
        assert_eq!(cloned.socket_path, config.socket_path);
    }

    #[test]
    fn test_resolve_api_config_falls_back_to_default() {
        let config = resolve_api_config(None, PathBuf::from("/run/empty.sock"));
        assert_eq!(config.socket_path, PathBuf::from("/run/empty.sock"));
    }
}
