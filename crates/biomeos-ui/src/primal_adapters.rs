//! Primal adapters for biomeOS UI

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal interface for all Primals
#[async_trait]
pub trait UniversalPrimalInterface {
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn get_capabilities(&self) -> Result<Vec<String>>;
    async fn coordinate_deployment(&self, request: DeploymentRequest) -> Result<DeploymentResponse>;
    async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus>;
    async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<()>;
    async fn get_logs(&self, service_id: &str, lines: Option<u32>) -> Result<Vec<String>>;
}

/// Primal adapter implementation
#[derive(Debug, Clone)]
pub struct PrimalAdapter {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    client: reqwest::Client,
}

#[async_trait]
impl UniversalPrimalInterface for PrimalAdapter {
    async fn health_check(&self) -> Result<HealthStatus> {
        let url = format!("{}/api/v1/health", self.endpoint);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            Ok(HealthStatus {
                status: "healthy".to_string(),
                message: None,
            })
        } else {
            Ok(HealthStatus {
                status: "unhealthy".to_string(),
                message: Some(format!("HTTP {}", response.status())),
            })
        }
    }
    
    async fn get_capabilities(&self) -> Result<Vec<String>> {
        Ok(self.capabilities.clone())
    }
    
    async fn coordinate_deployment(&self, request: DeploymentRequest) -> Result<DeploymentResponse> {
        let url = format!("{}/api/v1/coordinate", self.endpoint);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if response.status().is_success() {
            Ok(DeploymentResponse {
                deployment_id: request.deployment_id,
                status: "success".to_string(),
                message: "Deployment coordinated successfully".to_string(),
                endpoints: HashMap::new(),
            })
        } else {
            Ok(DeploymentResponse {
                deployment_id: request.deployment_id,
                status: "failed".to_string(),
                message: format!("Deployment failed: {}", response.status()),
                endpoints: HashMap::new(),
            })
        }
    }
    
    async fn get_service_status(&self, service_id: &str) -> Result<ServiceStatus> {
        let url = format!("{}/api/v1/services/{}/status", self.endpoint, service_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let status_data: serde_json::Value = response.json().await?;
            Ok(ServiceStatus {
                service_id: service_id.to_string(),
                status: status_data.get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                replicas: status_data.get("replicas")
                    .and_then(|r| r.as_u64())
                    .unwrap_or(0) as u32,
                endpoint: status_data.get("endpoint")
                    .and_then(|e| e.as_str())
                    .map(String::from),
            })
        } else {
            Ok(ServiceStatus {
                service_id: service_id.to_string(),
                status: "unknown".to_string(),
                replicas: 0,
                endpoint: None,
            })
        }
    }
    
    async fn scale_service(&self, service_id: &str, replicas: u32) -> Result<()> {
        let url = format!("{}/api/v1/services/{}/scale", self.endpoint, service_id);
        let scale_request = serde_json::json!({
            "replicas": replicas
        });
        
        let response = self.client.post(&url).json(&scale_request).send().await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Scale operation failed: {}", response.status()))
        }
    }
    
    async fn get_logs(&self, service_id: &str, lines: Option<u32>) -> Result<Vec<String>> {
        let mut url = format!("{}/api/v1/services/{}/logs", self.endpoint, service_id);
        
        if let Some(line_count) = lines {
            url.push_str(&format!("?lines={}", line_count));
        }
        
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let logs_data: serde_json::Value = response.json().await?;
            
            if let Some(logs_array) = logs_data.get("logs").and_then(|l| l.as_array()) {
                Ok(logs_array.iter()
                    .filter_map(|log| log.as_str().map(String::from))
                    .collect())
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![format!("Failed to retrieve logs: {}", response.status())])
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub deployment_id: String,
    pub manifest: serde_json::Value,
    pub primal_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub endpoints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service_id: String,
    pub status: String,
    pub replicas: u32,
    pub endpoint: Option<String>,
} 