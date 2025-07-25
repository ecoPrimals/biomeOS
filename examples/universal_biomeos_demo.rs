//! Universal biomeOS Manager Demo
//!
//! This example demonstrates how the new universal biomeOS manager works
//! using capability-based primal discovery instead of hardcoded implementations.
//!
//! It can discover and work with any primal (current or future) that provides
//! the needed capabilities.

use biomeos_core::{universal_biomeos_manager::*, BiomeOSConfig, BiomeResult};
// Removed unused imports: PrimalCapability, PrimalType
use std::collections::HashMap;

#[tokio::main]
async fn main() -> BiomeResult<()> {
    println!("🌱 Universal biomeOS Manager Demo");
    println!("==================================");
    println!();

    // Initialize the universal manager
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Initialize the system
    match manager.initialize().await {
        Ok(_) => println!("✅ biomeOS Manager initialized successfully"),
        Err(e) => println!("❌ Failed to initialize: {}", e),
    }

    // Discover available primals
    println!("\n🔍 Discovering available primals...");
    match manager.discover_primals().await {
        Ok(primals) => {
            println!("Found {} primals:", primals.len());

            for primal in &primals {
                println!("  📦 {} ({})", primal.id, primal.endpoint);
                println!("     Type: {}", primal.primal_type.category);
                println!("     Capabilities: {}", primal.capabilities.len());
                println!("     Health: {:?}", primal.health);
                println!();
            }

            // Group primals by capability
            let mut capability_groups: HashMap<String, Vec<&DiscoveryResult>> = HashMap::new();

            for primal in &primals {
                for capability in &primal.capabilities {
                    capability_groups
                        .entry(capability.name.clone())
                        .or_insert_with(Vec::new)
                        .push(primal);
                }
            }

            // Show capability-based organization
            println!("🎯 Primals organized by capabilities:");
            for (capability, primals) in capability_groups {
                println!("  {} ({} primals)", capability, primals.len());
                for primal in primals {
                    println!("    - {}", primal.id);
                }
            }
        }
        Err(e) => println!("❌ Discovery failed: {}", e),
    }

    // Test health monitoring
    println!("\n🏥 System Health Check:");
    let health = manager.get_system_health().await;
    println!("  Overall Status: {:?}", health.overall_status);
    println!("  Resource Usage:");
    println!("    CPU: {:.1}%", health.resource_usage.cpu_usage_percent);
    println!(
        "    Memory: {:.1}%",
        health.resource_usage.memory_usage_percent
    );
    println!("    Disk: {:.1}%", health.resource_usage.disk_usage_percent);

    // Test network discovery
    println!("\n🌐 Network Discovery:");
    match manager.discover_network_scan().await {
        Ok(network_primals) => {
            println!("  Found {} primals via network scan", network_primals.len());
            for primal in network_primals {
                println!("    🔗 {} at {}", primal.id, primal.endpoint);
            }
        }
        Err(e) => println!("❌ Network discovery failed: {}", e),
    }

    // Test static discovery
    println!("\n📋 Static Configuration Discovery:");
    match manager.discover_static().await {
        Ok(static_primals) => {
            println!("  Found {} primals in static config", static_primals.len());
            for primal in static_primals {
                println!("    📌 {} at {}", primal.id, primal.endpoint);
            }
        }
        Err(e) => println!("❌ Static discovery failed: {}", e),
    }

    println!("\n✨ Demo completed successfully!");
    println!("   The universal manager can discover and work with any primal type");
    println!("   without hardcoded dependencies or assumptions.");

    Ok(())
}
