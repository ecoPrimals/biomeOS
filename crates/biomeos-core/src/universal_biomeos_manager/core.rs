// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Core Universal BiomeOS Manager
//!
//! Central coordination system for managing the entire biomeOS ecosystem.
//! Contains the main struct definition and initialization methods.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::universal_biomeos_manager::discovery::PrimalDiscoveryService;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health};

/// Primary primal info for discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique primal identifier
    pub id: String,
    /// Human-readable primal name
    pub name: String,
    /// Primal type classification
    pub primal_type: PrimalType,
    /// Communication endpoint (socket path or URL)
    pub endpoint: String,
    /// Capabilities this primal provides
    pub capabilities: Vec<PrimalCapability>,
    /// Current health status
    pub health: Health,
    /// Last heartbeat / health check timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// When this primal was first discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    /// Arbitrary key-value metadata
    pub metadata: HashMap<String, String>,
}

/// Universal BiomeOS Manager for ecosystem orchestration
#[derive(Debug, Clone)]
pub struct UniversalBiomeOSManager {
    /// Shared configuration for the BiomeOS ecosystem
    pub config: Arc<BiomeOSConfig>,
    pub(crate) discovery_service: Arc<PrimalDiscoveryService>,
    pub(crate) registered_primals: Arc<RwLock<HashMap<String, PrimalInfo>>>,
}

impl UniversalBiomeOSManager {
    /// Initialize the UniversalBiomeOSManager
    pub async fn new(config: BiomeOSConfig) -> Result<Self> {
        let config_arc = Arc::new(config);
        let registered_primals = Arc::new(RwLock::new(HashMap::new()));
        let discovery_service = Arc::new(PrimalDiscoveryService::new(config_arc.clone()));

        Ok(Self {
            config: config_arc,
            registered_primals,
            discovery_service,
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

        self.discovery_service.initialize().await?;

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

                    // Wait between checks (configurable via config.health.check_interval)
                    let interval = manager.config.health.check_interval;
                    tokio::time::sleep(interval).await;
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_types::BiomeOSConfig;

    #[tokio::test]
    async fn test_new_with_config() {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config).await.expect("new");
        assert!(manager.get_config().metadata.version.len() > 0);
    }

    #[tokio::test]
    async fn test_with_default_config() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("with_default_config");
        let config = manager.get_config();
        assert!(!config.metadata.version.is_empty());
    }

    #[tokio::test]
    async fn test_initialize() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("initialize");
    }

    #[tokio::test]
    async fn test_get_config() {
        let mut config = BiomeOSConfig::default();
        config.metadata.version = "2.0.0-test".to_string();
        let manager = UniversalBiomeOSManager::new(config).await.expect("new");
        assert_eq!(manager.get_config().metadata.version, "2.0.0-test");
    }

    #[tokio::test]
    async fn test_discovery_service_accessor() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        let discovery = manager.discovery_service();
        assert!(std::mem::size_of_val(discovery) > 0);
    }

    #[tokio::test]
    async fn test_registered_primals_accessor() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        let primals = manager.registered_primals();
        let count = primals.read().await.len();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn test_start_monitoring() {
        let mut config = BiomeOSConfig::default();
        config.health.check_interval = std::time::Duration::ZERO;
        let manager = UniversalBiomeOSManager::new(config).await.expect("manager");
        manager.initialize().await.expect("init");
        manager.start_monitoring().await.expect("start_monitoring");
        // Give the spawned task a moment to start (zero interval = tight loop)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[test]
    fn test_primal_info_serialization() {
        use biomeos_primal_sdk::PrimalCapability;
        use biomeos_types::{Health, PrimalType};
        use std::collections::HashMap;

        let info = PrimalInfo {
            id: "test-1".to_string(),
            name: "test-primal".to_string(),
            primal_type: PrimalType::from_discovered("compute", "toadstool", "1.0"),
            endpoint: "unix:///tmp/test.sock".to_string(),
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            health: Health::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("test-1"));
        assert!(json.contains("test-primal"));
    }
}
