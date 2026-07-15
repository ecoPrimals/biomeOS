// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_types::{JsonRpcRequest, JsonRpcResponse};

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test_method"));
    assert!(
        request
            .id
            .as_ref()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn test_jsonrpc_response_deserialization() {
    let json = r#"{"jsonrpc":"2.0","result":{"success":true},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id.as_u64().unwrap(), 1);
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_jsonrpc_error_response() {
    let json = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid request");
}

#[test]
fn test_jsonrpc_response_with_null_result() {
    let json = r#"{"jsonrpc":"2.0","result":null,"id":42}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert!(
        response.result.is_none(),
        "serde maps JSON null to None for Option<Value>"
    );
    assert_eq!(response.id.as_u64().unwrap(), 42);
}

#[test]
fn test_jsonrpc_response_nested_result() {
    let json = r#"{"jsonrpc":"2.0","result":{"nested":{"value":123}},"id":1}"#;
    let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
    let result = response.result.unwrap();
    assert_eq!(result["nested"]["value"], 123);
}

#[test]
fn test_jsonrpc_request_params_empty_object() {
    let request = JsonRpcRequest::new("ping", serde_json::json!({}));
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"params\":{}"));
    assert!(
        request
            .id
            .as_ref()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn test_jsonrpc_request_params_nested() {
    let params = serde_json::json!({"capability": "encryption", "family_id": null});
    let request = JsonRpcRequest::new("discover", params);
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("discover"));
    assert!(json.contains("capability"));
}
