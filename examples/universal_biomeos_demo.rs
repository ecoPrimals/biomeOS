// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Universal biomeOS Manager Demo
//!
//! This example demonstrates how the new universal biomeOS manager works
//! using capability-based primal discovery instead of hardcoded implementations.
//!
//! It can discover and work with any primal (current or future) that provides
//! the needed capabilities.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_types::BiomeOSConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌱 Universal biomeOS Manager Demo");
    println!("==================================");
    println!();

    // Initialize the universal manager
    let config = BiomeOSConfig::default();
    let manager = match UniversalBiomeOSManager::new(config) {
        Ok(manager) => {
            println!("✅ biomeOS Manager initialized successfully");
            manager
        }
        Err(e) => {
            println!("❌ Failed to initialize: {e}");
            return Err(e);
        }
    };

    // Discover available primals using 5-tier socket discovery
    println!("\n🔍 Discovering available primals...");
    match manager.discover().await {
        Ok(endpoints) => {
            println!("Found {} endpoints:", endpoints.len());

            for endpoint in &endpoints {
                println!("  🔗 Discovered: {endpoint}");
            }

            if endpoints.is_empty() {
                println!(
                    "  ℹ️  No endpoints discovered - this is normal for isolated environments"
                );
            }
        }
        Err(e) => println!("❌ Discovery failed: {e}"),
    }

    // Test health monitoring
    println!("\n🏥 System Health Check:");
    let health = manager.get_system_health();
    println!("  Overall Status: {:?}", health.health);
    println!("  System ID: {}", health.id);
    println!("  Components: {}", health.components.len());
    println!("  Metrics available: {}", health.metrics.custom.len());

    // Test socket discovery
    println!("\n🌐 Socket Discovery:");
    match manager.discover().await {
        Ok(primals) => {
            println!("  Found {} primals via socket discovery", primals.len());
            for endpoint in primals {
                println!("    🔗 Endpoint: {endpoint}");
            }
        }
        Err(e) => println!("❌ Socket discovery failed: {e}"),
    }

    // Test primal registration
    println!("\n📋 Registered Primals:");
    let registered_primals = manager.get_registered_primals().await;
    println!("  Found {} registered primals", registered_primals.len());
    for primal in registered_primals {
        println!("    📌 {} ({})", primal.name, primal.endpoint);
    }

    println!("\n✨ Demo completed successfully!");
    println!("   The universal manager can discover and work with any primal type");
    println!("   without hardcoded dependencies or assumptions.");

    Ok(())
}
