//! # ml-processor
//!
//! Machine learning task processor
//! 
//! This primal was generated using the BiomeOS Primal SDK.

use biomeos_primal_sdk::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;


/// Configuration for this primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlProcessorConfig {
    /// Custom configuration fields
    pub custom_field: String,
}

impl Default for MlProcessorConfig {
    fn default() -> Self {
        Self {
            custom_field: "default_value".to_string(),
        }
    }
}

/// The main primal implementation
pub struct MlProcessor {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    config: MlProcessorConfig,
}

impl MlProcessor {
    /// Create a new instance of this primal
    pub fn new(config: MlProcessorConfig) -> Self {
        let metadata = PrimalMetadata {
            name: "ml-processor".to_string(),
            primal_type: Community { name: "ml-processor", category: AI },
            version: "0.1.0".to_string(),
            description: "Machine learning task processor".to_string(),
            author: "ML Team <ml@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            documentation: None,
            keywords: vec!["biomeos", "primal"],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![AI, Custom { name: "MachineLearning", description: "Machine learning processing" }];
        
        Self {
            metadata,
            capabilities,
            config,
        }
    }
}

#[async_trait]
impl EcoPrimal for MlProcessor {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        tracing::info!("Initializing ml-processor primal");
        // Add your initialization logic here
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        tracing::debug!("Handling request: {}", request.method);
        
        match request.method.as_str() {
            "ping" => {
                Ok(PrimalResponse::success(
                    request.request_id,
                    serde_json::json!({"message": "pong"})
                ))
            }
            "get_config" => {
                Ok(PrimalResponse::success(
                    request.request_id,
                    serde_json::to_value(&self.config).unwrap()
                ))
            }
            _ => {
                Err(PrimalError::InvalidRequest(
                    format!("Unknown method: {}", request.method)
                ))
            }
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        // Implement your health check logic here
        PrimalHealth::healthy()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        tracing::info!("Shutting down ml-processor primal");
        // Add your cleanup logic here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_primal_creation() {
        let config = MlProcessorConfig::default();
        let primal = MlProcessor::new(config);
        
        assert_eq!(primal.metadata().name, "ml-processor");
        assert!(!primal.capabilities().is_empty());
    }
    
    #[tokio::test]
    async fn test_ping_request() {
        let config = MlProcessorConfig::default();
        let primal = MlProcessor::new(config);
        
        let request = PrimalRequest::new("ping", serde_json::json!({}));
        let response = primal.handle_request(request).await.unwrap();
        
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let config = MlProcessorConfig::default();
        let primal = MlProcessor::new(config);
        
        let health = primal.health_check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
