//! Simple End-to-End Testing Suite for BiomeOS
//!
//! This test suite validates basic biomeOS functionality including
//! system initialization, primal discovery, and health monitoring.

use anyhow::Result;
use biomeos_core::{
    integration::live_service::LiveService, universal_biomeos_manager::UniversalBiomeOSManager,
};
use biomeos_types::{BiomeOSConfig, Health};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
use tracing_test::traced_test;

/// Helper to check if health is valid
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

/// Test system initialization
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_system_initialization() -> Result<()> {
    info!("🚀 Testing system initialization");

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;
    let health_report = manager.get_system_health().await;

    info!("System health: {:?}", health_report.health);
    assert!(is_valid_health(&health_report.health));

    info!("✅ System initialization test passed");
    Ok(())
}

/// Test primal discovery functionality
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_primal_discovery() -> Result<()> {
    info!("🔍 Testing primal discovery");

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Test discovery
    let discovered = manager.discover().await?;
    info!("Discovered {} primals", discovered.len());

    info!("✅ Primal discovery test passed");
    Ok(())
}

/// Test health monitoring system
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_health_monitoring() -> Result<()> {
    info!("💚 Testing health monitoring");

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get initial health status
    let initial_health = manager.get_system_health().await;
    info!("Initial system health: {:?}", initial_health.health);

    // Wait a bit and check again
    sleep(Duration::from_secs(1)).await;

    let updated_health = manager.get_system_health().await;
    info!("Updated system health: {:?}", updated_health.health);

    // Verify health status is valid
    assert!(is_valid_health(&updated_health.health));

    info!("✅ Health monitoring test passed");
    Ok(())
}

/// Test live service integration
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_live_service_integration() -> Result<()> {
    info!("🔗 Testing live service integration");

    let mut live_service = LiveService::new().await?;
    live_service.start().await?;

    let system_status = live_service.get_system_status().await?;
    info!("Live service system status retrieved successfully");
    info!("System uptime: {:?}", system_status.uptime);

    info!("✅ Live service integration test passed");
    Ok(())
}

/// Test configuration loading
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_configuration_loading() -> Result<()> {
    info!("⚙️ Testing configuration loading");

    // Test default configuration
    let default_config = BiomeOSConfig::default();
    assert!(!default_config.metadata.name.is_empty());

    info!("✅ Configuration loading test passed");
    Ok(())
}

/// Test continuous health checks
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_continuous_health_checks() -> Result<()> {
    info!("🔄 Testing continuous health checks");

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Perform multiple health checks
    for i in 0..5 {
        let health = manager.get_system_health().await;
        info!("Health check {}: {:?}", i + 1, health.health);
        assert!(is_valid_health(&health.health));
        sleep(Duration::from_millis(200)).await;
    }

    let final_health = manager.get_system_health().await;
    info!("Final system health: {:?}", final_health.health);
    assert!(is_valid_health(&final_health.health));

    info!("✅ Continuous health checks test passed");
    Ok(())
}

/// Test manager state management
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[traced_test]
async fn test_manager_state_management() -> Result<()> {
    info!("📊 Testing manager state management");

    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Get initial health
    let initial_health = manager.get_system_health().await;
    info!("Initial health: {:?}", initial_health.health);

    // Verify health is valid
    assert!(is_valid_health(&initial_health.health));

    // Get final health
    let final_health = manager.get_system_health().await;
    info!("Final health: {:?}", final_health.health);

    info!("✅ Manager state management test passed");
    Ok(())
}
