// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! # Gaia IDE — Sovereign Agentic Editor
//!
//! Local Rust server that serves the Gaia IDE frontend and proxies API
//! requests to Forgejo (git forge) and Ollama (local AI) on the LAN.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Gaia IDE Server                    │
//! │                  localhost:3141 (π!)                  │
//! ├─────────────────────────────────────────────────────┤
//! │  GET /              → gaia-ide.html (Monaco Editor)  │
//! │  GET /static/*      → JS/CSS/assets from hud-site    │
//! │  /api/forge/*       → proxy to Forgejo API           │
//! │  /api/ollama/*      → proxy to Ollama API            │
//! │  /api/fs/*          → local filesystem read/write    │
//! │  /api/terminal      → WebSocket terminal (planned)   │
//! │  /api/status        → service discovery status        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p gaia-ide
//! # Opens http://localhost:3141 in your default browser
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests use unwrap/expect for concise assertions"
    )
)]

mod proxy;
mod discovery;
mod filesystem;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;

use crate::discovery::ServiceRegistry;

/// Server state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Discovered services on the LAN
    pub services: Arc<ServiceRegistry>,
    /// Path to the hud-site public directory
    pub site_root: PathBuf,
    /// HTTP client for proxying
    pub http_client: reqwest::Client,
}

// reqwest is not in workspace deps, so we use hyper directly
// Actually let's use a simpler approach with axum's built-in client

fn main() {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gaia_ide=info,tower_http=info".parse().unwrap_or_default()),
        )
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
        eprintln!("Failed to create tokio runtime: {e}");
        std::process::exit(1);
    });

    rt.block_on(async_main());
}

async fn async_main() {
    info!("🌿 Gaia IDE — Sovereign Agentic Editor");
    info!("   AGPL-3.0 · No fees · No cloud · Forever free");

    // Discover site root — look for hud-site relative to the biomeOS workspace
    let site_root = find_site_root();
    info!("Site root: {}", site_root.display());

    // Discover services on the LAN
    let services = Arc::new(discovery::discover_services().await);
    info!("Forge: {}", services.forge_url.as_deref().unwrap_or("not found"));
    info!("Ollama: {}", services.ollama_url.as_deref().unwrap_or("not found"));

    let state = Arc::new(AppState {
        services: services.clone(),
        site_root: site_root.clone(),
        http_client: dummy_client(),
    });

    // Build router
    let app = Router::new()
        // OpenAI-compatible API — Cursor/Windsurf/any tool can use localhost:3141
        // as a drop-in replacement for api.openai.com or api.anthropic.com
        .route("/v1/{*path}", get(proxy::openai_compat_proxy).post(proxy::openai_compat_proxy))
        // API routes (proxied)
        .route("/api/status", get(status_handler))
        .route("/api/forge/{*path}", get(proxy::forge_proxy).post(proxy::forge_proxy))
        .route("/api/ollama/{*path}", get(proxy::ollama_proxy).post(proxy::ollama_proxy))
        .route("/api/fs/read", get(filesystem::read_file))
        .route("/api/fs/write", axum::routing::post(filesystem::write_file))
        .route("/api/fs/list", get(filesystem::list_dir))
        // Serve the IDE page at root
        .route("/", get(ide_handler))
        // Serve static files from hud-site/site/public
        .fallback_service(ServeDir::new(&site_root))
        // Middleware
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_methods(AllowMethods::any())
                .allow_headers(AllowHeaders::any()),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind to port 3141 (π!)
    let addr = SocketAddr::from(([127, 0, 0, 1], 3141));
    info!("🚀 Listening on http://{addr}");
    info!("   Open in browser: http://localhost:3141");

    // Auto-open browser
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        open_browser("http://localhost:3141");
    });

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {addr}: {e}");
            eprintln!("Is port 3141 already in use?");
            std::process::exit(1);
        });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| eprintln!("Server error: {e}"));

    info!("🌿 Gaia IDE shut down. lub-DUB 💚");
}

/// Serve the IDE HTML page
async fn ide_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Try to serve gaia-ide.html from site root
    let ide_path = state.site_root.join("gaia-ide.html");
    match tokio::fs::read_to_string(&ide_path).await {
        Ok(content) => {
            // Inject the local API base URL so the IDE connects to us instead of direct Forge/Ollama
            let injected = content.replace(
                "</head>",
                &format!(
                    r#"<script>
window.GAIA_LOCAL_API = "http://localhost:3141/api";
window.GAIA_FORGE = {};
window.GAIA_OLLAMA = {};
window.GAIA_LOCAL = true;
</script>
</head>"#,
                    serde_json::to_string(&state.services.forge_url).unwrap_or_default(),
                    serde_json::to_string(&state.services.ollama_url).unwrap_or_default(),
                ),
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))],
                injected,
            )
        }
        Err(_) => {
            // Fallback: embedded minimal IDE
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))],
                FALLBACK_IDE.to_string(),
            )
        }
    }
}

/// Status endpoint — reports discovered services
async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "gaia_ide": true,
        "version": env!("CARGO_PKG_VERSION"),
        "license": "AGPL-3.0-or-later",
        "forge": state.services.forge_url,
        "ollama": state.services.ollama_url,
        "site_root": state.site_root.display().to_string(),
        "primals_detected": state.services.primals,
        "ts": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Find the hud-site/site/public directory
fn find_site_root() -> PathBuf {
    // Check common locations
    let candidates = [
        // Relative to biomeOS workspace
        PathBuf::from("../hud-site/site/public"),
        // From the Development directory
        dirs_home().join("Development/hud-site/site/public"),
        // Current directory
        PathBuf::from("hud-site/site/public"),
        // Absolute fallback
        PathBuf::from(r"C:\Users\User\Development\hud-site\site\public"),
    ];

    for c in &candidates {
        if c.join("gaia-ide.html").exists() || c.join("gaia.html").exists() {
            return c.to_path_buf();
        }
    }

    // Default even if not found — ServeDir will 404 gracefully
    candidates.last().cloned().unwrap_or_default()
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Open the default browser
fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Graceful shutdown on Ctrl+C
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .unwrap_or_else(|_| ());
    info!("Shutdown signal received");
}

/// Dummy HTTP client (we'll use hyper directly in proxy module)
fn dummy_client() -> reqwest::Client {
    // This is a placeholder — actual proxying uses hyper
    reqwest::Client::new()
}

/// Fallback IDE HTML if gaia-ide.html is not found on disk
const FALLBACK_IDE: &str = r#"<!DOCTYPE html>
<html><head><title>Gaia IDE — Setup</title>
<style>*{margin:0;padding:0;box-sizing:border-box}body{font-family:monospace;background:#0d1117;color:#c9d1d9;display:flex;align-items:center;justify-content:center;height:100vh;text-align:center}a{color:#3fb950}</style>
</head><body>
<div>
<h1 style="color:#3fb950">🌿 Gaia IDE</h1>
<p style="margin:1rem 0">gaia-ide.html not found in site root.</p>
<p>Clone hud-site: <code>git clone https://github.com/ecoPrimals/hud hud-site</code></p>
<p style="margin-top:1rem"><a href="/api/status">Check API Status</a></p>
</div>
</body></html>"#;

// NOTE: reqwest is not in workspace dependencies.
// For now we use a minimal shim. The proxy module uses hyper directly.
mod reqwest {
    #[derive(Clone)]
    pub struct Client;
    impl Client {
        pub fn new() -> Self { Self }
    }
}
