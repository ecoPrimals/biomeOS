// SPDX-License-Identifier: AGPL-3.0-only
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
/// EVOLVED (Mar 11, 2026): Push-based graph events from GraphEventBroadcaster.
///
/// After subscribe, graph events are pushed in real-time as JSON-RPC notifications.
#[allow(
    clippy::too_many_lines,
    reason = "WebSocket message handling and subscription logic"
)]
async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: Arc<AppState>) {
    use axum::extract::ws::Message;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
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
                            Ok(req) => match req.method.as_ref() {
                                "events.subscribe" => {
                                    let params = req
                                        .params
                                        .clone()
                                        .unwrap_or_else(|| serde_json::json!({}));
                                    let filter: SubscriptionFilter =
                                        serde_json::from_value(params).unwrap_or_else(|e| {
                                            tracing::warn!("JSON parse fallback: {}", e);
                                            SubscriptionFilter::default()
                                        });
                                    let sub_id = format!("sub_{}", next_sub_id.fetch_add(1, Ordering::SeqCst));
                                    subscriptions.write().await.insert(sub_id.clone(), filter);
                                    JsonRpcResponse::success(
                                        req.id.clone().unwrap_or(serde_json::Value::Null),
                                        serde_json::json!({
                                            "subscription_id": sub_id,
                                            "success": true,
                                        }),
                                    )
                                }
                                "events.unsubscribe" => {
                                    let sub_id = req
                                        .params
                                        .as_ref()
                                        .and_then(|p| p.get("subscription_id"))
                                        .and_then(serde_json::Value::as_str)
                                        .unwrap_or("");
                                    let existed = subscriptions.write().await.remove(sub_id).is_some();
                                    JsonRpcResponse::success(
                                        req.id.clone().unwrap_or(serde_json::Value::Null),
                                        serde_json::json!({
                                            "success": existed,
                                            "subscription_id": sub_id,
                                        }),
                                    )
                                }
                                "events.list_subscriptions" => {
                                    let subs = subscriptions.read().await;
                                    let sub_list: Vec<_> = subs.keys().collect();
                                    JsonRpcResponse::success(
                                        req.id.clone().unwrap_or(serde_json::Value::Null),
                                        serde_json::json!({
                                            "subscriptions": sub_list,
                                            "count": sub_list.len(),
                                        }),
                                    )
                                }
                                _ => JsonRpcResponse::error(
                                    req.id.clone().unwrap_or(serde_json::Value::Null),
                                    JsonRpcError::method_not_found(),
                                ),
                            },
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
#[allow(
    clippy::too_many_lines,
    reason = "router setup with CORS, routes, and middleware"
)]
fn create_app_with_transport(state: AppState, force_sovereign: bool) -> Router {
    let shared_state = Arc::new(state);

    // Initialize Dark Forest gate
    let mut gate_config = dark_forest_gate::DarkForestGateConfig::from_env();
    if force_sovereign {
        gate_config = gate_config.force_sovereign();
    }
    let gate_state = dark_forest_gate::DarkForestGateState::new(gate_config);

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
    let rendezvous_state = Arc::new(handlers::rendezvous::RendezvousState::new(""));

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
    let router = router.layer(axum::middleware::from_fn_with_state(
        gate_state,
        dark_forest_gate::dark_forest_gate_middleware,
    ));

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

/// Serve on Unix socket only (production mode)
///
/// Unix sockets are inherently secure via filesystem permissions (0600).
/// The Dark Forest gate still applies for defense in depth, but Unix
/// socket connections are already limited to the local user.
pub async fn serve_unix_socket(socket_path: &std::path::Path, app: Router) -> anyhow::Result<()> {
    unix_server::serve_unix_socket(socket_path, app, None).await
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use axum::body::Body;
    use biomeos_test_utils::TestEnvGuard;
    use futures_util::{SinkExt, StreamExt};
    use http_body_util::BodyExt;
    use std::sync::OnceLock;
    use tokio_tungstenite::tungstenite::Message as WsMessage;
    use tower::ServiceExt;

    fn sovereign_env_lock() -> &'static tokio::sync::Mutex<()> {
        static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    #[tokio::test]
    async fn router_health_returns_json_when_gate_disabled() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(v["status"], "healthy");
    }

    #[tokio::test]
    async fn router_readiness_and_liveness_when_gate_disabled() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        for path in ["/api/v1/health/ready", "/api/v1/health/live"] {
            let app = create_app(state.clone());
            let response = app
                .oneshot(
                    axum::http::Request::builder()
                        .uri(path)
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK, "path {path}");
        }
    }

    #[tokio::test]
    async fn router_topology_forbidden_without_token_when_sovereign() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::remove("BIOMEOS_SOVEREIGN");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app_for_tcp(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/topology")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn router_well_known_bypasses_gate_when_sovereign() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::remove("BIOMEOS_SOVEREIGN");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app_for_tcp(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/.well-known/acme-challenge/token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_ne!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn router_health_bare_ok_when_sovereign_no_body() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::remove("BIOMEOS_SOVEREIGN");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app_for_tcp(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn events_ws_welcome_and_subscribe_roundtrip() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = axum::serve(listener, app);
        let join = tokio::spawn(async move {
            server.await.expect("serve");
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = format!("ws://{addr}/api/v1/events/ws");
        let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
            .await
            .expect("ws connect");
        let (mut write, mut read) = ws.split();
        let welcome = read.next().await.expect("welcome").expect("msg");
        let WsMessage::Text(text) = welcome else {
            panic!("expected text welcome");
        };
        assert!(text.contains("connection.established"));
        let sub = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "events.subscribe",
            "params": { "graph_id": "g1" },
            "id": 7
        });
        write
            .send(WsMessage::Text(sub.to_string()))
            .await
            .expect("send");
        let reply = read.next().await.expect("reply").expect("ok");
        let WsMessage::Text(reply_text) = reply else {
            panic!("expected text reply");
        };
        let v: serde_json::Value = serde_json::from_str(&reply_text).expect("json");
        assert!(v.get("result").is_some());
        let unsub = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "events.unsubscribe",
            "params": { "subscription_id": "sub_1" },
            "id": 8
        });
        write
            .send(WsMessage::Text(unsub.to_string()))
            .await
            .expect("unsub");
        let list = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "events.list_subscriptions",
            "id": 9
        });
        write
            .send(WsMessage::Text(list.to_string()))
            .await
            .expect("list");
        join.abort();
    }

    #[test]
    fn jsonrpc_error_helpers_standard_codes() {
        let mn = JsonRpcError::method_not_found();
        assert_eq!(mn.code, -32601);
        let pe = JsonRpcError::parse_error();
        assert_eq!(pe.code, -32700);
    }

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
            jsonrpc: JsonRpcVersion,
            result: Some(serde_json::json!({"data": "test"})),
            error: None,
            id: serde_json::json!(1),
        };
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
            id: serde_json::Value::Null,
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

    #[test]
    fn test_subscription_filter_empty() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: None,
            node_filter: None,
        };
        let json = serde_json::to_string(&filter).expect("serialize");
        let deserialized: SubscriptionFilter =
            serde_json::from_str(&json).expect("round-trip deserialize");
        assert!(deserialized.graph_id.is_none());
        assert!(deserialized.event_types.is_none());
    }

    #[test]
    fn test_api_error_into_response_internal() {
        let error = ApiError::Internal("test internal".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_api_error_into_response_discovery_failed() {
        let error = ApiError::DiscoveryFailed("no primals".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_api_error_into_response_not_found() {
        let error = ApiError::NotFound("resource".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_create_app_for_tcp_returns_router() {
        let state = AppState::builder()
            .build_with_defaults()
            .expect("should create state");
        let app = create_app_for_tcp(state);
        drop(app);
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "events.subscribe",
            "params": {"graph_id": "g1"},
            "id": 1
        }"#;
        let req: JsonRpcRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.method.as_ref(), "events.subscribe");
        assert_eq!(
            req.params
                .as_ref()
                .and_then(|p| p.get("graph_id"))
                .and_then(|v| v.as_str()),
            Some("g1")
        );
    }

    #[test]
    fn test_json_rpc_response_serialization_round_trip() {
        let response = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: Some(serde_json::json!({"subscription_id": "sub_1"})),
            error: None,
            id: serde_json::json!(42),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("sub_1"));
        assert!(json.contains("42"));
    }

    #[tokio::test]
    async fn test_api_error_json_body_contains_error_key() {
        let error = ApiError::Internal("test".to_string());
        let response = error.into_response();
        let (_, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert!(json.get("error").is_some());
        assert_eq!(json["error"].as_str(), Some("test"));
    }

    #[test]
    fn test_api_error_discovery_failed_status() {
        let error = ApiError::DiscoveryFailed("no primals".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_create_app_routes_registered() {
        let state = AppState::builder()
            .build_with_defaults()
            .expect("create state");
        let app = create_app(state);
        // Router should have routes - we can't easily inspect axum Router
        // but we verify it builds without panic
        let _ = app;
    }

    #[test]
    fn test_subscription_filter_deserialization_defaults() {
        let json = "{}";
        let filter: SubscriptionFilter = serde_json::from_str(json).expect("deserialize");
        assert!(filter.graph_id.is_none());
        assert!(filter.event_types.is_none());
        assert!(filter.node_filter.is_none());
    }

    #[tokio::test]
    async fn router_unknown_route_returns_404() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/route-that-does-not-exist")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn router_health_includes_security_headers() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get("strict-transport-security")
                .is_some()
        );
        assert!(response.headers().get("x-content-type-options").is_some());
        assert!(response.headers().get("content-security-policy").is_some());
        assert!(response.headers().get("x-frame-options").is_some());
        assert!(response.headers().get("referrer-policy").is_some());
        assert!(response.headers().get("cache-control").is_some());
    }

    #[tokio::test]
    async fn router_cors_permissive_reflects_origin_when_gate_disabled() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .header("origin", "http://localhost:3000")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get("access-control-allow-origin")
                .is_some()
        );
    }

    #[tokio::test]
    async fn router_post_body_over_limit_returns_413() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let oversized = vec![b'x'; 1024 * 1024 + 1];
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method(axum::http::Method::POST)
                    .uri("/api/v1/capabilities/discover")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(oversized))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn router_events_ws_invalid_json_and_unknown_method() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = axum::serve(listener, app);
        let join = tokio::spawn(async move {
            server.await.expect("serve");
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = format!("ws://{addr}/api/v1/events/ws");
        let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
            .await
            .expect("ws connect");
        let (mut write, mut read) = ws.split();
        let _welcome = read.next().await.expect("welcome").expect("msg");

        write
            .send(WsMessage::Text("not json".into()))
            .await
            .expect("send bad json");
        let parse_reply = read.next().await.expect("parse reply").expect("ok");
        let WsMessage::Text(parse_text) = parse_reply else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&parse_text).expect("json");
        assert_eq!(v["error"]["code"], -32700);

        let unknown = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "unknown.custom",
            "params": {}
        });
        write
            .send(WsMessage::Text(unknown.to_string()))
            .await
            .expect("send unknown");
        let method_reply = read.next().await.expect("method reply").expect("ok");
        let WsMessage::Text(method_text) = method_reply else {
            panic!("expected text");
        };
        let v2: serde_json::Value = serde_json::from_str(&method_text).expect("json");
        assert_eq!(v2["error"]["code"], -32601);

        join.abort();
    }

    #[tokio::test]
    async fn router_events_ws_binary_message_ignored_no_reply() {
        let _guard = sovereign_env_lock().lock().await;
        let _sovereign = TestEnvGuard::set("BIOMEOS_SOVEREIGN", "false");
        let state = AppState::builder().build_with_defaults().expect("state");
        let app = create_app(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = axum::serve(listener, app);
        let join = tokio::spawn(async move {
            server.await.expect("serve");
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let url = format!("ws://{addr}/api/v1/events/ws");
        let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
            .await
            .expect("ws connect");
        let (mut write, mut read) = ws.split();
        let _welcome = read.next().await.expect("welcome").expect("msg");

        write
            .send(WsMessage::Binary(vec![1, 2, 3]))
            .await
            .expect("binary");
        let next = tokio::time::timeout(std::time::Duration::from_millis(200), read.next()).await;
        assert!(
            next.is_err(),
            "binary frames are ignored; no JSON-RPC reply expected"
        );

        let ping = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "events.list_subscriptions",
        });
        write
            .send(WsMessage::Text(ping.to_string()))
            .await
            .expect("list");
        let after = read.next().await.expect("after binary").expect("ok");
        let WsMessage::Text(t) = after else {
            panic!("expected text");
        };
        let v: serde_json::Value = serde_json::from_str(&t).expect("json");
        assert!(v.get("result").is_some());

        join.abort();
    }
}
