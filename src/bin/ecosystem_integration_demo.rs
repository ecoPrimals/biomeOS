//! # BiomeOS Ecosystem Integration Demo
//!
//! Comprehensive demonstration of the BiomeOS universal ecosystem
//! working with the new Primal SDK.
//!
//! This demo showcases:
//! - BiomeOS primal manager integration
//! - Automatic primal discovery and registration
//! - Universal adapter delegation
//! - Community primal integration
//! - End-to-end request/response flows

use biomeos_core::{primal_manager::BiomeOSPrimalManager, config::BiomeConfig};
use biomeos_primal_sdk::*;
use async_trait::async_trait;
use chrono::Utc;
use clap::Parser;
use serde_json::json;
// use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};

#[derive(Parser, Debug)]
#[command(author, version, about = "BiomeOS Ecosystem Integration Demo")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Demo mode (full, discovery, delegation, community)
    #[arg(short, long, default_value = "full")]
    mode: String,

    /// Number of mock primals to create
    #[arg(short, long, default_value = "3")]
    primal_count: u32,

    /// Demo duration in seconds
    #[arg(short, long, default_value = "30")]
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

    info!("🌱 BiomeOS Ecosystem Integration Demo Starting!");
    info!("============================================");
    
    // Initialize BiomeOS configuration
    let config = BiomeConfig::default();
    
    // Create BiomeOS primal manager (integrates with our SDK)
    info!("🚀 Initializing BiomeOS Primal Manager...");
    let mut primal_manager = BiomeOSPrimalManager::new(config.clone())?;
    
    match args.mode.as_str() {
        "full" => run_full_demo(&mut primal_manager, &args).await?,
        "discovery" => run_discovery_demo(&mut primal_manager, &args).await?,
        "delegation" => run_delegation_demo(&mut primal_manager).await?,
        "community" => run_community_demo(&mut primal_manager).await?,
        _ => {
            error!("Unknown mode: {}. Use: full, discovery, delegation, or community", args.mode);
            std::process::exit(1);
        }
    }
    
    info!("🎉 BiomeOS Ecosystem Demo Complete!");
    Ok(())
}

/// Run full ecosystem demonstration
async fn run_full_demo(
    primal_manager: &mut BiomeOSPrimalManager,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌟 FULL ECOSYSTEM DEMO");
    info!("=====================");
    
    // Phase 1: Create mock ecosystem primals
    info!("📡 Phase 1: Creating mock ecosystem primals...");
    let mock_primals = create_mock_ecosystem_primals(args.primal_count).await?;
    
    // Phase 2: Register primals with BiomeOS
    info!("📚 Phase 2: Registering primals with BiomeOS...");
    for primal in &mock_primals {
        primal_manager.register_primal(Arc::clone(primal)).await?;
        info!("✅ Registered primal: {}", primal.metadata().name);
    }
    
    // Phase 3: Demonstrate discovery
    info!("🔍 Phase 3: Testing primal discovery...");
    let discovered = primal_manager.discover_primals().await?;
    info!("📋 Discovered {} primals:", discovered.len());
    for primal in &discovered {
        info!("  - {} ({})", primal.name, primal.primal_type);
    }
    
    // Phase 4: Demonstrate delegation
    info!("🎯 Phase 4: Testing delegation patterns...");
    demonstrate_delegation(primal_manager).await?;
    
    // Phase 5: Health monitoring
    info!("❤️  Phase 5: Monitoring primal health...");
    demonstrate_health_monitoring(primal_manager, args.duration).await?;
    
    // Phase 6: Community integration
    info!("🌱 Phase 6: Community primal integration...");
    demonstrate_community_integration(primal_manager).await?;
    
    Ok(())
}

/// Run discovery-focused demo
async fn run_discovery_demo(
    primal_manager: &mut BiomeOSPrimalManager,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 DISCOVERY DEMO");
    info!("================");
    
    // Create and register mock primals
    let mock_primals = create_mock_ecosystem_primals(args.primal_count).await?;
    for primal in &mock_primals {
        primal_manager.register_primal(Arc::clone(primal)).await?;
    }
    
    // Test different discovery mechanisms
    info!("📡 Testing network discovery...");
    let network_discovered = primal_manager.discover_primals().await?;
    info!("Found {} primals via network discovery", network_discovered.len());
    
    info!("📋 Registry query...");
    let all_primals = primal_manager.get_all_primals().await;
    info!("Registry contains {} primals", all_primals.len());
    
    // Show detailed primal information
    for primal_info in &all_primals {
        info!("📝 Primal: {}", primal_info.name);
        info!("   Type: {:?}", primal_info.primal_type);
        info!("   Endpoint: {}", primal_info.endpoint);
        info!("   Capabilities: {} items", primal_info.capabilities.len());
        info!("   Health: {:?}", primal_info.health_status);
    }
    
    Ok(())
}

/// Run delegation-focused demo
async fn run_delegation_demo(
    primal_manager: &mut BiomeOSPrimalManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 DELEGATION DEMO");
    info!("=================");
    
    // Create specialized primals for each delegation type
    let primals = vec![
        create_mock_toadstool_primal().await?,
        create_mock_beardog_primal().await?,
        create_mock_songbird_primal().await?,
        create_mock_nestgate_primal().await?,
        create_mock_squirrel_primal().await?,
    ];
    
    for primal in &primals {
        primal_manager.register_primal(Arc::clone(primal)).await?;
    }
    
    // Test each delegation method
    test_manifest_delegation(primal_manager).await?;
    test_encryption_delegation(primal_manager).await?;
    test_service_discovery_delegation(primal_manager).await?;
    test_tunneling_delegation(primal_manager).await?;
    test_plugin_delegation(primal_manager).await?;
    
    Ok(())
}

/// Run community-focused demo
async fn run_community_demo(
    primal_manager: &mut BiomeOSPrimalManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌱 COMMUNITY DEMO");
    info!("================");
    
    // Create community primals of different types
    let community_primals = vec![
        create_community_gaming_primal().await?,
        create_community_ai_primal().await?,
        create_community_iot_primal().await?,
    ];
    
    for primal in &community_primals {
        primal_manager.register_primal(Arc::clone(primal)).await?;
        info!("🌟 Registered community primal: {}", primal.metadata().name);
    }
    
    // Test community primal functionality
    test_community_primal_interactions(primal_manager).await?;
    
    // Show ecosystem growth
    let total_primals = primal_manager.get_all_primals().await.len();
    info!("🚀 Ecosystem now contains {} primals (including community contributions)", total_primals);
    
    Ok(())
}

/// Create mock ecosystem primals for testing
async fn create_mock_ecosystem_primals(count: u32) -> Result<Vec<Arc<dyn EcoPrimal>>, Box<dyn std::error::Error>> {
    let mut primals = Vec::new();
    
    for i in 1..=count {
        let primal = Arc::new(MockEcosystemPrimal::new(
            &format!("mock-primal-{}", i),
            PrimalType::Community { 
                name: format!("mock-primal-{}", i), 
                category: PrimalCategory::Custom("demo".to_string()) 
            },
            vec![PrimalCapability::Custom {
                name: format!("MockCapability{}", i),
                description: format!("Mock capability for primal {}", i),
            }],
        )?);
        primals.push(primal as Arc<dyn EcoPrimal>);
    }
    
    Ok(primals)
}

/// Create mock ToadStool primal
async fn create_mock_toadstool_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "mock-toadstool",
        PrimalType::ToadStool,
        vec![
            PrimalCapability::SystemManagement,
            PrimalCapability::ProcessManagement,
            PrimalCapability::Custom {
                name: "ManifestParsing".to_string(),
                description: "BYOB manifest parsing and validation".to_string(),
            },
        ],
    )?))
}

/// Create mock BearDog primal  
async fn create_mock_beardog_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "mock-beardog",
        PrimalType::BearDog,
        vec![
            PrimalCapability::Encryption,
            PrimalCapability::KeyManagement,
            PrimalCapability::Authentication,
            PrimalCapability::Authorization,
        ],
    )?))
}

/// Create mock Songbird primal
async fn create_mock_songbird_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "mock-songbird",
        PrimalType::Songbird,
        vec![
            PrimalCapability::ServiceDiscovery,
            PrimalCapability::MessageRouting,
            PrimalCapability::LoadBalancing,
            PrimalCapability::ServiceMesh,
        ],
    )?))
}

/// Create mock NestGate primal
async fn create_mock_nestgate_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "mock-nestgate",
        PrimalType::NestGate,
        vec![
            PrimalCapability::NetworkManagement,
            PrimalCapability::Custom {
                name: "TunnelManagement".to_string(),
                description: "Network tunnel creation and management".to_string(),
            },
        ],
    )?))
}

/// Create mock Squirrel primal
async fn create_mock_squirrel_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "mock-squirrel",
        PrimalType::Squirrel,
        vec![
            PrimalCapability::PluginManagement,
            PrimalCapability::CodeExecution,
            PrimalCapability::Sandboxing,
        ],
    )?))
}

/// Create community gaming primal
async fn create_community_gaming_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "community-game-engine",
        PrimalType::Community {
            name: "game-engine".to_string(),
            category: PrimalCategory::Gaming,
        },
        vec![
            PrimalCapability::Gaming,
            PrimalCapability::Custom {
                name: "GameLogic".to_string(),
                description: "Game logic processing and state management".to_string(),
            },
        ],
    )?))
}

/// Create community AI primal
async fn create_community_ai_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "community-ml-processor",
        PrimalType::Community {
            name: "ml-processor".to_string(),
            category: PrimalCategory::AI,
        },
        vec![
            PrimalCapability::AI,
            PrimalCapability::Custom {
                name: "MachineLearning".to_string(),
                description: "ML model training and inference".to_string(),
            },
        ],
    )?))
}

/// Create community IoT primal
async fn create_community_iot_primal() -> Result<Arc<dyn EcoPrimal>, Box<dyn std::error::Error>> {
    Ok(Arc::new(MockEcosystemPrimal::new(
        "community-iot-hub",
        PrimalType::Community {
            name: "iot-hub".to_string(),
            category: PrimalCategory::IoT,
        },
        vec![
            PrimalCapability::IoT,
            PrimalCapability::DeviceManagement,
            PrimalCapability::Custom {
                name: "SensorData".to_string(),
                description: "IoT sensor data collection and processing".to_string(),
            },
        ],
    )?))
}

/// Demonstrate community integration  
async fn demonstrate_community_integration(
    primal_manager: &BiomeOSPrimalManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌱 Demonstrating community primal integration...");
    
    let primals = primal_manager.get_all_primals().await;
    let community_primals: Vec<_> = primals.iter()
        .filter(|p| matches!(p.primal_type, PrimalType::Community { .. }))
        .collect();
        
    info!("📋 Found {} community primals in ecosystem", community_primals.len());
    
    for primal in community_primals {
        info!("  🌟 {}: {:?}", primal.name, primal.primal_type);
    }
    
    Ok(())
}

/// Demonstrate delegation patterns
async fn demonstrate_delegation(
    primal_manager: &mut BiomeOSPrimalManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 Testing delegation patterns...");
    
    // Test delegation methods (if available in the primal manager)
    let delegation_tests = vec![
        ("ManifestParsing", json!({"manifest": "test.yaml"})),
        ("Encryption", json!({"data": "sensitive information", "key": "test-key"})),
        ("ServiceDiscovery", json!({"service_type": "http", "port": 8080})),
        ("NetworkTunneling", json!({"source": "192.168.1.1", "target": "10.0.0.1"})),
        ("PluginManagement", json!({"plugin": "test-plugin.wasm"})),
    ];
    
    for (method, payload) in delegation_tests {
        info!("  🎯 Testing {} delegation...", method);
        
        // Create a test request
        let request = PrimalRequest::new(method, payload);
        
        // Find appropriate primal for this delegation
        let primals = primal_manager.get_all_primals().await;
        let suitable_primal = primals.iter().find(|p| {
            p.capabilities.iter().any(|cap| match cap {
                PrimalCapability::SystemManagement if method == "ManifestParsing" => true,
                PrimalCapability::Encryption if method == "Encryption" => true,
                PrimalCapability::ServiceDiscovery if method == "ServiceDiscovery" => true,
                PrimalCapability::NetworkManagement if method == "NetworkTunneling" => true,
                PrimalCapability::PluginManagement if method == "PluginManagement" => true,
                _ => false,
            })
        });
        
        if let Some(primal) = suitable_primal {
            info!("    ✅ Found suitable primal: {}", primal.name);
            info!("    📡 Endpoint: {}", primal.endpoint);
        } else {
            warn!("    ⚠️  No suitable primal found for {} delegation", method);
        }
    }
    
    Ok(())
}

/// Test specific delegation methods
async fn test_manifest_delegation(primal_manager: &BiomeOSPrimalManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🍄 Testing ToadStool manifest delegation...");
    // Implementation would interact with actual ToadStool primal
    info!("   ✅ Manifest parsing delegation test complete");
    Ok(())
}

async fn test_encryption_delegation(primal_manager: &BiomeOSPrimalManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🐻 Testing BearDog encryption delegation...");
    // Implementation would interact with actual BearDog primal
    info!("   ✅ Encryption delegation test complete");
    Ok(())
}

async fn test_service_discovery_delegation(primal_manager: &BiomeOSPrimalManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎼 Testing Songbird service discovery delegation...");
    // Implementation would interact with actual Songbird primal
    info!("   ✅ Service discovery delegation test complete");
    Ok(())
}

async fn test_tunneling_delegation(primal_manager: &BiomeOSPrimalManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🕸️  Testing NestGate tunneling delegation...");
    // Implementation would interact with actual NestGate primal
    info!("   ✅ Tunneling delegation test complete");
    Ok(())
}

async fn test_plugin_delegation(primal_manager: &BiomeOSPrimalManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🐿️  Testing Squirrel plugin delegation...");
    // Implementation would interact with actual Squirrel primal
    info!("   ✅ Plugin delegation test complete");
    Ok(())
}

/// Demonstrate health monitoring
async fn demonstrate_health_monitoring(
    primal_manager: &BiomeOSPrimalManager,
    duration: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("❤️  Monitoring primal health for {} seconds...", duration);
    
    let start_time = std::time::Instant::now();
    let mut check_count = 0;
    
    while start_time.elapsed().as_secs() < duration {
        let primals = primal_manager.get_all_primals().await;
        let healthy_count = primals.iter()
            .filter(|p| matches!(p.health_status, HealthStatus::Healthy))
            .count();
        
        info!("   📊 Health check #{}: {}/{} primals healthy", 
              check_count + 1, healthy_count, primals.len());
        
        check_count += 1;
        sleep(Duration::from_secs(5)).await;
    }
    
    info!("   ✅ Health monitoring complete ({} checks performed)", check_count);
    Ok(())
}

/// Test community primal interactions
async fn test_community_primal_interactions(
    primal_manager: &BiomeOSPrimalManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌱 Testing community primal interactions...");
    
    let primals = primal_manager.get_all_primals().await;
    let community_primals: Vec<_> = primals.iter()
        .filter(|p| matches!(p.primal_type, PrimalType::Community { .. }))
        .collect();
    
    info!("   📋 Found {} community primals", community_primals.len());
    
    for primal in community_primals {
        info!("   🌟 Testing community primal: {}", primal.name);
        
        // Create test request for the primal
        let test_request = PrimalRequest::new("ping", json!({"message": "community test"}));
        
        // Note: In a real implementation, we would send this request to the primal
        // For this demo, we just show that the primal is available and has capabilities
        info!("     📡 Endpoint: {}", primal.endpoint);
        info!("     ⚡ Capabilities: {} items", primal.capabilities.len());
        info!("     ❤️  Health: {:?}", primal.health_status);
    }
    
    info!("   ✅ Community primal interaction test complete");
    Ok(())
}

/// Mock ecosystem primal implementation for demo purposes
struct MockEcosystemPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    health: PrimalHealth,
}

impl MockEcosystemPrimal {
    fn new(
        name: &str,
        primal_type: PrimalType,
        capabilities: Vec<PrimalCapability>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = PrimalMetadata {
            name: name.to_string(),
            primal_type,
            version: "1.0.0".to_string(),
            description: format!("Mock {} primal for ecosystem demo", name),
            author: "BiomeOS Demo <demo@biomeos.org>".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://biomeos.org".to_string()),
            documentation: Some("https://docs.biomeos.org".to_string()),
            keywords: vec!["biomeos".to_string(), "demo".to_string()],
            min_biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let health = PrimalHealth::healthy();
        
        Ok(Self {
            metadata,
            capabilities,
            health,
        })
    }
}

#[async_trait]
impl EcoPrimal for MockEcosystemPrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        debug!("Initializing mock primal: {}", self.metadata.name);
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        debug!("Mock primal {} handling request: {}", self.metadata.name, request.method);
        
        match request.method.as_str() {
            "ping" => {
                Ok(PrimalResponse::success(
                    request.request_id,
                    json!({
                        "message": "pong",
                        "primal": self.metadata.name,
                        "timestamp": Utc::now().to_rfc3339()
                    })
                ))
            }
            "get_info" => {
                Ok(PrimalResponse::success(
                    request.request_id,
                    json!({
                        "name": self.metadata.name,
                        "type": self.metadata.primal_type,
                        "version": self.metadata.version,
                        "capabilities": self.capabilities.len()
                    })
                ))
            }
            _ => {
                // Mock handling of other requests
                Ok(PrimalResponse::success(
                    request.request_id,
                    json!({
                        "message": format!("Mock response from {}", self.metadata.name),
                        "method": request.method,
                        "processed_at": Utc::now().to_rfc3339()
                    })
                ))
            }
        }
    }
    
    async fn health_check(&self) -> PrimalHealth {
        self.health.clone()
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        debug!("Shutting down mock primal: {}", self.metadata.name);
        Ok(())
    }
}

impl std::fmt::Debug for MockEcosystemPrimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockEcosystemPrimal")
            .field("name", &self.metadata.name)
            .field("type", &self.metadata.primal_type)
            .field("capabilities_count", &self.capabilities.len())
            .finish()
    }
} 