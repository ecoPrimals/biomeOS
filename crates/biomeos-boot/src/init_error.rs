//! BiomeOS Boot Error Types
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
    /// Init system errors
    #[error("not running as PID 1 (current PID: {0})")]
    NotPid1(i32),

    #[error("init already running")]
    InitAlreadyRunning,

    /// Filesystem errors
    #[error("failed to mount {target} from {fs_source}: {errno}")]
    MountFailed {
        target: String,
        fs_source: String,
        errno: nix::errno::Errno,
    },

    #[error("filesystem {0} already mounted")]
    AlreadyMounted(String),

    #[error("failed to create directory {path}: {error}")]
    DirectoryCreation {
        path: PathBuf,
        error: String,
    },

    /// Hardware detection errors
    #[error("hardware detection failed: {0}")]
    HardwareDetection(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("unsupported architecture: {0}")]
    UnsupportedArchitecture(String),

    /// Network errors
    #[error("network configuration failed: {0}")]
    NetworkConfig(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to detect network interfaces")]
    NetworkInterfaceDetection,

    /// Boot parameter errors
    #[error("failed to read /proc/cmdline: {0}")]
    CmdlineRead(#[source] std::io::Error),

    #[error("invalid boot parameter: {0}")]
    InvalidBootParameter(String),

    #[error("missing required boot parameter: {0}")]
    MissingBootParameter(String),

    /// Shell and process errors
    #[error("failed to spawn shell: {0}")]
    ShellSpawn(#[source] std::io::Error),

    #[error("shell exited unexpectedly with code {0:?}")]
    ShellExited(Option<i32>),

    #[error("failed to execute {command}: {source}")]
    ProcessExecution {
        command: String,
        #[source]
        source: std::io::Error,
    },

    /// Console I/O errors
    #[error("failed to initialize console output: {0}")]
    ConsoleInit(#[source] std::io::Error),

    #[error("failed to write to console: {0}")]
    ConsoleWrite(#[source] std::io::Error),

    /// Device detection errors
    #[error("failed to detect USB device: {0}")]
    UsbDetection(String),

    #[error("BiomeOS USB not found at expected paths")]
    BiomeOsUsbNotFound,

    /// Emergency mode errors
    #[error("emergency mode failed: {0}")]
    EmergencyMode(String),

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Initialization errors
    #[error("initialization failed: {0}")]
    InitializationFailed(String),
    
    // Device management errors (new for boot_logger)
    #[error("device not found: {device}")]
    DeviceNotFound {
        device: String,
    },
    
    #[error("failed to open device {device}: {error}")]
    DeviceOpen {
        device: String,
        error: String,
    },
    
    #[error("failed to create device {device}: {error}")]
    DeviceCreation {
        device: String,
        error: String,
    },
    
    #[error("I/O error during {operation}: {error}")]
    IoError {
        operation: String,
        error: String,
    },
}

impl BootError {
    /// Create a mount failed error
    pub fn mount_failed(target: impl Into<String>, fs_source: impl Into<String>, errno: nix::errno::Errno) -> Self {
        Self::MountFailed {
            target: target.into(),
            fs_source: fs_source.into(),
            errno,
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::AlreadyMounted(_)
                | Self::BiomeOsUsbNotFound
                | Self::NetworkInterfaceDetection
        )
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
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

    #[test]
    fn test_error_severity() {
        let err = BootError::NotPid1(42);
        assert_eq!(err.severity(), ErrorSeverity::Fatal);

        let err = BootError::AlreadyMounted("/proc".to_string());
        assert_eq!(err.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_error_recoverability() {
        assert!(BootError::AlreadyMounted("/proc".to_string()).is_recoverable());
        assert!(!BootError::NotPid1(42).is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = BootError::NotPid1(42);
        assert_eq!(err.to_string(), "not running as PID 1 (current PID: 42)");
    }
}

