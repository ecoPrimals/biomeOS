//! Error types for BiomeOS deployment

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DeployError>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeployError {
    /// QEMU process errors
    #[error("QEMU process failed: {message}")]
    QemuProcess { message: String },

    /// QEMU configuration errors
    #[error("Invalid QEMU configuration: {message}")]
    QemuConfig { message: String },

    /// Network bridge errors
    #[error("Network bridge operation failed: {message}")]
    NetworkBridge { message: String },

    /// Topology parsing errors
    #[error("Failed to parse topology file {path}: {source}")]
    TopologyParse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    /// Topology validation errors
    #[error("Invalid topology: {message}")]
    TopologyValidation { message: String },

    /// File system errors
    #[error("File system error: {message}")]
    FileSystem { message: String },

    /// Health check errors
    #[error("Health check failed for VM {vm_name}: {message}")]
    HealthCheck { vm_name: String, message: String },

    /// Federation deployment errors
    #[error("Federation deployment failed: {message}")]
    FederationDeploy { message: String },

    /// Timeout errors
    #[error("Operation '{operation}' timed out after {timeout_secs}s")]
    Timeout {
        operation: String,
        timeout_secs: u64,
    },

    /// File not found errors
    #[error("File not found: {}", path.display())]
    FileNotFound { path: PathBuf },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic process errors
    #[error("Process execution error: {0}")]
    Process(#[from] nix::errno::Errno),
}
