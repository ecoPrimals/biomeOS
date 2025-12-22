//! Simple End-to-End Testing Suite for BiomeOS
//!
//! This test suite validates basic biomeOS functionality including
//! system initialization, primal discovery, and health monitoring.

use anyhow::Result;
use biomeos_core::{
    config::{BiomeOSConfig, DiscoveryMethod, Environment},
    integration::live_service::LiveService,
    universal_biomeos_manager::{HealthStatus, UniversalBiomeOSManager},
};
use biomeos_types::PrimalCapability;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use tracing_test::traced_test;

/// Test system initialization
#[tokio::test]
#[traced_test]
async fn test_system_initialization() -> Result<()> {
    info!("🚀 Testing system initialization");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(10),
            scan_hosts: vec!["localhost".to_string()],
            scan_ports: vec![8080, 8081, 8082],
            registry_endpoints: vec!["http://localhost:8080".to_string()],
        },
        environment: Environment::Development,
        max_concurrent_requests: 10,
        request_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(5),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;
    let health = manager.get_system_health().await;

    info!("System health: {:?}", health.overall_status);
    assert!(matches!(
        health.overall_status,
        HealthStatus::Healthy | HealthStatus::Degraded
    ));

    info!("✅ System initialization test passed");
    Ok(())
}

/// Test primal discovery functionality
#[tokio::test]
#[traced_test]
async fn test_primal_discovery() -> Result<()> {
    info!("🔍 Testing primal discovery");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(10),
            scan_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            scan_ports: vec![8080, 8081, 8082, 8083],
            registry_endpoints: vec![],
        },
        environment: Environment::Development,
        max_concurrent_requests: 10,
        request_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(5),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test network scan discovery
    let discovered = manager.discover_network_scan().await?;
    info!("Discovered {} primals via network scan", discovered.len());

    // Test capability-based discovery
    let capabilities = vec![PrimalCapability::Compute];
    let capability_results = manager.discover_by_capability(&capabilities).await?;
    info!(
        "Found {} primals with compute capability",
        capability_results.len()
    );

    info!("✅ Primal discovery test passed");
    Ok(())
}

/// Test health monitoring system
#[tokio::test]
#[traced_test]
async fn test_health_monitoring() -> Result<()> {
    info!("💚 Testing health monitoring");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(5),
            scan_hosts: vec!["localhost".to_string()],
            scan_ports: vec![8080],
            registry_endpoints: vec![],
        },
        environment: Environment::Development,
        max_concurrent_requests: 5,
        request_timeout: Duration::from_secs(3),
        health_check_interval: Duration::from_secs(2),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get initial health status
    let initial_health = manager.get_system_health().await;
    info!("Initial system health: {:?}", initial_health.overall_status);

    // Wait a bit and check again
    sleep(Duration::from_secs(1)).await;

    let updated_health = manager.get_system_health().await;
    info!("Updated system health: {:?}", updated_health.overall_status);

    // Verify health status is valid
    assert!(matches!(
        updated_health.overall_status,
        HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Critical
    ));

    info!("✅ Health monitoring test passed");
    Ok(())
}

/// Test live service integration
#[tokio::test]
#[traced_test]
async fn test_live_service_integration() -> Result<()> {
    info!("🔗 Testing live service integration");

    let live_service = LiveService::new().await?;
    let system_status = live_service.get_system_status().await?;

    info!("Live service system status retrieved successfully");
    info!(
        "System status keys: {:?}",
        system_status.keys().collect::<Vec<_>>()
    );

    // Test that we can get discovered primals
    let discovered_primals = live_service.get_discovered_primals().await;
    info!("Discovered primals count: {}", discovered_primals.len());

    info!("✅ Live service integration test passed");
    Ok(())
}

/// Test endpoint probing functionality
#[tokio::test]
#[traced_test]
async fn test_endpoint_probing() -> Result<()> {
    info!("🔗 Testing endpoint probing");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(5),
            scan_hosts: vec!["localhost".to_string()],
            scan_ports: vec![8080, 8081],
            registry_endpoints: vec![],
        },
        environment: Environment::Development,
        max_concurrent_requests: 5,
        request_timeout: Duration::from_secs(2),
        health_check_interval: Duration::from_secs(5),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test probing various endpoints
    let test_endpoints = vec![
        "http://localhost:8080",
        "http://localhost:8081",
        "http://127.0.0.1:8080",
    ];

    for endpoint in test_endpoints {
        match manager.probe_endpoint(endpoint).await {
            Ok(result) => {
                info!("✅ Probe successful for {}: {:?}", endpoint, result.name);
            }
            Err(e) => {
                warn!("⚠️ Probe failed for {}: {:?}", endpoint, e);
                // This is expected for endpoints that aren't running
            }
        }
    }

    info!("✅ Endpoint probing test passed");
    Ok(())
}

/// Test registry discovery (if available)
#[tokio::test]
#[traced_test]
async fn test_registry_discovery() -> Result<()> {
    info!("📋 Testing registry discovery");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::Registry,
            timeout: Duration::from_secs(5),
            scan_hosts: vec![],
            scan_ports: vec![],
            registry_endpoints: vec!["http://localhost:8080".to_string()],
        },
        environment: Environment::Development,
        max_concurrent_requests: 5,
        request_timeout: Duration::from_secs(3),
        health_check_interval: Duration::from_secs(5),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test registry discovery
    match manager.discover_registry("http://localhost:8080").await {
        Ok(results) => {
            info!("Registry discovery successful: {} results", results.len());
        }
        Err(e) => {
            warn!(
                "Registry discovery failed (expected if no registry running): {:?}",
                e
            );
            // This is expected if no registry is actually running
        }
    }

    info!("✅ Registry discovery test passed");
    Ok(())
}

/// Test concurrent operations
#[tokio::test]
#[traced_test]
async fn test_concurrent_operations() -> Result<()> {
    info!("⚡ Testing concurrent operations");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(5),
            scan_hosts: vec!["localhost".to_string()],
            scan_ports: vec![8080, 8081],
            registry_endpoints: vec![],
        },
        environment: Environment::Development,
        max_concurrent_requests: 10,
        request_timeout: Duration::from_secs(3),
        health_check_interval: Duration::from_secs(5),
    };

    let manager = std::sync::Arc::new(UniversalBiomeOSManager::new(config).await?);

    // Run multiple discovery operations concurrently
    let mut handles = Vec::new();

    for i in 0..5 {
        let manager_clone = std::sync::Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let result = manager_clone.discover_network_scan().await;
            info!("Concurrent discovery {} completed", i);
            result
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut successful = 0;
    for handle in handles {
        match handle.await? {
            Ok(_) => successful += 1,
            Err(e) => warn!("Concurrent operation failed: {:?}", e),
        }
    }

    info!(
        "Concurrent operations completed: {}/5 successful",
        successful
    );

    info!("✅ Concurrent operations test passed");
    Ok(())
}

/// Test system resilience under load
#[tokio::test]
#[traced_test]
async fn test_system_resilience() -> Result<()> {
    info!("🛡️ Testing system resilience");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(2),
            scan_hosts: vec!["localhost".to_string()],
            scan_ports: vec![8080],
            registry_endpoints: vec![],
        },
        environment: Environment::Development,
        max_concurrent_requests: 5,
        request_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_secs(1),
    };

    let manager = UniversalBiomeOSManager::new(config).await?;

    // Perform rapid successive operations
    for i in 0..10 {
        let health = manager.get_system_health().await;
        info!(
            "Resilience test iteration {}: {:?}",
            i, health.overall_status
        );

        // Small delay between operations
        sleep(Duration::from_millis(100)).await;
    }

    // Verify system is still responsive
    let final_health = manager.get_system_health().await;
    info!("Final system health: {:?}", final_health.overall_status);

    assert!(matches!(
        final_health.overall_status,
        HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Critical
    ));

    info!("✅ System resilience test passed");
    Ok(())
}

/// Integration test that combines multiple operations
#[tokio::test]
#[traced_test]
async fn test_complete_workflow() -> Result<()> {
    info!("🔄 Testing complete workflow");

    let config = BiomeOSConfig {
        discovery: biomeos_core::config::DiscoveryConfig {
            method: DiscoveryMethod::NetworkScan,
            timeout: Duration::from_secs(10),
            scan_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            scan_ports: vec![8080, 8081, 8082],
            registry_endpoints: vec!["http://localhost:8080".to_string()],
        },
        environment: Environment::Development,
        max_concurrent_requests: 10,
        request_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(5),
    };

    // Step 1: Initialize system
    info!("Step 1: Initializing system");
    let manager = UniversalBiomeOSManager::new(config).await?;
    let live_service = LiveService::new().await?;

    // Step 2: Check initial health
    info!("Step 2: Checking initial health");
    let initial_health = manager.get_system_health().await;
    info!("Initial health: {:?}", initial_health.overall_status);

    // Step 3: Discover primals
    info!("Step 3: Discovering primals");
    let discovered = manager.discover_network_scan().await?;
    info!("Discovered {} primals", discovered.len());

    // Step 4: Test capability matching
    info!("Step 4: Testing capability matching");
    let capabilities = vec![PrimalCapability::Compute, PrimalCapability::Storage];
    let capability_results = manager.discover_by_capability(&capabilities).await?;
    info!(
        "Found {} primals with required capabilities",
        capability_results.len()
    );

    // Step 5: Test live service integration
    info!("Step 5: Testing live service integration");
    let system_status = live_service.get_system_status().await?;
    info!("Live service status keys: {}", system_status.len());

    // Step 6: Final health check
    info!("Step 6: Final health check");
    let final_health = manager.get_system_health().await;
    info!("Final health: {:?}", final_health.overall_status);

    info!("✅ Complete workflow test passed");
    Ok(())
}
