// biomeOS API Server
// REST API for primal orchestration and discovery

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},  // ✅ Add post for trust evaluation
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod handlers;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    // Universal Primal Client will go here
    // For now, placeholder
    pub mock_mode: bool,
}

/// API error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Primal discovery failed: {0}")]
    DiscoveryFailed(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::DiscoveryFailed(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
        }));
        
        (status, body).into_response()
    }
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    mode: String,
}

/// Health check handler
async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: if state.mock_mode { "mock" } else { "live" }.to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "biomeos_api=info,tower_http=debug".to_string()),
        )
        .init();

    info!("🏗️  Starting biomeOS API Server v{}", env!("CARGO_PKG_VERSION"));

    // Create app state
    let mock_mode = std::env::var("BIOMEOS_MOCK_MODE")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    
    if mock_mode {
        warn!("⚠️  Running in MOCK MODE - using hardcoded test data");
        warn!("   Set BIOMEOS_MOCK_MODE=false for live primal discovery");
    } else {
        info!("✅ Running in LIVE MODE - will discover real primals");
    }
    
    let state = Arc::new(AppState { mock_mode });

    // Build router
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/primals/discovered", get(handlers::discovery::get_discovered_primals))
        .route("/api/v1/primals/list", get(handlers::discovery::get_discovered_primals)) // Alias
        .route("/api/v1/primals", get(handlers::discovery::get_discovered_primals)) // PetalTongue expects this!
        .route("/api/v1/topology", get(handlers::topology::get_topology))
        // ✅ NEW: Trust endpoints via Universal Primal Client
        .route("/api/v1/trust/evaluate", post(handlers::trust::evaluate_trust))
        .route("/api/v1/trust/identity", get(handlers::trust::get_identity))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind server
    let bind_addr = std::env::var("BIOMEOS_API_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    
    let addr: SocketAddr = bind_addr
        .parse()
        .expect("Invalid BIOMEOS_API_BIND_ADDR");

    info!("🚀 biomeOS API Server listening on http://{}", addr);
    info!("   Health: http://{}/api/v1/health", addr);
    info!("   Discovery: http://{}/api/v1/primals/discovered", addr);
    info!("   Topology: http://{}/api/v1/topology", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

