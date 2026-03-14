// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! JSON-RPC 2.0 wire types.
//!
//! Shared across all biomeOS crates to avoid duplicating the protocol format.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0").
    pub jsonrpc: String,
    /// Method name to invoke.
    pub method: String,
    /// Method parameters (optional per JSON-RPC 2.0 spec).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request identifier (string, number, or null for notifications). Omitted for notifications.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Parse a JSON-RPC request from a string.
    pub fn parse(request_line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(request_line.trim())
    }

    /// Create a new request with an auto-incrementing id.
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        static REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.into(),
            params: Some(params),
            id: Some(serde_json::Value::Number(serde_json::Number::from(id))),
        }
    }

    /// Create a notification (no id, no response expected).
    pub fn notification(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.into(),
            params: Some(params),
            id: None,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always "2.0").
    pub jsonrpc: String,
    /// Successful result payload (mutually exclusive with `error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error payload (mutually exclusive with `result`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request identifier echoed back.
    pub id: serde_json::Value,
}

impl JsonRpcResponse {
    /// Build a success response for the given request id.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Build an error response for the given request id.
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code.
    pub code: i64,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a parse error (-32700).
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Create an invalid request error (-32600).
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    /// Create a method not found error (-32601).
    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    /// Create an invalid params error (-32602).
    pub fn invalid_params(details: Option<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }

    /// Create an internal error (-32603).
    pub fn internal_error(details: Option<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_version_constant() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn request_parse_valid() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{"a":1},"id":1}"#;
        let req = JsonRpcRequest::parse(json).expect("parse");
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test");
        assert_eq!(req.params, Some(serde_json::json!({"a": 1})));
        assert_eq!(req.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn request_parse_trims_whitespace() {
        let json = "  \n  {\"jsonrpc\":\"2.0\",\"method\":\"m\",\"id\":1}  ";
        let req = JsonRpcRequest::parse(json).expect("parse");
        assert_eq!(req.method, "m");
    }

    #[test]
    fn request_parse_invalid_returns_error() {
        let err = JsonRpcRequest::parse("not json").unwrap_err();
        assert!(err.to_string().contains("expected"));
    }

    #[test]
    fn request_new_has_id_and_params() {
        let req = JsonRpcRequest::new("method", serde_json::json!({"x": 42}));
        assert_eq!(req.method, "method");
        assert_eq!(req.params, Some(serde_json::json!({"x": 42})));
        assert!(req.id.is_some());
    }

    #[test]
    fn request_notification_has_no_id() {
        let req = JsonRpcRequest::notification("notify", serde_json::json!({}));
        assert_eq!(req.method, "notify");
        assert_eq!(req.id, None);
    }

    #[test]
    fn response_success() {
        let id = serde_json::json!(1);
        let result = serde_json::json!({"ok": true});
        let resp = JsonRpcResponse::success(id.clone(), result.clone());
        assert_eq!(resp.result, Some(result));
        assert!(resp.error.is_none());
        assert_eq!(resp.id, id);
    }

    #[test]
    fn response_error() {
        let id = serde_json::json!(2);
        let err = JsonRpcError::method_not_found();
        let resp = JsonRpcResponse::error(id.clone(), err);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
        assert_eq!(resp.id, id);
    }

    #[test]
    fn error_parse_error() {
        let e = JsonRpcError::parse_error();
        assert_eq!(e.code, -32700);
        assert_eq!(e.message, "Parse error");
        assert!(e.data.is_none());
    }

    #[test]
    fn error_invalid_request() {
        let e = JsonRpcError::invalid_request();
        assert_eq!(e.code, -32600);
        assert_eq!(e.message, "Invalid Request");
    }

    #[test]
    fn error_method_not_found() {
        let e = JsonRpcError::method_not_found();
        assert_eq!(e.code, -32601);
        assert_eq!(e.message, "Method not found");
    }

    #[test]
    fn error_invalid_params_with_details() {
        let e = JsonRpcError::invalid_params(Some("bad param".into()));
        assert_eq!(e.code, -32602);
        assert_eq!(e.message, "Invalid params");
        assert_eq!(e.data, Some(serde_json::json!({"details": "bad param"})));
    }

    #[test]
    fn error_invalid_params_without_details() {
        let e = JsonRpcError::invalid_params(None);
        assert!(e.data.is_none());
    }

    #[test]
    fn error_internal_error_with_details() {
        let e = JsonRpcError::internal_error(Some("crash".into()));
        assert_eq!(e.code, -32603);
        assert_eq!(e.data, Some(serde_json::json!({"details": "crash"})));
    }

    #[test]
    fn roundtrip_serialize_request() {
        let req = JsonRpcRequest::new("ping", serde_json::json!({}));
        let s = serde_json::to_string(&req).expect("serialize");
        let parsed = JsonRpcRequest::parse(&s).expect("parse");
        assert_eq!(parsed.method, req.method);
    }
}
