//! API mode - HTTP/WebSocket API server (UniBin integration)
//!
//! Wires the biomeos-api library into the UniBin `biomeos api` subcommand.
//! This is the production API server — no separate binary needed.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

/// Run the biomeOS API server
///
/// Starts the Unix-socket-only JSON-RPC API server using the biomeos-api library.
/// HTTP bridge is removed — all communication is via Unix socket (TRUE PRIMAL).
pub async fn run(_port: Option<u16>, socket: Option<PathBuf>, unix_only: bool) -> Result<()> {
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

    let socket_path = socket.unwrap_or_else(|| config.socket_path.clone());

    if !unix_only {
        if let Some(port) = _port {
            warn!("HTTP mode (port {port}) is deprecated — using Unix socket only (TRUE PRIMAL)");
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
