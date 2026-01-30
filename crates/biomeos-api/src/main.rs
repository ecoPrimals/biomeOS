// biomeOS API Server
// REST API for primal orchestration and discovery

use tracing::{info, warn};

use biomeos_api::{create_app, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "biomeos_api=info,tower_http=debug".to_string()),
        )
        .init();

    info!(
        "🏗️  Starting biomeOS API Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Build application state using modern builder pattern
    let state = AppState::builder()
        .config_from_env()
        .build_with_defaults()?;

    let config = state.config().clone();

    if config.standalone_mode {
        warn!("⚠️  Running in STANDALONE MODE - graceful degradation without primals");
        warn!("   Set BIOMEOS_STANDALONE_MODE=false for live primal discovery");
    } else {
        info!("✅ Running in LIVE MODE - discovering real primals");
    }

    // Build router
    let app = create_app(state);

    // Start server (Unix socket PRIMARY, HTTP bridge optional)
    if config.enable_http_bridge {
        if let Some(bind_addr) = config.bind_addr {
            // Dual mode: Unix socket + HTTP bridge
            info!("🌉 Starting in DUAL MODE (Unix socket + HTTP bridge)");
            info!("   Unix socket: {}", config.socket_path.display());
            info!("   HTTP bridge: http://{}", bind_addr);
            info!("   ⚠️ HTTP is TEMPORARY and will be removed!");

            biomeos_api::serve_dual_mode(&config.socket_path, bind_addr, app).await?;
        } else {
            warn!("⚠️  HTTP bridge enabled but no bind_addr set!");
            warn!("   Falling back to Unix socket only");

            info!("🚀 biomeOS API Server starting (Unix socket only)");
            info!("   Socket: {}", config.socket_path.display());
            info!("   Protocol: JSON-RPC 2.0");
            info!("   Endpoints:");
            info!("     • /api/v1/health");
            info!("     • /api/v1/primals/discovered");
            info!("     • /api/v1/topology");
            info!("     • /api/v1/livespores");
            info!("     • /api/v1/events/stream (SSE)");
            info!("     • /api/v1/events/ws (WebSocket JSON-RPC 2.0)");

            biomeos_api::serve_unix_socket(&config.socket_path, app).await?;
        }
    } else {
        // Unix socket only (PRODUCTION mode!)
        info!("🚀 biomeOS API Server starting (Unix socket - PRODUCTION)");
        info!("   Socket: {}", config.socket_path.display());
        info!("   Protocol: JSON-RPC 2.0");
        info!("   Security: Owner-only (0600 permissions)");
        info!("   Port-free: ✅ TRUE PRIMAL architecture!");
        info!("");
        info!("   Endpoints:");
        info!("     • /api/v1/health");
        info!("     • /api/v1/primals/discovered");
        info!("     • /api/v1/topology");
        info!("     • /api/v1/livespores");
        info!("     • /api/v1/events/stream (SSE)");
        info!("     • /api/v1/events/ws (WebSocket JSON-RPC 2.0)");
        info!("");
        info!("   Connect via: {}", config.socket_path.display());

        biomeos_api::serve_unix_socket(&config.socket_path, app).await?;
    }

    Ok(())
}
