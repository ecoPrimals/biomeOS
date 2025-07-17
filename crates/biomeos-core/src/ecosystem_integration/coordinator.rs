//! # Ecosystem Coordinator
//!
//! Main coordination system for ecosystem integration between all Primals.
//! This module provides the primary interface for managing the ecosystem
//! and coordinating between different Primal types.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use super::messaging::{EcosystemMessageBus, InMemoryMessageBus, PrimalClient};
use super::service_registry::EcosystemServiceRegistry;
use super::types::EcosystemStatus;
use crate::BiomeResult;

/// Ecosystem coordinator that manages the integration between all Primals
pub struct EcosystemCoordinator {
    pub service_registry: EcosystemServiceRegistry,
    message_bus: Arc<dyn EcosystemMessageBus>,
    primal_clients: HashMap<String, Arc<dyn PrimalClient>>,
}

impl EcosystemCoordinator {
    pub fn new() -> Self {
        let message_bus = Arc::new(InMemoryMessageBus::new());
        let service_registry = EcosystemServiceRegistry::new(message_bus.clone());

        Self {
            service_registry,
            message_bus,
            primal_clients: HashMap::new(),
        }
    }

    /// Register a Primal client
    pub fn register_primal_client(&mut self, primal_type: String, client: Arc<dyn PrimalClient>) {
        self.primal_clients.insert(primal_type, client);
    }

    /// Initialize the ecosystem
    pub async fn initialize_ecosystem(&self) -> BiomeResult<()> {
        info!("Initializing ecosystem coordination");

        // Initialize all registered Primal clients
        for (primal_type, client) in &self.primal_clients {
            info!("Initializing {} client", primal_type);
            client.initialize().await?;
        }

        info!("Ecosystem coordination initialized successfully");
        Ok(())
    }

    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> BiomeResult<EcosystemStatus> {
        let health = self.service_registry.check_ecosystem_health().await?;
        let services = {
            let services = self.service_registry.services.read().await;
            services.len()
        };

        Ok(EcosystemStatus {
            health,
            total_services: services,
            active_primals: self.primal_clients.len(),
            uptime: Duration::from_secs(0), // Would track actual uptime
        })
    }

    /// Get message bus reference
    pub fn message_bus(&self) -> Arc<dyn EcosystemMessageBus> {
        self.message_bus.clone()
    }

    /// Get registered Primal clients
    pub fn get_primal_clients(&self) -> &HashMap<String, Arc<dyn PrimalClient>> {
        &self.primal_clients
    }

    /// Check if a specific Primal type is registered
    pub fn has_primal_client(&self, primal_type: &str) -> bool {
        self.primal_clients.contains_key(primal_type)
    }

    /// Get a specific Primal client
    pub fn get_primal_client(&self, primal_type: &str) -> Option<&Arc<dyn PrimalClient>> {
        self.primal_clients.get(primal_type)
    }

    /// Shutdown the ecosystem coordinator
    pub async fn shutdown(&self) -> BiomeResult<()> {
        info!("Shutting down ecosystem coordination");

        // Unregister all services
        let services = self.service_registry.list_services().await?;
        for service in services {
            if let Err(e) = self
                .service_registry
                .unregister_service(&service.service_id)
                .await
            {
                tracing::warn!("Failed to unregister service {}: {}", service.service_id, e);
            }
        }

        info!("Ecosystem coordination shutdown complete");
        Ok(())
    }
}

impl Default for EcosystemCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
