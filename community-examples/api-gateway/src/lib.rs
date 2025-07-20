//! # api-gateway
//!
//! RESTful API gateway for BiomeOS services
//! 
//! This primal was generated using the BiomeOS Primal SDK.

use biomeos_primal_sdk::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;
use axum::{routing::get, Router};

/// Configuration for this primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewayConfig {
    /// Custom configuration fields
    pub custom_field: String,
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        Self {
            custom_field: "default_value".to_string(),
        }
    }
}

/// The main primal implementation
pub struct ApiGateway {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    config: ApiGatewayConfig,
}

impl ApiGateway {
    /// Create a new instance of this primal
    pub fn new(config: ApiGatewayConfig) -> Self {
        let metadata = PrimalMetadata {
            name: "api-gateway".to_string(),
            primal_type: Community { name: "api-gateway", category: Communication },
            version: "0.1.0".to_string(),
            description: "RESTful API gateway for BiomeOS services".to_string(),
            author: "API Team <api@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            documentation: None,
            keywords: vec!["biomeos", "primal"],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![Custom { name: "WebService", description: "Web service API capability" }];
        
        Self {
            metadata,
            capabilities,
            config,
        }
    }
}

#[async_trait]
impl EcoPrimal for ApiGateway {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        tracing::info!("Initializing api-gateway primal");
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
        tracing::info!("Shutting down api-gateway primal");
        // Add your cleanup logic here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_primal_creation() {
        let config = ApiGatewayConfig::default();
        let primal = ApiGateway::new(config);
        
        assert_eq!(primal.metadata().name, "api-gateway");
        assert!(!primal.capabilities().is_empty());
    }
    
    #[tokio::test]
    async fn test_ping_request() {
        let config = ApiGatewayConfig::default();
        let primal = ApiGateway::new(config);
        
        let request = PrimalRequest::new("ping", serde_json::json!({}));
        let response = primal.handle_request(request).await.unwrap();
        
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let config = ApiGatewayConfig::default();
        let primal = ApiGateway::new(config);
        
        let health = primal.health_check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
