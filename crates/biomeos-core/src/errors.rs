//! Error types for biomeOS

use std::fmt;

#[derive(Debug)]
pub enum BiomeError {
    /// Configuration error
    Config(String),
    /// Configuration error (legacy)
    ConfigError(String),
    /// IO error
    Io(std::io::Error),
    /// IO error (legacy)
    IoError(std::io::Error),
    /// Serialization error
    Serialization(String),
    /// Network error
    Network(String),
    /// Network error (legacy)
    NetworkError(String),
    /// Authentication error
    Authentication(String),
    /// Authorization error
    Authorization(String),
    /// Security error
    Security(String),
    /// Validation error
    Validation(String),
    /// Validation error (legacy)
    ValidationError(String),
    /// Invalid input error
    InvalidInput(String),
    /// Invalid response error
    InvalidResponse(String),
    /// Resource not found
    NotFound(String),
    /// Feature not implemented
    NotImplemented(String),
    /// Operation timeout
    Timeout(String),
    /// Runtime error
    RuntimeError(String),
    /// Service error
    ServiceError(String),
    /// Resource error
    ResourceError(String),
    /// Resource exhausted
    ResourceExhausted(String),
    /// Sovereignty violation
    SovereigntyViolation(String),
    /// Vendor lock-in detected
    VendorLock(String),
    /// Primal not found
    PrimalNotFound(String),
    /// YAML parsing error
    YamlError(serde_yaml::Error),
    /// JSON parsing error
    JsonError(serde_json::Error),
    /// Generic error
    Generic(String),
}

impl fmt::Display for BiomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BiomeError::Config(msg) => write!(f, "Configuration error: {}", msg),
            BiomeError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            BiomeError::Io(err) => write!(f, "IO error: {}", err),
            BiomeError::IoError(err) => write!(f, "IO error: {}", err),
            BiomeError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            BiomeError::Network(msg) => write!(f, "Network error: {}", msg),
            BiomeError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BiomeError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            BiomeError::Authorization(msg) => write!(f, "Authorization error: {}", msg),
            BiomeError::Security(msg) => write!(f, "Security error: {}", msg),
            BiomeError::Validation(msg) => write!(f, "Validation error: {}", msg),
            BiomeError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            BiomeError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            BiomeError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            BiomeError::NotFound(msg) => write!(f, "Not found: {}", msg),
            BiomeError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            BiomeError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            BiomeError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            BiomeError::ServiceError(msg) => write!(f, "Service error: {}", msg),
            BiomeError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            BiomeError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            BiomeError::SovereigntyViolation(msg) => write!(f, "Sovereignty violation: {}", msg),
            BiomeError::VendorLock(msg) => write!(f, "Vendor lock detected: {}", msg),
            BiomeError::PrimalNotFound(msg) => write!(f, "Primal not found: {}", msg),
            BiomeError::YamlError(err) => write!(f, "YAML error: {}", err),
            BiomeError::JsonError(err) => write!(f, "JSON error: {}", err),
            BiomeError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for BiomeError {}

impl From<std::io::Error> for BiomeError {
    fn from(err: std::io::Error) -> Self {
        BiomeError::Io(err)
    }
}

impl From<serde_yaml::Error> for BiomeError {
    fn from(err: serde_yaml::Error) -> Self {
        BiomeError::YamlError(err)
    }
}

impl From<serde_json::Error> for BiomeError {
    fn from(err: serde_json::Error) -> Self {
        BiomeError::JsonError(err)
    }
}

/// Result type for biomeOS operations
pub type BiomeResult<T> = Result<T, BiomeError>;
