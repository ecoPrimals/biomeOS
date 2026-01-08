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
    #[error("Insufficient space on device: required {required} bytes, available {available} bytes")]
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

    /// Anyhow error (for generic fallback)
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

