// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use crate::dark_forest_gate::DarkForestGateConfig;
use axum::body::Body;
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tower::ServiceExt;

/// Gate config with sovereign checks disabled (equivalent to `BIOMEOS_SOVEREIGN=false`).
fn gate_disabled() -> DarkForestGateConfig {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_SOVEREIGN".to_string(), "false".to_string());
    DarkForestGateConfig::from_env_map(&env)
}

#[tokio::test]
async fn router_health_returns_json_when_gate_disabled() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    for path in ["/api/v1/health/ready", "/api/v1/health/live"] {
        let app = create_app_with_gate(state.clone(), gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
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
