// SPDX-License-Identifier: AGPL-3.0-only
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
