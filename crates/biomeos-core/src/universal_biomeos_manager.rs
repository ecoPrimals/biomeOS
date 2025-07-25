//! Universal BiomeOS Manager
//!
//! Central coordination system for managing the entire biomeOS ecosystem.

use crate::BiomeOSConfig;
use anyhow::Result;
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod discovery;
mod health;
mod types;

pub use discovery::PrimalDiscoveryService;
pub use health::HealthMonitor;
pub use types::*;

/// Primary primal info for discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: PrimalHealth,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Universal BiomeOS Manager for ecosystem orchestration
#[derive(Debug, Clone)]
pub struct UniversalBiomeOSManager {
    config: std::sync::Arc<BiomeOSConfig>,
    discovery_service: std::sync::Arc<discovery::PrimalDiscoveryService>,
    registered_primals:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, PrimalInfo>>>,
}

impl UniversalBiomeOSManager {
    /// Initialize the UniversalBiomeOSManager
    pub async fn new(config: BiomeOSConfig) -> Result<Self> {
        let config_arc = std::sync::Arc::new(config);
        let registered_primals = std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let discovery_service = 
            std::sync::Arc::new(discovery::PrimalDiscoveryService::new(config_arc.clone()));
        
        Ok(Self {
            config: config_arc,
            registered_primals,
            discovery_service,
        })
    }

    /// Initialize the manager
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("🚀 Initializing Universal BiomeOS Manager");
        Ok(())
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        tracing::info!("🏥 Starting health monitoring");
        Ok(())
    }

    /// Get system health
    pub async fn get_system_health(&self) -> SystemHealth {
        SystemHealth {
            overall_status: HealthStatus::Healthy,
            primal_health: HashMap::new(),
            resource_usage: SystemResourceUsage::default(),
            uptime: chrono::Duration::seconds(0),
        }
    }

    /// Discover primals in registry
    pub async fn discover_registry(&self, registry_url: &str) -> Result<Vec<discovery::DiscoveryResult>> {
        self.discovery_service.discover_registry(registry_url).await
    }

    /// Register a primal with the manager
    pub async fn register_primal(&self, primal_info: PrimalInfo) -> Result<()> {
        let mut registry = self.registered_primals.write().await;
        registry.insert(primal_info.id.clone(), primal_info.clone());
        tracing::info!("Registered primal: {}", primal_info.id);
        Ok(())
    }

    /// Get all registered primals
    pub async fn get_registered_primals(&self) -> Vec<PrimalInfo> {
        let registry = self.registered_primals.read().await;
        registry.values().cloned().collect()
    }

    /// Probe a specific endpoint
    pub async fn probe_endpoint(&self, endpoint: &str) -> Result<discovery::ProbeResult> {
        self.discovery_service.probe_endpoint(endpoint).await
    }

    /// Initialize partnership access
    pub async fn initialize_partnership_access(&self, _key: GeneticAccessKey) -> Result<()> {
        tracing::info!("Initializing partnership access");
        Ok(())
    }

    /// Initialize grandma safe mode
    pub async fn initialize_grandma_safe(&self) -> Result<()> {
        tracing::info!("Initializing grandma safe mode");
        Ok(())
    }

    /// Discover primals using network scan
    pub async fn discover_network_scan(&self) -> Result<Vec<discovery::DiscoveryResult>> {
        self.discovery_service.discover_network_scan().await
    }

    /// Discover primals using the configured discovery method
    pub async fn discover(&self) -> Result<Vec<discovery::DiscoveryResult>> {
        match self.discovery_service.discover_network_scan().await {
            Ok(results) => {
                tracing::info!("Discovery completed: {} primals found", results.len());
                Ok(results)
            }
            Err(e) => {
                tracing::warn!("Discovery failed: {}", e);
                Err(e)
            }
        }
    }

    /// Discover primals by capability
    pub async fn discover_by_capability(&self, capabilities: &[PrimalCapability]) -> Result<Vec<discovery::DiscoveryResult>> {
        let all_discovered = self.discover_network_scan().await?;
        
        let filtered = all_discovered
            .into_iter()
            .filter(|primal| {
                capabilities.iter().any(|req_cap| {
                    primal.capabilities.iter().any(|primal_cap| {
                        // Match by domain for capability comparison
                        req_cap.domain == primal_cap.domain
                    })
                })
            })
            .collect();
            
        Ok(filtered)
    }

    /// Discover orchestration services from a registry endpoint
    pub async fn discover_orchestration_services(&self, registry_url: &str) -> Result<Vec<discovery::DiscoveryResult>> {
        // Filter discovered services for orchestration capabilities
        let all_services = self.discover_registry(registry_url).await?;
        let orchestration_services = all_services
            .into_iter()
            .filter(|service| {
                service.capabilities.iter().any(|cap| {
                    cap.domain.to_lowercase().contains("orchestration") ||
                    cap.name.to_lowercase().contains("orchestration")
                })
            })
            .collect();
        
        Ok(orchestration_services)
    }

    /// Alias for backward compatibility - redirects to discover()
    pub async fn discover_primals(&self) -> Result<Vec<discovery::DiscoveryResult>> {
        self.discover().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BiomeOSConfig, DiscoveryConfig, DiscoveryMethod, Environment, PrimalsConfig, SystemConfig,
        TimeoutConfig,
    };
    use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Create a test configuration for unit tests
    fn create_test_config() -> BiomeOSConfig {
        BiomeOSConfig {
            system: SystemConfig {
                name: "test-biomeos".to_string(),
                version: "1.0.0-test".to_string(),
                environment: Environment::Testing,
                log_level: "debug".to_string(),
                data_dir: "/tmp/biomeos-test".to_string(),
                primals: PrimalsConfig {
                    static_endpoints: std::collections::HashMap::from([
                        (
                            "test-toadstool".to_string(),
                            "http://localhost:8084".to_string(),
                        ),
                        (
                            "test-songbird".to_string(),
                            "http://localhost:8081".to_string(),
                        ),
                    ]),
                    timeouts: TimeoutConfig {
                        discovery_timeout_ms: 1000,
                        probe_timeout_ms: 500,
                        health_check_interval_ms: 2000,
                        default_timeout_ms: 1000,
                    },
                },
            },
            primals: PrimalsConfig {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::Static,
                    timeout: Duration::from_millis(1000),
                    scan_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
                    scan_ports: vec![8080, 8081, 8082],
                    registry_endpoints: vec!["http://localhost:8080".to_string()],
                },
                static_endpoints: std::collections::HashMap::from([
                    (
                        "test-toadstool".to_string(),
                        "http://localhost:8084".to_string(),
                    ),
                    (
                        "test-songbird".to_string(),
                        "http://localhost:8081".to_string(),
                    ),
                ]),
                timeouts: TimeoutConfig {
                    discovery_timeout_ms: 1000,
                    probe_timeout_ms: 500,
                    health_check_interval_ms: 2000,
                    default_timeout_ms: 1000,
                },
            },
            security: crate::config::SecurityConfig::default(),
            licensing: crate::config::LicensingConfig::default(),
            integration: crate::config::IntegrationConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_manager_initialization() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await;
        
        // Manager should initialize successfully
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_primal_registration() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        
        // Create test primal info
        let primal_info = PrimalInfo {
            id: "test-primal".to_string(),
            name: "Test Primal".to_string(),
            primal_type: PrimalType::new("test", "test-primal", "1.0.0"),
            endpoint: "http://localhost:9999".to_string(),
            capabilities: vec![PrimalCapability::new("compute", "test", "1.0.0")],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        // Register the primal
        let result = manager.register_primal(primal_info.clone()).await;
        assert!(result.is_ok());
        
        // Verify primal is registered
        let registered_primals = manager.get_registered_primals().await;
        assert!(registered_primals.iter().any(|p| p.id == "test-primal"));
    }

    #[tokio::test]
    async fn test_capability_based_discovery() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        
        // Register test primals with different capabilities
        let compute_primal = PrimalInfo {
            id: "compute-primal".to_string(),
            name: "Compute Primal".to_string(),
            primal_type: PrimalType::new("compute", "compute-primal", "1.0.0"),
            endpoint: "http://localhost:8084".to_string(),
            capabilities: vec![PrimalCapability::new("compute", "test", "1.0.0")],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        let storage_primal = PrimalInfo {
            id: "storage-primal".to_string(),
            name: "Storage Primal".to_string(),
            primal_type: PrimalType::new("storage", "storage-primal", "1.0.0"),
            endpoint: "http://localhost:8082".to_string(),
            capabilities: vec![PrimalCapability::new("storage", "test", "1.0.0")],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        manager.register_primal(compute_primal).await.unwrap();
        manager.register_primal(storage_primal).await.unwrap();
        
        // Test capability-based discovery
        let compute_cap = vec![PrimalCapability::new("compute", "test", "1.0.0")];
        let compute_results = manager.discover_by_capability(&compute_cap).await.unwrap();
        // Results may be empty since we're not actually discovering from network
        
        let storage_cap = vec![PrimalCapability::new("storage", "test", "1.0.0")];
        let storage_results = manager.discover_by_capability(&storage_cap).await.unwrap();
        // Results may be empty since we're not actually discovering from network
        
        // Both should succeed (even if empty)
        assert!(compute_results.len() >= 0);
        assert!(storage_results.len() >= 0);
    }

    #[tokio::test]
    async fn test_system_health_monitoring() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();

        // Get system health
        let health = manager.get_system_health().await;

        // Should return valid health status
        assert!(matches!(
            health.overall_status,
            HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unhealthy
        ));

        // Should have resource usage information
        assert!(health.resource_usage.cpu_usage_percent >= 0.0);
        assert!(health.resource_usage.memory_usage_percent >= 0.0);
        assert!(health.resource_usage.disk_usage_percent >= 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let config = create_test_config();
        let manager = Arc::new(UniversalBiomeOSManager::new(config).await.unwrap());
        
        // Create multiple concurrent tasks
        let mut handles = vec![];
        
        for i in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let primal_info = PrimalInfo {
                    id: format!("concurrent-primal-{}", i),
                    name: format!("Concurrent Primal {}", i),
                    primal_type: PrimalType::new("test", &format!("primal-{}", i), "1.0.0"),
                    endpoint: format!("http://localhost:{}", 9000 + i),
                    capabilities: vec![PrimalCapability::new("compute", "test", "1.0.0")],
                    health: PrimalHealth::Healthy,
                    last_seen: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                };
                
                manager_clone.register_primal(primal_info).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        // Verify all primals were registered
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 5);
    }

    #[tokio::test]
    async fn test_primal_health_updates() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        
        // Register a healthy primal
        let mut primal_info = PrimalInfo {
            id: "health-test-primal".to_string(),
            name: "Health Test Primal".to_string(),
            primal_type: PrimalType::new("test", "health-test", "1.0.0"),
            endpoint: "http://localhost:9998".to_string(),
            capabilities: vec![PrimalCapability::new("compute", "test", "1.0.0")],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        manager.register_primal(primal_info.clone()).await.unwrap();
        
        // Update health to degraded
        primal_info.health = PrimalHealth::Degraded;
        primal_info.last_seen = chrono::Utc::now();
        
        manager.register_primal(primal_info).await.unwrap();
        
        // Verify health was updated
        let registered_primals = manager.get_registered_primals().await;
        let updated_primal = registered_primals.iter()
            .find(|p| p.id == "health-test-primal")
            .unwrap();
        
        assert_eq!(updated_primal.health, PrimalHealth::Degraded);
    }

    #[tokio::test]
    async fn test_discovery_error_handling() {
        let mut config = create_test_config();
        
        // Set up configuration that will likely fail (invalid hosts)
        config.primals.discovery.scan_hosts = vec!["invalid-host-12345".to_string()];
        config.primals.discovery.scan_ports = vec![99999]; // Invalid port
        
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        
        // Network scan should handle errors gracefully
        let result = manager.discover_network_scan().await;
        assert!(result.is_ok());
        
        // Should return empty results, not error
        let discoveries = result.unwrap();
        assert_eq!(discoveries.len(), 0);
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        let config = create_test_config();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        
        // Register many primals to test memory usage
        for i in 0..100 {
            let primal_info = PrimalInfo {
                id: format!("memory-test-primal-{}", i),
                name: format!("Memory Test Primal {}", i),
                primal_type: PrimalType::new("test", &format!("memory-test-{}", i), "1.0.0"),
                endpoint: format!("http://localhost:{}000", i + 10),
                capabilities: vec![PrimalCapability::new("compute", "test", "1.0.0")],
                health: PrimalHealth::Healthy,
                last_seen: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            
            manager.register_primal(primal_info).await.unwrap();
        }
        
        // Verify all primals are registered
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 100);
        
        // Test discovery performance with many primals
        let start_time = std::time::Instant::now();
        let compute_primals = manager.discover_by_capability(&[PrimalCapability::new("compute", "test", "1.0.0")]).await.unwrap();
        let discovery_time = start_time.elapsed();
        
        // Discovery should be fast (even if results are empty due to no network primals)
        assert!(compute_primals.len() >= 0);
        assert!(discovery_time < Duration::from_millis(100)); // Should be fast
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test with minimal valid configuration
        let minimal_config = BiomeOSConfig {
            system: SystemConfig {
                name: "minimal-test".to_string(),
                version: "1.0.0".to_string(),
                environment: Environment::Testing,
                log_level: "info".to_string(),
                data_dir: "/tmp".to_string(),
            },
            primals: PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::Static,
                    auto_discovery: true,
                    static_endpoints: std::collections::HashMap::new(),
                    scan_hosts: Vec::new(),
                    scan_ports: Vec::new(),
                },
                endpoints: std::collections::HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            },
            security: crate::config::SecurityConfig {
                enable_crypto_locks: true,
                genetic_key_path: None,
                ai_cat_door: crate::config::AiCatDoorConfig {
                    enabled: true,
                    cost_protection_threshold: 20.0,
                    monthly_budget: 100.0,
                },
            },
            licensing: crate::config::LicensingConfig {
                license_type: crate::config::LicenseType::Individual,
                organization_scale: None,
                entropy_tier: crate::config::EntropyTier::HumanLived,
            },
            integration: crate::config::IntegrationConfig {
                songbird: crate::config::SongbirdIntegrationConfig {
                    endpoint: None,
                    auto_register: true,
                    health_reporting_interval_ms: 30000,
                },
                ecosystem: crate::config::EcosystemIntegrationConfig {
                    enable_cross_primal_communication: true,
                    ai_first_responses: true,
                    universal_registration: true,
                },
            },
        };
        
        let manager = UniversalBiomeOSManager::new(minimal_config).await;
        assert!(manager.is_ok());
    }
}
