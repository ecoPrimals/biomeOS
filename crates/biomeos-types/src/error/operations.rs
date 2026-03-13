// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
