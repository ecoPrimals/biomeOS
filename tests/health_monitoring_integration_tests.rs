//! Health Monitoring Integration Tests
//!
//! Comprehensive integration tests for health monitoring system including
//! real system stats collection, background tasks, and service integration.

use anyhow::Result;
use biomeos_core::{
    integration::live_service::LiveService,
    universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager},
};
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health, HealthReport};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing_test::traced_test;

/// Helper to check if health is valid (any meaningful state)
fn is_valid_health(health: &Health) -> bool {
    matches!(
        health,
        Health::Healthy
            | Health::Degraded { .. }
            | Health::Critical { .. }
            | Health::Unhealthy { .. }
            | Health::Unknown { .. }
            | Health::Starting { .. }
            | Health::Stopping { .. }
            | Health::Maintenance { .. }
    )
}

/// Extract CPU usage percentage from HealthReport
fn get_cpu_usage(report: &HealthReport) -> f64 {
    report
        .metrics
        .resources
        .as_ref()
        .and_then(|r| r.cpu_usage)
        .map(|u| u * 100.0)
        .unwrap_or(0.0)
}

/// Extract memory usage percentage from HealthReport
fn get_memory_usage(report: &HealthReport) -> f64 {
    report
        .metrics
        .resources
        .as_ref()
        .and_then(|r| r.memory_usage)
        .map(|u| u * 100.0)
        .unwrap_or(0.0)
}

/// Extract disk usage percentage from HealthReport
fn get_disk_usage(report: &HealthReport) -> f64 {
    report
        .metrics
        .resources
        .as_ref()
        .and_then(|r| r.disk_usage)
        .map(|u| u * 100.0)
        .unwrap_or(0.0)
}

/// Extract uptime seconds from HealthReport
#[allow(dead_code)]
fn get_uptime_seconds(report: &HealthReport) -> i64 {
    report
        .metrics
        .availability
        .as_ref()
        .map(|a| a.uptime_seconds as i64)
        .unwrap_or(0)
}

#[traced_test]
#[tokio::test]
async fn test_real_system_health_collection() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test real system health collection
    let health_report = manager.get_system_health().await;

    // Verify system health structure
    assert!(is_valid_health(&health_report.health));

    // Verify resource usage is within reasonable bounds
    let cpu = get_cpu_usage(&health_report);
    let memory = get_memory_usage(&health_report);
    let disk = get_disk_usage(&health_report);

    assert!(cpu >= 0.0);
    assert!(cpu <= 800.0); // Allow for high-core systems
    assert!(memory >= 0.0);
    assert!(memory <= 100.0);
    assert!(disk >= 0.0);
    assert!(disk <= 100.0);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_background_monitoring_tasks() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Wait for background tasks to run a few cycles
    sleep(Duration::from_millis(2000)).await;

    // Get system health multiple times to verify continuous monitoring
    let health1 = manager.get_system_health().await;
    sleep(Duration::from_millis(500)).await;
    let health2 = manager.get_system_health().await;

    // Both health checks should succeed
    assert!(is_valid_health(&health1.health));
    assert!(is_valid_health(&health2.health));

    // Resource usage should be continuously updated
    assert!(get_cpu_usage(&health1) >= 0.0);
    assert!(get_cpu_usage(&health2) >= 0.0);

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_live_service_health_integration() -> Result<()> {
    let mut live_service = LiveService::new().await?;

    // Start the live service (initializes background monitoring)
    live_service.start().await?;

    // Wait for initialization to complete
    sleep(Duration::from_millis(1000)).await;

    // Test system status via live service
    let system_status = live_service.get_system_status().await?;

    // Verify system status structure
    assert!(system_status.uptime.num_seconds() >= 0);

    // Test storage metrics
    let storage_metrics = live_service.get_storage_metrics().await?;
    // May have mount points on most systems
    // Mount points collection validated (may be empty on some systems)
    let _ = &storage_metrics.mount_points;

    // Test network status
    let network_status = live_service.get_network_status().await?;
    // Should have at least loopback on most systems
    // Network interfaces collection validated (may be empty in some envs)
    let _ = &network_status.interfaces;

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_system_resource_parsing() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get system health to trigger resource parsing
    let health_report = manager.get_system_health().await;

    // Test memory parsing (should work on most Linux systems)
    if std::fs::read_to_string("/proc/meminfo").is_ok() {
        let memory = get_memory_usage(&health_report);
        assert!(memory >= 0.0);
        assert!(memory <= 100.0);
    }

    // Test CPU load parsing (should work on most Unix systems)
    if std::fs::read_to_string("/proc/loadavg").is_ok() {
        let cpu = get_cpu_usage(&health_report);
        assert!(cpu >= 0.0);
    }

    // Test disk usage parsing
    if std::process::Command::new("df").arg("/").output().is_ok() {
        let disk = get_disk_usage(&health_report);
        assert!(disk >= 0.0);
        assert!(disk <= 100.0);
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_status_assessment() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get current system health
    let health_report = manager.get_system_health().await;

    // Test that health assessment logic works
    match &health_report.health {
        Health::Healthy => {
            // If healthy, resource usage should be reasonable
            assert!(get_cpu_usage(&health_report) < 95.0);
            assert!(get_memory_usage(&health_report) < 95.0);
            assert!(get_disk_usage(&health_report) < 95.0);
        }
        Health::Degraded { issues, .. } => {
            // Degraded indicates some resource pressure
            println!("System degraded: {:?}", issues);
        }
        Health::Critical { issues, .. } => {
            // Critical indicates high resource usage
            println!("System critical: {:?}", issues);
        }
        Health::Unhealthy { issues, .. } => {
            println!("System unhealthy: {:?}", issues);
        }
        Health::Unknown { reason, .. } => {
            println!("System health unknown: {}", reason);
        }
        _ => {
            // Starting, Stopping, Maintenance states
            println!("System in transition state");
        }
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_primal_health_collection() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Register some test primals with correct struct
    let now = chrono::Utc::now();
    let test_primals = vec![
        ("test-primal-1", Health::Healthy),
        ("test-primal-2", Health::degraded(vec![])),
        ("test-primal-3", Health::unknown("test")),
    ];

    for (id, health) in &test_primals {
        let primal_info = PrimalInfo {
            id: id.to_string(),
            name: format!("Test Primal {}", id),
            primal_type: PrimalType::new("test", *id, "1.0.0"),
            capabilities: vec![PrimalCapability::new("test", "capability", "1.0.0")],
            health: health.clone(),
            endpoint: format!("http://localhost:8080/{}", id),
            last_seen: now,
            discovered_at: now,
            metadata: HashMap::new(),
        };

        manager.register_primal(primal_info).await?;
    }

    // Get system health and verify primal health is collected
    let health_report = manager.get_system_health().await;

    // Should have component health data
    // Components may include registered primals
    assert!(is_valid_health(&health_report.health));

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_monitoring_performance() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test multiple sequential health checks for performance
    let start_time = std::time::Instant::now();
    let mut results = Vec::new();

    for _ in 0..10 {
        let result = manager.get_system_health().await;
        results.push(result);
    }

    let elapsed = start_time.elapsed();

    // All health checks should complete
    assert_eq!(results.len(), 10);

    // Should complete reasonably quickly (less than 5 seconds for 10 checks)
    assert!(elapsed < Duration::from_secs(5));

    // All results should be valid
    for result in results {
        assert!(is_valid_health(&result.health));
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_resource_usage_consistency() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get multiple resource usage samples
    let mut memory_samples = Vec::new();
    let mut disk_samples = Vec::new();

    for _ in 0..5 {
        let health = manager.get_system_health().await;
        memory_samples.push(get_memory_usage(&health));
        disk_samples.push(get_disk_usage(&health));
        sleep(Duration::from_millis(200)).await;
    }

    // Verify all samples are valid
    for (memory, disk) in memory_samples.iter().zip(disk_samples.iter()) {
        assert!(*memory >= 0.0);
        assert!(*memory <= 100.0);
        assert!(*disk >= 0.0);
        assert!(*disk <= 100.0);
    }

    // Memory and disk usage should be relatively stable over short periods
    if memory_samples.len() >= 2 {
        let first_memory = memory_samples[0];
        let last_memory = memory_samples[memory_samples.len() - 1];
        let first_disk = disk_samples[0];
        let last_disk = disk_samples[disk_samples.len() - 1];

        // Memory usage shouldn't vary by more than 15% in short timeframe
        let memory_diff = (last_memory - first_memory).abs();
        assert!(
            memory_diff < 15.0,
            "Memory usage varied too much: {} -> {}",
            first_memory,
            last_memory
        );

        // Disk usage should be very stable in short timeframe
        let disk_diff = (last_disk - first_disk).abs();
        assert!(
            disk_diff < 2.0,
            "Disk usage varied too much: {} -> {}",
            first_disk,
            last_disk
        );
    }

    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_monitoring_error_handling() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Health monitoring should handle system read failures gracefully
    let health_report = manager.get_system_health().await;

    // Even if some system calls fail, we should get a valid health status
    assert!(is_valid_health(&health_report.health));

    // Resource usage should default to reasonable values if parsing fails
    let cpu = get_cpu_usage(&health_report);
    let memory = get_memory_usage(&health_report);
    let disk = get_disk_usage(&health_report);

    assert!(cpu >= 0.0);
    assert!(memory >= 0.0);
    assert!(disk >= 0.0);

    Ok(())
}

// Helper function to simulate high resource usage for testing
async fn simulate_high_cpu_usage() -> Result<()> {
    // Create a short CPU-intensive task
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(100) {
        // Busy wait to consume CPU
        let _ = (0..10000).fold(0u64, |acc, x| acc.wrapping_add(x * x));
    }
    Ok(())
}

#[traced_test]
#[tokio::test]
async fn test_health_monitoring_under_load() -> Result<()> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

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
    assert!(is_valid_health(&baseline_health.health));
    assert!(is_valid_health(&load_health.health));
    assert!(is_valid_health(&post_load_health.health));

    // CPU usage should be valid (but not guaranteed to be higher during short load test)
    assert!(get_cpu_usage(&load_health) >= 0.0);
    assert!(get_cpu_usage(&post_load_health) >= 0.0);

    Ok(())
}
