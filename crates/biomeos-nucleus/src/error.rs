// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Error types for NUCLEUS
//!
//! Following deep debt principles:
//! - Clear, descriptive error messages
//! - Contextual information preserved
//! - Easy to pattern match

use std::path::PathBuf;

/// Result type for NUCLEUS operations
pub type Result<T> = std::result::Result<T, Error>;

/// NUCLEUS error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Primal discovery failed (Layer 1: Songbird)
    #[error("Discovery failed: {reason}")]
    DiscoveryFailed {
        /// Reason for failure
        reason: String,
        /// Primal capability being discovered
        capability: Option<String>,
    },

    /// Identity verification failed (Layer 2: `BearDog`)
    #[error("Identity verification failed for {primal}: {reason}")]
    IdentityVerificationFailed {
        /// Primal that failed verification
        primal: String,
        /// Reason for failure
        reason: String,
    },

    /// Capability verification failed (Layer 3: Direct query)
    #[error("Capability verification failed: expected {expected:?}, got {actual:?}")]
    CapabilityMismatch {
        /// Expected capabilities
        expected: Vec<String>,
        /// Actual capabilities
        actual: Vec<String>,
    },

    /// Trust evaluation failed (Layer 4: `BearDog`)
    #[error("Trust evaluation failed: {reason}")]
    TrustEvaluationFailed {
        /// Reason for failure
        reason: String,
        /// Trust level achieved (if any)
        achieved_level: Option<String>,
    },

    /// Unix socket connection failed
    #[error("Failed to connect to socket {path}: {source}")]
    SocketConnectionFailed {
        /// Socket path
        path: PathBuf,
        /// Underlying error
        source: std::io::Error,
    },

    /// JSON-RPC request failed
    #[error("JSON-RPC request failed: {method} - {reason}")]
    JsonRpcFailed {
        /// RPC method
        method: String,
        /// Reason for failure
        reason: String,
    },

    /// Primal not found
    #[error("No primal found with capability: {capability}")]
    PrimalNotFound {
        /// Capability being searched for
        capability: String,
    },

    /// Invalid primal response
    #[error("Invalid response from {primal}: {reason}")]
    InvalidResponse {
        /// Primal name
        primal: String,
        /// Reason
        reason: String,
    },

    /// Timeout
    #[error("Operation timed out after {seconds}s: {operation}")]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration
        seconds: u64,
    },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error (for wrapping other errors)
    #[error("NUCLEUS error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Create a discovery failed error
    pub fn discovery_failed(reason: impl Into<String>, capability: Option<String>) -> Self {
        Self::DiscoveryFailed {
            reason: reason.into(),
            capability,
        }
    }

    /// Create an identity verification failed error
    pub fn identity_verification_failed(
        primal: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::IdentityVerificationFailed {
            primal: primal.into(),
            reason: reason.into(),
        }
    }

    /// Create a capability mismatch error
    #[must_use]
    pub const fn capability_mismatch(expected: Vec<String>, actual: Vec<String>) -> Self {
        Self::CapabilityMismatch { expected, actual }
    }

    /// Create a trust evaluation failed error
    pub fn trust_evaluation_failed(
        reason: impl Into<String>,
        achieved_level: Option<String>,
    ) -> Self {
        Self::TrustEvaluationFailed {
            reason: reason.into(),
            achieved_level,
        }
    }

    /// Create a socket connection failed error
    pub fn socket_connection_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::SocketConnectionFailed {
            path: path.into(),
            source,
        }
    }

    /// Create a JSON-RPC failed error
    pub fn jsonrpc_failed(method: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::JsonRpcFailed {
            method: method.into(),
            reason: reason.into(),
        }
    }

    /// Create a primal not found error
    pub fn primal_not_found(capability: impl Into<String>) -> Self {
        Self::PrimalNotFound {
            capability: capability.into(),
        }
    }

    /// Create an invalid response error
    pub fn invalid_response(primal: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidResponse {
            primal: primal.into(),
            reason: reason.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, seconds: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            seconds,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_failed() {
        let e = Error::discovery_failed("socket not found", Some("security".to_string()));
        let msg = e.to_string();
        assert!(msg.contains("Discovery failed"));
        assert!(msg.contains("socket not found"));
    }

    #[test]
    fn test_identity_verification_failed() {
        let e = Error::identity_verification_failed("beardog", "invalid signature");
        let msg = e.to_string();
        assert!(msg.contains("beardog"));
        assert!(msg.contains("invalid signature"));
    }

    #[test]
    fn test_capability_mismatch() {
        let e =
            Error::capability_mismatch(vec!["security".to_string()], vec!["discovery".to_string()]);
        let msg = e.to_string();
        assert!(msg.contains("Capability verification failed"));
    }

    #[test]
    fn test_trust_evaluation_failed() {
        let e = Error::trust_evaluation_failed("no trust", Some("partial".to_string()));
        let msg = e.to_string();
        assert!(msg.contains("Trust evaluation failed"));
    }

    #[test]
    fn test_socket_connection_failed() {
        let path = PathBuf::from("/tmp/missing.sock");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let e = Error::socket_connection_failed(path, io_err);
        let msg = e.to_string();
        assert!(msg.contains("/tmp/missing.sock"));
    }

    #[test]
    fn test_jsonrpc_failed() {
        let e = Error::jsonrpc_failed("health.check", "timeout");
        let msg = e.to_string();
        assert!(msg.contains("health.check"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_primal_not_found() {
        let e = Error::primal_not_found("security");
        let msg = e.to_string();
        assert!(msg.contains("security"));
    }

    #[test]
    fn test_invalid_response() {
        let e = Error::invalid_response("songbird", "malformed JSON");
        let msg = e.to_string();
        assert!(msg.contains("songbird"));
        assert!(msg.contains("malformed JSON"));
    }

    #[test]
    fn test_timeout() {
        let e = Error::timeout("discovery", 30);
        let msg = e.to_string();
        assert!(msg.contains("30"));
        assert!(msg.contains("discovery"));
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        let e: Error = json_err.unwrap_err().into();
        let msg = e.to_string();
        assert!(msg.contains("Serialization") || msg.contains("error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: Error = io_err.into();
        let msg = e.to_string();
        assert!(msg.contains("IO") || msg.contains("denied"));
    }

    #[test]
    fn test_result_type() {
        let _: Result<()> = Ok(());
        let err: Result<()> = Err(Error::primal_not_found("test"));
        assert!(err.is_err());
    }
}
