//! Error types for biomeOS

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for biomeOS operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BiomeError {
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    /// Configuration error (simple)
    #[error("Config error: {0}")]
    Config(String),
    
    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    /// Primal error
    #[error("Primal error: {message}")]
    PrimalError { message: String },
    
    /// Primal not found
    #[error("Primal not found: {0}")]
    PrimalNotFound(String),
    
    /// Security error
    #[error("Security error: {message}")]
    SecurityError { message: String },
    
    /// Security error (simple)
    #[error("Security error: {0}")]
    Security(String),
    
    /// Sovereignty violation
    #[error("Sovereignty violation: {0}")]
    SovereigntyViolation(String),
    
    /// Vendor lock detected
    #[error("Vendor lock detected: {0}")]
    VendorLock(String),
    
    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    /// IO error
    #[error("IO error: {message}")]
    IoError { message: String },
    
    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    /// Runtime error for toadStool integration
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    /// Validation error for manifest validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Generic error
    #[error("Error: {message}")]
    Generic { message: String },
}

impl From<std::io::Error> for BiomeError {
    fn from(err: std::io::Error) -> Self {
        BiomeError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for BiomeError {
    fn from(err: serde_json::Error) -> Self {
        BiomeError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for BiomeError {
    fn from(err: serde_yaml::Error) -> Self {
        BiomeError::SerializationError {
            message: err.to_string(),
        }
    }
}

/// Result type for biomeOS operations
pub type BiomeResult<T> = Result<T, BiomeError>;
