//! Error types for biomeOS manifest system

use thiserror::Error;

/// Manifest-specific errors
#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Parsing error: {0}")]
    ParsingError(String),
    
    #[error("Schema error: {0}")]
    SchemaError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

impl From<serde_yaml::Error> for ManifestError {
    fn from(err: serde_yaml::Error) -> Self {
        ManifestError::ParsingError(err.to_string())
    }
}

impl From<ManifestError> for biomeos_core::BiomeError {
    fn from(err: ManifestError) -> Self {
        biomeos_core::BiomeError::Generic(
            err.to_string(),
        )
    }
} 