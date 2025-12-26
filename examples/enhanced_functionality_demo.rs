//! Enhanced BiomeOS Functionality Demo
//!
//! Demonstrates primal registration and capability-based discovery

use anyhow::Result;
use biomeos_core::universal_biomeos_manager::PrimalInfo;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use biomeos_types::{BiomeOSConfig, Health};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌱 BiomeOS Enhanced Functionality Demo");
    println!("======================================");

    // Initialize BiomeOS with default configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    println!("✅ BiomeOS manager initialized");

    // Create some example primals to register
    let now = chrono::Utc::now();

    let toadstool_primal = PrimalInfo {
        id: "toadstool-001".to_string(),
        name: "ToadStool Container Manager".to_string(),
        primal_type: PrimalType::new("compute", "toadstool", "1.0.0"),
        capabilities: vec![
            PrimalCapability::new("compute", "container_orchestration", "1.0.0"),
            PrimalCapability::new("compute", "vm_management", "1.0.0"),
            PrimalCapability::new("system", "resource_management", "1.0.0"),
        ],
        health: Health::Healthy,
        endpoint: "http://localhost:8084".to_string(),
        last_seen: now,
        discovered_at: now,
        metadata: HashMap::new(),
    };

    let songbird_primal = PrimalInfo {
        id: "songbird-001".to_string(),
        name: "Songbird Service Discovery".to_string(),
        primal_type: PrimalType::new("networking", "songbird", "2.0.0"),
        capabilities: vec![
            PrimalCapability::new("networking", "service_discovery", "2.0.0"),
            PrimalCapability::new("networking", "load_balancing", "2.0.0"),
            PrimalCapability::new("system", "orchestration", "2.0.0"),
        ],
        health: Health::Healthy,
        endpoint: "http://localhost:8080".to_string(),
        last_seen: now,
        discovered_at: now,
        metadata: HashMap::new(),
    };

    let nestgate_primal = PrimalInfo {
        id: "nestgate-001".to_string(),
        name: "NestGate Storage Manager".to_string(),
        primal_type: PrimalType::new("storage", "nestgate", "1.5.0"),
        capabilities: vec![
            PrimalCapability::new("storage", "persistent_volumes", "1.5.0"),
            PrimalCapability::new("storage", "file_systems", "1.5.0"),
            PrimalCapability::new("system", "data_management", "1.5.0"),
        ],
        health: Health::Healthy,
        endpoint: "http://localhost:8082".to_string(),
        last_seen: now,
        discovered_at: now,
        metadata: HashMap::new(),
    };

    // Register the primals
    println!("\n🔧 Registering Primals:");
    manager.register_primal(toadstool_primal).await?;
    println!("  ✅ Registered Toadstool compute primal");

    manager.register_primal(songbird_primal).await?;
    println!("  ✅ Registered Songbird networking primal");

    manager.register_primal(nestgate_primal).await?;
    println!("  ✅ Registered NestGate storage primal");

    // Test getting all registered primals
    println!("\n📋 Currently Registered Primals:");
    let primals = manager.get_registered_primals().await;
    for primal in &primals {
        println!(
            "  - {} ({:?}): {:?}",
            primal.name, primal.primal_type, primal.health
        );
        println!("    Capabilities:");
        for cap in &primal.capabilities {
            println!("      • {:?}", cap);
        }
    }

    // Test capability-based discovery
    println!("\n🔍 Testing Capability-Based Discovery:");

    // Find compute primals
    let compute_caps = vec![PrimalCapability::new(
        "compute",
        "container_orchestration",
        "1.0.0",
    )];
    let compute_primals = manager.discover_by_capability(&compute_caps).await?;
    println!(
        "  Compute primals (container orchestration): {} found",
        compute_primals.len()
    );

    // Find storage primals
    let storage_caps = vec![PrimalCapability::new(
        "storage",
        "persistent_volumes",
        "1.5.0",
    )];
    let storage_primals = manager.discover_by_capability(&storage_caps).await?;
    println!(
        "  Storage primals (persistent volumes): {} found",
        storage_primals.len()
    );

    // Find network primals
    let network_caps = vec![PrimalCapability::new(
        "networking",
        "service_discovery",
        "2.0.0",
    )];
    let network_primals = manager.discover_by_capability(&network_caps).await?;
    println!(
        "  Network primals (service discovery): {} found",
        network_primals.len()
    );

    // Test discovery for a capability no primal has
    let ai_caps = vec![PrimalCapability::new("ai", "ml_inference", "1.0.0")];
    let ai_primals = manager.discover_by_capability(&ai_caps).await?;
    println!("  AI primals (ML inference): {} found", ai_primals.len());

    // Get system health
    println!("\n💚 System Health:");
    let health = manager.get_system_health().await;
    println!("  Overall status: {:?}", health.health);

    // Get primal statistics
    println!("\n📈 Primal Statistics:");
    let stats = manager.get_primal_statistics().await;
    println!("  Total: {}", stats.total);
    println!("  Healthy: {}", stats.healthy);
    println!("  Degraded: {}", stats.degraded);
    println!("  Unhealthy: {}", stats.unhealthy);

    println!("\n✨ Demo completed successfully!");

    Ok(())
}
