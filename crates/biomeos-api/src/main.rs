// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![forbid(unsafe_code)]

//! biomeOS API Server
//!
//! REST API for primal orchestration and discovery.
//! Sovereign mode (Dark Forest gate) is ENABLED by default.
//! All connections must prove family lineage before any interaction.

use tracing::{info, warn};

use biomeos_api::{AppState, Config, create_app};

/// Resolve API server configuration from environment (extracted for testability)
pub(crate) fn resolve_api_server_config() -> anyhow::Result<(AppState, Config)> {
    let state = AppState::builder()
        .config_from_env()
        .build_with_defaults()?;
    let config = state.config().clone();
    Ok((state, config))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "biomeos_api=info,tower_http=debug".to_string()),
        )
        .init();

    info!("Starting biomeOS API Server v{}", env!("CARGO_PKG_VERSION"));

    let (state, config) = resolve_api_server_config()?;

    if config.standalone_mode {
        warn!("Running in STANDALONE MODE - graceful degradation without primals");
    } else {
        info!("Running in LIVE MODE - discovering real primals");
    }

    // Start server (Unix socket only — HTTP bridge removed in v0.5.0)
    if config.enable_http_bridge {
        warn!("HTTP bridge is deprecated and removed. Using Unix socket only.");
        if config.bind_addr.is_some() {
            warn!("  bind_addr is ignored — use Unix socket for PetalTongue/JSON-RPC");
        }
    }

    let app = create_app(state);

    info!("biomeOS API Server starting (Unix socket - PRODUCTION)");
    info!("  Socket: {}", config.socket_path.display());
    info!("  Security: Owner-only (0600) + Dark Forest gate");
    info!("  Protocol: JSON-RPC 2.0");
    info!("  Port-free: TRUE PRIMAL architecture");

    biomeos_api::serve_unix_socket(&config.socket_path, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    #[ignore = "env-dependent; run with --test-threads=1"]
    fn test_resolve_api_server_config() {
        let (state, config) = resolve_api_server_config().unwrap();
        assert!(!config.socket_path.as_os_str().is_empty());
        let _ = state;
    }
}
