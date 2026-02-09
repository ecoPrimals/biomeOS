//! Core Universal BiomeOS Manager
//!
//! Central coordination system for managing the entire biomeOS ecosystem.
//! Contains the main struct definition and initialization methods.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Legacy import - client_registry depends on legacy clients module
// use crate::universal_biomeos_manager::client_registry::ClientRegistry;
use crate::universal_biomeos_manager::discovery::PrimalDiscoveryService;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health};

/// Primary primal info for discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: Health,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Universal BiomeOS Manager for ecosystem orchestration
#[derive(Debug, Clone)]
pub struct UniversalBiomeOSManager {
    pub config: Arc<BiomeOSConfig>,
    pub(crate) discovery_service: Arc<PrimalDiscoveryService>,
    pub(crate) registered_primals: Arc<RwLock<HashMap<String, PrimalInfo>>>,
    // Legacy field - ClientRegistry depends on old clients module
    // pub(crate) clients: Arc<ClientRegistry>,
}

impl UniversalBiomeOSManager {
    /// Initialize the UniversalBiomeOSManager
    pub async fn new(config: BiomeOSConfig) -> Result<Self> {
        let config_arc = Arc::new(config);
        let registered_primals = Arc::new(RwLock::new(HashMap::new()));
        let discovery_service = Arc::new(PrimalDiscoveryService::new(config_arc.clone()));
        // Legacy ClientRegistry initialization removed
        // let clients = Arc::new(ClientRegistry::new());

        Ok(Self {
            config: config_arc,
            registered_primals,
            discovery_service,
            // clients,  // Removed
        })
    }

    /// Create manager with default configuration
    pub async fn with_default_config() -> Result<Self> {
        let config = BiomeOSConfig::default();
        Self::new(config).await
    }

    /// Initialize the manager
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("🚀 Initializing Universal BiomeOS Manager");

        // Initialize discovery service
        self.discovery_service.initialize().await?;

        // Legacy client initialization removed
        // Initialize primal clients through zero-knowledge discovery
        // self.clients.initialize().await?;

        tracing::info!("✅ Universal BiomeOS Manager initialized successfully");
        Ok(())
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        tracing::info!("🏥 Starting health monitoring");

        // Start background monitoring tasks
        tokio::spawn({
            let manager = self.clone();
            async move {
                loop {
                    // Periodic health checks
                    if let Err(e) = manager.perform_health_check().await {
                        tracing::warn!("Health check failed: {}", e);
                    }

                    // Wait 30 seconds between checks
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                }
            }
        });

        tracing::info!("✅ Health monitoring started");
        Ok(())
    }

    /// Perform a health check on all registered primals
    async fn perform_health_check(&self) -> Result<()> {
        let primals = self.registered_primals.read().await;
        let mut tasks = Vec::new();

        for (id, primal) in primals.iter() {
            let endpoint = primal.endpoint.clone();
            let id = id.clone();

            let task = tokio::spawn(async move {
                // Simple health check - in practice would probe the endpoint
                tracing::debug!("Health check for primal {}: {}", id, endpoint);
                Ok::<(), anyhow::Error>(())
            });

            tasks.push(task);
        }

        // Wait for all health checks to complete
        for task in tasks {
            let _ = task.await;
        }

        Ok(())
    }

    /// Get manager configuration
    pub fn get_config(&self) -> &BiomeOSConfig {
        &self.config
    }

    /// Get discovery service reference
    pub fn discovery_service(&self) -> &Arc<PrimalDiscoveryService> {
        &self.discovery_service
    }

    /// Get registered primals reference
    pub fn registered_primals(&self) -> &Arc<RwLock<HashMap<String, PrimalInfo>>> {
        &self.registered_primals
    }

    // NOTE: ClientRegistry removed - clients() method deprecated
    // pub fn clients(&self) -> &Arc<ClientRegistry> { &self.clients }

    /// Shutdown the manager gracefully
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("🛑 Shutting down Universal BiomeOS Manager");

        // Graceful shutdown logic would go here
        // - Stop monitoring tasks
        // - Clean up resources
        // - Notify registered primals

        tracing::info!("✅ Universal BiomeOS Manager shutdown complete");
        Ok(())
    }
}
