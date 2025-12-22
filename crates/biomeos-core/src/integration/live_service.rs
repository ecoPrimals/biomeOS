// MIGRATED TO UNIFIED TYPES: Integration module updated for biomeos-types
use biomeos_types::{BiomeOSConfig, Health};
use crate::universal_biomeos_manager::UniversalBiomeOSManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct LiveService {
    pub config: BiomeOSConfig,
    pub universal_manager: UniversalBiomeOSManager,
}

impl LiveService {
    /// Create a new live service instance
    pub async fn new() -> Result<Self> {
        let config = BiomeOSConfig::default();
        let universal_manager = UniversalBiomeOSManager::new(config.clone()).await?;

        Ok(Self {
            config,
            universal_manager,
        })
    }

    /// Start the live service
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting BiomeOS Live Service");

        // Initialize the manager (this is async)
        self.universal_manager.initialize().await?;

        // Start monitoring loops
        self.start_monitoring_loops().await?;

        info!("BiomeOS Live Service started successfully");
        Ok(())
    }

    /// Start monitoring loops
    async fn start_monitoring_loops(&self) -> Result<()> {
        // Start the universal manager monitoring
        // Start universal manager monitoring (implementation pending manager fixes)
        tracing::info!("Live service monitoring initialized");
        // self.universal_manager.start_monitoring().await?; // Uncomment when universal manager compiles

        // Start periodic health checks
        let manager_clone = self.universal_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let health_report = manager_clone.get_system_health().await;
                debug!(
                    "Periodic health check: system={:?}, subject={}",
                    health_report.health,
                    health_report.subject.name
                );
            }
        });

        // Start service discovery refresh
        let manager_clone = self.universal_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                // ✅ DISCOVERY FUNCTIONALITY RE-ENABLED
                // Use the unified discovery system to find available primals
                match manager_clone.discover_network_scan().await {
                    Ok(discovered_services) => {
                        if !discovered_services.is_empty() {
                            info!("Discovery refresh found {} services", discovered_services.len());
                            for service in &discovered_services {
                                debug!("Discovered service: {}", service);
                            }
                        } else {
                            debug!("Discovery refresh: no new services found");
                        }
                    }
                    Err(e) => {
                        warn!("Discovery refresh failed: {}", e);
                    }
                }
                
                // Also attempt static discovery for configured endpoints
                match manager_clone.discover().await {
                    Ok(static_services) => {
                        if !static_services.is_empty() {
                            debug!("Static discovery found {} services", static_services.len());
                        }
                    }
                    Err(e) => {
                        debug!("Static discovery failed: {}", e);
                    }
                }
                
                // Also attempt registry discovery if configured
                if let Some(ref registry) = manager_clone.config.discovery.registry {
                    match manager_clone.discover_registry(&registry.url).await {
                        Ok(registry_services) => {
                            if !registry_services.is_empty() {
                                info!("Registry discovery found {} services", registry_services.len());
                            }
                        }
                        Err(e) => {
                            debug!("Registry discovery failed (may be expected): {}", e);
                        }
                    }
                }
            }
        });

        info!("Starting monitoring loops");
        Ok(())
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        debug!("Getting system status");

        let health_report = self.universal_manager.get_system_health().await;

        // Create placeholder uptime and resource usage since the structure changed
        let duration = chrono::Utc::now() - health_report.generated_at;
        let uptime = chrono::Duration::try_seconds(duration.num_seconds()).unwrap_or_default();
        
        // Re-enabled: Get discovered primals from the universal manager
        let mut primals_map = HashMap::new();
        
        // Try to get discovered primals from the universal manager
        match self.universal_manager.discover().await {
            Ok(_primal_endpoints) => {
                // Get registered primals for detailed information (returns Vec<PrimalInfo> directly)
                let registered_primals = self.universal_manager.get_registered_primals().await;
                for primal_info in registered_primals {
                    let primal_status = PrimalStatus {
                        name: primal_info.name,
                        health: primal_info.health,
                        endpoint: primal_info.endpoint,
                        discovered_at: primal_info.discovered_at,
                    };
                    primals_map.insert(primal_info.id, primal_status);
                }
                
                debug!("Found {} registered primals", primals_map.len());
            }
            Err(e) => {
                warn!("Failed to discover primals: {}", e);
            }
        }

        let status = SystemStatus {
            uptime,
            resource_usage: health_report.metrics.resources.unwrap_or_else(default_resource_metrics),
            health_status: health_report.health,
            primals: primals_map,
        };

        Ok(status)
    }

    /// Get discovered primals (re-enabled with proper discovery integration)
    pub async fn get_discovered_primals(&self) -> Vec<String> {
        debug!("Getting discovered primals from universal manager");
        
        match self.universal_manager.discover().await {
            Ok(primal_endpoints) => {
                info!("Discovered {} primals", primal_endpoints.len());
                primal_endpoints
            }
            Err(e) => {
                warn!("Failed to discover primals: {}", e);
                Vec::new()
            }
        }
    }

    /// Get raw discovered primals (re-enabled with proper discovery integration)
    pub async fn get_raw_discovered_primals(&self) -> Result<Vec<String>> {
        debug!("Getting raw discovered primals from universal manager");
        
        // Use network scan discovery for raw results
        match self.universal_manager.discover_network_scan().await {
            Ok(raw_endpoints) => {
                info!("Network scan discovered {} raw endpoints", raw_endpoints.len());
                Ok(raw_endpoints)
            }
            Err(e) => {
                warn!("Failed to perform network scan discovery: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Discover primals by registry (new method)
    pub async fn discover_primals_by_registry(&self, registry_url: &str) -> Result<Vec<String>> {
        debug!("Discovering primals from registry: {}", registry_url);
        
        match self.universal_manager.discover_registry(registry_url).await {
            Ok(registry_endpoints) => {
                info!("Registry discovery found {} primals", registry_endpoints.len());
                Ok(registry_endpoints)
            }
            Err(e) => {
                warn!("Failed to discover primals from registry {}: {}", registry_url, e);
                Ok(Vec::new())
            }
        }
    }

    /// Discover primals by capability (new method)
    pub async fn discover_primals_by_capability(&self, capabilities: &[biomeos_primal_sdk::PrimalCapability]) -> Result<Vec<String>> {
        debug!("Discovering primals by capabilities: {:?}", capabilities);
        
        match self.universal_manager.discover_by_capability(capabilities).await {
            Ok(capability_endpoints) => {
                info!("Capability-based discovery found {} primals", capability_endpoints.len());
                Ok(capability_endpoints)
            }
            Err(e) => {
                warn!("Failed to discover primals by capability: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Get comprehensive primal information (new method)
    pub async fn get_primal_info(&self, primal_id: &str) -> Result<Option<crate::universal_biomeos_manager::PrimalInfo>> {
        debug!("Getting primal information for: {}", primal_id);
        
        // Get all registered primals and search for the requested one
        let registered_primals = self.universal_manager.get_registered_primals().await;
        
        for primal_info in registered_primals {
            if primal_info.id == primal_id {
                debug!("Found primal info for {}: {:?}", primal_id, primal_info.name);
                return Ok(Some(primal_info));
            }
        }
        
        debug!("No primal info found for: {}", primal_id);
        Ok(None)
    }

    /// Get storage metrics
    pub async fn get_storage_metrics(&self) -> Result<StorageMetrics> {
        debug!("Getting storage metrics");

        let total_space = 0u64;
        let used_space = 0u64;
        let mut mount_points = Vec::new();

        // Check common mount points
        let common_mounts = vec!["/", "/home", "/var", "/tmp"];

        for mount_point in common_mounts {
            if let Ok(_metadata) = std::fs::metadata(mount_point) {
                // Basic filesystem check - in real implementation would use statvfs
                mount_points.push(MountPoint {
                    path: mount_point.to_string(),
                    filesystem: "unknown".to_string(),
                    total_bytes: 1024 * 1024 * 1024 * 100, // 100GB default
                    used_bytes: 1024 * 1024 * 1024 * 50,   // 50GB used
                    available_bytes: 1024 * 1024 * 1024 * 50, // 50GB available
                });
            }
        }

        Ok(StorageMetrics {
            total_space,
            used_space,
            available_space: total_space - used_space,
            mount_points,
        })
    }

    /// Get network status
    pub async fn get_network_status(&self) -> Result<NetworkStatus> {
        debug!("Getting network status");

        // Basic network status - in real implementation would check actual interfaces
        Ok(NetworkStatus {
            interfaces: vec![],
            total_bytes_sent: 0,
            total_bytes_received: 0,
            active_connections: 0,
        })
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        debug!("Performing comprehensive health check");

        let system_status = self.get_system_status().await?;
        let storage_metrics = self.get_storage_metrics().await?;
        let network_status = self.get_network_status().await?;

        // Assess overall system health using unified health system
        let overall_healthy = matches!(system_status.health_status, Health::Healthy);

        Ok(HealthCheckResult {
            overall_healthy,
            system_status,
            storage_metrics,
            network_status,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub uptime: chrono::Duration,
    pub resource_usage: biomeos_types::ResourceMetrics,
    pub health_status: Health,
    pub primals: HashMap<String, PrimalStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub name: String,
    pub health: biomeos_types::Health,
    pub endpoint: String,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub mount_points: Vec<MountPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub path: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub interfaces: Vec<NetworkInterface>,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub is_up: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub overall_healthy: bool,
    pub system_status: SystemStatus,
    pub storage_metrics: StorageMetrics,
    pub network_status: NetworkStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStatus {
    pub name: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub is_active: bool,
}

// Helper functions for type conversion and defaults (avoiding orphan rule issues)
pub fn default_resource_metrics() -> biomeos_types::ResourceMetrics {
    biomeos_types::ResourceMetrics {
        cpu_usage: Some(0.0),
        memory_usage: Some(0.0),
        disk_usage: Some(0.0),
        network_io: None,
    }
}
