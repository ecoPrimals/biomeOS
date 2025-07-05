//! Error types for biomeOS

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for biomeOS operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BiomeError {
    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
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
    
    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    /// IO error
    #[error("IO error: {message}")]
    IoError { message: String },
    
    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
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
