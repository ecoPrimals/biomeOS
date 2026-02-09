//! JSON-RPC 2.0 protocol types and utilities
//!
//! Provides types for parsing and handling JSON-RPC 2.0 requests and responses.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: u64,
}

impl JsonRpcRequest {
    /// Parse a JSON-RPC request from a string
    ///
    /// Named `parse` to avoid confusion with `std::str::FromStr::from_str`
    pub fn parse(request_line: &str) -> Result<Self> {
        serde_json::from_str(request_line.trim()).context("Failed to parse JSON-RPC request")
    }
}

/// Create a JSON-RPC error response
pub fn error_response(code: i32, message: String, id: Option<u64>) -> Value {
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
pub fn success_response(result: Value, id: u64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

/// Create a JSON-RPC internal error response
pub fn internal_error_response(error: &anyhow::Error, id: Option<u64>) -> Value {
    error_response(-32603, format!("Internal error: {}", error), id)
}

/// Create a JSON-RPC method not found error response
pub fn method_not_found_response(method: &str, id: u64) -> Value {
    error_response(-32601, format!("Method not found: {}", method), Some(id))
}
