//! # Ecosystem Messaging
//!
//! Message bus and communication protocols for ecosystem integration.
//! This module provides the messaging infrastructure for communication
//! between different Primals in the ecosystem.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use super::types::{EcosystemMessage, EcosystemMessageType};
use crate::BiomeResult;

/// Message bus for ecosystem communication
#[async_trait]
pub trait EcosystemMessageBus: Send + Sync {
    /// Send a message to a specific Primal
    async fn send(&self, message: EcosystemMessage) -> BiomeResult<()>;

    /// Broadcast a message to all Primals
    async fn broadcast(&self, message: EcosystemMessage) -> BiomeResult<()>;

    /// Subscribe to messages of a specific type
    async fn subscribe(&self, message_type: EcosystemMessageType) -> BiomeResult<()>;
}

/// In-memory message bus implementation for testing and development
pub struct InMemoryMessageBus {
    subscribers:
        Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<EcosystemMessage>>>>>,
}

impl Default for InMemoryMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EcosystemMessageBus for InMemoryMessageBus {
    async fn send(&self, message: EcosystemMessage) -> BiomeResult<()> {
        debug!(
            "Sending message: {} -> {}",
            message.from_primal, message.to_primal
        );

        let subscribers = self.subscribers.read().await;
        if let Some(senders) = subscribers.get(&message.to_primal) {
            for sender in senders {
                if let Err(e) = sender.send(message.clone()) {
                    warn!("Failed to send message to subscriber: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn broadcast(&self, message: EcosystemMessage) -> BiomeResult<()> {
        debug!("Broadcasting message from: {}", message.from_primal);

        let subscribers = self.subscribers.read().await;
        for senders in subscribers.values() {
            for sender in senders {
                if let Err(e) = sender.send(message.clone()) {
                    warn!("Failed to broadcast message to subscriber: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn subscribe(&self, _message_type: EcosystemMessageType) -> BiomeResult<()> {
        // Implementation would set up subscription channels
        // For now, this is a placeholder
        Ok(())
    }
}

/// Trait for ecosystem communication
#[async_trait]
pub trait EcosystemCommunication: Send + Sync {
    /// Send a message to another Primal
    async fn send_message(&self, message: EcosystemMessage) -> BiomeResult<()>;

    /// Handle an incoming message
    async fn handle_message(
        &mut self,
        message: EcosystemMessage,
    ) -> BiomeResult<Option<EcosystemMessage>>;

    /// Broadcast status to the ecosystem
    async fn broadcast_status(&self) -> BiomeResult<()>;

    /// Register with the ecosystem
    async fn register_service(
        &self,
        registration: super::types::EcosystemServiceRegistration,
    ) -> BiomeResult<()>;

    /// Deregister from the ecosystem
    async fn deregister_service(&self, service_id: &str) -> BiomeResult<()>;
}

/// Trait for Primal clients
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Initialize the Primal client
    async fn initialize(&self) -> BiomeResult<()>;

    /// Get Primal health status
    async fn health_check(&self) -> BiomeResult<crate::HealthStatus>;

    /// Send a message to this Primal
    async fn send_message(
        &self,
        message: EcosystemMessage,
    ) -> BiomeResult<Option<EcosystemMessage>>;
}
