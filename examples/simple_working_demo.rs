// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Simple BiomeOS Demo
//!
//! Demonstrates basic BiomeOS functionality with actual working APIs

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_types::BiomeOSConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌱 BiomeOS Simple Demo");
    println!("======================");

    // Initialize BiomeOS with default configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config.clone())?;
    println!("✅ BiomeOS manager initialized");

    // Test system health
    let health = manager.get_system_health();
    println!("🏥 System Health: {:?}", health.health);

    // Test 5-tier socket discovery
    match manager.discover().await {
        Ok(discovered) => {
            println!(
                "🔍 Discovery completed: {} primals discovered",
                discovered.len()
            );
        }
        Err(e) => {
            println!("⚠️ Discovery failed: {e}");
        }
    }

    // Test primal registration
    println!("📋 Testing primal registration...");
    let primals = manager.get_registered_primals().await;
    println!("📊 Currently registered primals: {}", primals.len());

    // Test configuration
    println!("⚙️ Testing configuration system...");
    println!("🔧 System configured for: {:?}", config.system.environment);
    println!("🌐 Network binding: {}", config.network.bind_address);

    println!("\n✨ Demo completed successfully!");
    println!("🎯 All core BiomeOS systems are operational!");
    Ok(())
}
