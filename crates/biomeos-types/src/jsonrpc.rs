// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC 2.0 wire types.
//!
//! Shared across all biomeOS crates to avoid duplicating the protocol format.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// JSON-RPC 2.0 protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// Zero-allocation JSON-RPC 2.0 version marker.
///
/// Always serializes as `"2.0"` and rejects any other value on deserialization.
/// Eliminates a `String` heap allocation per request/response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct JsonRpcVersion;

impl Serialize for JsonRpcVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(JSONRPC_VERSION)
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VersionVisitor;

        impl serde::de::Visitor<'_> for VersionVisitor {
            type Value = JsonRpcVersion;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "the string \"{JSONRPC_VERSION}\"")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<JsonRpcVersion, E> {
                if v == JSONRPC_VERSION {
                    Ok(JsonRpcVersion)
                } else {
                    Err(E::custom(format!(
                        "expected JSON-RPC version \"{JSONRPC_VERSION}\", got \"{v}\""
                    )))
                }
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<JsonRpcVersion, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl std::fmt::Display for JsonRpcVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(JSONRPC_VERSION)
    }
}

impl PartialEq<str> for JsonRpcVersion {
    fn eq(&self, other: &str) -> bool {
        other == JSONRPC_VERSION
    }
}

impl PartialEq<&str> for JsonRpcVersion {
    fn eq(&self, other: &&str) -> bool {
        *other == JSONRPC_VERSION
    }
}

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0"). Zero-allocation marker type.
    pub jsonrpc: JsonRpcVersion,
    /// Method name to invoke. Uses `Arc<str>` for zero-copy cloning on the hot path.
    pub method: Arc<str>,
    /// Method parameters (optional per JSON-RPC 2.0 spec).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request identifier (string, number, or null for notifications). Omitted for notifications.
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

/// Parsed JSON-RPC input — either a single request or a batch (Section 6).
#[derive(Debug, Clone)]
pub enum JsonRpcInput {
    /// A single JSON-RPC request object.
    Single(JsonRpcRequest),
    /// A batch of JSON-RPC requests (array).
    Batch(Vec<JsonRpcRequest>),
}

impl JsonRpcInput {
    /// Parse a JSON-RPC input line which may be a single object or a batch array.
    ///
    /// Per JSON-RPC 2.0 Section 6:
    /// - `{}` → `Single`
    /// - `[{}, {}]` → `Batch`
    /// - `[]` → error (empty batch is invalid)
    pub fn parse(input: &str) -> Result<Self, JsonRpcError> {
        let trimmed = input.trim();
        let value: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|_| JsonRpcError::parse_error())?;

        match value {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    return Err(JsonRpcError::invalid_request());
                }
                let mut requests = Vec::with_capacity(arr.len());
                for item in arr {
                    let req: JsonRpcRequest = serde_json::from_value(item)
                        .map_err(|_| JsonRpcError::invalid_request())?;
                    requests.push(req);
                }
                Ok(Self::Batch(requests))
            }
            serde_json::Value::Object(_) => {
                let req: JsonRpcRequest =
                    serde_json::from_value(value).map_err(|_| JsonRpcError::invalid_request())?;
                Ok(Self::Single(req))
            }
            _ => Err(JsonRpcError::invalid_request()),
        }
    }
}

impl JsonRpcRequest {
    fn next_request_id() -> serde_json::Value {
        static REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        serde_json::Value::Number(serde_json::Number::from(id))
    }

    /// Parse a JSON-RPC request from a string.
    pub fn parse(request_line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(request_line.trim())
    }

    /// Create a new request with an auto-incrementing id.
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::JsonRpcRequest;
    /// let req = JsonRpcRequest::new("method", serde_json::json!({}));
    /// assert_eq!(req.method.as_ref(), "method");
    /// assert!(req.id.is_some());
    /// ```
    pub fn new(method: impl AsRef<str>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            method: Arc::from(method.as_ref()),
            params: Some(params),
            id: Some(Self::next_request_id()),
        }
    }

    /// Serialize a newline-terminated request line without cloning `params`.
    ///
    /// Borrows `params` for serialization only — use on hot paths where callers
    /// hold a shared `serde_json::Value` reference.
    pub fn serialize_line(
        method: impl AsRef<str>,
        params: &serde_json::Value,
    ) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct BorrowedRequest<'a> {
            jsonrpc: JsonRpcVersion,
            method: &'a str,
            params: &'a serde_json::Value,
            id: serde_json::Value,
        }

        let mut line = serde_json::to_string(&BorrowedRequest {
            jsonrpc: JsonRpcVersion,
            method: method.as_ref(),
            params,
            id: Self::next_request_id(),
        })?;
        line.push('\n');
        Ok(line)
    }

    /// Create a notification (no id, no response expected).
    pub fn notification(method: impl AsRef<str>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            method: Arc::from(method.as_ref()),
            params: Some(params),
            id: None,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always "2.0"). Zero-allocation marker type.
    pub jsonrpc: JsonRpcVersion,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::JsonRpcResponse;
    /// let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"ok": true}));
    /// assert!(resp.result.is_some());
    /// assert!(resp.error.is_none());
    /// assert_eq!(resp.id, serde_json::json!(1));
    /// ```
    #[must_use]
    pub const fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Build an error response for the given request id.
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::{JsonRpcError, JsonRpcResponse};
    /// let err = JsonRpcError::method_not_found();
    /// let resp = JsonRpcResponse::error(serde_json::json!(1), err);
    /// assert!(resp.result.is_none());
    /// assert!(resp.error.is_some());
    /// assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    /// ```
    #[must_use]
    pub const fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
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
    #[must_use]
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Create an invalid request error (-32600).
    #[must_use]
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    /// Create a method not found error (-32601).
    #[must_use]
    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    /// Create an invalid params error (-32602).
    #[must_use]
    pub fn invalid_params(details: Option<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }

    /// Create an internal error (-32603).
    #[must_use]
    pub fn internal_error(details: Option<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }

    /// Create a permission denied error (-32001).
    ///
    /// Used by the `MethodGate` when a caller lacks scope for a protected method.
    #[must_use]
    pub fn permission_denied(method: &str) -> Self {
        Self {
            code: -32_001,
            message: format!("Permission denied: method '{method}' requires a capability token"),
            data: Some(serde_json::json!({"method": method})),
        }
    }

    /// Create an unauthorized error (-32000).
    ///
    /// Used when caller identity cannot be established at all.
    #[must_use]
    pub fn unauthorized(reason: &str) -> Self {
        Self {
            code: -32_000,
            message: format!("Unauthorized: {reason}"),
            data: None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[path = "jsonrpc_tests.rs"]
mod tests;
