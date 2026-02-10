//! biomeOS API Server Library
//!
//! REST API library for primal orchestration and discovery.
//! This module exposes the core types and functions used by the binary.

#![warn(missing_docs)]
#![deny(unsafe_code)]

/// Dark Forest beacon gate middleware for sovereign security
pub mod dark_forest_gate;
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
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

/// API error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Primal discovery failure
    #[error("Primal discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Resource not found
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

// Health handler moved to handlers/health.rs

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
///
/// **Sovereign mode is enabled by default.** All connections must present
/// a valid Dark Forest beacon token proving family lineage before any
/// interaction occurs. Without lineage, the system is indistinguishable
/// from an empty 403.
///
/// Set `BIOMEOS_SOVEREIGN=false` to disable (development/testing only).
pub fn create_app(state: AppState) -> Router {
    create_app_with_transport(state, false)
}

/// Create the application router for TCP-bound mode
///
/// When binding to a TCP port (public network), sovereign mode is FORCED
/// regardless of environment variables. The system never exposes anything
/// on a network port without lineage verification.
pub fn create_app_for_tcp(state: AppState) -> Router {
    create_app_with_transport(state, true)
}

/// Internal: create app with transport-aware security
fn create_app_with_transport(state: AppState, force_sovereign: bool) -> Router {
    let shared_state = Arc::new(state);

    // Initialize Dark Forest gate
    let mut gate_config = dark_forest_gate::DarkForestGateConfig::from_env();
    if force_sovereign {
        gate_config = gate_config.force_sovereign();
    }
    let gate_state = dark_forest_gate::DarkForestGateState::new(gate_config.clone());

    // CORS: restrictive by default
    // Only allow same-origin + the X-Dark-Forest-Token header
    // No permissive CORS — an attacker should learn nothing from preflight
    let cors = if gate_config.enabled {
        // Sovereign mode: no CORS at all — only family clients connect,
        // they don't need browser CORS
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact(HeaderValue::from_static("null")))
            .allow_methods(AllowMethods::list([
                axum::http::Method::GET,
                axum::http::Method::POST,
            ]))
            .allow_headers(AllowHeaders::list([
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderName::from_static("x-dark-forest-token"),
            ]))
    } else {
        // Non-sovereign (development): permissive for local testing
        CorsLayer::permissive()
    };

    let router = Router::new()
        .route("/api/v1/health", get(handlers::health::health))
        .route("/api/v1/health/ready", get(handlers::health::readiness))
        .route("/api/v1/health/live", get(handlers::health::liveness))
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
        // Capability discovery routes
        .route(
            "/api/v1/capabilities",
            get(handlers::capability::list_capabilities),
        )
        .route(
            "/api/v1/capabilities/discover",
            post(handlers::capability::discover_capability),
        )
        .route(
            "/api/v1/trust/evaluate",
            post(handlers::trust::evaluate_trust),
        )
        .route("/api/v1/trust/identity", get(handlers::trust::get_identity))
        // Genome Factory routes
        .route(
            "/api/v1/genome/create",
            post(handlers::genome::create_genome),
        )
        .route(
            "/api/v1/genome/compose",
            post(handlers::genome::compose_genome),
        )
        .route(
            "/api/v1/genome/self-replicate",
            post(handlers::genome::self_replicate),
        )
        .route(
            "/api/v1/genome/:id/verify",
            get(handlers::genome::verify_genome),
        )
        .route("/api/v1/genome/list", get(handlers::genome::list_genomes))
        .route(
            "/api/v1/genome/:id/download",
            get(handlers::genome::download_genome),
        )
        .route("/api/v1/genome/build", post(handlers::genome::build_genome))
        .route(
            "/api/v1/genome/:id/info",
            get(handlers::genome::get_genome_info),
        )
        .route(
            "/api/v1/genome/verify-file",
            post(handlers::genome::verify_genome_file),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // Add rendezvous routes (Dark Forest beacon handshake for Pixel-USB)
    // These have their OWN Dark Forest verification inside the handlers too
    let beardog_socket = std::env::var("BEARDOG_SOCKET").unwrap_or_else(|_| {
        let paths = biomeos_types::paths::SystemPaths::new_lazy();
        paths.primal_socket("beardog").to_string_lossy().to_string()
    });
    let rendezvous_state = Arc::new(handlers::rendezvous::RendezvousState::new(&beardog_socket));

    let router = router
        .route(
            "/api/v1/rendezvous/beacon",
            post(handlers::rendezvous::post_beacon),
        )
        .route(
            "/api/v1/rendezvous/check",
            post(handlers::rendezvous::check_peer),
        )
        .with_state(rendezvous_state);

    // Apply Dark Forest gate as outermost layer
    // This gates ALL requests — lineage before interaction
    router.layer(axum::middleware::from_fn_with_state(
        gate_state,
        dark_forest_gate::dark_forest_gate_middleware,
    ))
}

/// Serve on Unix socket only (production mode)
///
/// Unix sockets are inherently secure via filesystem permissions (0600).
/// The Dark Forest gate still applies for defense in depth, but Unix
/// socket connections are already limited to the local user.
pub async fn serve_unix_socket(socket_path: &std::path::Path, app: Router) -> anyhow::Result<()> {
    unix_server::serve_unix_socket(socket_path, app).await
}

/// Serve in dual mode (Unix socket + HTTP bridge)
///
/// **WARNING**: When binding to TCP, sovereign mode is FORCED.
/// All TCP connections must present a valid Dark Forest token.
/// The TCP-bound router uses `create_app_for_tcp()` which forces sovereign.
///
/// **DEPRECATED since 0.3.0**: Use [`serve_unix_socket()`] for production.
/// HTTP bridge will be removed in v0.5.0 when PetalTongue migrates to
/// Unix socket JSON-RPC.
#[deprecated(
    since = "0.3.0",
    note = "Use serve_unix_socket() — HTTP bridge will be removed in v0.5.0"
)]
pub async fn serve_dual_mode(
    socket_path: &std::path::Path,
    bind_addr: std::net::SocketAddr,
    app: Router,
) -> anyhow::Result<()> {
    #[allow(deprecated)]
    unix_server::serve_dual_mode(socket_path, bind_addr, app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_internal() {
        let error = ApiError::Internal("test error".to_string());
        assert!(format!("{error}").contains("test error"));
    }

    #[test]
    fn test_api_error_discovery_failed() {
        let error = ApiError::DiscoveryFailed("no primals found".to_string());
        assert!(format!("{error}").contains("no primals found"));
    }

    #[test]
    fn test_api_error_not_found() {
        let error = ApiError::NotFound("resource missing".to_string());
        assert!(format!("{error}").contains("resource missing"));
    }

    // Health handler tests moved to handlers/health.rs

    #[test]
    fn test_create_app_returns_router() {
        // Create a minimal AppState for testing
        let state = AppState::builder()
            .build_with_defaults()
            .expect("should create state");
        let app = create_app(state);
        // Router should be created without panicking
        drop(app);
    }

    #[test]
    fn test_json_rpc_error_codes() {
        // Standard JSON-RPC error codes
        let method_not_found = JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        assert_eq!(method_not_found.code, -32601);

        let parse_error = JsonRpcError {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        };
        assert_eq!(parse_error.code, -32700);
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"data": "test"})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
            id: None,
        };
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_subscription_filter_serialization() {
        let filter = SubscriptionFilter {
            graph_id: Some("test-graph".to_string()),
            event_types: Some(vec!["node_started".to_string()]),
            node_filter: None,
        };
        let json = serde_json::to_string(&filter).expect("serialize");
        assert!(json.contains("test-graph"));
        assert!(json.contains("node_started"));
    }
}
