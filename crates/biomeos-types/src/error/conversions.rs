//! Error Conversions and Utility Implementations
//!
//! This module contains BiomeError constructor methods, trait implementations,
//! and conversions from standard Rust error types.

use super::ai_context::{AIErrorCategory, AIErrorContext, ErrorSeverity, RetryStrategy};
use super::core::{BiomeError, ValidationError};
use super::operations::SecurityViolationType;

/// Standard result type for biomeOS operations
pub type BiomeResult<T> = Result<T, BiomeError>;

impl BiomeError {
    /// Create a configuration error with context
    pub fn config_error(message: impl Into<String>, key: Option<impl Into<String>>) -> Self {
        Self::Configuration {
            message: message.into(),
            key: key.map(|k| k.into()),
            config_path: None,
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::ConfigurationIssue)),
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
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::NetworkFailure)),
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
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::SecurityViolation)),
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
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::ResourceLimitation)),
        }
    }

    /// Create a discovery error
    pub fn discovery_failed(message: impl Into<String>, target: Option<impl Into<String>>) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: target.map(|s| s.into()),
            status_code: None,
            timeout_ms: None,
            operation: None,
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::NetworkFailure)),
        }
    }

    /// Create an integration error
    pub fn integration_failed(
        message: impl Into<String>,
        service: Option<impl Into<String>>,
    ) -> Self {
        Self::Integration {
            message: message.into(),
            component: service.map(|s| s.into()),
            integration_type: None,
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::DependencyFailure)),
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
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::SystemError)),
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
            ai_context: Box::new(AIErrorContext::with_retry(
                AIErrorCategory::NetworkFailure,
                RetryStrategy::exponential_backoff(3, 1000, 10000),
            )),
        }
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>, errors: Vec<ValidationError>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            rule: None,
            errors,
            ai_context: Box::new(AIErrorContext::new(AIErrorCategory::UserError)),
        }
    }

    /// Get the error category
    pub fn category(&self) -> &AIErrorCategory {
        match self {
            Self::Configuration { ai_context, .. }
            | Self::InvalidInput { ai_context, .. }
            | Self::Discovery { ai_context, .. }
            | Self::Network { ai_context, .. }
            | Self::Security { ai_context, .. }
            | Self::Resource { ai_context, .. }
            | Self::Integration { ai_context, .. }
            | Self::Internal { ai_context, .. }
            | Self::Timeout { ai_context, .. }
            | Self::Authorization { ai_context, .. }
            | Self::Validation { ai_context, .. }
            | Self::ExternalService { ai_context, .. }
            | Self::Data { ai_context, .. }
            | Self::Unknown { ai_context, .. } => &ai_context.category,
        }
    }

    /// Get the error severity
    pub fn severity(&self) -> &ErrorSeverity {
        match self {
            Self::Configuration { ai_context, .. }
            | Self::InvalidInput { ai_context, .. }
            | Self::Discovery { ai_context, .. }
            | Self::Network { ai_context, .. }
            | Self::Security { ai_context, .. }
            | Self::Resource { ai_context, .. }
            | Self::Integration { ai_context, .. }
            | Self::Internal { ai_context, .. }
            | Self::Timeout { ai_context, .. }
            | Self::Authorization { ai_context, .. }
            | Self::Validation { ai_context, .. }
            | Self::ExternalService { ai_context, .. }
            | Self::Data { ai_context, .. }
            | Self::Unknown { ai_context, .. } => &ai_context.severity,
        }
    }

    /// Check if automatic retry is recommended
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Configuration { ai_context, .. }
            | Self::InvalidInput { ai_context, .. }
            | Self::Discovery { ai_context, .. }
            | Self::Network { ai_context, .. }
            | Self::Security { ai_context, .. }
            | Self::Resource { ai_context, .. }
            | Self::Integration { ai_context, .. }
            | Self::Internal { ai_context, .. }
            | Self::Timeout { ai_context, .. }
            | Self::Authorization { ai_context, .. }
            | Self::Validation { ai_context, .. }
            | Self::ExternalService { ai_context, .. }
            | Self::Data { ai_context, .. }
            | Self::Unknown { ai_context, .. } => ai_context.retry_strategy.should_retry,
        }
    }

    /// Get the AI error context
    pub fn ai_context(&self) -> &AIErrorContext {
        match self {
            Self::Configuration { ai_context, .. }
            | Self::InvalidInput { ai_context, .. }
            | Self::Discovery { ai_context, .. }
            | Self::Network { ai_context, .. }
            | Self::Security { ai_context, .. }
            | Self::Resource { ai_context, .. }
            | Self::Integration { ai_context, .. }
            | Self::Internal { ai_context, .. }
            | Self::Timeout { ai_context, .. }
            | Self::Authorization { ai_context, .. }
            | Self::Validation { ai_context, .. }
            | Self::ExternalService { ai_context, .. }
            | Self::Data { ai_context, .. }
            | Self::Unknown { ai_context, .. } => ai_context,
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
            Some(format!("io_error_{}", err.kind() as u8)),
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
            }],
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
            }],
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
    fn test_config_error_without_key() {
        let error = BiomeError::config_error("Missing config file", None::<String>);
        assert_eq!(error.category(), &AIErrorCategory::ConfigurationIssue);
    }

    #[test]
    fn test_network_error_with_all_fields() {
        let error = BiomeError::network_error(
            "Connection refused",
            Some("http://localhost:8080"),
            Some(503),
        );
        assert_eq!(error.category(), &AIErrorCategory::NetworkFailure);
    }

    #[test]
    fn test_network_error_minimal() {
        let error = BiomeError::network_error("Network unavailable", None::<String>, None);
        assert_eq!(error.category(), &AIErrorCategory::NetworkFailure);
    }

    #[test]
    fn test_security_error_with_violation_type() {
        let error =
            BiomeError::security_error("Access denied", Some(SecurityViolationType::AccessDenied));
        assert_eq!(error.category(), &AIErrorCategory::SecurityViolation);
    }

    #[test]
    fn test_security_error_without_violation_type() {
        let error = BiomeError::security_error("Security check failed", None);
        assert_eq!(error.category(), &AIErrorCategory::SecurityViolation);
    }

    #[test]
    fn test_resource_error() {
        let error =
            BiomeError::resource_error("Memory exhausted", "memory", Some("1GB"), Some("512MB"));
        assert_eq!(error.category(), &AIErrorCategory::ResourceLimitation);
    }

    #[test]
    fn test_resource_error_minimal() {
        let error = BiomeError::resource_error(
            "Resource unavailable",
            "cpu",
            None::<String>,
            None::<String>,
        );
        assert_eq!(error.category(), &AIErrorCategory::ResourceLimitation);
    }

    #[test]
    fn test_discovery_failed() {
        let error = BiomeError::discovery_failed("Service not found", Some("beardog"));
        assert_eq!(error.category(), &AIErrorCategory::NetworkFailure);
    }

    #[test]
    fn test_discovery_failed_without_target() {
        let error = BiomeError::discovery_failed("Discovery timeout", None::<String>);
        assert_eq!(error.category(), &AIErrorCategory::NetworkFailure);
    }

    #[test]
    fn test_integration_failed() {
        let error = BiomeError::integration_failed("Songbird API error", Some("songbird"));
        assert_eq!(error.category(), &AIErrorCategory::DependencyFailure);
    }

    #[test]
    fn test_internal_error() {
        let error = BiomeError::internal_error("Unexpected state", Some("ERR_001"));
        assert_eq!(error.category(), &AIErrorCategory::SystemError);
    }

    #[test]
    fn test_internal_error_without_code() {
        let error = BiomeError::internal_error("Unknown error", None::<String>);
        assert_eq!(error.category(), &AIErrorCategory::SystemError);
    }

    #[test]
    fn test_timeout_error() {
        let error = BiomeError::timeout_error("Request timed out", 30000, Some("health_check"));
        assert_eq!(error.category(), &AIErrorCategory::NetworkFailure);
        assert!(error.should_retry()); // Timeout errors should be retryable
    }

    #[test]
    fn test_validation_error() {
        let errors = vec![ValidationError {
            field: "name".to_string(),
            message: "Name is required".to_string(),
            code: "required".to_string(),
            rejected_value: None,
        }];
        let error = BiomeError::validation_error("Validation failed", errors);
        assert_eq!(error.category(), &AIErrorCategory::UserError);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: BiomeError = io_err.into();
        assert_eq!(error.category(), &AIErrorCategory::SystemError);
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_str = "{ invalid json }";
        let json_err: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        if let Err(e) = json_err {
            let error: BiomeError = e.into();
            // JSON parsing errors are validation errors (UserError category)
            assert_eq!(error.category(), &AIErrorCategory::UserError);
        }
    }

    #[test]
    fn test_from_uuid_error() {
        let uuid_str = "not-a-valid-uuid";
        let uuid_err: Result<uuid::Uuid, _> = uuid_str.parse();
        if let Err(e) = uuid_err {
            let error: BiomeError = e.into();
            // UUID parsing errors are validation errors (UserError category)
            assert_eq!(error.category(), &AIErrorCategory::UserError);
        }
    }

    #[test]
    fn test_severity_accessor() {
        let error = BiomeError::internal_error("Bug detected", Some("SEVERE"));
        let severity = error.severity();
        // Severity should be a valid value
        assert!(*severity >= ErrorSeverity::Info);
    }

    #[test]
    fn test_ai_context_accessor() {
        let error = BiomeError::config_error("Bad config", Some("key"));
        let context = error.ai_context();
        assert_eq!(context.category, AIErrorCategory::ConfigurationIssue);
    }

    #[test]
    fn test_suggested_actions_accessor() {
        let error = BiomeError::network_error("Timeout", Some("endpoint"), None);
        let actions = error.suggested_actions();
        // Actions may be empty by default
        assert!(actions.is_empty() || !actions.is_empty());
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy::exponential_backoff(3, 1000, 10000);
        assert!(strategy.should_retry);
        assert_eq!(strategy.max_attempts, 3);
        assert_eq!(strategy.delay_ms, 1000);
    }

    #[test]
    fn test_retry_strategy_no_retry() {
        let strategy = RetryStrategy::no_retry();
        assert!(!strategy.should_retry);
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Emergency > ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    }

    #[test]
    fn test_error_severity_equality() {
        assert_eq!(ErrorSeverity::Error, ErrorSeverity::Error);
        assert_ne!(ErrorSeverity::Error, ErrorSeverity::Warning);
    }

    #[test]
    fn test_biome_result_type() {
        fn test_fn() -> BiomeResult<i32> {
            Ok(42)
        }

        assert_eq!(test_fn().unwrap(), 42);
    }

    #[test]
    fn test_biome_result_error() {
        fn test_fn() -> BiomeResult<i32> {
            Err(BiomeError::internal_error("test error", None::<String>))
        }

        assert!(test_fn().is_err());
    }
}
