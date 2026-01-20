//! Error types for Neural API Client
//!
//! Modern idiomatic Rust error handling using thiserror.

use std::io;
use thiserror::Error;

/// Neural API Client errors
#[derive(Debug, Error)]
pub enum NeuralApiError {
    /// Failed to connect to Neural API
    #[error("Failed to connect to Neural API: {0}")]
    ConnectionError(String),
    
    /// JSON-RPC error from server
    #[error("JSON-RPC error {code}: {message}")]
    RpcError {
        /// Error code (JSON-RPC standard codes)
        code: i32,
        /// Error message
        message: String,
    },
    
    /// Request timeout
    #[error("Request timeout after {0}ms")]
    Timeout(u64),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Neural API not found
    #[error("Neural API not found at {0}")]
    NotFound(String),
}

/// Standard JSON-RPC error codes
#[allow(dead_code)]
impl NeuralApiError {
    /// Parse error (-32700)
    pub const PARSE_ERROR: i32 = -32700;
    
    /// Invalid request (-32600)
    pub const INVALID_REQUEST: i32 = -32600;
    
    /// Method not found (-32601)
    pub const METHOD_NOT_FOUND: i32 = -32601;
    
    /// Invalid params (-32602)
    pub const INVALID_PARAMS: i32 = -32602;
    
    /// Internal error (-32603)
    pub const INTERNAL_ERROR: i32 = -32603;
}

