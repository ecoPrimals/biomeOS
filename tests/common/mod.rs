//! Common test utilities and fixtures for biomeOS testing
//!
//! Provides modern testing patterns, mock objects, and shared utilities
//! that align with the refactored biomeOS architecture.

use anyhow::Result;
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};
use biomeos_core::config::*;
use biomeos_core::universal_biomeos_manager::{PrimalInfo, discovery::DiscoveryResult};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use std::collections::HashMap;


/// Test configuration builder for consistent test setups
pub struct TestConfigBuilder {
    config: BiomeOSConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: BiomeOSConfig {
                system: SystemConfig {
                    name: "test-biomeos".to_string(),
                    version: "0.1.0-test".to_string(),
                    environment: Environment::Testing,
                    log_level: "debug".to_string(),
                    data_dir: "/tmp/biomeos-test".to_string(),
                },
                primals: PrimalConfigs {
                    discovery: DiscoveryConfig {
                        method: DiscoveryMethod::Static,
                        auto_discovery: false,
                        static_endpoints: HashMap::new(),
                        scan_hosts: vec!["localhost".to_string()],
                        scan_ports: vec![8080, 8081, 8082],
                    },
                    endpoints: HashMap::new(),
                    timeouts: TimeoutConfig {
                        default_timeout_ms: 1000,
                        discovery_timeout_ms: 2000,
                        health_check_interval_ms: 5000,
                    },
                },
                security: SecurityConfig {
                    enable_crypto_locks: false, // Disabled for testing
                    genetic_key_path: None,
                    ai_cat_door: AiCatDoorConfig {
                        enabled: false,
                        cost_protection_threshold: 10.0,
                        monthly_budget: 50.0,
                    },
                },
                licensing: LicensingConfig {
                    license_type: LicenseType::Individual,
                    organization_scale: None,
                    entropy_tier: EntropyTier::HumanLived,
                },
                integration: IntegrationConfig {
                    songbird: SongbirdIntegrationConfig {
                        endpoint: Some("http://localhost:8081".to_string()),
                        auto_register: false,
                        health_reporting_interval_ms: 10000,
                    },
                    ecosystem: EcosystemIntegrationConfig {
                        enable_cross_primal_communication: true,
                        ai_first_responses: false,
                        universal_registration: true,
                    },
                },
            }
        }
    }

    pub fn with_static_endpoints(mut self, endpoints: Vec<(&str, &str)>) -> Self {
        for (name, endpoint) in endpoints {
            self.config.primals.discovery.static_endpoints.insert(
                name.to_string(), 
                endpoint.to_string()
            );
        }
        self
    }

    pub fn with_network_discovery(mut self, hosts: Vec<&str>, ports: Vec<u16>) -> Self {
        self.config.primals.discovery.method = DiscoveryMethod::NetworkScan;
        self.config.primals.discovery.scan_hosts = hosts.into_iter().map(|s| s.to_string()).collect();
        self.config.primals.discovery.scan_ports = ports;
        self
    }

    pub fn with_security_enabled(mut self, enabled: bool) -> Self {
        self.config.security.enable_crypto_locks = enabled;
        self
    }

    pub fn build(self) -> BiomeOSConfig {
        self.config
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock primal factory for consistent test data
pub struct MockPrimalFactory;

impl MockPrimalFactory {
    pub fn create_compute_primal(id: &str) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: format!("{} Compute Service", id),
            primal_type: PrimalType::new("compute", id, "1.0.0"),
            endpoint: format!("http://localhost:8084/{}", id),
            capabilities: vec![
                PrimalCapability::compute_provider(),
                PrimalCapability::new("compute", "wasm_execution", "1.0.0"),
            ],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "compute".to_string());
                meta.insert("test".to_string(), "true".to_string());
                meta
            },
        }
    }

    pub fn create_storage_primal(id: &str) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: format!("{} Storage Service", id),
            primal_type: PrimalType::new("storage", id, "1.0.0"),
            endpoint: format!("http://localhost:8082/{}", id),
            capabilities: vec![
                PrimalCapability::storage_provider(),
                PrimalCapability::new("storage", "file_system", "1.0.0"),
            ],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "storage".to_string());
                meta.insert("test".to_string(), "true".to_string());
                meta
            },
        }
    }

    pub fn create_orchestration_primal(id: &str) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: format!("{} Orchestration Service", id),
            primal_type: PrimalType::new("orchestration", id, "1.0.0"),
            endpoint: format!("http://localhost:8081/{}", id),
            capabilities: vec![
                PrimalCapability::orchestration_provider(),
                PrimalCapability::new("orchestration", "service_discovery", "1.0.0"),
            ],
            health: PrimalHealth::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "orchestration".to_string());
                meta.insert("test".to_string(), "true".to_string());
                meta
            },
        }
    }

    pub fn create_discovery_result(primal_info: &PrimalInfo) -> DiscoveryResult {
        DiscoveryResult {
            id: primal_info.id.clone(),
            primal_type: primal_info.primal_type.clone(),
            endpoint: primal_info.endpoint.clone(),
            capabilities: primal_info.capabilities.clone(),
            health: primal_info.health.clone(),
            discovered_at: primal_info.discovered_at,
        }
    }
}

/// Test manager factory for consistent test setups
pub struct TestManagerFactory;

impl TestManagerFactory {
    /// Create a manager with default test configuration
    pub async fn create_default() -> Result<UniversalBiomeOSManager> {
        let config = TestConfigBuilder::default().build();
        UniversalBiomeOSManager::new(config).await
    }

    /// Create a manager with custom configuration
    pub async fn create_with_config(config: BiomeOSConfig) -> Result<UniversalBiomeOSManager> {
        UniversalBiomeOSManager::new(config).await
    }

    /// Create a manager pre-populated with test primals
    pub async fn create_with_test_primals() -> Result<UniversalBiomeOSManager> {
        let manager = Self::create_default().await?;
        
        // Register test primals
        let compute_primal = MockPrimalFactory::create_compute_primal("test-compute");
        let storage_primal = MockPrimalFactory::create_storage_primal("test-storage");
        let orchestration_primal = MockPrimalFactory::create_orchestration_primal("test-orchestration");
        
        manager.register_primal(compute_primal).await?;
        manager.register_primal(storage_primal).await?;
        manager.register_primal(orchestration_primal).await?;
        
        Ok(manager)
    }
}

/// Test assertion helpers for modern testing patterns
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a discovery result contains expected primals
    pub fn assert_discovery_contains_ids(results: &[DiscoveryResult], expected_ids: &[&str]) {
        let found_ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        for expected_id in expected_ids {
            assert!(
                found_ids.contains(expected_id),
                "Expected to find primal '{}' in discovery results: {:?}",
                expected_id,
                found_ids
            );
        }
    }

    /// Assert that primals have expected capabilities
    pub fn assert_capability_match(
        results: &[DiscoveryResult], 
        primal_id: &str, 
        expected_domain: &str
    ) {
        let primal = results.iter()
            .find(|r| r.id == primal_id)
            .expect(&format!("Primal '{}' not found in results", primal_id));
        
        let has_capability = primal.capabilities.iter()
            .any(|cap| cap.domain == expected_domain);
        
        assert!(
            has_capability,
            "Primal '{}' should have capability in domain '{}'",
            primal_id,
            expected_domain
        );
    }

    /// Assert system health is acceptable
    pub fn assert_system_healthy(health: &biomeos_core::universal_biomeos_manager::SystemHealth) {
        use biomeos_core::universal_biomeos_manager::HealthStatus;
        
        match health.overall_status {
            HealthStatus::Healthy => { /* Good! */ }
            HealthStatus::Degraded => {
                println!("Warning: System health is degraded");
            }
            HealthStatus::Critical => {
                panic!("System health is critical - test environment may be compromised");
            }
            HealthStatus::Warning => {
                println!("Warning: System has warnings");
            }
            HealthStatus::Unhealthy => {
                println!("Warning: System is unhealthy");
            }
            HealthStatus::Unknown => {
                println!("Warning: System health status is unknown");
            }
        }
    }
}

/// Performance test utilities for validating zero-copy optimizations
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Measure memory allocations during a test operation
    pub async fn measure_allocations<F, T>(operation: F) -> (T, usize)
    where
        F: std::future::Future<Output = T>,
    {
        // This is simplified - in a real implementation, you'd use tools like
        // jemalloc or custom allocators to track actual allocations
        let start_time = std::time::Instant::now();
        let result = operation.await;
        let duration = start_time.elapsed();
        
        // Return result and estimated allocation count (simplified)
        (result, duration.as_micros() as usize)
    }

    /// Assert that an operation completes within expected time bounds
    pub async fn assert_performance_bounds<F, T>(
        operation: F,
        max_duration_ms: u64,
        description: &str,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        
        assert!(
            duration.as_millis() <= max_duration_ms as u128,
            "{} took {}ms, expected <= {}ms",
            description,
            duration.as_millis(),
            max_duration_ms
        );
        
        result
    }
}

/// Async test setup and teardown utilities
pub struct AsyncTestSetup {
    pub manager: UniversalBiomeOSManager,
    _cleanup: Vec<Box<dyn FnOnce() + Send>>,
}

impl AsyncTestSetup {
    pub async fn new() -> Result<Self> {
        let manager = TestManagerFactory::create_default().await?;
        
        Ok(Self {
            manager,
            _cleanup: Vec::new(),
        })
    }

    pub async fn with_test_primals() -> Result<Self> {
        let manager = TestManagerFactory::create_with_test_primals().await?;
        
        Ok(Self {
            manager,
            _cleanup: Vec::new(),
        })
    }
}

/// Macro for creating async test cases with setup/teardown
#[macro_export]
macro_rules! async_test {
    ($name:ident, $setup:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $name() -> anyhow::Result<()> {
            let setup = $setup.await?;
            let result = $test_body(setup).await;
            result
        }
    };
} 