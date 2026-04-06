// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! IPC resilience utilities absorbed from the ecosystem.
//!
//! Provides phase-tagged IPC errors and centralized JSON-RPC result extraction,
//! adopted from loamSpine, petalTongue, sweetGrass, primalSpring, and healthSpring.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::jsonrpc::JsonRpcResponse;

/// Phase of an IPC call where the error occurred.
///
/// Absorbed from loamSpine/petalTongue/sweetGrass — enables callers to decide
/// retry strategy based on *where* the failure happened rather than just *what*
/// the failure was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IpcErrorPhase {
    /// Socket/TCP connection could not be established.
    Connect,
    /// Connection established but writing the request failed.
    Write,
    /// Request sent but reading the response failed or timed out.
    Read,
    /// Response received but was not valid JSON.
    InvalidJson,
    /// Valid JSON but not a valid JSON-RPC 2.0 response.
    InvalidRpc,
    /// Valid JSON-RPC response with an `error` object.
    ApplicationError,
    /// The entire call exceeded its deadline.
    Timeout,
}

impl IpcErrorPhase {
    /// Whether this phase typically indicates a transient failure worth retrying.
    #[must_use]
    pub const fn is_retriable(self) -> bool {
        matches!(
            self,
            Self::Connect | Self::Write | Self::Read | Self::Timeout
        )
    }

    /// Whether a timeout is the likely root cause.
    #[must_use]
    pub const fn is_timeout_likely(self) -> bool {
        matches!(self, Self::Read | Self::Timeout)
    }

    /// Whether the remote explicitly reported "method not found" (-32601).
    #[must_use]
    pub const fn is_application_error(self) -> bool {
        matches!(self, Self::ApplicationError)
    }
}

impl fmt::Display for IpcErrorPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::Write => write!(f, "write"),
            Self::Read => write!(f, "read"),
            Self::InvalidJson => write!(f, "invalid-json"),
            Self::InvalidRpc => write!(f, "invalid-rpc"),
            Self::ApplicationError => write!(f, "application-error"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

/// Result of extracting the `result` field from a JSON-RPC response.
///
/// Centralizes the pattern used across 5+ springs where ad-hoc
/// `response["result"]` access was replaced with typed extraction.
///
/// # Errors
///
/// Returns a phase-tagged error when the response contains an `error` object
/// or when the `result` field is missing.
pub fn extract_rpc_result(
    response: &JsonRpcResponse,
    primal: &str,
) -> Result<serde_json::Value, RpcExtractionError> {
    if let Some(ref err) = response.error {
        return Err(RpcExtractionError {
            phase: IpcErrorPhase::ApplicationError,
            primal: primal.to_owned(),
            code: Some(err.code),
            message: err.message.clone(),
        });
    }

    response.result.clone().ok_or_else(|| RpcExtractionError {
        phase: IpcErrorPhase::InvalidRpc,
        primal: primal.to_owned(),
        code: None,
        message: "response missing both result and error fields".to_owned(),
    })
}

/// Extract only the error from a JSON-RPC response, if present.
///
/// Returns `None` for successful responses, `Some(error)` for error responses.
#[must_use]
pub fn extract_rpc_error(response: &JsonRpcResponse, primal: &str) -> Option<RpcExtractionError> {
    response.error.as_ref().map(|err| RpcExtractionError {
        phase: IpcErrorPhase::ApplicationError,
        primal: primal.to_owned(),
        code: Some(err.code),
        message: err.message.clone(),
    })
}

/// Phase-tagged RPC extraction error.
///
/// Carries the IPC phase, the primal that produced it, and the optional
/// JSON-RPC error code for downstream classification.
#[derive(Debug, Clone)]
pub struct RpcExtractionError {
    /// Phase of the IPC call where extraction failed.
    pub phase: IpcErrorPhase,
    /// Primal (or endpoint) identifier.
    pub primal: String,
    /// JSON-RPC error code, if the remote provided one.
    pub code: Option<i64>,
    /// Human-readable error message.
    pub message: String,
}

impl RpcExtractionError {
    /// Whether this is a JSON-RPC "method not found" (-32601).
    #[must_use]
    pub fn is_method_not_found(&self) -> bool {
        self.code == Some(-32601)
    }

    /// Whether this error is likely retriable.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        self.phase.is_retriable()
    }
}

impl fmt::Display for RpcExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(
                f,
                "RPC {} error from {}: [{}] {}",
                self.phase, self.primal, code, self.message
            )
        } else {
            write!(
                f,
                "RPC {} error from {}: {}",
                self.phase, self.primal, self.message
            )
        }
    }
}

impl std::error::Error for RpcExtractionError {}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use crate::jsonrpc::{JsonRpcError, JsonRpcResponse, JsonRpcVersion};

    #[test]
    fn extract_result_success() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
        let result = extract_rpc_result(&resp, "beardog").unwrap();
        assert_eq!(result, serde_json::json!({"status": "ok"}));
    }

    #[test]
    fn extract_result_error_response() {
        let resp = JsonRpcResponse::error(serde_json::json!(1), JsonRpcError::method_not_found());
        let err = extract_rpc_result(&resp, "songbird").unwrap_err();
        assert_eq!(err.phase, IpcErrorPhase::ApplicationError);
        assert!(err.is_method_not_found());
        assert_eq!(err.code, Some(-32601));
        assert!(err.to_string().contains("songbird"));
    }

    #[test]
    fn extract_result_missing_both_fields() {
        let resp = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: None,
            error: None,
            id: serde_json::json!(1),
        };
        let err = extract_rpc_result(&resp, "toadstool").unwrap_err();
        assert_eq!(err.phase, IpcErrorPhase::InvalidRpc);
        assert!(!err.is_method_not_found());
    }

    #[test]
    fn extract_error_none_for_success() {
        let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!(true));
        assert!(extract_rpc_error(&resp, "beardog").is_none());
    }

    #[test]
    fn extract_error_some_for_error() {
        let resp = JsonRpcResponse::error(
            serde_json::json!(1),
            JsonRpcError::internal_error(Some("crash".into())),
        );
        let err = extract_rpc_error(&resp, "nestgate").unwrap();
        assert_eq!(err.code, Some(-32603));
    }

    #[test]
    fn phase_is_retriable() {
        assert!(IpcErrorPhase::Connect.is_retriable());
        assert!(IpcErrorPhase::Write.is_retriable());
        assert!(IpcErrorPhase::Read.is_retriable());
        assert!(IpcErrorPhase::Timeout.is_retriable());
        assert!(!IpcErrorPhase::ApplicationError.is_retriable());
        assert!(!IpcErrorPhase::InvalidJson.is_retriable());
        assert!(!IpcErrorPhase::InvalidRpc.is_retriable());
    }

    #[test]
    fn phase_is_timeout_likely() {
        assert!(IpcErrorPhase::Read.is_timeout_likely());
        assert!(IpcErrorPhase::Timeout.is_timeout_likely());
        assert!(!IpcErrorPhase::Connect.is_timeout_likely());
    }

    #[test]
    fn phase_display() {
        assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
        assert_eq!(IpcErrorPhase::InvalidJson.to_string(), "invalid-json");
        assert_eq!(
            IpcErrorPhase::ApplicationError.to_string(),
            "application-error"
        );
    }

    #[test]
    fn extraction_error_display_with_code() {
        let err = RpcExtractionError {
            phase: IpcErrorPhase::ApplicationError,
            primal: "beardog".to_owned(),
            code: Some(-32601),
            message: "Method not found".to_owned(),
        };
        let s = err.to_string();
        assert!(s.contains("beardog"));
        assert!(s.contains("-32601"));
        assert!(s.contains("Method not found"));
    }

    #[test]
    fn extraction_error_display_without_code() {
        let err = RpcExtractionError {
            phase: IpcErrorPhase::InvalidRpc,
            primal: "toadstool".to_owned(),
            code: None,
            message: "missing result".to_owned(),
        };
        let s = err.to_string();
        assert!(s.contains("toadstool"));
        assert!(s.contains("missing result"));
        assert!(!s.contains('['));
    }

    #[test]
    fn extraction_error_is_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<RpcExtractionError>();
    }
}
