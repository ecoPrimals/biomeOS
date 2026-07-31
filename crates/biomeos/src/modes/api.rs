// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unified API mode — HTTP/WebSocket + Neural API JSON-RPC (G22 convergence)
//!
//! A single `biomeos api` process serves BOTH protocols:
//! - HTTP/WebSocket (axum) for UI, dashboards, external tools
//! - JSON-RPC (Neural API) for primal IPC, graph execution, composition
//!
//! This eliminates split-brain between separate processes and ensures
//! single-restart = full composition recovery for springs+gardens.

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

/// Run the unified biomeOS API server (G22 convergence)
///
/// Launches BOTH the HTTP/WebSocket server AND the Neural API JSON-RPC server
/// in a single process. Default transport is Unix socket (TRUE PRIMAL). When
/// `--port` is provided, a TCP listener is bound alongside UDS for mobile/Android
/// substrates where Unix sockets are unavailable.
pub async fn run(
    port: Option<u16>,
    socket: Option<PathBuf>,
    _unix_only: bool,
    bind: Option<String>,
) -> Result<()> {
    info!("biomeOS Unified API Server (G22 convergence)");

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

    info!("biomeOS Unified API Server starting");
    info!("  HTTP Socket: {}", socket_path.display());
    if let Some(p) = port {
        info!("  TCP Port: {p} (alongside UDS for mobile/cross-gate)");
    }
    if let Some(ref addr) = bind {
        info!("  Bind Address: {addr}");
    }
    info!("  Protocols: HTTP/WebSocket + JSON-RPC (Neural API)");

    // G22: Launch Neural API server alongside HTTP (single process, dual protocol)
    let family_id = biomeos_core::family_discovery::get_family_id();
    let neural_socket = socket_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/tmp"))
        .join(format!("neural-api-{family_id}.sock"));

    info!("  Neural API Socket: {}", neural_socket.display());

    let neural_bind = bind.clone();
    tokio::spawn(async move {
        if let Err(e) = super::neural_api::run(
            PathBuf::from("graphs"),
            family_id,
            Some(neural_socket),
            None,
            false,
            neural_bind,
            true, // btsp_optional: accept plain JSON-RPC from local callers
        )
        .await
        {
            tracing::error!("Neural API server exited: {e}");
        }
    });

    let env_tcp_only = biomeos_types::env_config::is_tcp_only_bind_mode();

    let tcp_handle = if let Some(tcp_port) = port {
        let tcp_app = app.clone();
        let bind_host = bind.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = biomeos_api::serve_tcp(tcp_port, tcp_app, bind_host.as_deref()).await {
                tracing::error!("API TCP server error: {e}");
            }
        }))
    } else {
        None
    };

    if env_tcp_only && tcp_handle.is_some() {
        info!("PRIMAL_BIND_MODE=tcp_only — skipping UDS bind, serving TCP only");
        if let Some(handle) = tcp_handle {
            handle.await.ok();
        }
        return Ok(());
    }

    match biomeos_api::serve_unix_socket(&socket_path, app).await {
        Ok(()) => Ok(()),
        Err(e) => {
            if let Some(handle) = tcp_handle {
                warn!(
                    "UDS bind failed ({}), running on TCP only. \
                     This is expected on SELinux/Android substrates.",
                    e
                );
                handle.await.ok();
                Ok(())
            } else {
                Err(e)
            }
        }
    }?;

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

        let result = run(None, Some(socket_dir), true, None).await;
        assert!(
            result.is_err(),
            "run with directory as socket path should fail: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("socket")
                || err.to_string().contains("remove")
                || err.to_string().contains("bind")
                || err.to_string().contains("transport"),
            "Expected socket-related error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_run_fails_when_socket_path_parent_nonexistent() {
        let socket_path = PathBuf::from("/nonexistent-parent-xyz-12345/biomeos.sock");

        let result = run(None, Some(socket_path), true, None).await;
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

        let run_handle =
            tokio::spawn(async move { run(None, Some(path_for_spawn), true, None).await });

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
