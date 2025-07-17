//! API contract middleware and enforcement

use super::traits::*;
use super::types::*;
use super::validation::*;
use crate::BiomeResult;
use std::sync::Arc;

/// API contract enforcement middleware
pub struct ApiContractMiddleware {
    /// Validator
    validator: ApiContractValidator,
    /// Metrics collector
    metrics: Arc<dyn ApiMetricsCollector>,
}

impl ApiContractMiddleware {
    /// Create new middleware
    pub fn new(validator: ApiContractValidator, metrics: Arc<dyn ApiMetricsCollector>) -> Self {
        Self { validator, metrics }
    }

    /// Process API request
    pub async fn process_request(&self, request: &serde_json::Value) -> BiomeResult<()> {
        // Validate request
        let validation_result = self.validator.validate_request(request)?;

        if !validation_result.valid {
            return Err(crate::BiomeError::InvalidInput(format!(
                "Request validation failed: {:?}",
                validation_result.errors
            )));
        }

        Ok(())
    }

    /// Process API response
    pub async fn process_response(&self, response: &serde_json::Value) -> BiomeResult<()> {
        // Validate response
        let validation_result = self.validator.validate_response(response)?;

        if !validation_result.valid {
            return Err(crate::BiomeError::InvalidInput(format!(
                "Response validation failed: {:?}",
                validation_result.errors
            )));
        }

        Ok(())
    }

    /// Record API metrics
    pub async fn record_metrics(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
    ) {
        self.metrics
            .record_api_call(endpoint, method, status_code, duration_ms)
            .await;
    }

    /// Record API error
    pub async fn record_error(&self, endpoint: &str, method: &str, error_category: &ErrorCategory) {
        self.metrics
            .record_api_error(endpoint, method, error_category)
            .await;
    }

    /// Record validation error
    pub async fn record_validation_error(&self, endpoint: &str, validation_type: &str) {
        self.metrics
            .record_validation_error(endpoint, validation_type)
            .await;
    }
}

/// Default API metrics collector implementation
pub struct DefaultApiMetricsCollector;

#[async_trait::async_trait]
impl ApiMetricsCollector for DefaultApiMetricsCollector {
    async fn record_api_call(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
    ) {
        // TODO: Implement actual metrics recording
        tracing::info!(
            endpoint = endpoint,
            method = method,
            status_code = status_code,
            duration_ms = duration_ms,
            "API call recorded"
        );
    }

    async fn record_api_error(&self, endpoint: &str, method: &str, error_category: &ErrorCategory) {
        // TODO: Implement actual error metrics recording
        tracing::error!(
            endpoint = endpoint,
            method = method,
            error_category = ?error_category,
            "API error recorded"
        );
    }

    async fn record_validation_error(&self, endpoint: &str, validation_type: &str) {
        // TODO: Implement actual validation error metrics recording
        tracing::error!(
            endpoint = endpoint,
            validation_type = validation_type,
            "Validation error recorded"
        );
    }
}
