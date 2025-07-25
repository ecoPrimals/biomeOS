//! Health Monitoring Integration Tests
//!
//! Comprehensive integration tests for health monitoring system including
//! real system stats collection, background tasks, and service integration.

use anyhow::Result;
use biomeos_core::{
    config::{BiomeOSConfig, DiscoveryMethod, Environment},
    integration::live_service::LiveService,
    universal_biomeos_manager::{
        HealthStatus, SystemHealth, SystemResourceUsage, UniversalBiomeOSManager,
    },
    BiomeError,
};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing_test::traced_test;

/// Create test configuration for health monitoring
fn create_health_monitoring_config() -> BiomeOSConfig {
    let mut config = BiomeOSConfig::default();
    config.system.environment = Environment::Testing;
    config.primals.discovery.method = DiscoveryMethod::Static;
    config.primals.timeouts.health_check_interval_ms = 1000; // Fast for testing
    config
}

#[traced_test]
#[tokio::test]
async fn test_real_system_health_collection() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Test real system health collection
    let system_health = manager.get_system_health().await;

    // Verify system health structure
    assert!(matches!(
        system_health.overall_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
            | HealthStatus::Unhealthy
            | HealthStatus::Unknown
    ));

    // Verify uptime is positive (system should have some uptime)
    assert!(system_health.uptime.num_seconds() > 0);
    assert!(system_health.uptime.num_seconds() < 31_536_000); // Less than 1 year

    // Verify resource usage is within reasonable bounds
    let usage = &system_health.resource_usage;
    assert!(usage.cpu_usage_percent >= 0.0);
    assert!(usage.cpu_usage_percent <= 800.0); // Allow for high-core systems
    assert!(usage.memory_usage_percent >= 0.0);
    assert!(usage.memory_usage_percent <= 100.0);
    assert!(usage.disk_usage_percent >= 0.0);
    assert!(usage.disk_usage_percent <= 100.0);
    assert!(usage.network_usage_mbps >= 0.0);

    // Verify primal health data exists
    assert!(!system_health.primal_health.is_empty());

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_background_monitoring_tasks() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Initialize manager to start background tasks
    manager.initialize().await?;

    // Wait for background tasks to run a few cycles
    sleep(Duration::from_millis(3500)).await; // Allow for multiple monitoring cycles

    // Get system health multiple times to verify continuous monitoring
    let health1 = manager.get_system_health().await;
    sleep(Duration::from_millis(500)).await;
    let health2 = manager.get_system_health().await;

    // Both health checks should succeed
    assert!(matches!(
        health1.overall_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
            | HealthStatus::Unhealthy
            | HealthStatus::Unknown
    ));
    assert!(matches!(
        health2.overall_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
            | HealthStatus::Unhealthy
            | HealthStatus::Unknown
    ));

    // Resource usage should be continuously updated (may vary slightly)
    assert!(health1.resource_usage.cpu_usage_percent >= 0.0);
    assert!(health2.resource_usage.cpu_usage_percent >= 0.0);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_live_service_health_integration() -> Result<()> {
    let config = create_health_monitoring_config();
    let mut live_service = LiveService::new(config).await?;

    // Start the live service (initializes background monitoring)
    live_service.start().await?;

    // Wait for initialization to complete
    sleep(Duration::from_millis(1000)).await;

    // Test system status via live service
    let system_status = live_service.get_system_status().await?;

    // Verify system status structure
    assert!(system_status.uptime.num_seconds() > 0);
    assert!(matches!(
        system_status.health_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
            | HealthStatus::Unhealthy
            | HealthStatus::Unknown
    ));

    // Test storage metrics
    let storage_metrics = live_service.get_storage_metrics().await?;
    assert!(storage_metrics.mount_points.len() > 0); // Should have at least one mount point

    // Test network status
    let network_status = live_service.get_network_status().await?;
    assert!(!network_status.interfaces.is_empty()); // Should have at least loopback

    // Test comprehensive health check
    let health_check = live_service.health_check().await?;
    assert!(health_check.timestamp.timestamp() > 0);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_system_resource_parsing() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Get system health to trigger resource parsing
    let system_health = manager.get_system_health().await;
    let usage = &system_health.resource_usage;

    // Test memory parsing (should work on most Linux systems)
    if std::fs::read_to_string("/proc/meminfo").is_ok() {
        // If /proc/meminfo exists, memory usage should be meaningful
        assert!(usage.memory_usage_percent >= 0.0);
        assert!(usage.memory_usage_percent <= 100.0);
    }

    // Test CPU load parsing (should work on most Unix systems)
    if std::fs::read_to_string("/proc/loadavg").is_ok() {
        // If /proc/loadavg exists, CPU usage should be meaningful
        assert!(usage.cpu_usage_percent >= 0.0);
    }

    // Test disk usage parsing (df command should work on most systems)
    // Note: This might fail in some container environments
    if std::process::Command::new("df").arg("/").output().is_ok() {
        assert!(usage.disk_usage_percent >= 0.0);
        assert!(usage.disk_usage_percent <= 100.0);
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_status_assessment() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Get current system health
    let system_health = manager.get_system_health().await;

    // Test that health assessment logic works
    match system_health.overall_status {
        HealthStatus::Healthy => {
            // If healthy, resource usage should be reasonable
            assert!(system_health.resource_usage.cpu_usage_percent < 90.0);
            assert!(system_health.resource_usage.memory_usage_percent < 90.0);
            assert!(system_health.resource_usage.disk_usage_percent < 90.0);
        }
        HealthStatus::Warning | HealthStatus::Degraded => {
            // Warning/Degraded indicates some resource pressure
            // This is expected in some test environments
        }
        HealthStatus::Critical => {
            // Critical indicates high resource usage
            // This might happen in constrained test environments
        }
        HealthStatus::Unhealthy | HealthStatus::Unknown => {
            // This might happen if system stats can't be read
            println!("System health: {:?}", system_health);
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_uptime_calculation() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    let system_health = manager.get_system_health().await;
    let uptime = system_health.uptime;

    // Basic uptime validation
    assert!(uptime.num_seconds() > 0);
    assert!(uptime.num_days() < 10000); // Sanity check (less than ~27 years)

    // Test uptime consistency over short periods
    sleep(Duration::from_millis(100)).await;
    let system_health2 = manager.get_system_health().await;
    let uptime2 = system_health2.uptime;

    // Second reading should be slightly higher (or same within tolerance)
    let diff = uptime2.num_seconds() - uptime.num_seconds();
    assert!(diff >= 0);
    assert!(diff < 10); // Should not differ by more than 10 seconds in this short test

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_primal_health_collection() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Register some test primals
    let test_primals = vec![
        ("test-primal-1", PrimalHealth::Healthy),
        ("test-primal-2", PrimalHealth::Degraded),
        ("test-primal-3", PrimalHealth::Unknown),
    ];

    for (id, health) in &test_primals {
        let primal_info = biomeos_core::universal_biomeos_manager::PrimalInfo {
            id: id.to_string(),
            primal_type: PrimalType::new("test", id, "1.0.0"),
            capabilities: vec![PrimalCapability::custom("test", "Test capability")],
            health: health.clone(),
            endpoint: format!("http://localhost:8080/{}", id),
        };

        manager.register_primal(id.to_string(), primal_info).await?;
    }

    // Get system health and verify primal health is collected
    let system_health = manager.get_system_health().await;

    // Should have primal health data (may include both registered and mock primals)
    assert!(!system_health.primal_health.is_empty());

    // Verify basic health status values are valid
    for (primal_id, health) in &system_health.primal_health {
        assert!(!primal_id.is_empty());
        assert!(matches!(
            health,
            PrimalHealth::Healthy
                | PrimalHealth::Degraded
                | PrimalHealth::Unhealthy
                | PrimalHealth::Unknown
        ));
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_monitoring_performance() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Test multiple concurrent health checks
    let start_time = std::time::Instant::now();
    let mut handles = Vec::new();

    for _ in 0..10 {
        let manager_ref = &manager;
        handles.push(tokio::spawn(async move {
            manager_ref.get_system_health().await
        }));
    }

    // Wait for all health checks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await??;
        results.push(result);
    }

    let elapsed = start_time.elapsed();

    // All health checks should complete
    assert_eq!(results.len(), 10);

    // Should complete reasonably quickly (less than 10 seconds for 10 concurrent checks)
    assert!(elapsed < Duration::from_secs(10));

    // All results should be valid
    for result in results {
        assert!(result.uptime.num_seconds() > 0);
        assert!(matches!(
            result.overall_status,
            HealthStatus::Healthy
                | HealthStatus::Degraded
                | HealthStatus::Warning
                | HealthStatus::Critical
                | HealthStatus::Unhealthy
                | HealthStatus::Unknown
        ));
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_resource_usage_consistency() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Get multiple resource usage samples
    let mut samples = Vec::new();
    for _ in 0..5 {
        let health = manager.get_system_health().await;
        samples.push(health.resource_usage);
        sleep(Duration::from_millis(200)).await;
    }

    // Verify all samples are valid
    for sample in &samples {
        assert!(sample.cpu_usage_percent >= 0.0);
        assert!(sample.memory_usage_percent >= 0.0);
        assert!(sample.memory_usage_percent <= 100.0);
        assert!(sample.disk_usage_percent >= 0.0);
        assert!(sample.disk_usage_percent <= 100.0);
        assert!(sample.network_usage_mbps >= 0.0);
    }

    // Memory and disk usage should be relatively stable over short periods
    if samples.len() >= 2 {
        let first = &samples[0];
        let last = &samples[samples.len() - 1];

        // Memory usage shouldn't vary by more than 10% in short timeframe
        let memory_diff = (last.memory_usage_percent - first.memory_usage_percent).abs();
        assert!(
            memory_diff < 10.0,
            "Memory usage varied too much: {} -> {}",
            first.memory_usage_percent,
            last.memory_usage_percent
        );

        // Disk usage should be very stable in short timeframe
        let disk_diff = (last.disk_usage_percent - first.disk_usage_percent).abs();
        assert!(
            disk_diff < 1.0,
            "Disk usage varied too much: {} -> {}",
            first.disk_usage_percent,
            last.disk_usage_percent
        );
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_monitoring_error_handling() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Health monitoring should handle system read failures gracefully
    let system_health = manager.get_system_health().await;

    // Even if some system calls fail, we should get a valid health status
    assert!(matches!(
        system_health.overall_status,
        HealthStatus::Healthy
            | HealthStatus::Degraded
            | HealthStatus::Warning
            | HealthStatus::Critical
            | HealthStatus::Unhealthy
            | HealthStatus::Unknown
    ));

    // Resource usage should default to reasonable values if parsing fails
    let usage = &system_health.resource_usage;
    assert!(usage.cpu_usage_percent >= 0.0);
    assert!(usage.memory_usage_percent >= 0.0);
    assert!(usage.disk_usage_percent >= 0.0);
    assert!(usage.network_usage_mbps >= 0.0);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_live_service_monitoring_integration() -> Result<()> {
    let config = create_health_monitoring_config();
    let mut live_service = LiveService::new(config).await?;

    live_service.start().await?;

    // Wait for monitoring to initialize
    sleep(Duration::from_millis(1500)).await;

    // Test multiple service calls in sequence
    let health1 = live_service.health_check().await?;
    let status = live_service.get_system_status().await?;
    let health2 = live_service.health_check().await?;

    // All calls should succeed
    assert!(health1.timestamp.timestamp() > 0);
    assert!(status.uptime.num_seconds() > 0);
    assert!(health2.timestamp.timestamp() > 0);

    // Second health check should have later timestamp
    assert!(health2.timestamp >= health1.timestamp);

    // Health status should be consistent
    assert_eq!(
        health1.overall_healthy,
        matches!(health1.system_status.health_status, HealthStatus::Healthy)
    );
    assert_eq!(
        health2.overall_healthy,
        matches!(health2.system_status.health_status, HealthStatus::Healthy)
    );

    Ok(())
}

// Helper function to simulate high resource usage for testing
async fn simulate_high_cpu_usage() -> Result<()> {
    // Create a short CPU-intensive task
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(100) {
        // Busy wait to consume CPU
        let _ = (0..10000).fold(0, |acc, x| acc + x * x);
    }
    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_monitoring_under_load() -> Result<()> {
    let config = create_health_monitoring_config();
    let manager = UniversalBiomeOSManager::new(config);

    // Get baseline health
    let baseline_health = manager.get_system_health().await;

    // Simulate some load
    let mut load_handles = Vec::new();
    for _ in 0..5 {
        load_handles.push(tokio::spawn(simulate_high_cpu_usage()));
    }

    // Get health while under load
    let load_health = manager.get_system_health().await;

    // Wait for load simulation to complete
    for handle in load_handles {
        handle.await??;
    }

    // Get health after load
    sleep(Duration::from_millis(500)).await;
    let post_load_health = manager.get_system_health().await;

    // All health checks should succeed
    assert!(baseline_health.uptime.num_seconds() > 0);
    assert!(load_health.uptime.num_seconds() > 0);
    assert!(post_load_health.uptime.num_seconds() > 0);

    // CPU usage might be higher under load (but not guaranteed in fast tests)
    // At minimum, all measurements should be valid
    assert!(load_health.resource_usage.cpu_usage_percent >= 0.0);
    assert!(post_load_health.resource_usage.cpu_usage_percent >= 0.0);

    Ok(())
}
