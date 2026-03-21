// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    Process(#[from] rustix::io::Errno),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn qemu_process_error_display() {
        let e = DeployError::QemuProcess {
            message: "qemu died".into(),
        };
        let s = e.to_string();
        assert!(s.contains("qemu died"));
        assert!(s.contains("QEMU"));
    }

    #[test]
    fn qemu_config_error_display() {
        let e = DeployError::QemuConfig {
            message: "invalid config".into(),
        };
        assert!(e.to_string().contains("invalid config"));
    }

    #[test]
    fn network_bridge_error_display() {
        let e = DeployError::NetworkBridge {
            message: "bridge failed".into(),
        };
        assert!(e.to_string().contains("bridge failed"));
    }

    #[test]
    fn topology_validation_error_display() {
        let e = DeployError::TopologyValidation {
            message: "invalid topology".into(),
        };
        assert!(e.to_string().contains("invalid topology"));
    }

    #[test]
    fn file_system_error_display() {
        let e = DeployError::FileSystem {
            message: "disk full".into(),
        };
        assert!(e.to_string().contains("disk full"));
    }

    #[test]
    fn health_check_error_display() {
        let e = DeployError::HealthCheck {
            vm_name: "vm1".into(),
            message: "unhealthy".into(),
        };
        let s = e.to_string();
        assert!(s.contains("vm1"));
        assert!(s.contains("unhealthy"));
    }

    #[test]
    fn federation_deploy_error_display() {
        let e = DeployError::FederationDeploy {
            message: "deploy failed".into(),
        };
        assert!(e.to_string().contains("deploy failed"));
    }

    #[test]
    fn timeout_error_display() {
        let e = DeployError::Timeout {
            operation: "boot".into(),
            timeout_secs: 60,
        };
        let s = e.to_string();
        assert!(s.contains("boot"));
        assert!(s.contains("60"));
    }

    #[test]
    fn file_not_found_error_display() {
        let e = DeployError::FileNotFound {
            path: PathBuf::from("/tmp/missing"),
        };
        assert!(e.to_string().contains("missing"));
    }

    #[test]
    fn config_error_display() {
        let e = DeployError::ConfigError {
            message: "bad config".into(),
        };
        assert!(e.to_string().contains("bad config"));
    }

    #[test]
    fn debug_impl() {
        let e = DeployError::ConfigError {
            message: "test".into(),
        };
        let _ = format!("{:?}", e);
    }
}
