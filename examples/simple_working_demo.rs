//! Simple BiomeOS Demo
//!
//! Demonstrates basic BiomeOS functionality with actual working APIs

use anyhow::Result;
use biomeos::universal_ui::{BiomeOSUI, UIFeatures, UIMode};
use biomeos_core::{BiomeOSConfig, UniversalBiomeOSManager};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌱 BiomeOS Simple Demo");
    println!("======================");

    // Initialize BiomeOS with default configuration
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);

    // Initialize the manager
    manager.initialize().await?;
    println!("✅ BiomeOS manager initialized");

    // Test system health
    let health = manager.get_system_health().await;
    println!("🏥 System Health: {:?}", health.overall_status);

    // Test network discovery (will return empty for now, but won't error)
    match manager.discover_network_scan().await {
        Ok(discovered) => {
            println!(
                "🔍 Network scan completed: {} services discovered",
                discovered.len()
            );
        }
        Err(e) => {
            println!("⚠️ Network scan failed: {}", e);
        }
    }

    // Test UI initialization
    let ui = BiomeOSUI::new(UIMode::Terminal);
    match ui.initialize().await {
        Ok(_) => println!("🎨 UI system initialized in Terminal mode"),
        Err(e) => println!("⚠️ UI initialization failed: {}", e),
    }

    let features = UIFeatures {
        dashboard_enabled: true,
        monitoring_enabled: true,
        primal_management: true,
        system_controls: false,
        advanced_features: false,
    };

    let ui_with_features = BiomeOSUI::new(UIMode::Auto).with_features(features);
    match ui_with_features.render().await {
        Ok(output) => println!("🖥️ UI rendered: {}", output),
        Err(e) => println!("⚠️ UI render failed: {}", e),
    }

    println!("\n✨ Demo completed successfully!");
    Ok(())
}
