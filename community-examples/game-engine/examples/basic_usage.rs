//! Basic usage example for game-engine primal

use game_engine_primal::*;
use biomeos_primal_sdk::*;
use tokio;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("🌱 game-engine Primal Example");
    println!("====================");
    
    // Create configuration
    let config = GameEngineConfig::default();
    println!("✅ Created configuration");
    
    // Create primal instance
    let primal = GameEngine::new(config);
    println!("✅ Created primal instance");
    
    // Initialize primal
    let sdk_config = PrimalConfig::default();
    primal.initialize(&sdk_config).await?;
    println!("✅ Initialized primal");
    
    // Display metadata
    let metadata = primal.metadata();
    println!("📋 Primal Information:");
    println!("   Name: {}", metadata.name);
    println!("   Version: {}", metadata.version);
    println!("   Type: {:?}", metadata.primal_type);
    println!("   Description: {}", metadata.description);
    
    // Display capabilities
    let capabilities = primal.capabilities();
    println!("⚡ Capabilities:");
    for capability in capabilities {
        println!("   - {:?}", capability);
    }
    
    // Test health check
    let health = primal.health_check().await;
    println!("❤️  Health Status: {:?}", health.status);
    
    // Test request handling
    println!("🔄 Testing requests...");
    
    // Ping request
    let ping_request = PrimalRequest::new("ping", serde_json::json!({}));
    match primal.handle_request(ping_request).await {
        Ok(response) => {
            println!("✅ Ping response: {}", response.payload);
        }
        Err(e) => {
            println!("❌ Ping failed: {}", e);
        }
    }
    
    // Config request
    let config_request = PrimalRequest::new("get_config", serde_json::json!({}));
    match primal.handle_request(config_request).await {
        Ok(response) => {
            println!("✅ Config response: {}", response.payload);
        }
        Err(e) => {
            println!("❌ Config failed: {}", e);
        }
    }
    
    // Shutdown primal
    primal.shutdown().await?;
    println!("✅ Primal shut down successfully");
    
    println!("🎉 Example completed successfully!");
    
    Ok(())
}
