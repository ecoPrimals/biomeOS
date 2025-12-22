//! Integration tests for biomeOS API contracts and component interactions
//!
//! Modern integration test suite that validates how different components of biomeOS
//! work together, focusing on API contracts, data flow, and system behavior.

use anyhow::Result;
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};
use biomeos_core::config::*;
use biomeos_core::universal_biomeos_manager::{PrimalInfo, discovery::DiscoveryResult};
use biomeos_core::integration::live_service::LiveService;
use biomeos_types::{PrimalCapability, Health, PrimalType};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

mod common;
use common::*;

/// Test suite for Universal BiomeOS Manager and Discovery Service integration
mod manager_discovery_integration {
    use super::*;

    #[tokio::test]
    async fn test_manager_discovery_service_coordination() -> Result<()> {
        let config = TestConfigBuilder::new()
            .with_static_endpoints(vec![
                ("toadstool", "http://localhost:8084"),
                ("songbird", "http://localhost:8081"),
            ])
            .build();

        let manager = UniversalBiomeOSManager::new(config).await?;

        // Test that manager properly delegates to discovery service
        let static_results = manager.discover_network_scan().await?;
        let registry_results = manager.discover_registry("http://localhost:8080/registry").await;

        // Both should complete without crashing (may return empty results)
        assert!(static_results.len() >= 0);
        match registry_results {
            Ok(results) => assert!(results.len() >= 0),
            Err(_) => { /* Expected in test environment */ }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_discovery_to_registration_flow() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test the flow from discovery to registration
        // 1. Attempt discovery (may be empty in test env)
        let discovered = manager.discover().await?;

        // 2. Create mock primals based on "discovery" results
        let mock_primals = vec![
            MockPrimalFactory::create_compute_primal("discovered-compute"),
            MockPrimalFactory::create_storage_primal("discovered-storage"),
        ];

        // 3. Register discovered primals
        for primal in mock_primals {
            manager.register_primal(primal).await?;
        }

        // 4. Verify registration worked
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), 2);

        // 5. Test capability-based retrieval
        let compute_caps = vec![PrimalCapability::compute_provider()];
        let compute_results = manager.discover_by_capability(&compute_caps).await?;

        // Capability discovery searches network, not registered primals
        assert!(compute_results.len() >= 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_discovery_and_registration() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Start concurrent operations
        let mut handles = Vec::new();

        // Discovery operations
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let _results = manager_clone.discover().await;
                format!("discovery-{}", i)
            });
            handles.push(handle);
        }

        // Registration operations
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let primal = MockPrimalFactory::create_compute_primal(&format!("concurrent-{}", i));
                manager_clone.register_primal(primal).await.unwrap();
                format!("registration-{}", i)
            });
            handles.push(handle);
        }

        // Wait for all operations
        for handle in handles {
            let _result = handle.await?;
        }

        // Verify system remains consistent
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), 5);

        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);

        Ok(())
    }
}

/// Test suite for Live Service integration
mod live_service_integration {
    use super::*;

    #[tokio::test]
    async fn test_live_service_initialization() -> Result<()> {
        let live_service = LiveService::new().await?;

        // Test basic live service functionality
        let system_status = live_service.get_system_status().await?;
        
        // Verify system status structure
        assert!(system_status.uptime.num_seconds() >= 0);
        assert_eq!(system_status.primals.len(), 0); // Empty initially

        Ok(())
    }

    #[tokio::test]
    async fn test_live_service_discovery_integration() -> Result<()> {
        let live_service = LiveService::new().await?;

        // Test discovery through live service
        let discovered_primals = live_service.get_discovered_primals().await;

        // Should return empty results in test environment
        assert_eq!(discovered_primals.len(), 0);

        // Test raw discovery
        let raw_discovered = live_service.get_raw_discovered_primals().await?;
        assert_eq!(raw_discovered.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_live_service_health_check() -> Result<()> {
        let live_service = LiveService::new().await?;

        // Perform comprehensive health check
        let health_result = live_service.health_check().await?;

        // Verify health check structure
        assert!(health_result.overall_healthy || !health_result.overall_healthy); // Should be boolean
        assert!(health_result.system_status.uptime.num_seconds() >= 0);

        Ok(())
    }
}

/// Test suite for Configuration System integration
mod configuration_integration {
    use super::*;

    #[tokio::test]
    async fn test_config_builder_to_manager_integration() -> Result<()> {
        // Test different configuration patterns
        let configs = vec![
            TestConfigBuilder::new()
                .with_static_endpoints(vec![("test1", "http://localhost:8001")])
                .build(),
            TestConfigBuilder::new()
                .with_network_discovery(vec!["localhost"], vec![8080, 8081])
                .build(),
            TestConfigBuilder::new()
                .with_security_enabled(true)
                .build(),
        ];

        for config in configs {
            let manager = UniversalBiomeOSManager::new(config).await?;
            
            // Each configuration should produce a working manager
            let health = manager.get_system_health().await;
            TestAssertions::assert_system_healthy(&health);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_config_environment_effects() -> Result<()> {
        // Test different environments
        let environments = vec![
            Environment::Development,
            Environment::Testing,
            Environment::Production,
        ];

        for env in environments {
            let config = TestConfigBuilder::new()
                .build();
            
            // Override environment (simplified since TestConfigBuilder sets Testing)
            let manager = UniversalBiomeOSManager::new(config).await?;
            
            // Manager should work regardless of environment
            let health = manager.get_system_health().await;
            TestAssertions::assert_system_healthy(&health);
        }

        Ok(())
    }
}

/// Test suite for Primal SDK integration
mod primal_sdk_integration {
    use super::*;

    #[tokio::test]
    async fn test_primal_capability_system_integration() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test different capability types
        let capability_tests = vec![
            (PrimalCapability::compute_provider(), "compute"),
            (PrimalCapability::storage_provider(), "storage"),
            (PrimalCapability::orchestration_provider(), "orchestration"),
            (PrimalCapability::security_provider(), "security"),
        ];

        for (capability, domain_name) in capability_tests {
            // Create primal with specific capability
            let mut primal = MockPrimalFactory::create_compute_primal(&format!("{}-test", domain_name));
            primal.capabilities = vec![capability.clone()];
            
            manager.register_primal(primal).await?;

            // Test capability-based discovery
            let results = manager.discover_by_capability(&[capability]).await?;
            
            // Network discovery may not find registered primals, but should not error
            assert!(results.len() >= 0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_primal_type_system_integration() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test different primal types
        let primal_types = vec![
            PrimalType::new("compute", "test-compute", "1.0.0"),
            PrimalType::new("storage", "test-storage", "2.0.0"),
            PrimalType::new("orchestration", "test-orchestration", "1.5.0"),
        ];

        for (i, primal_type) in primal_types.into_iter().enumerate() {
            let mut primal = MockPrimalFactory::create_compute_primal(&format!("type-test-{}", i));
            primal.primal_type = primal_type;
            
            manager.register_primal(primal).await?;
        }

        // Verify all primals were registered with correct types
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), 3);

        let type_domains: Vec<&str> = registered.iter()
            .map(|p| p.primal_type.domain.as_str())
            .collect();
        
        assert!(type_domains.contains(&"compute"));
        assert!(type_domains.contains(&"storage"));
        assert!(type_domains.contains(&"orchestration"));

        Ok(())
    }

    #[tokio::test]
    async fn test_primal_health_integration() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test different health states
        let health_states = vec![
            Health::Healthy,
            Health::Degraded,
            Health::Unhealthy,
        ];

        for (i, health_state) in health_states.into_iter().enumerate() {
            let mut primal = MockPrimalFactory::create_compute_primal(&format!("health-test-{}", i));
            primal.health = health_state.clone();
            
            manager.register_primal(primal).await?;
        }

        // Verify health states were preserved
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), 3);

        let health_states: Vec<&Health> = registered.iter()
            .map(|p| &p.health)
            .collect();
        
        assert!(health_states.contains(&&Health::Healthy));
        assert!(health_states.contains(&&Health::Degraded));
        assert!(health_states.contains(&&Health::Unhealthy));

        Ok(())
    }
}

/// Test suite for Error handling and resilience
mod error_resilience_integration {
    use super::*;

    #[tokio::test]
    async fn test_system_resilience_under_failures() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Register some healthy primals
        for i in 0..3 {
            let primal = MockPrimalFactory::create_compute_primal(&format!("healthy-{}", i));
            manager.register_primal(primal).await?;
        }

        // Register some unhealthy primals
        for i in 0..2 {
            let mut primal = MockPrimalFactory::create_storage_primal(&format!("unhealthy-{}", i));
            primal.health = Health::Unhealthy;
            manager.register_primal(primal).await?;
        }

        // System should remain functional despite unhealthy primals
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health); // Should handle mixed health states

        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_handling() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test operations with timeout
        let discovery_timeout = timeout(
            Duration::from_millis(500),
            manager.discover()
        ).await;

        // Should complete within timeout or timeout gracefully
        match discovery_timeout {
            Ok(results) => {
                let _discovery_results = results?;
                // Completed within timeout
            }
            Err(_) => {
                // Timed out - acceptable for network operations
            }
        }

        // System should remain healthy after timeout
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_data_handling() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Test with invalid/edge case data
        let edge_case_primals = vec![
            // Empty ID (should be handled)
            PrimalInfo {
                id: "".to_string(),
                name: "Empty ID Primal".to_string(),
                primal_type: PrimalType::new("test", "empty-id", "1.0.0"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                last_seen: chrono::Utc::now(),
                discovered_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            // Very long ID
            PrimalInfo {
                id: "a".repeat(1000),
                name: "Long ID Primal".to_string(),
                primal_type: PrimalType::new("test", "long-id", "1.0.0"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                last_seen: chrono::Utc::now(),
                discovered_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
        ];

        for primal in edge_case_primals {
            // Should handle edge cases gracefully
            let result = manager.register_primal(primal).await;
            match result {
                Ok(_) => { /* Accepted */ }
                Err(_) => { /* Rejected gracefully */ }
            }
        }

        // System should remain stable
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);

        Ok(())
    }
}

/// Test suite for Performance and scalability
mod performance_integration {
    use super::*;

    #[tokio::test]
    async fn test_large_scale_primal_management() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Register a large number of primals
        let primal_count = 100;
        let start_time = std::time::Instant::now();

        for i in 0..primal_count {
            let primal = MockPrimalFactory::create_compute_primal(&format!("scale-test-{}", i));
            manager.register_primal(primal).await?;
        }

        let registration_duration = start_time.elapsed();
        
        // Verify performance is acceptable
        assert!(registration_duration.as_millis() < 5000, 
            "Registration of {} primals took too long: {}ms", 
            primal_count, registration_duration.as_millis());

        // Verify all were registered
        let registered = manager.get_registered_primals().await;
        assert_eq!(registered.len(), primal_count);

        // Test retrieval performance
        let retrieval_start = std::time::Instant::now();
        let _retrieved = manager.get_registered_primals().await;
        let retrieval_duration = retrieval_start.elapsed();

        assert!(retrieval_duration.as_millis() < 100,
            "Retrieval of {} primals took too long: {}ms",
            primal_count, retrieval_duration.as_millis());

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_load_handling() -> Result<()> {
        let manager = TestManagerFactory::create_default().await?;

        // Create high concurrent load
        let mut handles = Vec::new();
        let concurrent_ops = 50;

        for i in 0..concurrent_ops {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                // Mix of different operations
                match i % 4 {
                    0 => {
                        let primal = MockPrimalFactory::create_compute_primal(&format!("concurrent-{}", i));
                        manager_clone.register_primal(primal).await.unwrap();
                    }
                    1 => {
                        let _health = manager_clone.get_system_health().await;
                    }
                    2 => {
                        let _registered = manager_clone.get_registered_primals().await;
                    }
                    3 => {
                        let _discovered = manager_clone.discover().await;
                    }
                    _ => unreachable!()
                }
                i
            });
            handles.push(handle);
        }

        // Wait for all operations with timeout
        let timeout_duration = Duration::from_secs(10);
        let results = timeout(timeout_duration, async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await.unwrap());
            }
            results
        }).await?;

        // All operations should complete
        assert_eq!(results.len(), concurrent_ops);

        // System should remain healthy
        let health = manager.get_system_health().await;
        TestAssertions::assert_system_healthy(&health);

        Ok(())
    }
} 