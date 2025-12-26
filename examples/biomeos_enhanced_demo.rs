//! Enhanced BiomeOS System Demo
//!
//! This demonstrates:
//! 1. ✅ Universal capability-based discovery (agnostic to specific primals)
//! 2. ✅ Real-time health monitoring
//! 3. ✅ Dynamic primal registration and management
//! 4. ✅ Comprehensive system resource tracking
//! 5. ✅ Resilient error handling and recovery

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Enhanced BiomeOS System Demo");
    info!("================================================");

    // 1. Initialize BiomeOS with enhanced configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    info!("✅ BiomeOS Universal Manager initialized");

    // 2. Demonstrate real-time health monitoring
    info!("\n📊 Getting System Health...");

    // Get and display current system health
    let health_report = manager.get_system_health().await;
    info!("🏥 System Health Report:");
    info!("   Overall Status: {:?}", health_report.health);
    info!("   Report ID: {:?}", health_report.id);

    match &health_report.health {
        Health::Healthy => info!("💚 System is running optimally!"),
        Health::Degraded { issues, .. } => {
            warn!("💛 System has {} issues", issues.len());
        }
        Health::Critical { issues, .. } => {
            error!("🔴 System requires attention ({} issues)", issues.len());
        }
        Health::Unhealthy { issues, .. } => {
            error!("❌ System is unhealthy ({} issues)", issues.len());
        }
        _ => info!("ℹ️  System status: {:?}", health_report.health),
    }

    // 3. Demonstrate capability-based discovery
    info!("\n🔍 Testing Capability-Based Discovery...");

    // Create test primal registration
    let now = chrono::Utc::now();
    let test_primal = biomeos_core::PrimalInfo {
        id: "demo-compute-primal".to_string(),
        name: "Demo Compute Service".to_string(),
        primal_type: PrimalType::new("compute", "demo", "1.0.0"),
        capabilities: vec![
            PrimalCapability::new("compute", "provider", "1.0.0"),
            PrimalCapability::new("gpu", "accelerator", "1.0.0"),
        ],
        health: Health::Healthy,
        endpoint: "http://localhost:8001".to_string(),
        last_seen: now,
        discovered_at: now,
        metadata: HashMap::new(),
    };

    manager.register_primal(test_primal).await?;
    info!("✅ Registered demo compute primal");

    // Search by capability
    let compute_caps = vec![PrimalCapability::new("compute", "provider", "1.0.0")];
    let found = manager.discover_by_capability(&compute_caps).await?;
    info!("🔎 Found {} primals with compute capability", found.len());

    // 4. Demonstrate primal management
    info!("\n🌿 Primal Management Demo...");

    let primals = manager.get_registered_primals().await;
    info!("📋 Currently registered primals: {}", primals.len());

    for primal in &primals {
        info!(
            "   - {} ({:?}): {:?}",
            primal.name, primal.primal_type, primal.health
        );
    }

    // 5. Primal statistics
    info!("\n📈 Primal Statistics...");
    let stats = manager.get_primal_statistics().await;
    info!("   Total primals: {}", stats.total);
    info!("   Healthy primals: {}", stats.healthy);
    info!("   Degraded primals: {}", stats.degraded);
    info!("   Unhealthy primals: {}", stats.unhealthy);

    // 6. Continuous monitoring demo
    info!("\n⏰ Starting Continuous Monitoring Demo (5 seconds)...");

    for i in 1..=5 {
        sleep(Duration::from_secs(1)).await;
        let health = manager.get_system_health().await;
        info!("   [{}/5] System status: {:?}", i, health.health);
    }

    info!("\n✨ Enhanced BiomeOS Demo Complete!");
    info!("================================================");
    info!("The demo showed:");
    info!("  ✅ Universal manager initialization");
    info!("  ✅ Health monitoring");
    info!("  ✅ Primal registration");
    info!("  ✅ Capability-based discovery");
    info!("  ✅ System statistics");

    Ok(())
}
