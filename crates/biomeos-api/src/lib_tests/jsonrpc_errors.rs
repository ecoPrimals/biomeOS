// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::*;

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
