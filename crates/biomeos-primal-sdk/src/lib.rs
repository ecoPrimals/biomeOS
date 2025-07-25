//! biomeOS Primal SDK

pub mod types;

pub use types::*;

/// Core trait for biomeOS primals
#[async_trait::async_trait]
pub trait EcoPrimal: Send + Sync {
    /// Get primal metadata
    fn metadata(&self) -> &PrimalMetadata;

    /// Get primal capabilities
    fn capabilities(&self) -> &[PrimalCapability];

    /// Initialize the primal
    async fn initialize(&self, config: &PrimalConfig) -> PrimalResult<()>;

    /// Handle a primal request
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;

    /// Get health status
    async fn health_check(&self) -> PrimalResult<PrimalHealth>;

    /// Shutdown the primal
    async fn shutdown(&self) -> PrimalResult<()>;
}
