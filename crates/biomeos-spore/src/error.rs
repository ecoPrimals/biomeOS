//! Error types for spore operations

use std::path::PathBuf;
use thiserror::Error;

/// Result type for spore operations
pub type SporeResult<T> = Result<T, SporeError>;

/// Errors that can occur during spore operations
#[derive(Error, Debug)]
pub enum SporeError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Seed file not found
    #[error("Seed file not found: {0}")]
    SeedFileNotFound(PathBuf),

    /// Invalid seed length
    #[error("Invalid seed length: expected {expected}, found {found}")]
    InvalidSeedLength { expected: u64, found: u64 },

    /// Invalid path (non-UTF8)
    #[error("Invalid path: cannot convert to UTF-8")]
    InvalidPath,

    /// Binary not found (genetic material missing)
    #[error("Genetic material not found: {0}")]
    BinaryNotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Device not found
    #[error("USB device not found: {0}")]
    DeviceNotFound(PathBuf),

    /// Insufficient space
    #[error(
        "Insufficient space on device: required {required} bytes, available {available} bytes"
    )]
    InsufficientSpace { required: u64, available: u64 },

    /// Mount error
    #[error("Failed to mount device: {0}")]
    MountError(String),

    /// Verification failed
    #[error("Spore verification failed: {0}")]
    VerificationFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML deserialization error
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Anyhow error (for generic fallback)
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    /// Validation error (Dark Forest)
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Serialization error (Dark Forest)
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error (Dark Forest)
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// System error (Dark Forest)
    #[error("System error: {0}")]
    SystemError(String),

    /// Wrapped I/O error (Dark Forest)
    #[error("I/O error: {0}")]
    IoError(std::io::Error),
}
