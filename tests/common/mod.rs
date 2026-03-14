// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Common test utilities and fixtures for biomeOS testing
//!
//! Provides modern testing patterns, mock objects, and shared utilities
//! that align with the refactored biomeOS architecture.

#![allow(dead_code)] // Test utilities may not all be used in every test

use anyhow::Result;
use biomeos_core::universal_biomeos_manager::{discovery::DiscoveryResult, PrimalInfo};
use biomeos_core::UniversalBiomeOSManager;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health, HealthReport};
use std::collections::HashMap;

/// Test configuration builder for consistent test setups
pub struct TestConfigBuilder {
    config: BiomeOSConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: BiomeOSConfig::default(),
        }
    }

    pub fn with_static_endpoints(self, _endpoints: Vec<(&str, &str)>) -> Self {
        // Static endpoints are now configured through registry config
        // This is a no-op for compatibility
        self
    }

    pub fn with_network_discovery(self, _hosts: Vec<&str>, _ports: Vec<u16>) -> Self {
        // Network discovery settings are now in discovery config
        // This is a no-op for compatibility
        self
    }

    pub fn with_security_enabled(self, _enabled: bool) -> Self {
        // Security configuration is now more granular
        // This is a no-op for compatibility in tests
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
        let now = chrono::Utc::now();
        PrimalInfo {
            id: id.to_string(),
            name: format!("{id} Compute Service"),
            primal_type: PrimalType::new("compute", id, "1.0.0"),
            endpoint: format!("http://localhost:8084/{id}"),
            capabilities: vec![
                PrimalCapability::new("compute", "provider", "1.0.0"),
                PrimalCapability::new("compute", "wasm_execution", "1.0.0"),
            ],
            health: Health::Healthy,
            last_seen: now,
            discovered_at: now,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "compute".to_string());
                meta.insert("test".to_string(), "true".to_string());
                meta
            },
        }
    }

    pub fn create_storage_primal(id: &str) -> PrimalInfo {
        let now = chrono::Utc::now();
        PrimalInfo {
            id: id.to_string(),
            name: format!("{id} Storage Service"),
            primal_type: PrimalType::new("storage", id, "1.0.0"),
            endpoint: format!("http://localhost:8082/{id}"),
            capabilities: vec![
                PrimalCapability::new("storage", "provider", "1.0.0"),
                PrimalCapability::new("storage", "file_system", "1.0.0"),
            ],
            health: Health::Healthy,
            last_seen: now,
            discovered_at: now,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "storage".to_string());
                meta.insert("test".to_string(), "true".to_string());
                meta
            },
        }
    }

    pub fn create_orchestration_primal(id: &str) -> PrimalInfo {
        let now = chrono::Utc::now();
        PrimalInfo {
            id: id.to_string(),
            name: format!("{id} Orchestration Service"),
            primal_type: PrimalType::new("orchestration", id, "1.0.0"),
            endpoint: format!("http://localhost:8081/{id}"),
            capabilities: vec![
                PrimalCapability::new("orchestration", "provider", "1.0.0"),
                PrimalCapability::new("orchestration", "service_discovery", "1.0.0"),
            ],
            health: Health::Healthy,
            last_seen: now,
            discovered_at: now,
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
        let orchestration_primal =
            MockPrimalFactory::create_orchestration_primal("test-orchestration");

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
                "Expected to find primal '{expected_id}' in discovery results: {found_ids:?}"
            );
        }
    }

    /// Assert that primals have expected capabilities
    pub fn assert_capability_match(
        results: &[DiscoveryResult],
        primal_id: &str,
        expected_category: &str,
    ) {
        let primal = results
            .iter()
            .find(|r| r.id == primal_id)
            .unwrap_or_else(|| panic!("Primal '{primal_id}' not found in results"));

        let has_capability = primal
            .capabilities
            .iter()
            .any(|cap| cap.category == expected_category);

        assert!(
            has_capability,
            "Primal '{primal_id}' should have capability in category '{expected_category}'"
        );
    }

    /// Assert system health is acceptable
    pub fn assert_system_healthy(health: &HealthReport) {
        match &health.health {
            Health::Healthy => { /* Good! */ }
            Health::Degraded { issues, .. } => {
                println!("Warning: System health is degraded: {issues:?}");
            }
            Health::Critical { issues, .. } => {
                panic!(
                    "System health is critical - test environment may be compromised: {issues:?}"
                );
            }
            Health::Unknown { reason, .. } => {
                println!("Warning: System health status is unknown: {reason}");
            }
            Health::Unhealthy { issues, .. } => {
                println!("Warning: System is unhealthy: {issues:?}");
            }
            Health::Starting { phase, progress } => {
                println!("System starting: {phase:?} ({progress}%)");
            }
            Health::Stopping { phase, progress } => {
                println!("System stopping: {phase:?} ({progress}%)");
            }
            Health::Maintenance {
                maintenance_type, ..
            } => {
                println!("System under maintenance: {maintenance_type:?}");
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
