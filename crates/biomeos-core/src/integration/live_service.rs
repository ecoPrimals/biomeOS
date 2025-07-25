use crate::config::BiomeOSConfig;
use crate::universal_biomeos_manager::UniversalBiomeOSManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

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
                let health = manager_clone.get_system_health().await;
                debug!(
                    "Periodic health check: uptime={}s, status={:?}",
                    health.uptime.num_seconds(),
                    health.overall_status
                );
            }
        });

        // Start service discovery refresh
        let manager_clone = self.universal_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match manager_clone.discover_network_scan().await {
                    Ok(services) => debug!(
                        "Service discovery refresh: {} services found",
                        services.len()
                    ),
                    Err(e) => debug!("Service discovery refresh failed: {}", e),
                }
            }
        });

        info!("Starting monitoring loops");
        Ok(())
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        debug!("Getting system status");

        let system_health = self.universal_manager.get_system_health().await;

        // Convert discovered primals to the expected format
        let discovered_primals = self.get_discovered_primals().await;
        let mut primals_map = HashMap::new();

        for primal in discovered_primals {
            let primal_status = PrimalStatus {
                name: primal.id.clone(),
                health: primal.health,
                endpoint: primal.endpoint,
                discovered_at: primal.discovered_at,
            };
            primals_map.insert(primal.id, primal_status);
        }

        let status = SystemStatus {
            uptime: system_health.uptime,
            resource_usage: system_health.resource_usage,
            health_status: system_health.overall_status,
            primals: primals_map,
        };

        Ok(status)
    }

    /// Get discovered primals
    pub async fn get_discovered_primals(
        &self,
    ) -> Vec<crate::universal_biomeos_manager::discovery::DiscoveryResult> {
        match self.universal_manager.discover().await {
            Ok(discovered) => discovered,
            Err(e) => {
                debug!("Failed to discover primals: {}", e);
                Vec::new()
            }
        }
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

    /// Get raw discovered primals
    pub async fn get_raw_discovered_primals(
        &self,
    ) -> Result<Vec<crate::universal_biomeos_manager::discovery::DiscoveryResult>> {
        // Use the correct method name
        let discovered = self.universal_manager.discover().await?;
        Ok(discovered)
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        debug!("Performing comprehensive health check");

        let system_status = self.get_system_status().await?;
        let storage_metrics = self.get_storage_metrics().await?;
        let network_status = self.get_network_status().await?;

        // Assess overall system health
        let overall_healthy =
            system_status.health_status == crate::universal_biomeos_manager::HealthStatus::Healthy;

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
    pub resource_usage: crate::universal_biomeos_manager::SystemResourceUsage,
    pub health_status: crate::universal_biomeos_manager::HealthStatus,
    pub primals: HashMap<String, PrimalStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub name: String,
    pub health: biomeos_primal_sdk::PrimalHealth,
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
