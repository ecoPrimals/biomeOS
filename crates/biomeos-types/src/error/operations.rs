//! Operation Types for Error Context
//!
//! This module contains various operation type enums that provide
//! context for different categories of errors in the biomeOS system.

use serde::{Deserialize, Serialize};

/// Network operations for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    Connect,
    Send,
    Receive,
    Disconnect,
    Resolve,
    Handshake,
}

/// Security violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    AuthenticationFailure,
    AuthorizationDenied,
    TokenExpired,
    InvalidCredentials,
    AccessDenied,
    RateLimitExceeded,
    SuspiciousActivity,
}

/// Resource operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceOperation {
    Allocate,
    Deallocate,
    Monitor,
    Scale,
    Limit,
}

/// Data operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    Read,
    Write,
    Update,
    Delete,
    Validate,
    Transform,
    Migrate,
}
