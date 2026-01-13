//! Error types for Universal Primal Client

use std::fmt;

/// Result type alias
pub type Result<T> = std::result::Result<T, ApiError>;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    /// HTTP request failed
    RequestFailed {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Response parsing failed
    ParseError {
        message: String,
        body: Option<String>,
    },

    /// Primal not found
    PrimalNotFound { capability: String },

    /// Unauthorized access
    Unauthorized { message: String },

    /// Forbidden
    Forbidden { message: String },

    /// Resource not found
    NotFound { resource: String },

    /// Server error
    ServerError { status: u16, message: String },

    /// Timeout
    Timeout { operation: String },

    /// Schema error
    SchemaError { message: String },

    /// Discovery error
    DiscoveryError { message: String },

    /// Trust verification failed
    TrustVerificationFailed { primal_id: String, reason: String },

    /// Configuration error
    ConfigError { message: String },

    /// Generic error
    Other { message: String },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestFailed { message, .. } => {
                write!(f, "Request failed: {}", message)
            }
            Self::ParseError { message, .. } => {
                write!(f, "Parse error: {}", message)
            }
            Self::PrimalNotFound { capability } => {
                write!(f, "Primal not found with capability: {}", capability)
            }
            Self::Unauthorized { message } => {
                write!(f, "Unauthorized: {}", message)
            }
            Self::Forbidden { message } => {
                write!(f, "Forbidden: {}", message)
            }
            Self::NotFound { resource } => {
                write!(f, "Not found: {}", resource)
            }
            Self::ServerError { status, message } => {
                write!(f, "Server error {}: {}", status, message)
            }
            Self::Timeout { operation } => {
                write!(f, "Timeout: {}", operation)
            }
            Self::SchemaError { message } => {
                write!(f, "Schema error: {}", message)
            }
            Self::DiscoveryError { message } => {
                write!(f, "Discovery error: {}", message)
            }
            Self::TrustVerificationFailed { primal_id, reason } => {
                write!(f, "Trust verification failed for {}: {}", primal_id, reason)
            }
            Self::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            Self::Other { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RequestFailed { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestFailed {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError {
            message: err.to_string(),
            body: None,
        }
    }
}
