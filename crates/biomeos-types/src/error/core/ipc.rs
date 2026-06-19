// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use thiserror::Error;

/// Structured error type for IPC communication failures.
///
/// Follows the healthSpring `SendError` pattern for typed error handling.
#[derive(Debug, Error)]
pub enum IpcError {
    /// Connection to primal failed
    #[error("connection failed to {primal}: {source}")]
    ConnectionFailed {
        /// Primal or endpoint identifier
        primal: String,
        /// Underlying connection error
        source: anyhow::Error,
    },
    /// Request timed out
    #[error("request to {primal} timed out after {timeout_ms}ms")]
    Timeout {
        /// Primal or endpoint identifier
        primal: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },
    /// JSON-RPC protocol error from the remote side
    #[error("JSON-RPC error {code} from {primal}: {message}")]
    JsonRpcError {
        /// Primal or endpoint identifier
        primal: String,
        /// JSON-RPC error code
        code: i32,
        /// Error message from remote
        message: String,
    },
    /// Response missing required `result` field
    #[error("missing result in response from {primal}")]
    MissingResult {
        /// Primal or endpoint identifier
        primal: String,
    },
    /// Serialization/deserialization error
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl IpcError {
    /// Whether this error is a method-not-found (-32601) — caller may want to try another primal
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::JsonRpcError { code: -32601, .. })
    }

    /// Whether this error is a timeout — caller may want to retry
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn ipc_error_is_method_not_found() {
        let err = IpcError::JsonRpcError {
            primal: "beardog".to_string(),
            code: -32601,
            message: "Method not found".to_string(),
        };
        assert!(err.is_method_not_found());
    }

    #[test]
    fn ipc_error_is_method_not_found_false_for_other_codes() {
        let err = IpcError::JsonRpcError {
            primal: "beardog".to_string(),
            code: -32600,
            message: "Invalid request".to_string(),
        };
        assert!(!err.is_method_not_found());
    }

    #[test]
    fn ipc_error_is_timeout() {
        let err = IpcError::Timeout {
            primal: "beardog".to_string(),
            timeout_ms: 5000,
        };
        assert!(err.is_timeout());
    }

    #[test]
    fn ipc_error_is_timeout_false_for_other_variants() {
        let err = IpcError::MissingResult {
            primal: "beardog".to_string(),
        };
        assert!(!err.is_timeout());
    }

    #[test]
    fn ipc_error_serialization_from_serde() {
        let json = "invalid json {{{";
        let err: IpcError = serde_json::from_str::<serde_json::Value>(json)
            .unwrap_err()
            .into();
        assert!(matches!(err, IpcError::Serialization(_)));
    }
}
