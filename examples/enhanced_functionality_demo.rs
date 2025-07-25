//! Enhanced BiomeOS Functionality Demo
//!
//! Demonstrates primal registration and capability-based discovery

use anyhow::Result;
use biomeos_core::universal_biomeos_manager::PrimalInfo;
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌱 BiomeOS Enhanced Functionality Demo");
    println!("======================================");

    // Initialize BiomeOS with default configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Initialize the manager
    manager.initialize().await?;
    println!("✅ BiomeOS manager initialized");

    // Create some example primals to register
    let toadstool_primal = PrimalInfo {
        id: "toadstool-001".to_string(),
        primal_type: PrimalType::new("compute", "toadstool", "1.0.0"),
        capabilities: vec![
            PrimalCapability::new("compute", "container_orchestration", "1.0"),
            PrimalCapability::new("compute", "vm_management", "1.0"),
            PrimalCapability::new("system", "resource_management", "1.0"),
        ],
        health: PrimalHealth::Healthy,
        discovered_at: chrono::Utc::now(),
    };

    let songbird_primal = PrimalInfo {
        id: "songbird-001".to_string(),
        primal_type: PrimalType::new("networking", "songbird", "2.0.0"),
        capabilities: vec![
            PrimalCapability::new("networking", "service_discovery", "2.0"),
            PrimalCapability::new("networking", "load_balancing", "2.0"),
            PrimalCapability::new("system", "orchestration", "2.0"),
        ],
        health: PrimalHealth::Healthy,
        discovered_at: chrono::Utc::now(),
    };

    let nestgate_primal = PrimalInfo {
        id: "nestgate-001".to_string(),
        primal_type: PrimalType::new("storage", "nestgate", "1.5.0"),
        capabilities: vec![
            PrimalCapability::new("storage", "persistent_volumes", "1.5"),
            PrimalCapability::new("storage", "file_systems", "1.5"),
            PrimalCapability::new("system", "data_management", "1.5"),
        ],
        health: PrimalHealth::Healthy,
        discovered_at: chrono::Utc::now(),
    };

    // Register the primals
    println!("\n🔧 Registering Primals:");
    manager
        .register_primal("toadstool-001".to_string(), toadstool_primal)
        .await?;
    println!("  ✅ Registered Toadstool compute primal");

    manager
        .register_primal("songbird-001".to_string(), songbird_primal)
        .await?;
    println!("  ✅ Registered Songbird networking primal");

    manager
        .register_primal("nestgate-001".to_string(), nestgate_primal)
        .await?;
    println!("  ✅ Registered NestGate storage primal");

    // Test getting all registered primals
    let registered = manager.get_registered_primals();
    println!("\n📋 Registered Primals: {} total", registered.len());
    for primal in &registered {
        println!(
            "  🎯 {}: {} capabilities",
            primal.id,
            primal.capabilities.len()
        );
    }

    // Test capability-based discovery
    println!("\n🔍 Testing Capability-Based Discovery:");

    // Look for compute capabilities
    let compute_capabilities = vec![PrimalCapability::new(
        "compute",
        "container_orchestration",
        "1.0",
    )];

    match manager.discover_by_capability(&compute_capabilities).await {
        Ok(results) => {
            println!(
                "  ✅ Found {} primals with compute capabilities:",
                results.len()
            );
            for result in results {
                println!("    - {}: {}", result.id, result.endpoint);
            }
        }
        Err(e) => println!("  ❌ Compute discovery failed: {}", e),
    }

    // Look for networking capabilities
    let networking_capabilities = vec![PrimalCapability::new(
        "networking",
        "service_discovery",
        "2.0",
    )];

    match manager
        .discover_by_capability(&networking_capabilities)
        .await
    {
        Ok(results) => {
            println!(
                "  ✅ Found {} primals with networking capabilities:",
                results.len()
            );
            for result in results {
                println!("    - {}: {}", result.id, result.endpoint);
            }
        }
        Err(e) => println!("  ❌ Networking discovery failed: {}", e),
    }

    // Look for storage capabilities
    let storage_capabilities = vec![PrimalCapability::new(
        "storage",
        "persistent_volumes",
        "1.5",
    )];

    match manager.discover_by_capability(&storage_capabilities).await {
        Ok(results) => {
            println!(
                "  ✅ Found {} primals with storage capabilities:",
                results.len()
            );
            for result in results {
                println!("    - {}: {}", result.id, result.endpoint);
            }
        }
        Err(e) => println!("  ❌ Storage discovery failed: {}", e),
    }

    // Test complex capability requirements (multiple capabilities)
    let complex_capabilities = vec![
        PrimalCapability::new("system", "resource_management", "1.0"),
        PrimalCapability::new("compute", "container_orchestration", "1.0"),
    ];

    match manager.discover_by_capability(&complex_capabilities).await {
        Ok(results) => {
            println!(
                "  ✅ Found {} primals with complex capabilities (compute + system):",
                results.len()
            );
            for result in results {
                println!("    - {}: {}", result.id, result.endpoint);
            }
        }
        Err(e) => println!("  ❌ Complex discovery failed: {}", e),
    }

    // Test system health
    let health = manager.get_system_health().await;
    println!("\n🏥 System Health: {:?}", health.overall_status);

    println!("\n✨ Enhanced functionality demo completed successfully!");
    println!("   Demonstrated: Registration, capability discovery, health monitoring");

    Ok(())
}
