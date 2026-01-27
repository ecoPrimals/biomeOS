//! Error types for graph operations.

use thiserror::Error;

/// Result type for graph operations.
pub type Result<T> = std::result::Result<T, GraphError>;

/// Errors that can occur during graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(String),

    /// TOML parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation error (structural issues)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Cyclic dependency detected
    #[error("Cyclic dependency: {0}")]
    CyclicDependency(String),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Capability not found
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
}
