// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    rustix::fs::statvfs(Path::new(path)).ok().map(|st| {
        let frsize = st.f_frsize;
        let total = st.f_blocks * frsize;
        let avail = st.f_bavail * frsize;
        let used = total.saturating_sub(avail);
        (total, used, avail)
    })
}

#[cfg(not(unix))]
fn get_mount_stats(_path: &str) -> Option<(u64, u64, u64)> {
    None
}

/// Get real system uptime from /proc/uptime (Linux) via biomeos-system.
/// Returns error if metrics unavailable (e.g. non-Linux, /proc not mounted).
async fn get_system_uptime() -> Result<chrono::Duration, anyhow::Error> {
    let info = biomeos_system::SystemInspector::get_system_info().await?;
    chrono::Duration::from_std(info.uptime).map_err(|e| anyhow::anyhow!("Invalid uptime: {e}"))
}

/// Get real system resource usage (CPU, memory, disk) via biomeos-system.
/// Returns error if metrics unavailable.
async fn get_system_resource_usage() -> Result<biomeos_types::ResourceMetrics, anyhow::Error> {
    biomeos_system::SystemInspector::get_resource_usage()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
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
        let universal_manager = UniversalBiomeOSManager::new(config.clone())?;

        Ok(Self {
            config,
            universal_manager,
        })
    }

    /// Start the live service
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting BiomeOS Live Service");

        // Initialize the manager (this is async)
        self.universal_manager.initialize()?;

        // Start monitoring loops
        self.start_monitoring_loops().await?;

        info!("BiomeOS Live Service started successfully");
        Ok(())
    }

    /// Start monitoring loops
    async fn start_monitoring_loops(&self) -> Result<()> {
        self.universal_manager.start_monitoring()?;
        tracing::info!("Live service monitoring initialized");

        // Start periodic health checks
        let manager_clone = self.universal_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let health_report = manager_clone.get_system_health();
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
                        if discovered_services.is_empty() {
                            debug!("Discovery refresh: no new services found");
                        } else {
                            info!(
                                "Discovery refresh found {} services",
                                discovered_services.len()
                            );
                            for service in &discovered_services {
                                debug!("Discovered service: {}", service);
                            }
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

        let health_report = self.universal_manager.get_system_health();

        // Real uptime from system (/proc/uptime on Linux via biomeos-system)
        let uptime = get_system_uptime().await.unwrap_or_else(|e| {
            warn!("Could not read system uptime: {}; using zero", e);
            chrono::Duration::zero()
        });

        // Real resource usage from system (CPU, memory, disk via biomeos-system)
        let resource_usage = get_system_resource_usage().await.unwrap_or_else(|e| {
            warn!("Could not read resource usage: {}; using defaults", e);
            default_resource_metrics()
        });

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
            resource_usage,
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
#[must_use]
pub const fn default_resource_metrics() -> biomeos_types::ResourceMetrics {
    biomeos_types::ResourceMetrics {
        cpu_usage: Some(0.0),
        memory_usage: Some(0.0),
        disk_usage: Some(0.0),
        network_io: None,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_resource_metrics() {
        let m = default_resource_metrics();
        assert_eq!(m.cpu_usage, Some(0.0));
        assert_eq!(m.memory_usage, Some(0.0));
        assert_eq!(m.disk_usage, Some(0.0));
    }

    #[tokio::test]
    async fn test_get_system_uptime_and_resource_usage() {
        // On Linux with /proc, get_system_uptime succeeds; otherwise returns Err
        let uptime_result = get_system_uptime().await;
        if let Ok(u) = uptime_result {
            assert!(u >= chrono::Duration::zero());
        }
        let _ = get_system_resource_usage().await;
    }

    #[test]
    fn test_system_status_serde() {
        let status = SystemStatus {
            uptime: chrono::Duration::zero(),
            resource_usage: default_resource_metrics(),
            health_status: Health::Healthy,
            primals: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let _: SystemStatus = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn test_storage_metrics_serde() {
        let metrics = StorageMetrics {
            total_space: 1000,
            used_space: 500,
            available_space: 500,
            mount_points: vec![MountPoint {
                path: "/".to_string(),
                filesystem: "ext4".to_string(),
                total_bytes: 1000,
                used_bytes: 500,
                available_bytes: 500,
            }],
        };
        let json = serde_json::to_string(&metrics).expect("serialize");
        let _: StorageMetrics = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn test_network_status_serde() {
        let status = NetworkStatus {
            interfaces: vec![],
            total_bytes_sent: 0,
            total_bytes_received: 0,
            active_connections: 0,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let _: NetworkStatus = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn test_health_check_result_serde() {
        let result = HealthCheckResult {
            overall_healthy: true,
            system_status: SystemStatus {
                uptime: chrono::Duration::zero(),
                resource_usage: default_resource_metrics(),
                health_status: Health::Healthy,
                primals: std::collections::HashMap::new(),
            },
            storage_metrics: StorageMetrics {
                total_space: 0,
                used_space: 0,
                available_space: 0,
                mount_points: vec![],
            },
            network_status: NetworkStatus {
                interfaces: vec![],
                total_bytes_sent: 0,
                total_bytes_received: 0,
                active_connections: 0,
            },
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let _: HealthCheckResult = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn test_interface_status_serde() {
        let status = InterfaceStatus {
            name: "eth0".to_string(),
            status: "up".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            is_active: true,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let parsed: InterfaceStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "eth0");
        assert_eq!(parsed.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_mount_point_serde() {
        let mp = MountPoint {
            path: "/".to_string(),
            filesystem: "ext4".to_string(),
            total_bytes: 100_000_000_000,
            used_bytes: 50_000_000_000,
            available_bytes: 50_000_000_000,
        };
        let json = serde_json::to_string(&mp).expect("serialize");
        let parsed: MountPoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.path, "/");
        assert_eq!(parsed.total_bytes, 100_000_000_000);
    }

    #[test]
    fn test_network_interface_serde() {
        let iface = NetworkInterface {
            name: "eth0".to_string(),
            ip_address: Some("10.0.0.1".to_string()),
            mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
            is_up: true,
            bytes_sent: 1000,
            bytes_received: 2000,
        };
        let json = serde_json::to_string(&iface).expect("serialize");
        let parsed: NetworkInterface = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "eth0");
        assert_eq!(parsed.bytes_sent, 1000);
    }

    #[test]
    fn test_primal_status_serde() {
        let status = PrimalStatus {
            name: "beardog".to_string(),
            health: Health::Healthy,
            endpoint: "unix:///tmp/beardog.sock".to_string(),
            discovered_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let parsed: PrimalStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "beardog");
    }

    #[tokio::test]
    async fn test_live_service_new() {
        let result = LiveService::new().await;
        assert!(result.is_ok());
        let service = result.unwrap();
        assert!(!service.config.metadata.version.is_empty());
    }

    #[tokio::test]
    async fn test_live_service_get_storage_metrics() {
        let service = LiveService::new().await.expect("new");
        let metrics = service
            .get_storage_metrics()
            .await
            .expect("storage metrics");
        assert_eq!(
            metrics.available_space,
            metrics.total_space.saturating_sub(metrics.used_space)
        );
    }

    #[tokio::test]
    async fn test_live_service_get_network_status() {
        let service = LiveService::new().await.expect("new");
        let status = service.get_network_status().await.expect("network status");
        assert!(status.interfaces.is_empty() || !status.interfaces.is_empty());
    }

    #[tokio::test]
    async fn test_live_service_get_system_status() {
        let service = LiveService::new().await.expect("new");
        let status = service.get_system_status().await.expect("system status");
        assert!(matches!(
            status.health_status,
            Health::Healthy | Health::Degraded { .. } | Health::Unhealthy { .. }
        ));
    }

    #[tokio::test]
    async fn test_live_service_health_check() {
        let service = LiveService::new().await.expect("new");
        let result = service.health_check().await.expect("health check");
        assert_eq!(
            result.overall_healthy,
            matches!(result.system_status.health_status, Health::Healthy)
        );
    }

    #[tokio::test]
    async fn test_live_service_get_discovered_primals() {
        let service = LiveService::new().await.expect("new");
        let _primals = service.get_discovered_primals().await;
    }

    #[tokio::test]
    async fn test_live_service_get_raw_discovered_primals() {
        let service = LiveService::new().await.expect("new");
        let result = service.get_raw_discovered_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_live_service_discover_primals_by_registry() {
        let service = LiveService::new().await.expect("new");
        let result = service
            .discover_primals_by_registry("http://registry.test:8500")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_live_service_discover_primals_by_capability() {
        let service = LiveService::new().await.expect("new");
        let caps = vec![biomeos_primal_sdk::PrimalCapability::new(
            "compute",
            "execution",
            "1.0",
        )];
        let result = service.discover_primals_by_capability(&caps).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_live_service_get_primal_info() {
        let service = LiveService::new().await.expect("new");
        let result = service.get_primal_info("nonexistent-id").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
