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

use crate::clients::base::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ToadStool compute and execution client
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::toadstool::ToadStoolClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let toadstool = ToadStoolClient::new("http://localhost:8080");
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
    http: PrimalHttpClient,
    endpoint: String,
}

impl ToadStoolClient {
    /// Create a new ToadStool client
    ///
    /// # Arguments
    /// * `endpoint` - ToadStool endpoint URL (e.g., `http://localhost:8080`)
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }

    /// Get resource usage metrics for a service
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
    /// let toadstool = ToadStoolClient::new("http://localhost:8080");
    /// let metrics = toadstool.get_resource_usage("my-service").await?;
    /// println!("CPU: {}%", metrics.cpu_percent);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resource_usage(&self, service_id: &str) -> Result<ResourceMetrics> {
        let response = self
            .http
            .get(&format!("/api/v1/services/{}/metrics", service_id))
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse resource metrics: {}", e))
    }

    /// Deploy a workload
    ///
    /// # Arguments
    /// * `manifest` - Workload deployment manifest
    ///
    /// # Returns
    /// Deployment information including the deployment ID
    ///
    /// # Errors
    /// Returns an error if deployment fails.
    pub async fn deploy_workload(&self, manifest: &WorkloadManifest) -> Result<DeploymentInfo> {
        let response = self
            .http
            .post("/api/v1/workloads/deploy", serde_json::to_value(manifest)?)
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse deployment info: {}", e))
    }

    /// Scale a service to a target number of replicas
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
    /// let toadstool = ToadStoolClient::new("http://localhost:8080");
    /// let result = toadstool.scale_service("my-service", 5).await?;
    /// println!("Scaled from {} to {}", result.previous_replicas, result.target_replicas);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<ScaleResult> {
        let body = serde_json::json!({
            "replicas": replicas
        });

        let response = self
            .http
            .post(&format!("/api/v1/services/{}/scale", service_id), body)
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse scale result: {}", e))
    }

    /// Get the current number of replicas for a service
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails or the service is not found.
    pub async fn get_service_replicas(&self, service_id: &str) -> Result<u32> {
        let response = self
            .http
            .get(&format!("/api/v1/services/{}/status", service_id))
            .await?;

        response["replicas"]
            .as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| anyhow::anyhow!("No replicas field in status response"))
    }

    /// Get service status
    ///
    /// # Arguments
    /// * `service_id` - Service identifier
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus> {
        let response = self
            .http
            .get(&format!("/api/v1/services/{}/status", service_id))
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse service status: {}", e))
    }
}

#[async_trait]
impl PrimalClient for ToadStoolClient {
    fn name(&self) -> &str {
        "toadstool"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
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

    #[test]
    fn test_toadstool_client_creation() {
        let client = ToadStoolClient::new("http://localhost:8080");
        assert_eq!(client.name(), "toadstool");
        assert_eq!(client.endpoint(), "http://localhost:8080");
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
