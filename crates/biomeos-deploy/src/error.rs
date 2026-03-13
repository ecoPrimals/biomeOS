// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Error types for BiomeOS deployment

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for deployment operations
pub type Result<T> = std::result::Result<T, DeployError>;

/// Deployment error types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeployError {
    /// QEMU process errors
    #[error("QEMU process failed: {message}")]
    QemuProcess {
        /// Error message
        message: String,
    },

    /// QEMU configuration errors
    #[error("Invalid QEMU configuration: {message}")]
    QemuConfig {
        /// Error message
        message: String,
    },

    /// Network bridge errors
    #[error("Network bridge operation failed: {message}")]
    NetworkBridge {
        /// Error message
        message: String,
    },

    /// Topology parsing errors
    #[error("Failed to parse topology file {path}: {source}")]
    TopologyParse {
        /// Path to the topology file
        path: PathBuf,
        /// Underlying YAML parse error
        #[source]
        source: serde_yaml::Error,
    },

    /// Topology validation errors
    #[error("Invalid topology: {message}")]
    TopologyValidation {
        /// Validation error message
        message: String,
    },

    /// File system errors
    #[error("File system error: {message}")]
    FileSystem {
        /// Error message
        message: String,
    },

    /// Health check errors
    #[error("Health check failed for VM {vm_name}: {message}")]
    HealthCheck {
        /// Name of the VM that failed health check
        vm_name: String,
        /// Error message
        message: String,
    },

    /// Federation deployment errors
    #[error("Federation deployment failed: {message}")]
    FederationDeploy {
        /// Error message
        message: String,
    },

    /// Timeout errors
    #[error("Operation '{operation}' timed out after {timeout_secs}s")]
    Timeout {
        /// Name of the operation that timed out
        operation: String,
        /// Timeout duration in seconds
        timeout_secs: u64,
    },

    /// File not found errors
    #[error("File not found: {}", path.display())]
    FileNotFound {
        /// Path to the missing file
        path: PathBuf,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigError {
        /// Error message
        message: String,
    },

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic process errors
    #[error("Process execution error: {0}")]
    Process(#[from] nix::errno::Errno),
}
