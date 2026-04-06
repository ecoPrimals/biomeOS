// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Operation Types for Error Context
//!
//! This module contains various operation type enums that provide
//! context for different categories of errors in the biomeOS system.

use serde::{Deserialize, Serialize};

/// Network operations for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    /// Establishing a connection
    Connect,
    /// Sending data
    Send,
    /// Receiving data
    Receive,
    /// Disconnecting
    Disconnect,
    /// DNS resolution
    Resolve,
    /// TLS/protocol handshake
    Handshake,
}

/// Security violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    /// Authentication failed
    AuthenticationFailure,
    /// Authorization denied
    AuthorizationDenied,
    /// Token has expired
    TokenExpired,
    /// Invalid credentials provided
    InvalidCredentials,
    /// Access denied to resource
    AccessDenied,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Suspicious activity detected
    SuspiciousActivity,
}

/// Resource operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceOperation {
    /// Allocating resources
    Allocate,
    /// Deallocating resources
    Deallocate,
    /// Monitoring resource usage
    Monitor,
    /// Scaling resources
    Scale,
    /// Setting resource limits
    Limit,
}

/// Data operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    /// Reading data
    Read,
    /// Writing data
    Write,
    /// Updating existing data
    Update,
    /// Deleting data
    Delete,
    /// Validating data
    Validate,
    /// Transforming data
    Transform,
    /// Migrating data
    Migrate,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn network_operation_debug() {
        let ops = [
            NetworkOperation::Connect,
            NetworkOperation::Send,
            NetworkOperation::Receive,
            NetworkOperation::Disconnect,
            NetworkOperation::Resolve,
            NetworkOperation::Handshake,
        ];
        for op in ops {
            let _ = format!("{:?}", op);
        }
    }

    #[test]
    fn security_violation_type_debug() {
        let types = [
            SecurityViolationType::AuthenticationFailure,
            SecurityViolationType::AuthorizationDenied,
            SecurityViolationType::TokenExpired,
            SecurityViolationType::InvalidCredentials,
            SecurityViolationType::AccessDenied,
            SecurityViolationType::RateLimitExceeded,
            SecurityViolationType::SuspiciousActivity,
        ];
        for t in types {
            let _ = format!("{:?}", t);
        }
    }

    #[test]
    fn resource_operation_debug() {
        let ops = [
            ResourceOperation::Allocate,
            ResourceOperation::Deallocate,
            ResourceOperation::Monitor,
            ResourceOperation::Scale,
            ResourceOperation::Limit,
        ];
        for op in ops {
            let _ = format!("{:?}", op);
        }
    }

    #[test]
    fn data_operation_debug() {
        let ops = [
            DataOperation::Read,
            DataOperation::Write,
            DataOperation::Update,
            DataOperation::Delete,
            DataOperation::Validate,
            DataOperation::Transform,
            DataOperation::Migrate,
        ];
        for op in ops {
            let _ = format!("{:?}", op);
        }
    }

    #[test]
    fn network_operation_serialize_roundtrip() {
        let op = NetworkOperation::Connect;
        let json = serde_json::to_string(&op).expect("serialize");
        let parsed: NetworkOperation = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, NetworkOperation::Connect));
    }

    #[test]
    fn security_violation_type_serialize_roundtrip() {
        let t = SecurityViolationType::TokenExpired;
        let json = serde_json::to_string(&t).expect("serialize");
        let parsed: SecurityViolationType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, SecurityViolationType::TokenExpired));
    }
}
