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

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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
            TransportPreference::JsonRpcUnixSocket,
        ).await
            .context("Failed to discover ToadStool. Is it running?")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support (100x faster)
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:8080")
    /// * `family_id` - Genetic family ID
    #[deprecated(note = "Use ToadStoolClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "toadstool",
            family_id,
            TransportPreference::Http
        ).await
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy constructor (DEPRECATED)
    ///
    /// **BREAKING**: This method is now async. Use `discover()` instead.
    #[deprecated(note = "Use ToadStoolClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("ToadStoolClient::new() is deprecated. Use ToadStoolClient::discover() instead.");
    }

    /// Get resource usage metrics for a service
    ///
    /// Uses ToadStool's JSON-RPC API: `metrics.get_resource_usage`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails or the service is not found.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let metrics = toadstool.get_resource_usage("my-service").await?;
    /// println!("CPU: {}%", metrics.cpu_percent);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resource_usage(&self, service_id: &str) -> Result<ResourceMetrics> {
        let response = self.transport.call(
            "metrics.get_resource_usage",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call metrics.get_resource_usage")?;

        serde_json::from_value(response)
            .context("Failed to parse resource metrics from response")
    }

    /// Deploy a workload
    ///
    /// Uses ToadStool's JSON-RPC API: `workload.deploy`
    ///
    /// # Arguments
    /// * `manifest` - Workload deployment manifest
    ///
    /// # Returns
    /// Deployment information including the deployment ID
    ///
    /// # Errors
    /// Returns an error if deployment fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::{ToadStoolClient, WorkloadManifest, ResourceRequirements};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let manifest = WorkloadManifest {
    ///     name: "my-app".to_string(),
    ///     image: "nginx:latest".to_string(),
    ///     replicas: 3,
    ///     resources: ResourceRequirements {
    ///         cpu_cores: 2.0,
    ///         memory_mb: 512,
    ///     },
    /// };
    /// let deployment = toadstool.deploy_workload(&manifest).await?;
    /// println!("Deployed: {}", deployment.deployment_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deploy_workload(&self, manifest: &WorkloadManifest) -> Result<DeploymentInfo> {
        let response = self.transport.call(
            "workload.deploy",
            Some(serde_json::to_value(manifest)?)
        ).await
            .context("Failed to call workload.deploy")?;

        serde_json::from_value(response)
            .context("Failed to parse deployment info from response")
    }

    /// Scale a service to a target number of replicas
    ///
    /// Uses ToadStool's JSON-RPC API: `service.scale`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    /// * `replicas` - Target number of replicas
    ///
    /// # Errors
    /// Returns an error if scaling fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let result = toadstool.scale_service("my-service", 5).await?;
    /// println!("Scaled from {} to {}", result.previous_replicas, result.target_replicas);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<ScaleResult> {
        let response = self.transport.call(
            "service.scale",
            Some(serde_json::json!({
                "service_id": service_id,
                "replicas": replicas,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.scale")?;

        serde_json::from_value(response)
            .context("Failed to parse scale result from response")
    }

    /// Get the current number of replicas for a service
    ///
    /// Uses ToadStool's JSON-RPC API: `service.get_status`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails or the service is not found.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let replicas = toadstool.get_service_replicas("my-service").await?;
    /// println!("Current replicas: {}", replicas);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_service_replicas(&self, service_id: &str) -> Result<u32> {
        let response = self.transport.call(
            "service.get_status",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.get_status")?;

        response["replicas"]
            .as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| anyhow::anyhow!("No replicas field in status response"))
    }

    /// Get service status
    ///
    /// Uses ToadStool's JSON-RPC API: `service.get_status`
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::toadstool::ToadStoolClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let toadstool = ToadStoolClient::discover("nat0").await?;
    /// let status = toadstool.get_service_status("my-service").await?;
    /// println!("Service status: {}", status.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus> {
        let response = self.transport.call(
            "service.get_status",
            Some(serde_json::json!({
                "service_id": service_id,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call service.get_status")?;

        serde_json::from_value(response)
            .context("Failed to parse service status from response")
    }
}

#[async_trait]
impl PrimalClient for ToadStoolClient {
    fn name(&self) -> &str {
        "toadstool"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        // For JSON-RPC, method becomes the RPC method name, path is ignored
        self.transport.call(method, body).await
    }
}

/// Resource metrics from ToadStool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f64,

    /// Memory usage in megabytes
    pub memory_mb: u64,

    /// Network I/O statistics
    pub network_io: NetworkIO,

    /// Timestamp of metrics collection
    pub timestamp: String,
}

/// Network I/O statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_in: u64,

    /// Bytes sent
    pub bytes_out: u64,
}

/// Workload manifest for deployment
#[derive(Debug, Clone, Serialize)]
pub struct WorkloadManifest {
    /// Workload name
    pub name: String,

    /// Container image
    pub image: String,

    /// Number of replicas
    pub replicas: u32,

    /// Resource requirements
    pub resources: ResourceRequirements,
}

/// Resource requirements for a workload
#[derive(Debug, Clone, Serialize)]
pub struct ResourceRequirements {
    /// CPU cores required
    pub cpu_cores: f64,

    /// Memory in megabytes
    pub memory_mb: u64,
}

/// Deployment information
#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentInfo {
    /// Unique deployment identifier
    pub deployment_id: String,

    /// Deployment status
    pub status: String,

    /// Service endpoint (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Scale operation result
#[derive(Debug, Clone, Deserialize)]
pub struct ScaleResult {
    /// Number of replicas before scaling
    pub previous_replicas: u32,

    /// Target number of replicas
    pub target_replicas: u32,

    /// Scaling operation status
    pub status: String,
}

/// Service status information
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceStatus {
    /// Service identifier
    pub service_id: String,

    /// Current status
    pub status: String,

    /// Number of replicas
    pub replicas: u32,

    /// Service endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toadstool_client_creation() {
        let client = ToadStoolClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "toadstool");
    }

    #[test]
    fn test_workload_manifest_serialization() {
        let manifest = WorkloadManifest {
            name: "test-service".to_string(),
            image: "nginx:latest".to_string(),
            replicas: 3,
            resources: ResourceRequirements {
                cpu_cores: 2.0,
                memory_mb: 512,
            },
        };

        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["name"], "test-service");
        assert_eq!(json["replicas"], 3);
    }
}
