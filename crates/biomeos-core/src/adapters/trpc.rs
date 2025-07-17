//! tRPC Universal Adapter
//!
//! This module provides tRPC-based bidirectional communication adapter that can work
//! with any primal supporting tRPC protocol. This enables true bidirectional
//! communication with real-time event streaming and subscription capabilities.

use super::{
    PrimalEvent, UniversalCommConfig, UniversalPrimalAdapter, UniversalRequest, UniversalResponse,
};
use crate::{BiomeError, BiomeResult, PrimalCapability, PrimalHealth};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// tRPC message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TrpcMessage {
    /// Query message
    Query {
        id: String,
        method: String,
        params: serde_json::Value,
    },
    /// Mutation message
    Mutation {
        id: String,
        method: String,
        params: serde_json::Value,
    },
    /// Subscription message
    Subscription {
        id: String,
        method: String,
        params: serde_json::Value,
    },
    /// Result message
    Result {
        id: String,
        result: serde_json::Value,
    },
    /// Error message
    Error { id: String, error: TrpcError },
}

/// tRPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// tRPC Universal Adapter (simplified implementation)
pub struct TrpcUniversalAdapter {
    config: UniversalCommConfig,
    capabilities_cache: Option<Vec<PrimalCapability>>,
}

impl TrpcUniversalAdapter {
    pub fn new(config: UniversalCommConfig) -> BiomeResult<Self> {
        Ok(Self {
            config,
            capabilities_cache: None,
        })
    }
}

#[async_trait]
impl UniversalPrimalAdapter for TrpcUniversalAdapter {
    async fn initialize(&mut self, config: UniversalCommConfig) -> BiomeResult<()> {
        self.config = config;
        // For now, this is a stub implementation
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn discover_capabilities(&self) -> BiomeResult<Vec<PrimalCapability>> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> BiomeResult<PrimalHealth> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn execute_operation(
        &self,
        _request: UniversalRequest,
    ) -> BiomeResult<UniversalResponse> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn subscribe_events(&self, _event_types: Vec<String>) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn send_event(&self, _event: PrimalEvent) -> BiomeResult<()> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn get_status(&self) -> BiomeResult<serde_json::Value> {
        Err(BiomeError::NotImplemented(
            "tRPC adapter not yet fully implemented".to_string(),
        ))
    }

    async fn close(&mut self) -> BiomeResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    #[test]
    fn test_trpc_adapter_creation() {
        let config = UniversalCommConfig {
            endpoint: "ws://localhost:8080".to_string(),
            protocol: super::super::CommunicationProtocol::Trpc,
            timeout: Duration::from_secs(30),
            bidirectional: super::super::BidirectionalConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let adapter = TrpcUniversalAdapter::new(config);
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_trpc_message_serialization() {
        let message = TrpcMessage::Query {
            id: "test-id".to_string(),
            method: "test.method".to_string(),
            params: serde_json::json!({"param": "value"}),
        };

        let serialized = serde_json::to_string(&message);
        assert!(serialized.is_ok());

        let deserialized: TrpcMessage = serde_json::from_str(&serialized.unwrap()).unwrap();
        match deserialized {
            TrpcMessage::Query { id, method, .. } => {
                assert_eq!(id, "test-id");
                assert_eq!(method, "test.method");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
