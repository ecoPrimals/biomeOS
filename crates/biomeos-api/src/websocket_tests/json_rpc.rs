// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_types::JsonRpcVersion;

use super::super::*;

#[test]
fn test_json_rpc_error_codes() {
    assert_eq!(JsonRpcError::parse_error().code, -32700);
    assert_eq!(JsonRpcError::invalid_request().code, -32600);
    assert_eq!(JsonRpcError::method_not_found().code, -32601);
    assert_eq!(JsonRpcError::invalid_params(None).code, -32602);
    assert_eq!(JsonRpcError::internal_error(None).code, -32603);
}

#[test]
fn test_json_rpc_request_deserialization() {
    let json =
        r#"{"jsonrpc": "2.0", "id": 1, "method": "test.method", "params": {"key": "value"}}"#;
    let request: JsonRpcRequest = serde_json::from_str(json).expect("deserialize");

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, Some(serde_json::json!(1)));
    assert_eq!(request.method.as_ref(), "test.method");
}

#[test]
fn test_json_rpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: Some(serde_json::json!({"success": true})),
        error: None,
        id: serde_json::json!(1),
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("2.0"));
    assert!(json.contains("success"));
}

#[test]
fn test_json_rpc_error_with_data() {
    let error = JsonRpcError::invalid_params(Some("missing required field".to_string()));
    assert_eq!(error.code, -32602);
    assert!(error.data.is_some());
}

#[test]
fn test_json_rpc_error_serialization() {
    let error = JsonRpcError::internal_error(Some("detail".to_string()));
    let json = serde_json::to_string(&error).expect("serialize");
    assert!(json.contains("-32603"));
    assert!(json.contains("Internal error"));
    assert!(json.contains("detail"));
}

#[test]
fn test_json_rpc_response_error_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: None,
        error: Some(JsonRpcError::invalid_params(Some(
            "missing field".to_string(),
        ))),
        id: serde_json::json!("req-1"),
    };
    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("error"));
    assert!(json.contains("-32602"));
}

#[test]
fn test_json_rpc_parse_error_response() {
    let err = JsonRpcError::parse_error();
    let resp = JsonRpcResponse::error(serde_json::Value::Null, err);
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("-32700"));
    assert!(json.contains("Parse error"));
}

#[test]
fn test_json_rpc_invalid_request_response() {
    let err = JsonRpcError::invalid_request();
    let resp = JsonRpcResponse::error(serde_json::json!(1), err);
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("-32600"));
}
