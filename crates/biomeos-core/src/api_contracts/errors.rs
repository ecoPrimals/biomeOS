//! Error handling for API contracts

use super::types::{
    ApiError, ApiResponse, ErrorCategory, ResponseMetadata, ResponseStatus, RetryInfo,
    RetryStrategy,
};
use crate::BiomeError;

impl From<BiomeError> for ApiError {
    fn from(error: BiomeError) -> Self {
        match error {
            BiomeError::ConfigError(msg) | BiomeError::Config(msg) => ApiError {
                code: "CONFIG_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Configuration,
                retry_info: None,
            },
            BiomeError::Validation(msg) | BiomeError::ValidationError(msg) => ApiError {
                code: "VALIDATION_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Validation,
                retry_info: None,
            },
            BiomeError::InvalidInput(msg) => ApiError {
                code: "INVALID_INPUT".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Validation,
                retry_info: None,
            },
            BiomeError::InvalidResponse(msg) => ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Validation,
                retry_info: None,
            },
            BiomeError::Security(msg) => ApiError {
                code: "SECURITY_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Authorization,
                retry_info: None,
            },
            BiomeError::NetworkError(msg) | BiomeError::Network(msg) => ApiError {
                code: "NETWORK_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::NetworkError,
                retry_info: Some(RetryInfo {
                    retry_recommended: true,
                    retry_after_seconds: Some(5),
                    max_retries: Some(3),
                    retry_strategy: Some(RetryStrategy::ExponentialBackoff {
                        base_delay: 1000,
                        max_delay: 30000,
                    }),
                }),
            },
            BiomeError::Authentication(msg) => ApiError {
                code: "AUTHENTICATION_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Authentication,
                retry_info: None,
            },
            BiomeError::Authorization(msg) => ApiError {
                code: "AUTHORIZATION_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Authorization,
                retry_info: None,
            },
            BiomeError::SovereigntyViolation(msg) => ApiError {
                code: "SOVEREIGNTY_VIOLATION".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Authorization,
                retry_info: None,
            },
            BiomeError::VendorLock(msg) => ApiError {
                code: "VENDOR_LOCK".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Configuration,
                retry_info: None,
            },
            BiomeError::Timeout(msg) => ApiError {
                code: "TIMEOUT".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::Timeout,
                retry_info: Some(RetryInfo {
                    retry_recommended: true,
                    retry_after_seconds: Some(10),
                    max_retries: Some(2),
                    retry_strategy: Some(RetryStrategy::FixedDelay { delay_seconds: 5 }),
                }),
            },
            BiomeError::NotFound(msg) | BiomeError::PrimalNotFound(msg) => ApiError {
                code: "NOT_FOUND".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::NotFound,
                retry_info: None,
            },
            BiomeError::ResourceExhausted(msg) => ApiError {
                code: "RESOURCE_EXHAUSTED".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::ResourceExhausted,
                retry_info: Some(RetryInfo {
                    retry_recommended: true,
                    retry_after_seconds: Some(60),
                    max_retries: Some(3),
                    retry_strategy: Some(RetryStrategy::ExponentialBackoff {
                        base_delay: 5000,
                        max_delay: 60000,
                    }),
                }),
            },
            BiomeError::RuntimeError(msg) => ApiError {
                code: "RUNTIME_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
            BiomeError::ServiceError(msg) => ApiError {
                code: "SERVICE_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::ServiceUnavailable,
                retry_info: Some(RetryInfo {
                    retry_recommended: true,
                    retry_after_seconds: Some(30),
                    max_retries: Some(3),
                    retry_strategy: Some(RetryStrategy::FixedDelay { delay_seconds: 30 }),
                }),
            },
            BiomeError::ResourceError(msg) => ApiError {
                code: "RESOURCE_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
            BiomeError::IoError(err) | BiomeError::Io(err) => ApiError {
                code: "IO_ERROR".to_string(),
                message: format!("IO error: {}", err),
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
            BiomeError::YamlError(err) => ApiError {
                code: "YAML_ERROR".to_string(),
                message: format!("YAML parsing error: {}", err),
                details: None,
                trace_id: None,
                category: ErrorCategory::Validation,
                retry_info: None,
            },
            BiomeError::JsonError(err) => ApiError {
                code: "JSON_ERROR".to_string(),
                message: format!("JSON parsing error: {}", err),
                details: None,
                trace_id: None,
                category: ErrorCategory::Validation,
                retry_info: None,
            },
            BiomeError::NotImplemented(msg) => ApiError {
                code: "NOT_IMPLEMENTED".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
            BiomeError::Serialization(msg) => ApiError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
            BiomeError::Generic(msg) => ApiError {
                code: "GENERIC_ERROR".to_string(),
                message: msg,
                details: None,
                trace_id: None,
                category: ErrorCategory::InternalError,
                retry_info: None,
            },
        }
    }
}

/// Helper function to handle API errors and convert them to standardized responses
pub fn handle_api_error<T>(error: BiomeError) -> ApiResponse<T> {
    let api_error = ApiError::from(error);
    let status = match api_error.category {
        ErrorCategory::Authentication => ResponseStatus::Unauthorized,
        ErrorCategory::Authorization => ResponseStatus::Forbidden,
        ErrorCategory::NotFound => ResponseStatus::NotFound,
        ErrorCategory::Timeout => ResponseStatus::Timeout,
        _ => ResponseStatus::Error,
    };

    create_api_response(None, status, Some(api_error))
}

/// Helper function to create standardized API responses
pub fn create_api_response<T>(
    data: Option<T>,
    status: ResponseStatus,
    error: Option<ApiError>,
) -> ApiResponse<T> {
    ApiResponse {
        data,
        status,
        error,
        metadata: ResponseMetadata {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            request_id: uuid::Uuid::new_v4().to_string(),
            api_version: "1.0".to_string(),
            primal_id: "biomeos-core".to_string(),
            processing_time_ms: 0,
            rate_limit: None,
        },
    }
}
