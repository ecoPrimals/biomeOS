//! API mode - HTTP/WebSocket API server
//!
//! Note: For full API server, use neural-api-server binary directly.
//! This mode provides a stub for unified CLI entry point.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

pub async fn run(port: Option<u16>, socket: Option<PathBuf>, unix_only: bool) -> Result<()> {
    info!("🌐 biomeOS API Server");

    if unix_only {
        let socket_path = socket.unwrap_or_else(|| PathBuf::from("/tmp/biomeos-api.sock"));
        info!("Unix socket mode: {}", socket_path.display());

        info!("⚠️  Full API server requires biomeos-api library refactoring");
        info!("   Socket would be: {}", socket_path.display());
    } else if let Some(socket_path) = socket {
        info!("Unix socket mode: {}", socket_path.display());
        info!("⚠️  Full API server requires biomeos-api library refactoring");
    } else {
        let port = port.unwrap_or(3000);
        warn!("⚠️  HTTP mode is deprecated! Use --unix-only for production.");
        info!("Would start HTTP API server on port {}", port);
        info!("⚠️  Full API server requires biomeos-api library refactoring");
    }

    info!("");
    info!("For now, biomeos-api binary should be used directly.");

    Ok(())
}
