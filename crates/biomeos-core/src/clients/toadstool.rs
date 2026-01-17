// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! ToadStool client for compute execution and resource metrics
//!
//! ToadStool is the compute and execution primal. It provides:
//! - Workload deployment and management
//! - Resource usage metrics
//! - Service scaling
//! - Performance monitoring
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_core::clients::toadstool::ToadStoolClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let toadstool = ToadStoolClient::discover("nat0").await?;
//!
//!     // Get resource metrics for a service
//!     let metrics = toadstool.get_resource_usage("service-123").await?;
//!     println!("CPU: {}%, Memory: {} MB", metrics.cpu_percent, metrics.memory_mb);
//!
//!     Ok(())
//! }
//! ```

mod types;

// Re-export all types for backward compatibility
pub use types::*;

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;

/// ToadStool compute and execution client
///
/// Uses JSON-RPC 2.0 over Unix sockets for fast, secure communication.
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::toadstool::ToadStoolClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Auto-discover via Unix socket
///     let toadstool = ToadStoolClient::discover("nat0").await?;
///
///     // Get resource metrics for a service
///     let metrics = toadstool.get_resource_usage("service-123").await?;
///     println!("CPU: {}%, Memory: {} MB", metrics.cpu_percent, metrics.memory_mb);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ToadStoolClient {
    transport: TransportClient,
    family_id: String,
}

impl ToadStoolClient {
    /// Auto-discover ToadStool via Unix socket
    ///
    /// Searches for ToadStool's Unix socket in XDG runtime directory.
    /// Falls back to HTTP if Unix socket not available.
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID (e.g., "nat0")
    ///
    /// # Returns
    /// ToadStoolClient configured with JSON-RPC over Unix socket (primary)
    /// or HTTP (fallback)
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::toadstool::ToadStoolClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let toadstool = ToadStoolClient::discover("nat0").await?;
    ///     let metrics = toadstool.get_resource_usage("service-123").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover_with_preference(
            "toadstool",
            family_id,
            TransportPreference::UnixSocket,
        )
        .await
        .context("Failed to discover ToadStool. Is it running?")?;

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    #[deprecated(note = "Use ToadStoolClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "toadstool",
            family_id,
            TransportPreference::Auto, // ✅ Evolved: Auto-discover secure transport
        )
        .await
        .context("Failed to discover ToadStool via secure transport")?;

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    #[deprecated(note = "Use ToadStoolClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("ToadStoolClient::new() is deprecated. Use ToadStoolClient::discover() instead.");
    }

    /// Get resource usage metrics for a service (JSON-RPC: compute.get_resource_usage)
    pub async fn get_resource_usage(&self, service_id: &str) -> Result<ResourceMetrics> {
        let response = self
            .transport
            .call(
                "compute.get_resource_usage",
                Some(serde_json::json!({
                    "service_id": service_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call compute.get_resource_usage")?;

        serde_json::from_value(response).context("Failed to parse resource metrics from response")
    }

    /// Deploy a workload (JSON-RPC: compute.deploy)
    pub async fn deploy_workload(&self, manifest: &WorkloadManifest) -> Result<DeploymentInfo> {
        let response = self
            .transport
            .call(
                "compute.deploy",
                Some(serde_json::json!({
                    "manifest": manifest,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call compute.deploy")?;

        serde_json::from_value(response).context("Failed to parse deployment info from response")
    }

    /// Scale a service (JSON-RPC: compute.scale)
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<ScaleResult> {
        let response = self
            .transport
            .call(
                "compute.scale",
                Some(serde_json::json!({
                    "service_id": service_id,
                    "replicas": replicas,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call compute.scale")?;

        serde_json::from_value(response).context("Failed to parse scale result from response")
    }

    /// Get service status (JSON-RPC: compute.get_status)
    pub async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus> {
        let response = self
            .transport
            .call(
                "compute.get_status",
                Some(serde_json::json!({
                    "service_id": service_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call compute.get_status")?;

        serde_json::from_value(response).context("Failed to parse service status from response")
    }

    /// Stop a service (JSON-RPC: compute.stop)
    pub async fn stop_service(&self, service_id: &str) -> Result<()> {
        self.transport
            .call(
                "compute.stop",
                Some(serde_json::json!({
                    "service_id": service_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call compute.stop")?;

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Collaborative Intelligence API
    // ═══════════════════════════════════════════════════════════════════════

    /// Estimate resources for an execution graph (JSON-RPC: collab.estimate_resources)
    ///
    /// Analyzes a graph of operations and estimates total resource requirements.
    ///
    /// # Arguments
    /// * `graph` - Execution graph with nodes and edges
    ///
    /// # Returns
    /// Resource estimate including CPU, memory, GPU, duration, and parallelism
    pub async fn estimate_resources(&self, graph: &ExecutionGraph) -> Result<ResourceEstimate> {
        let response = self
            .transport
            .call(
                "collab.estimate_resources",
                Some(serde_json::json!({
                    "graph": graph,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call collab.estimate_resources")?;

        serde_json::from_value(response).context("Failed to parse resource estimate from response")
    }

    /// Validate resource availability (JSON-RPC: collab.validate_availability)
    ///
    /// Checks if system has sufficient resources to execute a graph.
    ///
    /// # Arguments
    /// * `graph` - Execution graph to validate
    ///
    /// # Returns
    /// Validation result with availability status, gaps, and warnings
    pub async fn validate_availability(
        &self,
        graph: &ExecutionGraph,
    ) -> Result<AvailabilityValidation> {
        let response = self
            .transport
            .call(
                "collab.validate_availability",
                Some(serde_json::json!({
                    "graph": graph,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call collab.validate_availability")?;

        serde_json::from_value(response)
            .context("Failed to parse availability validation from response")
    }

    /// Get optimization suggestions (JSON-RPC: collab.optimize_graph)
    ///
    /// Analyzes a graph and suggests optimizations for better performance.
    ///
    /// # Arguments
    /// * `graph` - Execution graph to optimize
    ///
    /// # Returns
    /// Optimization suggestions with estimated speedup
    pub async fn optimize_graph(&self, graph: &ExecutionGraph) -> Result<OptimizationSuggestions> {
        let response = self
            .transport
            .call(
                "collab.optimize_graph",
                Some(serde_json::json!({
                    "graph": graph,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call collab.optimize_graph")?;

        serde_json::from_value(response)
            .context("Failed to parse optimization suggestions from response")
    }
}

#[async_trait]
impl PrimalClient for ToadStoolClient {
    fn name(&self) -> &str {
        "toadstool"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test using harvested binary from plasmidBin/
    ///
    /// Start ToadStool manually:
    /// ```bash
    /// ./plasmidBin/primals/toadstool --family nat0
    /// ```
    #[ignore = "Requires running ToadStool from plasmidBin/primals/toadstool"]
    #[tokio::test]
    async fn test_toadstool_client_creation() {
        let client = ToadStoolClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "toadstool");
    }
}
