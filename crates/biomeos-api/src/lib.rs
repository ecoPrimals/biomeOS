// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! biomeOS API Server Library
//!
//! REST API library for primal orchestration and discovery.
//! This module exposes the core types and functions used by the binary.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Shared beacon verification — single source of truth for Dark Forest token verification
pub mod beacon_verification;
/// Dark Forest beacon gate middleware for sovereign security
pub mod dark_forest_gate;
mod handlers;
mod state;
mod unix_server;
mod websocket;

use biomeos_types::JSONRPC_VERSION;
#[cfg(test)]
use biomeos_types::JsonRpcVersion;
pub use state::{AppState, AppStateBuilder, Config};
pub use websocket::{
    GraphEventWebSocketServer, JsonRpcError, JsonRpcRequest, JsonRpcResponse, SubscriptionFilter,
};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
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
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::DiscoveryFailed(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
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
/// EVOLVED (Mar 11, 2026): Push-based graph events from `GraphEventBroadcaster`.
///
/// After subscribe, graph events are pushed in real-time as JSON-RPC notifications.
async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: Arc<AppState>) {
    use axum::extract::ws::Message;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU64;
    use tokio::sync::RwLock;

    let (mut sender, mut receiver) = socket.split();
    let subscriptions: Arc<RwLock<HashMap<String, SubscriptionFilter>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let next_sub_id = AtomicU64::new(1);

    let welcome = serde_json::json!({
        "jsonrpc": JSONRPC_VERSION,
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

    // Spawn a task that forwards broadcaster events to the WebSocket sender
    let (push_tx, mut push_rx) = tokio::sync::mpsc::channel::<String>(256);
    let broadcaster = state.event_broadcaster().clone();
    let subs_for_push = subscriptions.clone();
    tokio::spawn(async move {
        let mut rx = broadcaster.subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let subs = subs_for_push.read().await;
                    if subs.is_empty() {
                        continue;
                    }
                    let notification = serde_json::json!({
                        "jsonrpc": JSONRPC_VERSION,
                        "method": "events.notification",
                        "params": event,
                    });
                    if push_tx.send(notification.to_string()).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("WebSocket event forwarder lagged, skipped {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let response = match serde_json::from_str::<JsonRpcRequest>(&text) {
                            Ok(req) => {
                                websocket::dispatch_ws_method(
                                    req.method.as_ref(),
                                    req.params,
                                    req.id,
                                    &subscriptions,
                                    &next_sub_id,
                                )
                                .await
                            }
                            Err(_) => JsonRpcResponse::error(
                                serde_json::Value::Null,
                                JsonRpcError::parse_error(),
                            ),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            let _ = sender.send(Message::Text(json)).await;
                        }
                    }
                    Some(Ok(_)) => {} // Ignore non-text messages
                    _ => break, // Connection closed or error
                }
            }
            pushed = push_rx.recv() => {
                match pushed {
                    Some(json) => {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
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
    let gate = dark_forest_gate::DarkForestGateConfig::from_env();
    create_app_with_gate_inner(state, gate)
}

/// Create the application router with an explicit Dark Forest gate configuration.
pub fn create_app_with_gate(
    state: AppState,
    gate_config: dark_forest_gate::DarkForestGateConfig,
) -> Router {
    create_app_with_gate_inner(state, gate_config)
}

/// Create the application router for TCP-bound mode
///
/// When binding to a TCP port (public network), sovereign mode is FORCED
/// regardless of environment variables. The system never exposes anything
/// on a network port without lineage verification.
pub fn create_app_for_tcp(state: AppState) -> Router {
    let gate = dark_forest_gate::DarkForestGateConfig::from_env().force_sovereign();
    create_app_with_gate_inner(state, gate)
}

/// TCP mode with an explicit gate config (sovereign flag is forced on).
pub fn create_app_for_tcp_with_gate(
    state: AppState,
    gate_config: dark_forest_gate::DarkForestGateConfig,
) -> Router {
    create_app_with_gate_inner(state, gate_config.force_sovereign())
}

#[expect(
    clippy::too_many_lines,
    reason = "flat axum route table; splitting fights Router<AppState> inference"
)]
fn register_api_routes(
    shared_state: Arc<AppState>,
    gate_state: dark_forest_gate::DarkForestGateState,
) -> Router {
    // CORS: restrictive by default
    // Only allow same-origin + the X-Dark-Forest-Token header
    // No permissive CORS — an attacker should learn nothing from preflight
    let cors = if gate_state.config.enabled {
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
        // Genome Distribution API (wateringHole/genomeBin)
        .route(
            "/api/v1/genome/dist/manifest",
            get(handlers::genome_dist::get_manifest),
        )
        .route(
            "/api/v1/genome/dist/:primal/latest",
            get(handlers::genome_dist::get_latest),
        )
        .route(
            "/api/v1/genome/dist/checksum/:primal/:version/:arch",
            get(handlers::genome_dist::get_checksum),
        )
        .route(
            "/api/v1/genome/dist/:primal/:version/:arch",
            get(handlers::genome_dist::download_binary),
        )
        .route(
            "/api/v1/genome/dist/update-livespore",
            post(handlers::genome_dist::update_livespore),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // Add rendezvous routes (Dark Forest beacon handshake for Pixel-USB)
    // Uses Tower Atomic capability routing — no direct primal socket wiring
    let rendezvous_state = Arc::new(handlers::rendezvous::RendezvousState::new());

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

    // Apply Dark Forest gate
    // This gates ALL requests — lineage before interaction
    router.layer(axum::middleware::from_fn_with_state(
        gate_state,
        dark_forest_gate::dark_forest_gate_middleware,
    ))
}

fn apply_security_headers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Security headers - applied as outermost layer to ALL responses
    // These headers are defense-in-depth even through Cloudflare proxy
    router
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        ))
        // Request body size limit - prevent oversized payloads (1MB max)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
}

fn create_app_with_gate_inner(
    state: AppState,
    gate_config: dark_forest_gate::DarkForestGateConfig,
) -> Router {
    let shared_state = Arc::new(state);
    let gate_state = dark_forest_gate::DarkForestGateState::new(gate_config);
    let router = register_api_routes(shared_state, gate_state);
    apply_security_headers(router)
}

/// Serve on Unix socket only (production mode)
///
/// Unix sockets are inherently secure via filesystem permissions (0600).
/// The Dark Forest gate still applies for defense in depth, but Unix
/// socket connections are already limited to the local user.
pub async fn serve_unix_socket(socket_path: &std::path::Path, app: Router) -> anyhow::Result<()> {
    unix_server::serve_unix_socket(socket_path, app, None).await
}

/// Serve the API over TCP alongside the Unix socket.
///
/// Binds a TCP listener on `0.0.0.0:<port>` and serves the same JSON-RPC
/// Axum router. Intended for mobile/Android substrates where Unix sockets
/// are unavailable or for cross-gate orchestration.
pub async fn serve_tcp(port: u16, app: Router) -> anyhow::Result<()> {
    let addr = biomeos_types::constants::endpoints::production_tcp_bind_addr(port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind TCP port {port}: {e}"))?;
    tracing::info!(
        "API TCP listener bound: {}:{port}",
        biomeos_types::constants::endpoints::PRODUCTION_BIND_ADDRESS
    );
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("API TCP server error: {e}"))
}

#[cfg(test)]
mod lib_tests;
