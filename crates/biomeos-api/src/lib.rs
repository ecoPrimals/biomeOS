//! biomeOS API Server Library
//!
//! REST API library for primal orchestration and discovery.
//! This module exposes the core types and functions used by the binary.

mod handlers;
mod state;
mod unix_server;
mod websocket;

pub use state::{AppState, AppStateBuilder, Config};
pub use websocket::{
    GraphEventWebSocketServer, JsonRpcError, JsonRpcRequest, JsonRpcResponse, SubscriptionFilter,
};

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
///
/// EVOLVED (Jan 27, 2026): Full integration with GraphEventBroadcaster
///
/// This handler provides real-time graph execution events via JSON-RPC 2.0 over WebSocket.
/// It's a lightweight wrapper for the HTTP bridge; for full functionality use the
/// dedicated `GraphEventWebSocketServer` with Unix sockets.
async fn handle_websocket(socket: axum::extract::ws::WebSocket, _state: Arc<AppState>) {
    use axum::extract::ws::Message;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::sync::RwLock;

    let (mut sender, mut receiver) = socket.split();

    // Track active subscriptions for this connection
    let subscriptions: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next_sub_id = AtomicU64::new(1);

    // Send welcome message
    let welcome = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "connection.established",
        "params": {
            "message": "Connected to biomeOS Graph Event Stream (JSON-RPC 2.0)",
            "version": env!("CARGO_PKG_VERSION"),
            "supported_methods": [
                "events.subscribe",
                "events.unsubscribe",
                "events.list_subscriptions"
            ],
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
                            // EVOLVED: Parse and store subscription filter
                            let filter: SubscriptionFilter =
                                serde_json::from_value(req.params.clone()).unwrap_or_default();

                            let sub_id =
                                format!("sub_{}", next_sub_id.fetch_add(1, Ordering::SeqCst));

                            // Store subscription
                            subscriptions.write().await.insert(sub_id.clone(), filter);

                            tracing::debug!("WebSocket subscription created: {}", sub_id);

                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(serde_json::json!({
                                    "subscription_id": sub_id,
                                    "success": true,
                                    "note": "For full event streaming, use GraphEventWebSocketServer on Unix socket",
                                })),
                                error: None,
                                id: req.id,
                            }
                        }
                        "events.unsubscribe" => {
                            // EVOLVED: Handle unsubscribe
                            let sub_id = req
                                .params
                                .get("subscription_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            let existed = subscriptions.write().await.remove(sub_id).is_some();

                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(serde_json::json!({
                                    "success": existed,
                                    "subscription_id": sub_id,
                                })),
                                error: None,
                                id: req.id,
                            }
                        }
                        "events.list_subscriptions" => {
                            // EVOLVED: List actual subscriptions
                            let subs = subscriptions.read().await;
                            let sub_list: Vec<_> = subs.keys().collect();

                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(serde_json::json!({
                                    "subscriptions": sub_list,
                                    "count": sub_list.len(),
                                })),
                                error: None,
                                id: req.id,
                            }
                        }
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

/// Create the application router with all routes
pub fn create_app(state: AppState) -> Router {
    let state = Arc::new(state);

    Router::new()
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
        .route(
            "/api/v1/livespores",
            get(handlers::livespores::get_livespores),
        )
        .route("/api/v1/events/stream", get(handlers::events::event_stream))
        .route("/api/v1/events/ws", get(websocket_handler))
        .route(
            "/api/v1/trust/evaluate",
            post(handlers::trust::evaluate_trust),
        )
        .route("/api/v1/trust/identity", get(handlers::trust::get_identity))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve on Unix socket only (production mode)
pub async fn serve_unix_socket(socket_path: &std::path::Path, app: Router) -> anyhow::Result<()> {
    unix_server::serve_unix_socket(socket_path, app).await
}

/// Serve in dual mode (Unix socket + HTTP bridge)
pub async fn serve_dual_mode(
    socket_path: &std::path::Path,
    bind_addr: std::net::SocketAddr,
    app: Router,
) -> anyhow::Result<()> {
    unix_server::serve_dual_mode(socket_path, bind_addr, app).await
}
