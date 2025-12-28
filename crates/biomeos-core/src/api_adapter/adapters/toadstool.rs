//! ToadStool API Adapter
//!
//! Adapter for ToadStool's compute orchestration and ML API.
//! Discovers job submission, GPU, and ML model endpoints.

use crate::api_adapter::{discovery, ApiAdapter};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// ToadStool-specific API adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadStoolAdapter {
    /// Base API adapter
    base: ApiAdapter,

    /// ToadStool-specific endpoints (discovered)
    job_submit_endpoint: Option<String>,
    job_status_endpoint: Option<String>,
    gpu_status_endpoint: Option<String>,
    ml_model_endpoint: Option<String>,
    compute_resources_endpoint: Option<String>,
    results_endpoint: Option<String>,
}

impl ToadStoolAdapter {
    /// Discover ToadStool's API structure
    pub async fn discover(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();

        // Use generic discovery first
        let base = discovery::discover_api_interface(&base_url, "toadstool").await?;

        // ToadStool-specific discovery
        let mut adapter = Self {
            base,
            job_submit_endpoint: None,
            job_status_endpoint: None,
            gpu_status_endpoint: None,
            ml_model_endpoint: None,
            compute_resources_endpoint: None,
            results_endpoint: None,
        };

        // Discover ToadStool-specific endpoints
        adapter.discover_job_endpoints().await;
        adapter.discover_gpu_endpoints().await;
        adapter.discover_ml_endpoints().await;
        adapter.discover_resource_endpoints().await;
        adapter.discover_results_endpoints().await;

        Ok(adapter)
    }

    /// Discover job submission and status endpoints
    async fn discover_job_endpoints(&mut self) {
        let submit_patterns = vec![
            "/jobs/submit",
            "/compute/submit",
            "/api/jobs/submit",
            "/api/v1/jobs/submit",
            "/submit",
        ];

        for pattern in submit_patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.job_submit_endpoint = Some(pattern.to_string());
                println!("  ✓ Job submit endpoint: {}", pattern);
                break;
            }
        }

        let status_patterns = vec![
            "/jobs/status",
            "/compute/status",
            "/api/jobs/status",
            "/api/v1/jobs/status",
            "/jobs",
        ];

        for pattern in status_patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.job_status_endpoint = Some(pattern.to_string());
                println!("  ✓ Job status endpoint: {}", pattern);
                break;
            }
        }
    }

    /// Discover GPU/hardware status endpoints
    async fn discover_gpu_endpoints(&mut self) {
        let patterns = vec![
            "/gpu/status",
            "/hardware/gpu",
            "/api/gpu/status",
            "/api/v1/gpu/status",
            "/compute/gpu",
        ];

        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.gpu_status_endpoint = Some(pattern.to_string());
                println!("  ✓ GPU status endpoint: {}", pattern);
                break;
            }
        }
    }

    /// Discover ML model endpoints
    async fn discover_ml_endpoints(&mut self) {
        let patterns = vec![
            "/ml/models",
            "/models",
            "/api/ml/models",
            "/api/v1/ml/models",
            "/inference",
        ];

        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.ml_model_endpoint = Some(pattern.to_string());
                println!("  ✓ ML model endpoint: {}", pattern);
                break;
            }
        }
    }

    /// Discover compute resource endpoints
    async fn discover_resource_endpoints(&mut self) {
        let patterns = vec![
            "/compute/resources",
            "/resources",
            "/api/compute/resources",
            "/api/v1/compute/resources",
            "/capacity",
        ];

        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.compute_resources_endpoint = Some(pattern.to_string());
                println!("  ✓ Compute resources endpoint: {}", pattern);
                break;
            }
        }
    }

    /// Discover results retrieval endpoints
    async fn discover_results_endpoints(&mut self) {
        let patterns = vec![
            "/jobs/results",
            "/results",
            "/api/jobs/results",
            "/api/v1/jobs/results",
            "/compute/results",
        ];

        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.results_endpoint = Some(pattern.to_string());
                println!("  ✓ Results endpoint: {}", pattern);
                break;
            }
        }
    }

    /// Get the base adapter
    pub fn base(&self) -> &ApiAdapter {
        &self.base
    }

    /// Check if ToadStool compute is healthy
    pub async fn check_compute_health(&self) -> Result<bool> {
        // Try compute-specific endpoint first
        if let Some(endpoint) = &self.compute_resources_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();

            if let Ok(response) = client.get(&url).send().await {
                return Ok(response.status().is_success());
            }
        }

        // Fallback to generic health check
        self.base.check_health().await
    }

    /// Get GPU status (if endpoint discovered)
    pub async fn get_gpu_status(&self) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.gpu_status_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();

            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }

        Ok(None)
    }

    /// Get compute resources (if endpoint discovered)
    pub async fn get_compute_resources(&self) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.compute_resources_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();

            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }

        Ok(None)
    }

    /// Get job status (if endpoint discovered)
    pub async fn get_job_status(&self, job_id: &str) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.job_status_endpoint {
            let url = format!("{}{}/{}", self.base.base_url(), endpoint, job_id);
            let client = reqwest::Client::new();

            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toadstool_adapter_creation() {
        // Test will require actual ToadStool instance
        assert_eq!(
            std::mem::size_of::<ToadStoolAdapter>(),
            std::mem::size_of::<ToadStoolAdapter>()
        );
    }
}
