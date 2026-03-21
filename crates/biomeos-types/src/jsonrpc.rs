// SPDX-License-Identifier: AGPL-3.0-only
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
        static REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            jsonrpc: JsonRpcVersion,
            method: Arc::from(method.as_ref()),
            params: Some(params),
            id: Some(serde_json::Value::Number(serde_json::Number::from(id))),
        }
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
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
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
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
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
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_version_constant() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn test_jsonrpc_version_marker_display() {
        assert_eq!(JsonRpcVersion.to_string(), "2.0");
        assert!(JsonRpcVersion == "2.0");
    }

    #[test]
    fn request_parse_valid() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{"a":1},"id":1}"#;
        let req = JsonRpcRequest::parse(json).expect("parse");
        assert!(req.jsonrpc == "2.0");
        assert_eq!(req.method.as_ref(), "test");
        assert_eq!(req.params, Some(serde_json::json!({"a": 1})));
        assert_eq!(req.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn test_jsonrpc_request_parse_valid() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{"a":1},"id":1}"#;
        let req = JsonRpcRequest::parse(json).unwrap();
        assert!(req.jsonrpc == "2.0");
        assert_eq!(req.method.as_ref(), "test");
        assert!(req.params.is_some());
        assert!(req.id.is_some());
    }

    #[test]
    fn request_parse_trims_whitespace() {
        let json = "  \n  {\"jsonrpc\":\"2.0\",\"method\":\"m\",\"id\":1}  ";
        let req = JsonRpcRequest::parse(json).expect("parse");
        assert_eq!(req.method.as_ref(), "m");
    }

    #[test]
    fn request_parse_invalid_returns_error() {
        let err = JsonRpcRequest::parse("not json").unwrap_err();
        assert!(err.to_string().contains("expected"));
    }

    #[test]
    fn request_new_has_id_and_params() {
        let req = JsonRpcRequest::new("method", serde_json::json!({"x": 42}));
        assert_eq!(req.method.as_ref(), "method");
        assert_eq!(req.params, Some(serde_json::json!({"x": 42})));
        assert!(req.id.is_some());
        assert_eq!(req.jsonrpc, JsonRpcVersion);
    }

    #[test]
    fn request_notification_has_no_id() {
        let req = JsonRpcRequest::notification("notify", serde_json::json!({}));
        assert_eq!(req.method.as_ref(), "notify");
        assert_eq!(req.id, None);
    }

    #[test]
    fn test_jsonrpc_request_parse_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"notify","params":{}}"#;
        let req = JsonRpcRequest::parse(json).unwrap();
        assert_eq!(req.method.as_ref(), "notify");
        assert!(req.id.is_none() || req.id == Some(serde_json::Value::Null));
    }

    #[test]
    fn test_jsonrpc_version_rejects_wrong_version() {
        let json = r#"{"jsonrpc":"1.0","method":"test","id":1}"#;
        let result = JsonRpcRequest::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_jsonrpc_request_parse_invalid_json() {
        let result = JsonRpcRequest::parse("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_jsonrpc_request_parse_trimmed() {
        let json = "  \n  {\"jsonrpc\":\"2.0\",\"method\":\"m\",\"id\":1}  ";
        let req = JsonRpcRequest::parse(json).unwrap();
        assert_eq!(req.method.as_ref(), "m");
    }

    #[test]
    fn test_jsonrpc_request_new() {
        let req = JsonRpcRequest::new("method", serde_json::json!({"x": 1}));
        assert!(req.jsonrpc == "2.0");
        assert_eq!(req.method.as_ref(), "method");
        assert_eq!(req.params, Some(serde_json::json!({"x": 1})));
        assert!(req.id.is_some());
    }

    #[test]
    fn test_jsonrpc_request_notification() {
        let req = JsonRpcRequest::notification("notify", serde_json::json!({}));
        assert!(req.jsonrpc == "2.0");
        assert_eq!(req.method.as_ref(), "notify");
        assert!(req.id.is_none());
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
    fn test_jsonrpc_response_success() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"result": "ok"}));
        assert!(resp.jsonrpc == "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.id, serde_json::json!(1));
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
    fn test_jsonrpc_response_error() {
        let err = JsonRpcError::method_not_found();
        let resp = JsonRpcResponse::error(serde_json::json!(1), err);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn error_parse_error() {
        let e = JsonRpcError::parse_error();
        assert_eq!(e.code, -32700);
        assert_eq!(e.message, "Parse error");
        assert!(e.data.is_none());
    }

    #[test]
    fn test_jsonrpc_error_parse_error() {
        let err = JsonRpcError::parse_error();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("Parse"));
    }

    #[test]
    fn error_invalid_request() {
        let e = JsonRpcError::invalid_request();
        assert_eq!(e.code, -32600);
        assert_eq!(e.message, "Invalid Request");
    }

    #[test]
    fn test_jsonrpc_error_invalid_request() {
        let err = JsonRpcError::invalid_request();
        assert_eq!(err.code, -32600);
    }

    #[test]
    fn error_method_not_found() {
        let e = JsonRpcError::method_not_found();
        assert_eq!(e.code, -32601);
        assert_eq!(e.message, "Method not found");
    }

    #[test]
    fn test_jsonrpc_error_method_not_found() {
        let err = JsonRpcError::method_not_found();
        assert_eq!(err.code, -32601);
    }

    #[test]
    fn error_invalid_params_with_details() {
        let e = JsonRpcError::invalid_params(Some("bad param".into()));
        assert_eq!(e.code, -32602);
        assert_eq!(e.message, "Invalid params");
        assert_eq!(e.data, Some(serde_json::json!({"details": "bad param"})));
    }

    #[test]
    fn test_jsonrpc_error_invalid_params_none() {
        let err = JsonRpcError::invalid_params(None);
        assert_eq!(err.code, -32602);
        assert!(err.data.is_none());
    }

    #[test]
    fn error_invalid_params_without_details() {
        let e = JsonRpcError::invalid_params(None);
        assert!(e.data.is_none());
    }

    #[test]
    fn test_jsonrpc_error_invalid_params_some() {
        let err = JsonRpcError::invalid_params(Some("missing field".into()));
        assert_eq!(err.code, -32602);
        assert!(err.data.is_some());
    }

    #[test]
    fn error_internal_error_with_details() {
        let e = JsonRpcError::internal_error(Some("crash".into()));
        assert_eq!(e.code, -32603);
        assert_eq!(e.data, Some(serde_json::json!({"details": "crash"})));
    }

    #[test]
    fn test_jsonrpc_error_internal_error_none() {
        let err = JsonRpcError::internal_error(None);
        assert_eq!(err.code, -32603);
    }

    #[test]
    fn test_jsonrpc_error_internal_error_some() {
        let err = JsonRpcError::internal_error(Some("panic".into()));
        assert_eq!(err.code, -32603);
        assert!(err.data.is_some());
    }

    #[test]
    fn roundtrip_serialize_request() {
        let req = JsonRpcRequest::new("ping", serde_json::json!({}));
        let s = serde_json::to_string(&req).expect("serialize");
        let parsed = JsonRpcRequest::parse(&s).expect("parse");
        assert_eq!(parsed.method.as_ref(), req.method.as_ref());
    }

    #[test]
    fn test_jsonrpc_request_serde_roundtrip() {
        let req = JsonRpcRequest::new("test", serde_json::json!({}));
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.method.as_ref(), parsed.method.as_ref());
    }

    #[test]
    fn test_jsonrpc_response_serde_roundtrip() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"data": true}));
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.result.is_some());
    }

    #[test]
    fn test_jsonrpc_error_serde_roundtrip() {
        let err = JsonRpcError::method_not_found();
        let json = serde_json::to_string(&err).unwrap();
        let parsed: JsonRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(err.code, parsed.code);
        assert_eq!(err.message, parsed.message);
    }

    #[test]
    fn test_jsonrpc_error_debug() {
        let err = JsonRpcError::parse_error();
        let s = format!("{:?}", err);
        assert!(s.contains("-32700") || s.contains("Parse"));
    }

    #[test]
    fn jsonrpc_input_parse_single_object() {
        let input = r#"{"jsonrpc":"2.0","method":"test","id":1}"#;
        let parsed = JsonRpcInput::parse(input).expect("parse single");
        match parsed {
            JsonRpcInput::Single(req) => assert_eq!(req.method.as_ref(), "test"),
            JsonRpcInput::Batch(_) => panic!("expected Single"),
        }
    }

    #[test]
    fn jsonrpc_input_parse_batch_array() {
        let input =
            r#"[{"jsonrpc":"2.0","method":"a","id":1},{"jsonrpc":"2.0","method":"b","id":2}]"#;
        let parsed = JsonRpcInput::parse(input).expect("parse batch");
        match parsed {
            JsonRpcInput::Batch(reqs) => {
                assert_eq!(reqs.len(), 2);
                assert_eq!(reqs[0].method.as_ref(), "a");
                assert_eq!(reqs[1].method.as_ref(), "b");
            }
            JsonRpcInput::Single(_) => panic!("expected Batch"),
        }
    }

    #[test]
    fn jsonrpc_input_parse_empty_array_is_invalid() {
        let input = "[]";
        let err = JsonRpcInput::parse(input).unwrap_err();
        assert_eq!(err.code, -32600, "empty array should be invalid request");
    }

    #[test]
    fn jsonrpc_input_parse_invalid_json() {
        let err = JsonRpcInput::parse("not json").unwrap_err();
        assert_eq!(err.code, -32700, "bad json should be parse error");
    }

    #[test]
    fn jsonrpc_input_parse_primitive_is_invalid() {
        let err = JsonRpcInput::parse("42").unwrap_err();
        assert_eq!(err.code, -32600);
    }
}
