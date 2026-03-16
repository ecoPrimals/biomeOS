// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! JSON-RPC 2.0 protocol types and utilities
//!
//! Provides types for parsing and handling JSON-RPC 2.0 requests and responses.

use serde_json::{json, Value};

pub use biomeos_types::JsonRpcRequest;

/// Create a JSON-RPC error response
pub fn error_response(code: i32, message: String, id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        },
        "id": id
    })
}

/// Create a JSON-RPC success response
pub fn success_response(result: Value, id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

/// Create a JSON-RPC internal error response
pub fn internal_error_response(error: &anyhow::Error, id: Option<Value>) -> Value {
    error_response(-32603, format!("Internal error: {error}"), id)
}

/// Create a JSON-RPC method not found error response
pub fn method_not_found_response(method: &str, id: Value) -> Value {
    error_response(-32601, format!("Method not found: {method}"), Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_parse_valid() {
        let json = r#"{"jsonrpc":"2.0","method":"test.method","params":{"a":1},"id":42}"#;
        let req = JsonRpcRequest::parse(json).expect("parse should succeed");
        assert_eq!(req.method.as_ref(), "test.method");
        assert_eq!(req.id.as_ref().and_then(|v| v.as_u64()).unwrap(), 42);
        assert_eq!(req.params.as_ref().unwrap()["a"], 1);
    }

    #[test]
    fn test_jsonrpc_request_parse_with_whitespace() {
        let json = "  \n  {\"jsonrpc\":\"2.0\",\"method\":\"foo\",\"id\":1}  ";
        let req = JsonRpcRequest::parse(json).expect("parse should succeed");
        assert_eq!(req.method.as_ref(), "foo");
        assert_eq!(req.id.as_ref().and_then(|v| v.as_u64()).unwrap(), 1);
        assert!(req.params.is_none());
    }

    #[test]
    fn test_jsonrpc_request_parse_null_params() {
        let json = r#"{"jsonrpc":"2.0","method":"bar","params":null,"id":0}"#;
        let req = JsonRpcRequest::parse(json).expect("parse should succeed");
        assert_eq!(req.method.as_ref(), "bar");
        assert_eq!(req.id.as_ref().and_then(|v| v.as_u64()).unwrap(), 0);
    }

    #[test]
    fn test_jsonrpc_request_parse_invalid_json() {
        let err = JsonRpcRequest::parse("{invalid").expect_err("parse should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("parse")
                || msg.contains("JSON")
                || msg.contains("key")
                || msg.contains("error"),
            "error: {msg}"
        );
    }

    #[test]
    fn test_jsonrpc_request_parse_empty_string() {
        let err = JsonRpcRequest::parse("").expect_err("parse should fail");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_jsonrpc_request_parse_missing_method() {
        let json = r#"{"jsonrpc":"2.0","id":1}"#;
        let err = JsonRpcRequest::parse(json).expect_err("parse should fail without method");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_error_response() {
        let resp = error_response(-32600, "Parse error".to_string(), Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["error"]["code"], -32600);
        assert_eq!(resp["error"]["message"], "Parse error");
        assert_eq!(resp["id"], 1);
    }

    #[test]
    fn test_error_response_null_id() {
        let resp = error_response(-32603, "Internal".to_string(), None);
        assert!(resp["id"].is_null());
    }

    #[test]
    fn test_success_response() {
        let result = serde_json::json!({"ok": true});
        let resp = success_response(result.clone(), json!(99));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["result"], result);
        assert_eq!(resp["id"], 99);
    }

    #[test]
    fn test_internal_error_response() {
        let err = anyhow::anyhow!("Something broke");
        let resp = internal_error_response(&err, Some(json!(5)));
        assert_eq!(resp["error"]["code"], -32603);
        assert!(resp["error"]["message"]
            .as_str()
            .expect("message")
            .contains("Something broke"));
        assert_eq!(resp["id"], 5);
    }

    #[test]
    fn test_internal_error_response_no_id() {
        let err = anyhow::anyhow!("Fail");
        let resp = internal_error_response(&err, None);
        assert!(resp["id"].is_null());
    }

    #[test]
    fn test_method_not_found_response() {
        let resp = method_not_found_response("unknown.method", json!(123));
        assert_eq!(resp["error"]["code"], -32601);
        assert_eq!(resp["error"]["message"], "Method not found: unknown.method");
        assert_eq!(resp["id"], 123);
    }
}
