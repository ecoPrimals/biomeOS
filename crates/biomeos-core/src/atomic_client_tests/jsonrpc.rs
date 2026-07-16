use super::super::atomic_client::*;
use biomeos_types::JsonRpcVersion;
use serde_json::{Value, json};

#[test]
fn test_jsonrpc_request_creation() {
    let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method.as_ref(), "test_method");
    assert_eq!(request.params.as_ref().unwrap()["key"], "value");
    assert!(
        request
            .id
            .as_ref()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
}
// ========================================================================
// JSON-RPC Request/Response Tests
// ========================================================================

#[test]
fn test_jsonrpc_request_auto_increment_id() {
    let req1 = JsonRpcRequest::new("method1", Value::Null);
    let req2 = JsonRpcRequest::new("method2", Value::Null);
    let req3 = JsonRpcRequest::new("method3", Value::Null);

    // IDs should be sequential
    let id1 = req1
        .id
        .as_ref()
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let id2 = req2
        .id
        .as_ref()
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let id3 = req3
        .id
        .as_ref()
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    assert!(id2 > id1);
    assert!(id3 > id2);
}

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest::new("test_method", json!({"key": "value"}));
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test_method"));
    assert!(json.contains("key"));
    assert!(json.contains("value"));
    assert!(json.contains("2.0"));
}

#[test]
fn test_jsonrpc_response_with_result() {
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: Some(json!({"status": "ok"})),
        error: None,
        id: serde_json::json!(1),
    };
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_jsonrpc_response_with_error() {
    let error = JsonRpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };
    let response = JsonRpcResponse {
        jsonrpc: JsonRpcVersion,
        result: None,
        error: Some(error),
        id: serde_json::json!(1),
    };
    assert!(response.result.is_none());
    assert!(response.error.is_some());
}

#[test]
fn test_jsonrpc_error_with_data() {
    let error = JsonRpcError {
        code: -32000,
        message: "Server error".to_string(),
        data: Some(json!({"details": "Something went wrong"})),
    };
    assert_eq!(error.code, -32000);
    assert!(error.data.is_some());
}
#[test]
fn test_jsonrpc_request_different_methods() {
    let req1 = JsonRpcRequest::new("method_a", Value::Null);
    let req2 = JsonRpcRequest::new("method_b", json!({"param": 123}));

    assert_eq!(req1.method.as_ref(), "method_a");
    assert_eq!(req2.method.as_ref(), "method_b");
    assert_eq!(req2.params.as_ref().unwrap()["param"], 123);
}

#[test]
fn test_jsonrpc_request_complex_params() {
    let params = json!({
        "nested": {
            "key": "value",
            "array": [1, 2, 3]
        },
        "number": 42
    });
    let request = JsonRpcRequest::new("complex_method", params);
    let p = request.params.as_ref().unwrap();
    assert_eq!(p["number"], 42);
    assert_eq!(p["nested"]["key"], "value");
}
#[test]
fn test_jsonrpc_request_null_params() {
    let request = JsonRpcRequest::new("method", Value::Null);
    assert!(request.params.as_ref().unwrap().is_null());
}

#[test]
fn test_jsonrpc_request_empty_object_params() {
    let request = JsonRpcRequest::new("method", json!({}));
    let params = request.params.as_ref().unwrap();
    assert!(params.is_object());
    assert!(params.as_object().unwrap().is_empty());
}

#[test]
fn test_jsonrpc_request_array_params() {
    let request = JsonRpcRequest::new("method", json!([1, 2, 3]));
    let params = request.params.as_ref().unwrap();
    assert!(params.is_array());
    assert_eq!(params.as_array().unwrap().len(), 3);
}
