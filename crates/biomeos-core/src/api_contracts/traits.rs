//! API contract traits and interfaces

use super::lifecycle::*;
use super::requests::*;
use super::responses::*;
use super::types::*;
use crate::BiomeResult;
use async_trait::async_trait;

/// Universal API contract for all primals
#[async_trait]
pub trait PrimalApiContract: Send + Sync {
    /// Get primal information
    async fn get_primal_info(&self) -> BiomeResult<PrimalInfoResponse>;

    /// Health check endpoint
    async fn health_check(&self) -> BiomeResult<HealthCheckResponse>;

    /// Get primal capabilities
    async fn get_capabilities(&self) -> BiomeResult<CapabilitiesResponse>;

    /// Get resource status
    async fn get_resource_status(&self) -> BiomeResult<ResourceStatusResponse>;

    /// Get performance metrics
    async fn get_performance_metrics(&self) -> BiomeResult<PerformanceMetricsResponse>;

    /// Execute primal-specific operation
    async fn execute_operation(&self, request: OperationRequest) -> BiomeResult<OperationResponse>;

    /// Handle inter-primal communication
    async fn handle_inter_primal_message(
        &self,
        message: InterPrimalMessage,
    ) -> BiomeResult<InterPrimalResponse>;

    /// Get configuration
    async fn get_configuration(&self) -> BiomeResult<ConfigurationResponse>;

    /// Update configuration
    async fn update_configuration(
        &self,
        config: ConfigurationUpdate,
    ) -> BiomeResult<ConfigurationResponse>;

    /// Get logs
    async fn get_logs(&self, request: LogRequest) -> BiomeResult<LogResponse>;

    /// Get metrics
    async fn get_metrics(&self, request: MetricsRequest) -> BiomeResult<MetricsResponse>;

    /// Handle lifecycle event
    async fn handle_lifecycle_event(&self, event: LifecycleEvent)
        -> BiomeResult<LifecycleResponse>;
}

/// API metrics collector
#[async_trait]
pub trait ApiMetricsCollector: Send + Sync {
    /// Record API call
    async fn record_api_call(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
    );

    /// Record API error
    async fn record_api_error(&self, endpoint: &str, method: &str, error_category: &ErrorCategory);

    /// Record validation error
    async fn record_validation_error(&self, endpoint: &str, validation_type: &str);
}
