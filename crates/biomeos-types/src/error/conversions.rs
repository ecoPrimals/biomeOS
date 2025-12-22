//! Error Conversions and Utility Implementations
//!
//! This module contains BiomeError constructor methods, trait implementations,
//! and conversions from standard Rust error types.

use super::core::{BiomeError, ValidationError};
use super::ai_context::{AIErrorContext, AIErrorCategory, ErrorSeverity, RetryStrategy};
use super::operations::SecurityViolationType;

/// Standard result type for biomeOS operations
pub type BiomeResult<T> = Result<T, BiomeError>;

impl BiomeError {
    /// Create a configuration error with context
    pub fn config_error(
        message: impl Into<String>,
        key: Option<impl Into<String>>,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            key: key.map(|k| k.into()),
            config_path: None,
            ai_context: AIErrorContext::new(AIErrorCategory::ConfigurationIssue),
        }
    }

    /// Create a network error
    pub fn network_error(
        message: impl Into<String>,
        endpoint: Option<impl Into<String>>,
        status_code: Option<u16>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: endpoint.map(|e| e.into()),
            status_code,
            timeout_ms: None,
            operation: None,
            ai_context: AIErrorContext::new(AIErrorCategory::NetworkFailure),
        }
    }

    /// Create a security error
    pub fn security_error(
        message: impl Into<String>,
        violation_type: Option<SecurityViolationType>,
    ) -> Self {
        Self::Security {
            message: message.into(),
            context: None,
            auth_method: None,
            violation_type,
            ai_context: AIErrorContext::new(AIErrorCategory::SecurityViolation),
        }
    }

    /// Create a resource error
    pub fn resource_error(
        message: impl Into<String>,
        resource_type: impl Into<String>,
        requested: Option<impl Into<String>>,
        available: Option<impl Into<String>>,
    ) -> Self {
        Self::Resource {
            message: message.into(),
            resource_type: Some(resource_type.into()),
            requested: requested.map(|s| s.into()),
            available: available.map(|s| s.into()),
            operation: None,
            ai_context: AIErrorContext::new(AIErrorCategory::ResourceLimitation),
        }
    }

    pub fn discovery_failed(
        message: impl Into<String>,
        target: Option<impl Into<String>>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: target.map(|s| s.into()),
            status_code: None,
            timeout_ms: None,
            operation: None,
            ai_context: AIErrorContext::new(AIErrorCategory::NetworkFailure),
        }
    }

    pub fn integration_failed(
        message: impl Into<String>,
        service: Option<impl Into<String>>,
    ) -> Self {
        Self::Integration {
            message: message.into(),
            component: service.map(|s| s.into()),
            integration_type: None,
            ai_context: AIErrorContext::new(AIErrorCategory::DependencyFailure),
        }
    }

    /// Create an internal error
    pub fn internal_error(
        message: impl Into<String>,
        error_code: Option<impl Into<String>>,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            error_code: error_code.map(|c| c.into()),
            stack_trace: None,
            ai_context: AIErrorContext::new(AIErrorCategory::SystemError),
        }
    }

    /// Create a timeout error
    pub fn timeout_error(
        message: impl Into<String>,
        timeout_ms: u64,
        operation: Option<impl Into<String>>,
    ) -> Self {
        Self::Timeout {
            message: message.into(),
            timeout_ms,
            operation: operation.map(|o| o.into()),
            ai_context: AIErrorContext::with_retry(
                AIErrorCategory::NetworkFailure,
                RetryStrategy::exponential_backoff(3, 1000, 10000),
            ),
        }
    }

    /// Create a validation error
    pub fn validation_error(
        message: impl Into<String>,
        errors: Vec<ValidationError>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            rule: None,
            errors,
            ai_context: AIErrorContext::new(AIErrorCategory::UserError),
        }
    }

    /// Get the error category
    pub fn category(&self) -> &AIErrorCategory {
        match self {
            Self::Configuration { ai_context, .. } |
            Self::InvalidInput { ai_context, .. } |
            Self::Discovery { ai_context, .. } |
            Self::Network { ai_context, .. } |
            Self::Security { ai_context, .. } |
            Self::Resource { ai_context, .. } |
            Self::Integration { ai_context, .. } |
            Self::Internal { ai_context, .. } |
            Self::Timeout { ai_context, .. } |
            Self::Authorization { ai_context, .. } |
            Self::Validation { ai_context, .. } |
            Self::ExternalService { ai_context, .. } |
            Self::Data { ai_context, .. } |
            Self::Unknown { ai_context, .. } => &ai_context.category,
        }
    }

    /// Get the error severity
    pub fn severity(&self) -> &ErrorSeverity {
        match self {
            Self::Configuration { ai_context, .. } |
            Self::InvalidInput { ai_context, .. } |
            Self::Discovery { ai_context, .. } |
            Self::Network { ai_context, .. } |
            Self::Security { ai_context, .. } |
            Self::Resource { ai_context, .. } |
            Self::Integration { ai_context, .. } |
            Self::Internal { ai_context, .. } |
            Self::Timeout { ai_context, .. } |
            Self::Authorization { ai_context, .. } |
            Self::Validation { ai_context, .. } |
            Self::ExternalService { ai_context, .. } |
            Self::Data { ai_context, .. } |
            Self::Unknown { ai_context, .. } => &ai_context.severity,
        }
    }

    /// Check if automatic retry is recommended
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Configuration { ai_context, .. } |
            Self::InvalidInput { ai_context, .. } |
            Self::Discovery { ai_context, .. } |
            Self::Network { ai_context, .. } |
            Self::Security { ai_context, .. } |
            Self::Resource { ai_context, .. } |
            Self::Integration { ai_context, .. } |
            Self::Internal { ai_context, .. } |
            Self::Timeout { ai_context, .. } |
            Self::Authorization { ai_context, .. } |
            Self::Validation { ai_context, .. } |
            Self::ExternalService { ai_context, .. } |
            Self::Data { ai_context, .. } |
            Self::Unknown { ai_context, .. } => ai_context.retry_strategy.should_retry,
        }
    }

    /// Get the AI error context
    pub fn ai_context(&self) -> &AIErrorContext {
        match self {
            Self::Configuration { ai_context, .. } |
            Self::InvalidInput { ai_context, .. } |
            Self::Discovery { ai_context, .. } |
            Self::Network { ai_context, .. } |
            Self::Security { ai_context, .. } |
            Self::Resource { ai_context, .. } |
            Self::Integration { ai_context, .. } |
            Self::Internal { ai_context, .. } |
            Self::Timeout { ai_context, .. } |
            Self::Authorization { ai_context, .. } |
            Self::Validation { ai_context, .. } |
            Self::ExternalService { ai_context, .. } |
            Self::Data { ai_context, .. } |
            Self::Unknown { ai_context, .. } => ai_context,
        }
    }

    /// Get suggested actions for automation
    pub fn suggested_actions(&self) -> &[super::ai_context::SuggestedAction] {
        &self.ai_context().suggested_actions
    }
}

// Standard error conversions
impl From<std::io::Error> for BiomeError {
    fn from(err: std::io::Error) -> Self {
        BiomeError::internal_error(
            format!("IO error: {}", err),
            Some(format!("io_error_{}", err.kind() as u8))
        )
    }
}

impl From<serde_json::Error> for BiomeError {
    fn from(err: serde_json::Error) -> Self {
        BiomeError::validation_error(
            format!("JSON serialization error: {}", err),
            vec![ValidationError {
                field: "json".to_string(),
                message: err.to_string(),
                code: "invalid_json".to_string(),
                rejected_value: None,
            }]
        )
    }
}

impl From<uuid::Error> for BiomeError {
    fn from(err: uuid::Error) -> Self {
        BiomeError::validation_error(
            format!("UUID error: {}", err),
            vec![ValidationError {
                field: "uuid".to_string(),
                message: err.to_string(),
                code: "invalid_uuid".to_string(),
                rejected_value: None,
            }]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = BiomeError::config_error("Invalid config", Some("database.url"));
        assert_eq!(error.category(), &AIErrorCategory::ConfigurationIssue);
        assert!(!error.should_retry());
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy::exponential_backoff(3, 1000, 10000);
        assert!(strategy.should_retry);
        assert_eq!(strategy.max_attempts, 3);
        assert_eq!(strategy.delay_ms, 1000);
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Emergency > ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    }
} 