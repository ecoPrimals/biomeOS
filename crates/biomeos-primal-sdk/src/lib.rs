//! biomeOS Primal SDK

pub mod types;

pub use types::*;

/// Core trait for biomeOS primals
#[async_trait::async_trait]
pub trait EcoPrimal: Send + Sync {
    type Config;
    async fn initialize(&self, config: Self::Config) -> anyhow::Result<()>;
    async fn handle_request(&self, request: serde_json::Value) -> anyhow::Result<serde_json::Value>;
    async fn health_check(&self) -> anyhow::Result<PrimalHealth>;
    async fn shutdown(&self) -> anyhow::Result<()>;
}

/// Primal health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PrimalHealth {
    Healthy,
    Degraded, 
    Unhealthy,
}
