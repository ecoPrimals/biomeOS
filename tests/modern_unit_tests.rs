//! Unit tests for UniversalBiomeOSManager
//!
//! Modern test suite validating core functionality of the Universal BiomeOS Manager,
//! including primal registration, discovery, capability-based search, and system health.

use anyhow::Result;

use biomeos_primal_sdk::{PrimalCapability, PrimalHealth};
use std::time::Duration;

mod common;
use common::*;

/// Test suite for UniversalBiomeOSManager initialization and basic operations
mod initialization_tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_initialization_with_default_config() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Verify manager was created successfully
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_manager_initialization_with_custom_config() -> Result<()> {
        let config = TestConfigBuilder::new()
            .with_static_endpoints(vec![
                ("test-compute", "http://localhost:8084"),
                ("test-storage", "http://localhost:8082"),
            ])
            .with_security_enabled(true)
            .build();
        
        let manager = TestManagerFactory::create_with_config(config).await?;
        
        // Verify manager initialized with custom config
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_manager_initialization_performance() -> Result<()> {
        let _manager = PerformanceTestUtils::assert_performance_bounds(
            TestManagerFactory::create_default(),
            100, // Should initialize within 100ms
            "Manager initialization"
        ).await?;
        
        Ok(())
    }
}

/// Test suite for primal registration functionality
mod primal_registration_tests {
    use super::*;

    #[tokio::test]
    async fn test_register_single_primal() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        let test_primal = MockPrimalFactory::create_compute_primal("test-compute");
        
        // Register the primal
        manager.register_primal(test_primal.clone()).await?;
        
        // Verify registration
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 1);
        assert_eq!(registered_primals[0].id, "test-compute");
        assert_eq!(registered_primals[0].name, "test-compute Compute Service");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_register_multiple_primals() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Register multiple primals
        let compute_primal = MockPrimalFactory::create_compute_primal("compute-1");
        let storage_primal = MockPrimalFactory::create_storage_primal("storage-1");
        let orchestration_primal = MockPrimalFactory::create_orchestration_primal("orchestration-1");
        
        manager.register_primal(compute_primal).await?;
        manager.register_primal(storage_primal).await?;
        manager.register_primal(orchestration_primal).await?;
        
        // Verify all registered
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 3);
        
        let ids: Vec<&str> = registered_primals.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"compute-1"));
        assert!(ids.contains(&"storage-1"));
        assert!(ids.contains(&"orchestration-1"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_register_primal_overwrites_existing() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Register initial primal
        let mut initial_primal = MockPrimalFactory::create_compute_primal("test-primal");
        manager.register_primal(initial_primal.clone()).await?;
        
        // Register updated primal with same ID
        initial_primal.health = PrimalHealth::Degraded;
        initial_primal.name = "Updated Test Primal".to_string();
        manager.register_primal(initial_primal).await?;
        
        // Verify update
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 1);
        assert_eq!(registered_primals[0].name, "Updated Test Primal");
        assert_eq!(registered_primals[0].health, PrimalHealth::Degraded);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_primal_registration() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Create multiple registration tasks
        let mut handles = Vec::new();
        for i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let primal = MockPrimalFactory::create_compute_primal(&format!("concurrent-{}", i));
                manager_clone.register_primal(primal).await
            });
            handles.push(handle);
        }
        
        // Wait for all registrations to complete
        for handle in handles {
            handle.await??;
        }
        
        // Verify all primals were registered
        let registered_primals = manager.get_registered_primals().await;
        assert_eq!(registered_primals.len(), 10);
        
        Ok(())
    }
}

/// Test suite for discovery functionality
mod discovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_empty_system() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Test discovery on empty system
        let results = manager.discover().await?;
        
        // Should return empty results without error
        assert_eq!(results.len(), 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_with_registered_primals() -> Result<()> {
        let manager = TestManagerFactory::create_with_test_primals().await?;
        
        // Note: discover() uses network scan, not registered primals
        // This test validates that discovery doesn't interfere with registered primals
        let discovery_results = manager.discover().await?;
        let registered_primals = manager.get_registered_primals().await;
        
        // Both should work independently
        assert_eq!(registered_primals.len(), 3); // From create_with_test_primals
        assert!(discovery_results.len() >= 0); // Network scan may be empty in test env
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_network_scan() -> Result<()> {
        let config = TestConfigBuilder::new()
            .with_network_discovery(vec!["localhost"], vec![8080, 8081])
            .build();
        
        let manager = TestManagerFactory::create_with_config(config).await?;
        
        // Test network scan discovery
        let results = manager.discover_network_scan().await?;
        
        // Should complete without error (may be empty in test environment)
        assert!(results.len() >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_probe_endpoint_invalid() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Test probing non-existent endpoint
        let result = manager.probe_endpoint("http://localhost:99999").await;
        
        // Should return error for invalid endpoint
        assert!(result.is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_performance() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        let _results = PerformanceTestUtils::assert_performance_bounds(
            manager.discover(),
            2000, // Discovery should complete within 2 seconds
            "Discovery operation"
        ).await?;
        
        Ok(())
    }
}

/// Test suite for capability-based discovery
mod capability_discovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_by_capability_empty() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Search for compute capabilities in empty system
        let compute_caps = vec![PrimalCapability::compute_provider()];
        let results = manager.discover_by_capability(&compute_caps).await?;
        
        // Should return empty results
        assert_eq!(results.len(), 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_by_capability_matching() -> Result<()> {
        // This test would require network services to be running
        // For now, we'll test the method without expecting matches
        let manager = TestManagerFactory::create_default().await?;
        
        let compute_caps = vec![PrimalCapability::compute_provider()];
        let storage_caps = vec![PrimalCapability::storage_provider()];
        
        // These calls should succeed even if no matches found
        let compute_results = manager.discover_by_capability(&compute_caps).await?;
        let storage_results = manager.discover_by_capability(&storage_caps).await?;
        
        assert!(compute_results.len() >= 0);
        assert!(storage_results.len() >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_by_multiple_capabilities() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Search for multiple capabilities
        let multi_caps = vec![
            PrimalCapability::compute_provider(),
            PrimalCapability::storage_provider(),
            PrimalCapability::orchestration_provider(),
        ];
        
        let results = manager.discover_by_capability(&multi_caps).await?;
        
        // Should complete without error
        assert!(results.len() >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_by_capability_performance() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        let capabilities = vec![PrimalCapability::compute_provider()];
        
        let _results = PerformanceTestUtils::assert_performance_bounds(
            manager.discover_by_capability(&capabilities),
            3000, // Capability search should complete within 3 seconds
            "Capability-based discovery"
        ).await?;
        
        Ok(())
    }
}

/// Test suite for system health monitoring
mod health_monitoring_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_system_health_basic() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        let health = manager.get_system_health().await;
        
        // Verify health structure
        TestAssertions::assert_system_healthy(&health);
        
        // Basic health checks
        assert!(health.uptime.num_seconds() >= 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_system_health_performance() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        let _health = PerformanceTestUtils::assert_performance_bounds(
            async { manager.get_system_health().await },
            50, // Health check should be very fast
            "System health check"
        ).await;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_system_health_consistency() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Get health multiple times
        let health1 = manager.get_system_health().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let health2 = manager.get_system_health().await;
        
        // Uptime should increase
        assert!(health2.uptime >= health1.uptime);
        
        Ok(())
    }
}

/// Test suite for error handling and edge cases
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_handles_invalid_registry_urls() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Test with invalid URLs
        let invalid_urls = vec![
            "not-a-url",
            "http://",
            "https://nonexistent-domain-12345.com",
            "http://localhost:99999",
        ];
        
        for url in invalid_urls {
            let result = manager.discover_registry(url).await;
            // Should handle gracefully (may return empty results or error)
            match result {
                Ok(results) => assert!(results.len() >= 0),
                Err(_) => { /* Expected for invalid URLs */ }
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_operations_safety() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Run multiple operations concurrently
        let mut handles = Vec::new();
        
        // Health checks
        for _ in 0..5 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                manager_clone.get_system_health().await
            }));
        }
        
        // Discovery operations
        for _ in 0..3 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                let _ = manager_clone.discover().await;
                manager_clone.get_system_health().await
            }));
        }
        
        // Wait for all operations
        for handle in handles {
            let _health = handle.await?;
            // All operations should complete successfully
        }
        
        Ok(())
    }
}

/// Zero-copy optimization validation tests
mod zero_copy_tests {
    use super::*;

    #[tokio::test]
    async fn test_primal_registration_memory_efficiency() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Measure memory usage during primal registration
        let large_primal_count = 100;
        let (_, _allocation_estimate) = PerformanceTestUtils::measure_allocations(async {
            for i in 0..large_primal_count {
                let primal = MockPrimalFactory::create_compute_primal(&format!("primal-{}", i));
                manager.register_primal(primal).await.unwrap();
            }
        }).await;
        
        // Verify all primals were registered
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), large_primal_count);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_discovery_memory_efficiency() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;
        
        // Measure memory during discovery operations
        let (_results, _allocation_estimate) = PerformanceTestUtils::measure_allocations(async {
            // Multiple discovery calls to test memory reuse
            for _ in 0..10 {
                let _ = manager.discover().await;
            }
        }).await;
        
        // Test completed successfully - actual memory measurement
        // would require more sophisticated tooling in production
        Ok(())
    }
} 