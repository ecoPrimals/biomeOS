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
    InvalidSeedLength {
        /// Expected seed length in bytes
        expected: u64,
        /// Actual seed length found
        found: u64,
    },

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
    InsufficientSpace {
        /// Bytes required for the operation
        required: u64,
        /// Bytes available on the device
        available: u64,
    },

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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Error Display Messages ==========

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = SporeError::Io(io_err);
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_seed_file_not_found_display() {
        let err = SporeError::SeedFileNotFound(PathBuf::from("/media/usb/.family.seed"));
        assert_eq!(
            err.to_string(),
            "Seed file not found: /media/usb/.family.seed"
        );
    }

    #[test]
    fn test_invalid_seed_length_display() {
        let err = SporeError::InvalidSeedLength {
            expected: 32,
            found: 16,
        };
        assert_eq!(
            err.to_string(),
            "Invalid seed length: expected 32, found 16"
        );
    }

    #[test]
    fn test_invalid_path_display() {
        let err = SporeError::InvalidPath;
        assert_eq!(err.to_string(), "Invalid path: cannot convert to UTF-8");
    }

    #[test]
    fn test_binary_not_found_display() {
        let err = SporeError::BinaryNotFound("beardog-server".to_string());
        assert_eq!(
            err.to_string(),
            "Genetic material not found: beardog-server"
        );
    }

    #[test]
    fn test_invalid_config_display() {
        let err = SporeError::InvalidConfig("missing node_id".to_string());
        assert_eq!(
            err.to_string(),
            "Invalid configuration: missing node_id"
        );
    }

    #[test]
    fn test_device_not_found_display() {
        let err = SporeError::DeviceNotFound(PathBuf::from("/dev/sdb1"));
        assert_eq!(err.to_string(), "USB device not found: /dev/sdb1");
    }

    #[test]
    fn test_insufficient_space_display() {
        let err = SporeError::InsufficientSpace {
            required: 1_000_000_000,
            available: 500_000_000,
        };
        assert!(err.to_string().contains("1000000000"));
        assert!(err.to_string().contains("500000000"));
    }

    #[test]
    fn test_mount_error_display() {
        let err = SporeError::MountError("device busy".to_string());
        assert_eq!(err.to_string(), "Failed to mount device: device busy");
    }

    #[test]
    fn test_verification_failed_display() {
        let err = SporeError::VerificationFailed("hash mismatch".to_string());
        assert_eq!(
            err.to_string(),
            "Spore verification failed: hash mismatch"
        );
    }

    #[test]
    fn test_validation_failed_display() {
        let err = SporeError::ValidationFailed("beacon expired".to_string());
        assert_eq!(err.to_string(), "Validation failed: beacon expired");
    }

    #[test]
    fn test_serialization_error_display() {
        let err = SporeError::SerializationError("invalid utf8".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid utf8");
    }

    #[test]
    fn test_deserialization_error_display() {
        let err = SporeError::DeserializationError("unexpected token".to_string());
        assert_eq!(
            err.to_string(),
            "Deserialization error: unexpected token"
        );
    }

    #[test]
    fn test_system_error_display() {
        let err = SporeError::SystemError("clock skew detected".to_string());
        assert_eq!(err.to_string(), "System error: clock skew detected");
    }

    #[test]
    fn test_io_error_wrapped_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = SporeError::IoError(io_err);
        assert!(err.to_string().contains("access denied"));
    }

    // ========== From Conversions ==========

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: SporeError = io_err.into();
        assert!(matches!(err, SporeError::Io(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("not valid json").unwrap_err();
        let err: SporeError = json_err.into();
        assert!(matches!(err, SporeError::Serialization(_)));
    }

    #[test]
    fn test_from_toml_de_error() {
        let toml_err = toml::from_str::<toml::Value>("= invalid").unwrap_err();
        let err: SporeError = toml_err.into();
        assert!(matches!(err, SporeError::TomlDe(_)));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let err: SporeError = anyhow_err.into();
        assert!(matches!(err, SporeError::Anyhow(_)));
        assert!(err.to_string().contains("something went wrong"));
    }

    // ========== SporeResult Type Alias ==========

    #[test]
    fn test_spore_result_ok() {
        fn returns_ok() -> SporeResult<i32> {
            Ok(42)
        }
        let result = returns_ok();
        assert!(result.is_ok());
        assert_eq!(result.expect("should be ok"), 42);
    }

    #[test]
    fn test_spore_result_err() {
        let result: SporeResult<i32> = Err(SporeError::InvalidPath);
        assert!(result.is_err());
    }

    // ========== Debug Formatting ==========

    #[test]
    fn test_error_debug_format() {
        let err = SporeError::BinaryNotFound("songbird".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("BinaryNotFound"));
        assert!(debug.contains("songbird"));
    }
}
