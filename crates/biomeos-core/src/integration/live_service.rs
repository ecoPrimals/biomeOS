//! Live service monitoring and health-check infrastructure
//!
//! Migrated to unified types via `biomeos-types`.

use crate::universal_biomeos_manager::UniversalBiomeOSManager;
use anyhow::Result;
use biomeos_types::{BiomeOSConfig, Health};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Get (total, used, available) bytes for a mount point via statvfs.
#[cfg(unix)]
fn get_mount_stats(path: &str) -> Option<(u64, u64, u64)> {
    nix::sys::statvfs::statvfs(Path::new(path)).ok().map(|st| {
        let frsize = st.fragment_size() as u64;
        let total = st.blocks() as u64 * frsize;
        let avail = st.blocks_available() as u64 * frsize;
        let used = total.saturating_sub(avail);
        (total, used, avail)
    })
}

#[cfg(not(unix))]
fn get_mount_stats(_path: &str) -> Option<(u64, u64, u64)> {
    None
}

/// Runtime service that manages the biomeOS lifecycle
#[derive(Debug)]
pub struct LiveService {
    /// Active configuration
    pub config: BiomeOSConfig,
    /// Universal manager for primal orchestration
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
                    health_report.health, health_report.subject.name
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
                            info!(
                                "Discovery refresh found {} services",
                                discovered_services.len()
                            );
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
                                info!(
                                    "Registry discovery found {} services",
                                    registry_services.len()
                                );
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
            resource_usage: health_report
                .metrics
                .resources
                .unwrap_or_else(default_resource_metrics),
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
                info!(
                    "Network scan discovered {} raw endpoints",
                    raw_endpoints.len()
                );
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
                info!(
                    "Registry discovery found {} primals",
                    registry_endpoints.len()
                );
                Ok(registry_endpoints)
            }
            Err(e) => {
                warn!(
                    "Failed to discover primals from registry {}: {}",
                    registry_url, e
                );
                Ok(Vec::new())
            }
        }
    }

    /// Discover primals by capability (new method)
    pub async fn discover_primals_by_capability(
        &self,
        capabilities: &[biomeos_primal_sdk::PrimalCapability],
    ) -> Result<Vec<String>> {
        debug!("Discovering primals by capabilities: {:?}", capabilities);

        match self
            .universal_manager
            .discover_by_capability(capabilities)
            .await
        {
            Ok(capability_endpoints) => {
                info!(
                    "Capability-based discovery found {} primals",
                    capability_endpoints.len()
                );
                Ok(capability_endpoints)
            }
            Err(e) => {
                warn!("Failed to discover primals by capability: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Get comprehensive primal information (new method)
    pub async fn get_primal_info(
        &self,
        primal_id: &str,
    ) -> Result<Option<crate::universal_biomeos_manager::PrimalInfo>> {
        debug!("Getting primal information for: {}", primal_id);

        // Get all registered primals and search for the requested one
        let registered_primals = self.universal_manager.get_registered_primals().await;

        for primal_info in registered_primals {
            if primal_info.id == primal_id {
                debug!(
                    "Found primal info for {}: {:?}",
                    primal_id, primal_info.name
                );
                return Ok(Some(primal_info));
            }
        }

        debug!("No primal info found for: {}", primal_id);
        Ok(None)
    }

    /// Get storage metrics
    pub async fn get_storage_metrics(&self) -> Result<StorageMetrics> {
        debug!("Getting storage metrics");

        let mut total_space = 0u64;
        let mut used_space = 0u64;
        let mut mount_points = Vec::new();

        // Check common mount points
        let common_mounts = vec!["/", "/home", "/var", "/tmp"];

        for mount_point in common_mounts {
            if std::fs::metadata(mount_point).is_err() {
                continue;
            }
            let mp = mount_point.to_string();
            let (total, used, available) = get_mount_stats(mount_point).unwrap_or((0, 0, 0));
            total_space = total_space.saturating_add(total);
            used_space = used_space.saturating_add(used);
            mount_points.push(MountPoint {
                path: mp,
                filesystem: "unknown".to_string(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
            });
        }

        let available_space = total_space.saturating_sub(used_space);

        Ok(StorageMetrics {
            total_space,
            used_space,
            available_space,
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

/// Aggregate system status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// System uptime
    pub uptime: chrono::Duration,
    /// CPU / memory / disk resource usage
    pub resource_usage: biomeos_types::ResourceMetrics,
    /// Overall health assessment
    pub health_status: Health,
    /// Per-primal health status
    pub primals: HashMap<String, PrimalStatus>,
}

/// Status of a single discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    /// Primal name
    pub name: String,
    /// Health status
    pub health: biomeos_types::Health,
    /// Endpoint URL or socket path
    pub endpoint: String,
    /// When this primal was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Disk / storage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// Total disk space in bytes
    pub total_space: u64,
    /// Used disk space in bytes
    pub used_space: u64,
    /// Available disk space in bytes
    pub available_space: u64,
    /// Per-mount-point details
    pub mount_points: Vec<MountPoint>,
}

/// A single filesystem mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    /// Mount path
    pub path: String,
    /// Filesystem type
    pub filesystem: String,
    /// Total capacity in bytes
    pub total_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
}

/// Network status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Cumulative bytes sent
    pub total_bytes_sent: u64,
    /// Cumulative bytes received
    pub total_bytes_received: u64,
    /// Number of active TCP/UDP connections
    pub active_connections: u32,
}

/// A network interface and its traffic counters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g. "eth0")
    pub name: String,
    /// Primary IP address
    pub ip_address: Option<String>,
    /// MAC address
    pub mac_address: Option<String>,
    /// Whether the interface is up
    pub is_up: bool,
    /// Bytes sent on this interface
    pub bytes_sent: u64,
    /// Bytes received on this interface
    pub bytes_received: u64,
}

/// Result of a periodic health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Whether the system is healthy overall
    pub overall_healthy: bool,
    /// System status snapshot
    pub system_status: SystemStatus,
    /// Storage metrics
    pub storage_metrics: StorageMetrics,
    /// Network status
    pub network_status: NetworkStatus,
    /// When this check was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Status of a single network interface (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStatus {
    /// Interface name
    pub name: String,
    /// Human-readable status string
    pub status: String,
    /// IP address (if assigned)
    pub ip_address: Option<String>,
    /// Whether the interface is active
    pub is_active: bool,
}

/// Create default (zero) resource metrics
pub fn default_resource_metrics() -> biomeos_types::ResourceMetrics {
    biomeos_types::ResourceMetrics {
        cpu_usage: Some(0.0),
        memory_usage: Some(0.0),
        disk_usage: Some(0.0),
        network_io: None,
    }
}
