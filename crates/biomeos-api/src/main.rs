// biomeOS API Server
// REST API for primal orchestration and discovery

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod handlers;
mod state;
mod websocket;

pub use state::{AppState, Config};
pub use websocket::{
    GraphEventWebSocketServer, JsonRpcError, JsonRpcRequest, JsonRpcResponse, SubscriptionFilter,
};

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
        mode: if state.is_standalone_mode() {
            "standalone"
        } else {
            "live"
        }
        .to_string(),
    })
}

/// WebSocket upgrade handler for JSON-RPC 2.0 event streaming
async fn websocket_handler(
    State(state): State<Arc<AppState>>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: axum::extract::ws::WebSocket, _state: Arc<AppState>) {
    use axum::extract::ws::Message;

    let (mut sender, mut receiver) = socket.split();

    // TODO: Integrate with GraphEventBroadcaster from state
    // For now, send a welcome message
    let welcome = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "connection.established",
        "params": {
            "message": "Connected to biomeOS Graph Event Stream (JSON-RPC 2.0)",
            "version": env!("CARGO_PKG_VERSION"),
        }
    });

    if sender
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            // Parse JSON-RPC request
            let response = match serde_json::from_str::<JsonRpcRequest>(&text) {
                Ok(req) => {
                    // Handle methods
                    match req.method.as_str() {
                        "events.subscribe" => {
                            // TODO: Implement subscription logic
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(serde_json::json!({
                                    "subscription_id": format!("sub_{}", uuid::Uuid::new_v4()),
                                    "success": true,
                                })),
                                error: None,
                                id: req.id,
                            }
                        }
                        "events.list_subscriptions" => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::json!({
                                "subscriptions": [],
                                "count": 0,
                            })),
                            error: None,
                            id: req.id,
                        },
                        _ => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601,
                                message: "Method not found".to_string(),
                                data: None,
                            }),
                            id: req.id,
                        },
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: None,
                    }),
                    id: None,
                },
            };

            if let Ok(json) = serde_json::to_string(&response) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
    }
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

    let state = Arc::new(state);

    // Build router
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route(
            "/api/v1/primals/discovered",
            get(handlers::discovery::get_discovered_primals),
        )
        .route(
            "/api/v1/primals/list",
            get(handlers::discovery::get_discovered_primals),
        )
        .route(
            "/api/v1/primals",
            get(handlers::discovery::get_discovered_primals),
        )
        .route("/api/v1/topology", get(handlers::topology::get_topology))
        .route("/api/v1/events/stream", get(handlers::events::event_stream)) // SSE endpoint
        .route("/api/v1/events/ws", get(websocket_handler)) // WebSocket endpoint (JSON-RPC 2.0)
        .route(
            "/api/v1/trust/evaluate",
            post(handlers::trust::evaluate_trust),
        )
        .route("/api/v1/trust/identity", get(handlers::trust::get_identity))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    info!(
        "🚀 biomeOS API Server listening on http://{}",
        config.bind_addr
    );
    info!("   Health: http://{}/api/v1/health", config.bind_addr);
    info!(
        "   Discovery: http://{}/api/v1/primals/discovered",
        config.bind_addr
    );
    info!("   Topology: http://{}/api/v1/topology", config.bind_addr);
    info!(
        "   Events (SSE): http://{}/api/v1/events/stream",
        config.bind_addr
    );
    info!(
        "   Events (WebSocket JSON-RPC 2.0): ws://{}/api/v1/events/ws",
        config.bind_addr
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
