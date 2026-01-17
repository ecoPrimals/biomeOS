//! Primal client traits and common types

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Health status of a primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Primal is healthy and responsive
    Healthy,
    /// Primal is degraded (slow responses, partial functionality)
    Degraded,
    /// Primal is unhealthy (not responding, errors)
    Unhealthy,
    /// Primal health unknown (not yet checked)
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Core primal client trait
///
/// All primal clients should implement this trait for consistent interface.
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Get the primal's name/identifier
    fn name(&self) -> &str;

    /// Get the primal's endpoint
    fn endpoint(&self) -> String;

    /// Check if the primal is available and responsive
    async fn is_available(&self) -> bool;

    /// Perform health check
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Make a raw request to the primal
    ///
    /// # Arguments
    /// * `method` - The RPC method name
    /// * `path` - The HTTP path (deprecated, ignored for JSON-RPC)
    /// * `body` - Optional request body/parameters
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value>;
}
