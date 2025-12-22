//! # game-engine
//!
//! Lightweight game engine for BiomeOS
//! 
//! This primal was generated using the BiomeOS Primal SDK.

use biomeos_primal_sdk::{
    PrimalRequest, PrimalResponse, PrimalResult, EcoPrimal,
    PrimalMetadata, PrimalCapability, PrimalConfig, PrimalHealth,
    PrimalType, PrimalError
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

/// Utility function for safe JSON serialization with fallback
fn safe_json_serialize<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        tracing::warn!("JSON serialization failed: {}", e);
        serde_json::json!({ "error": "serialization_failed", "details": e.to_string() })
    })
}

/// Configuration for this primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEngineConfig {
    /// Custom configuration fields
    pub custom_field: String,
}

impl Default for GameEngineConfig {
    fn default() -> Self {
        Self {
            custom_field: "default_value".to_string(),
        }
    }
}

/// The main primal implementation
pub struct GameEngine {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    config: GameEngineConfig,
}

impl GameEngine {
    /// Create a new instance of this primal
    pub fn new(config: GameEngineConfig) -> Self {
        let metadata = PrimalMetadata {
            name: "game-engine".to_string(),
            primal_type: Community { name: "game-engine", category: Gaming },
            version: "0.1.0".to_string(),
            description: "Lightweight game engine for BiomeOS".to_string(),
            author: "GameDev <game@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            documentation: None,
            keywords: vec!["biomeos", "primal"],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![Gaming, Custom { name: "GameLogic", description: "Game logic processing" }];
        
        Self {
            metadata,
            capabilities,
            config,
        }
    }
}

#[async_trait]
impl EcoPrimal for GameEngine {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        tracing::info!("Initializing game-engine primal");
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
                    safe_json_serialize(&self.config)
                ))
            }
            _ => {
                Err(PrimalError::validation_error(
                    format!("Unknown method: {}", request.method),
                    vec![]
                ))
            }
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        // Implement your health check logic here
        PrimalHealth::healthy()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        tracing::info!("Shutting down game-engine primal");
        // Add your cleanup logic here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_primal_creation() {
        let config = GameEngineConfig::default();
        let primal = GameEngine::new(config);
        
        assert_eq!(primal.metadata().name, "game-engine");
        assert!(!primal.capabilities().is_empty());
    }
    
    #[tokio::test]
    async fn test_ping_request() {
        let config = GameEngineConfig::default();
        let primal = GameEngine::new(config);
        
        let request = PrimalRequest::new("ping", serde_json::json!({}));
                        let response = primal.handle_request(request).await
                    .unwrap_or_else(|e| {
                        tracing::error!("Request handling failed: {}", e);
                        PrimalResponse::error(
                            request.request_id,
                            "internal_error".to_string(),
                            Some(e.to_string())
                        )
                    });
        
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let config = GameEngineConfig::default();
        let primal = GameEngine::new(config);
        
        let health = primal.health_check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
