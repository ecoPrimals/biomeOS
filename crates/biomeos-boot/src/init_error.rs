// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Boot Error Types
//!
//! Comprehensive error handling for the boot system with detailed context.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for boot operations
pub type Result<T> = std::result::Result<T, BootError>;

/// Comprehensive boot error types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BootError {
    /// Not running as PID 1
    #[error("not running as PID 1 (current PID: {0})")]
    NotPid1(i32),

    /// Init system is already running
    #[error("init already running")]
    InitAlreadyRunning,

    /// Mount operation failed
    #[error("failed to mount {target} from {fs_source}: {errno}")]
    MountFailed {
        /// Mount target path
        target: String,
        /// Filesystem source device
        fs_source: String,
        /// System error number
        errno: rustix::io::Errno,
    },

    /// Filesystem is already mounted
    #[error("filesystem {0} already mounted")]
    AlreadyMounted(String),

    /// Failed to create a directory
    #[error("failed to create directory {path}: {error}")]
    DirectoryCreation {
        /// Directory path that failed
        path: PathBuf,
        /// Error details
        error: String,
    },

    /// Hardware detection failed
    #[error("hardware detection failed: {0}")]
    HardwareDetection(#[source] anyhow::Error),

    /// Unsupported CPU architecture
    #[error("unsupported architecture: {0}")]
    UnsupportedArchitecture(String),

    /// Network configuration failed
    #[error("network configuration failed: {0}")]
    NetworkConfig(#[source] anyhow::Error),

    /// Failed to detect network interfaces
    #[error("failed to detect network interfaces")]
    NetworkInterfaceDetection,

    /// Failed to read kernel command line
    #[error("failed to read /proc/cmdline: {0}")]
    CmdlineRead(#[source] std::io::Error),

    /// Invalid boot parameter value
    #[error("invalid boot parameter: {0}")]
    InvalidBootParameter(String),

    /// Required boot parameter is missing
    #[error("missing required boot parameter: {0}")]
    MissingBootParameter(String),

    /// Failed to spawn emergency shell
    #[error("failed to spawn shell: {0}")]
    ShellSpawn(#[source] std::io::Error),

    /// Shell exited unexpectedly
    #[error("shell exited unexpectedly with code {0:?}")]
    ShellExited(Option<i32>),

    /// Failed to execute a process
    #[error("failed to execute {command}: {source}")]
    ProcessExecution {
        /// Command that failed
        command: String,
        /// I/O error source
        #[source]
        source: std::io::Error,
    },

    /// Console initialization failed
    #[error("failed to initialize console output: {0}")]
    ConsoleInit(#[source] std::io::Error),

    /// Console write failed
    #[error("failed to write to console: {0}")]
    ConsoleWrite(#[source] std::io::Error),

    /// USB device detection failed
    #[error("failed to detect USB device: {0}")]
    UsbDetection(String),

    /// `BiomeOS` USB drive not found
    #[error("BiomeOS USB not found at expected paths")]
    BiomeOsUsbNotFound,

    /// Emergency mode failed
    #[error("emergency mode failed: {0}")]
    EmergencyMode(String),

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Initialization failed
    #[error("initialization failed: {0}")]
    InitializationFailed(String),

    /// Device not found
    #[error("device not found: {device}")]
    DeviceNotFound {
        /// Device path or name
        device: String,
    },

    /// Failed to open a device
    #[error("failed to open device {device}: {error}")]
    DeviceOpen {
        /// Device path or name
        device: String,
        /// Error details
        error: String,
    },

    /// Failed to create a device
    #[error("failed to create device {device}: {error}")]
    DeviceCreation {
        /// Device path or name
        device: String,
        /// Error details
        error: String,
    },

    /// I/O error during a specific operation
    #[error("I/O error during {operation}: {error}")]
    IoError {
        /// Operation that failed
        operation: String,
        /// Error details
        error: String,
    },
}

impl BootError {
    /// Create a mount failed error
    pub fn mount_failed(
        target: impl Into<String>,
        fs_source: impl Into<String>,
        errno: rustix::io::Errno,
    ) -> Self {
        Self::MountFailed {
            target: target.into(),
            fs_source: fs_source.into(),
            errno,
        }
    }

    /// Check if error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::AlreadyMounted(_) | Self::BiomeOsUsbNotFound | Self::NetworkInterfaceDetection
        )
    }

    /// Get error severity
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotPid1(_) | Self::InitAlreadyRunning => ErrorSeverity::Fatal,
            Self::MountFailed { .. } | Self::DirectoryCreation { .. } => ErrorSeverity::Critical,
            Self::AlreadyMounted(_) => ErrorSeverity::Info,
            Self::BiomeOsUsbNotFound | Self::NetworkInterfaceDetection => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational (not actually an error)
    Info,
    /// Warning (degraded functionality)
    Warning,
    /// Error (feature failed, but system can continue)
    Error,
    /// Critical (system stability at risk)
    Critical,
    /// Fatal (cannot continue)
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
            Self::Fatal => write!(f, "FATAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_severity() {
        let err = BootError::NotPid1(42);
        assert_eq!(err.severity(), ErrorSeverity::Fatal);

        let err = BootError::AlreadyMounted("/proc".to_string());
        assert_eq!(err.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_error_severity_all_variants() {
        assert_eq!(
            BootError::InitAlreadyRunning.severity(),
            ErrorSeverity::Fatal
        );
        assert_eq!(
            BootError::mount_failed("/mnt", "/dev/sda1", rustix::io::Errno::INVAL).severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            BootError::DirectoryCreation {
                path: PathBuf::from("/tmp"),
                error: "test".to_string(),
            }
            .severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            BootError::BiomeOsUsbNotFound.severity(),
            ErrorSeverity::Warning
        );
        assert_eq!(
            BootError::NetworkInterfaceDetection.severity(),
            ErrorSeverity::Warning
        );
        assert_eq!(
            BootError::InvalidBootParameter("x".to_string()).severity(),
            ErrorSeverity::Error
        );
    }

    #[test]
    fn test_error_recoverability() {
        assert!(BootError::AlreadyMounted("/proc".to_string()).is_recoverable());
        assert!(!BootError::NotPid1(42).is_recoverable());
        assert!(BootError::BiomeOsUsbNotFound.is_recoverable());
        assert!(BootError::NetworkInterfaceDetection.is_recoverable());
        assert!(!BootError::InvalidBootParameter("x".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = BootError::NotPid1(42);
        assert_eq!(err.to_string(), "not running as PID 1 (current PID: 42)");
    }

    #[test]
    fn test_mount_failed_constructor() {
        let err = BootError::mount_failed("/mnt", "/dev/sda1", rustix::io::Errno::INVAL);
        assert!(err.to_string().contains("mount"));
        assert!(err.to_string().contains("/mnt"));
    }

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Info.to_string(), "INFO");
        assert_eq!(ErrorSeverity::Warning.to_string(), "WARN");
        assert_eq!(ErrorSeverity::Error.to_string(), "ERROR");
        assert_eq!(ErrorSeverity::Critical.to_string(), "CRITICAL");
        assert_eq!(ErrorSeverity::Fatal.to_string(), "FATAL");
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Fatal > ErrorSeverity::Info);
        assert!(ErrorSeverity::Critical > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Error >= ErrorSeverity::Error);
    }

    #[test]
    fn test_error_severity_debug() {
        let s = format!("{:?}", ErrorSeverity::Fatal);
        assert!(s.contains("Fatal"));
    }

    #[test]
    fn test_io_error_display() {
        let err = BootError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(err.to_string().contains("I/O"));
    }

    #[test]
    fn test_device_not_found_display() {
        let err = BootError::DeviceNotFound {
            device: "/dev/sda".to_string(),
        };
        assert!(err.to_string().contains("/dev/sda"));
    }
}
