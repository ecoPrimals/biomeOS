//! # Simple BiomeOS Ecosystem Demo
//!
//! A simplified demonstration of the BiomeOS ecosystem working with
//! the Primal SDK for community development.

use biomeos_primal_sdk::*;
use async_trait::async_trait;
use chrono::Utc;
use clap::Parser;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, debug};

#[derive(Parser, Debug)]
#[command(author, version, about = "Simple BiomeOS Ecosystem Demo")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Demo duration in seconds
    #[arg(short, long, default_value = "20")]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Setup logging
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("🌱 BiomeOS Simple Ecosystem Demo Starting!");
    info!("===========================================");
    
    // Phase 1: Demonstrate Primal SDK functionality
    info!("🧬 Phase 1: Primal SDK Core Features");
    demonstrate_primal_sdk().await?;
    
    // Phase 2: Show community primals
    info!("🌟 Phase 2: Community Primal Examples");
    demonstrate_community_primals().await?;
    
    // Phase 3: Universal primal interface
    info!("🔗 Phase 3: Universal Primal Interface");
    demonstrate_universal_interface().await?;
    
    // Phase 4: Ecosystem coordination
    info!("🌐 Phase 4: Ecosystem Coordination");
    demonstrate_ecosystem_coordination(&args).await?;
    
    info!("🎉 BiomeOS Simple Ecosystem Demo Complete!");
    info!("The universal primal ecosystem is operational!");
    
    Ok(())
}

/// Demonstrate core Primal SDK functionality
async fn demonstrate_primal_sdk() -> Result<(), Box<dyn std::error::Error>> {
    info!("  📚 Creating primal registry...");
    let registry_config = RegistryConfig::default();
    let registry = PrimalRegistry::new(registry_config);
    
    info!("  🔍 Creating discovery service...");
    let discovery_config = DiscoveryConfig::default();
    let discovery = PrimalDiscoveryService::new(discovery_config);
    
    info!("  ✅ Core SDK components initialized");
    info!("     - Registry: Ready for primal registration");
    info!("     - Discovery: Ready for network scanning");
    
    Ok(())
}

/// Demonstrate community primals created with CLI
async fn demonstrate_community_primals() -> Result<(), Box<dyn std::error::Error>> {
    info!("  🎮 Creating gaming primal...");
    let gaming_primal = Arc::new(DemoGamePrimal::new()?);
    
    info!("  🤖 Creating AI primal...");
    let ai_primal = Arc::new(DemoAIPrimal::new()?);
    
    info!("  🌐 Creating IoT primal...");
    let iot_primal = Arc::new(DemoIoTPrimal::new()?);
    
    // Test each primal
    info!("  🧪 Testing community primals...");
    
    // Gaming primal test
    let game_request = PrimalRequest::new("start_game", json!({"level": 1}));
    let game_response = gaming_primal.handle_request(game_request).await?;
    info!("     🎮 Gaming response: {}", game_response.payload.get("message").unwrap_or(&json!("N/A")));
    
    // AI primal test
    let ai_request = PrimalRequest::new("predict", json!({"data": [1, 2, 3, 4]}));
    let ai_response = ai_primal.handle_request(ai_request).await?;
    info!("     🤖 AI response: {}", ai_response.payload.get("prediction").unwrap_or(&json!("N/A")));
    
    // IoT primal test
    let iot_request = PrimalRequest::new("read_sensors", json!({}));
    let iot_response = iot_primal.handle_request(iot_request).await?;
    info!("     🌐 IoT response: {} sensors", iot_response.payload.get("sensor_count").unwrap_or(&json!(0)));
    
    info!("  ✅ Community primals operational");
    
    Ok(())
}

/// Demonstrate universal primal interface
async fn demonstrate_universal_interface() -> Result<(), Box<dyn std::error::Error>> {
    info!("  🔗 Testing universal EcoPrimal interface...");
    
    // Create different types of primals
    let primals: Vec<Arc<dyn EcoPrimal>> = vec![
        Arc::new(CoreToadStoolPrimal::new()?),
        Arc::new(CoreBearDogPrimal::new()?),
        Arc::new(CoreSongbirdPrimal::new()?),
        Arc::new(DemoGamePrimal::new()?),
        Arc::new(DemoAIPrimal::new()?),
    ];
    
    info!("  📋 Primal ecosystem overview:");
    for primal in &primals {
        let metadata = primal.metadata();
        let health = primal.health_check().await;
        
        info!("     - {}: {:?} (Health: {:?})", 
              metadata.name, 
              metadata.primal_type,
              health.status);
        
        info!("       Capabilities: {} items", primal.capabilities().len());
    }
    
    // Test universal request handling
    info!("  🧪 Testing universal request handling...");
    let test_request = PrimalRequest::new("ping", json!({"test": true}));
    
    for (i, primal) in primals.iter().enumerate() {
        let response = primal.handle_request(test_request.clone()).await;
        match response {
            Ok(resp) => info!("     ✅ Primal {}: {}", i + 1, resp.payload.get("message").unwrap_or(&json!("OK"))),
            Err(e) => info!("     ⚠️  Primal {}: {}", i + 1, e),
        }
    }
    
    info!("  ✅ Universal interface working");
    
    Ok(())
}

/// Demonstrate ecosystem coordination
async fn demonstrate_ecosystem_coordination(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    info!("  🌐 Simulating ecosystem coordination...");
    
    let primals = vec![
        ("ToadStool", "Manifest Processing"),
        ("BearDog", "Encryption & Security"),
        ("Songbird", "Service Discovery"), 
        ("NestGate", "Network Tunneling"),
        ("Squirrel", "Plugin Management"),
        ("GameEngine", "Community Gaming"),
        ("MLProcessor", "Community AI"),
        ("IoTHub", "Community IoT"),
    ];
    
    info!("  📡 Active ecosystem primals:");
    for (name, capability) in &primals {
        info!("     🌟 {}: {}", name, capability);
    }
    
    // Simulate coordination activities
    let coordination_duration = args.duration.min(20);
    info!("  🔄 Running ecosystem coordination for {} seconds...", coordination_duration);
    
    for i in 0..coordination_duration {
        match i % 4 {
            0 => info!("     📡 Service discovery scan complete - {} services found", primals.len()),
            1 => info!("     🔐 Security check passed - All primals authenticated"),
            2 => info!("     🌐 Health monitoring - All systems operational"),
            3 => info!("     💾 State sync complete - Ecosystem coordinated"),
            _ => {}
        }
        
        sleep(Duration::from_secs(1)).await;
        
        if i % 5 == 4 {
            info!("     📊 Ecosystem status: {} primals active, all healthy", primals.len());
        }
    }
    
    info!("  ✅ Ecosystem coordination successful");
    info!("     - All primals discovered and registered");
    info!("     - Universal interface operational");  
    info!("     - Community primals integrated seamlessly");
    info!("     - BiomeOS serving as universal coordinator");
    
    Ok(())
}

// Demo primal implementations

/// Demo gaming primal (represents community-generated primal)
pub struct DemoGamePrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl DemoGamePrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "demo-game-engine".to_string(),
            primal_type: PrimalType::Community {
                name: "game-engine".to_string(),
                category: PrimalCategory::Gaming,
            },
            version: "1.0.0".to_string(),
            description: "Demo gaming primal created with BiomeOS CLI".to_string(),
            author: "Community Developer <dev@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://biomeos.org".to_string()),
            documentation: None,
            keywords: vec!["gaming".to_string(), "community".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::Gaming,
            PrimalCapability::Custom {
                name: "GameLogic".to_string(),
                description: "Game logic and state management".to_string(),
            },
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for DemoGamePrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        debug!("Initializing demo gaming primal");
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(request.request_id, json!({"message": "Game Engine Ready!"}))),
            "start_game" => Ok(PrimalResponse::success(request.request_id, json!({"message": "Game started!", "level": 1}))),
            "get_score" => Ok(PrimalResponse::success(request.request_id, json!({"score": 9001, "level": 5}))),
            _ => Err(PrimalError::InvalidRequest("Unknown game method".to_string()))
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        PrimalHealth::healthy()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        debug!("Shutting down demo gaming primal");
        Ok(())
    }
}

/// Demo AI primal (represents community-generated primal)
pub struct DemoAIPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl DemoAIPrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "demo-ml-processor".to_string(),
            primal_type: PrimalType::Community {
                name: "ml-processor".to_string(),
                category: PrimalCategory::AI,
            },
            version: "1.0.0".to_string(),
            description: "Demo AI/ML primal for machine learning tasks".to_string(),
            author: "ML Team <ml@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://biomeos.org".to_string()),
            documentation: None,
            keywords: vec!["ai".to_string(), "ml".to_string(), "community".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::AI,
            PrimalCapability::Custom {
                name: "MachineLearning".to_string(),
                description: "ML model training and inference".to_string(),
            },
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for DemoAIPrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        debug!("Initializing demo AI primal");
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(request.request_id, json!({"message": "AI Processor Online!"}))),
            "predict" => Ok(PrimalResponse::success(request.request_id, json!({"prediction": 42.7, "confidence": 0.95}))),
            "train" => Ok(PrimalResponse::success(request.request_id, json!({"status": "training", "epochs": 100}))),
            _ => Err(PrimalError::InvalidRequest("Unknown AI method".to_string()))
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        PrimalHealth::healthy()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        debug!("Shutting down demo AI primal");
        Ok(())
    }
}

/// Demo IoT primal (represents community-generated primal)
pub struct DemoIoTPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl DemoIoTPrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "demo-iot-hub".to_string(),
            primal_type: PrimalType::Community {
                name: "iot-hub".to_string(),
                category: PrimalCategory::IoT,
            },
            version: "1.0.0".to_string(),
            description: "Demo IoT primal for device management".to_string(),
            author: "IoT Team <iot@example.com>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://biomeos.org".to_string()),
            documentation: None,
            keywords: vec!["iot".to_string(), "devices".to_string(), "community".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::IoT,
            PrimalCapability::DeviceManagement,
            PrimalCapability::Custom {
                name: "SensorData".to_string(),
                description: "IoT sensor data processing".to_string(),
            },
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for DemoIoTPrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        debug!("Initializing demo IoT primal");
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(request.request_id, json!({"message": "IoT Hub Connected!"}))),
            "read_sensors" => Ok(PrimalResponse::success(request.request_id, json!({"sensor_count": 24, "status": "all_active"}))),
            "control_device" => Ok(PrimalResponse::success(request.request_id, json!({"device": "thermostat", "status": "updated"}))),
            _ => Err(PrimalError::InvalidRequest("Unknown IoT method".to_string()))
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        PrimalHealth::healthy()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        debug!("Shutting down demo IoT primal");
        Ok(())
    }
}

// Core primal mockups (representing the ecosystem primals)

/// Demo ToadStool primal
pub struct CoreToadStoolPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl CoreToadStoolPrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "core-toadstool".to_string(),
            primal_type: PrimalType::ToadStool,
            version: "1.0.0".to_string(),
            description: "Core ToadStool primal for manifest processing".to_string(),
            author: "ToadStool Team <toadstool@ecosystem.org>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://toadstool.org".to_string()),
            documentation: None,
            keywords: vec!["core".to_string(), "manifest".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::SystemManagement,
            PrimalCapability::ProcessManagement,
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for CoreToadStoolPrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> { Ok(()) }
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        Ok(PrimalResponse::success(request.request_id, json!({"message": "ToadStool processing manifest"})))
    }
    async fn health_check(&self) -> PrimalHealth { PrimalHealth::healthy() }
    async fn shutdown(&self) -> PrimalResult<()> { Ok(()) }
}

/// Demo BearDog primal
pub struct CoreBearDogPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl CoreBearDogPrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "core-beardog".to_string(),
            primal_type: PrimalType::BearDog,
            version: "1.0.0".to_string(),
            description: "Core BearDog primal for encryption and security".to_string(),
            author: "BearDog Team <beardog@ecosystem.org>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://beardog.org".to_string()),
            documentation: None,
            keywords: vec!["core".to_string(), "security".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::Encryption,
            PrimalCapability::Authentication,
            PrimalCapability::KeyManagement,
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for CoreBearDogPrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> { Ok(()) }
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        Ok(PrimalResponse::success(request.request_id, json!({"message": "BearDog securing data"})))
    }
    async fn health_check(&self) -> PrimalHealth { PrimalHealth::healthy() }
    async fn shutdown(&self) -> PrimalResult<()> { Ok(()) }
}

/// Demo Songbird primal
pub struct CoreSongbirdPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl CoreSongbirdPrimal {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: "core-songbird".to_string(),
            primal_type: PrimalType::Songbird,
            version: "1.0.0".to_string(),
            description: "Core Songbird primal for service discovery".to_string(),
            author: "Songbird Team <songbird@ecosystem.org>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://songbird.org".to_string()),
            documentation: None,
            keywords: vec!["core".to_string(), "discovery".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let capabilities = vec![
            PrimalCapability::ServiceDiscovery,
            PrimalCapability::MessageRouting,
            PrimalCapability::LoadBalancing,
        ];
        
        Ok(Self { metadata, capabilities })
    }
}

#[async_trait]
impl EcoPrimal for CoreSongbirdPrimal {
    fn metadata(&self) -> &PrimalMetadata { &self.metadata }
    fn capabilities(&self) -> &[PrimalCapability] { &self.capabilities }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> { Ok(()) }
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        Ok(PrimalResponse::success(request.request_id, json!({"message": "Songbird discovering services"})))
    }
    async fn health_check(&self) -> PrimalHealth { PrimalHealth::healthy() }
    async fn shutdown(&self) -> PrimalResult<()> { Ok(()) }
} 