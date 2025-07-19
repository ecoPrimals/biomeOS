//! Universal biomeOS Manager Demo
//!
//! This example demonstrates how the new universal biomeOS manager works
//! using capability-based primal discovery instead of hardcoded implementations.
//!
//! It can discover and work with any primal (current or future) that provides
//! the needed capabilities.

use biomeos_core::{
    universal_biomeos_manager::*,
    primal_clients::CapabilityCategory,
    BiomeResult,
};

#[tokio::main]
async fn main() -> BiomeResult<()> {
    println!("🌱 Universal biomeOS Manager Demo");
    println!("==================================");
    println!();
    
    // Create biomeOS manager with capability-based discovery
    let manager = create_biomeos_manager().await?;
    
    // Show ecosystem status
    println!("📊 Ecosystem Status:");
    let ecosystem = manager.get_ecosystem_health().await?;
    println!("   Health: {:?}", ecosystem.health);
    println!("   Primals: {}", ecosystem.active_primals);
    println!();
    
    // Show available capabilities
    println!("🔧 Available Capabilities:");
    let capabilities = manager.get_available_capabilities().await?;
    for (category, primals) in capabilities {
        println!("   {:?}: {} primals", category, primals.len());
    }
    println!();
    
    // Demonstrate orchestration capability
    if capabilities.contains_key(&CapabilityCategory::Orchestration) {
        println!("🚀 Testing Orchestration Capability:");
        
        let manifest = r#"
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: demo-biome
  namespace: demo
spec:
  services:
  - name: web-service
    image: nginx:alpine
    ports: [80]
    resources:
      cpu: "0.5"
      memory: "512Mi"
"#;
        
        match manager.deploy_biome(manifest).await {
            Ok(deployment_id) => {
                println!("   ✅ Deployed biome: {}", deployment_id);
            }
            Err(e) => {
                println!("   ❌ Deployment failed: {}", e);
            }
        }
    } else {
        println!("⚠️  No orchestration capability available");
    }
    println!();
    
    // Demonstrate service mesh capability
    if capabilities.contains_key(&CapabilityCategory::ServiceMesh) {
        println!("🔍 Testing Service Mesh Capability:");
        
        match manager.discover_services().await {
            Ok(services) => {
                println!("   ✅ Discovered {} services", services.len());
                for service in services.iter().take(3) {
                    println!("     - {}: {}", service.name, service.endpoint);
                }
            }
            Err(e) => {
                println!("   ❌ Service discovery failed: {}", e);
            }
        }
    } else {
        println!("⚠️  No service mesh capability available");
    }
    println!();
    
    // Demonstrate storage capability
    if capabilities.contains_key(&CapabilityCategory::Storage) {
        println!("💾 Testing Storage Capability:");
        
        match manager.create_storage_volume("10Gi", Some("fast-ssd".to_string())).await {
            Ok(volume_id) => {
                println!("   ✅ Created storage volume: {}", volume_id);
            }
            Err(e) => {
                println!("   ❌ Storage creation failed: {}", e);
            }
        }
    } else {
        println!("⚠️  No storage capability available");
    }
    println!();
    
    // Demonstrate security capability
    if capabilities.contains_key(&CapabilityCategory::Security) {
        println!("🔐 Testing Security Capability:");
        
        match manager.authenticate("demo-user", "demo-password").await {
            Ok(token) => {
                println!("   ✅ Authentication successful: {}...", &token[..20]);
            }
            Err(e) => {
                println!("   ❌ Authentication failed: {}", e);
            }
        }
    } else {
        println!("⚠️  No security capability available");
    }
    println!();
    
    // Demonstrate intelligence capability
    if capabilities.contains_key(&CapabilityCategory::Intelligence) {
        println!("🤖 Testing Intelligence Capability:");
        
        match manager.deploy_ai_agent(
            "demo-agent",
            "data-analyst",
            vec!["analysis".to_string(), "visualization".to_string()],
        ).await {
            Ok(agent_id) => {
                println!("   ✅ Deployed AI agent: {}", agent_id);
            }
            Err(e) => {
                println!("   ❌ AI agent deployment failed: {}", e);
            }
        }
    } else {
        println!("⚠️  No intelligence capability available");
    }
    println!();
    
    // Demonstrate custom capability
    println!("⚡ Testing Custom Capability:");
    let custom_params = serde_json::json!({
        "operation": "ping",
        "target": "localhost"
    });
    
    match manager.execute_custom_capability(
        CapabilityCategory::Custom("network-tools".to_string()),
        "ping",
        custom_params,
    ).await {
        Ok(result) => {
            println!("   ✅ Custom operation result: {:?}", result);
        }
        Err(e) => {
            println!("   ❌ Custom operation failed: {}", e);
        }
    }
    println!();
    
    // Show final ecosystem health
    println!("🏥 Final Ecosystem Health:");
    let final_ecosystem = manager.get_ecosystem_health().await?;
    println!("   Overall Health: {:?}", final_ecosystem.health);
    println!("   Active Primals: {}", final_ecosystem.active_primals);
    
    for (id, primal) in final_ecosystem.active_primals {
        println!("     - {}: {:?} ({})", id, primal.health, primal.endpoint);
    }
    println!();
    
    println!("🎉 Demo Complete!");
    println!();
    println!("Key Features Demonstrated:");
    println!("• Capability-based primal discovery");
    println!("• Universal orchestration (any primal can provide)");
    println!("• Service mesh integration (any primal can provide)");
    println!("• Storage management (any primal can provide)");
    println!("• Security operations (any primal can provide)");
    println!("• AI agent deployment (any primal can provide)");
    println!("• Custom capability execution (future primals)");
    println!("• Ecosystem health monitoring");
    println!();
    println!("💡 This system can work with:");
    println!("• Current primals: Toadstool, Songbird, NestGate, BearDog, Squirrel");
    println!("• Future primals: Any primal that implements the capability interface");
    println!("• Community primals: Third-party implementations");
    println!("• Custom primals: Your own capability providers");
    
    Ok(())
}

/// Example of creating a custom biomeOS configuration
#[allow(dead_code)]
async fn demo_custom_config() -> BiomeResult<()> {
    use biomeos_core::primal_clients::CapabilityRequirement;
    
    println!("🛠️  Creating custom biomeOS configuration...");
    
    // Create custom configuration
    let custom_config = BiomeOSConfig {
        auto_discovery: true,
        discovery_timeout: 60, // Longer timeout for slow networks
        required_capabilities: vec![
            CapabilityRequirement {
                category: CapabilityCategory::Orchestration,
                operations: vec![
                    "deploy_biome".to_string(),
                    "scale_service".to_string(),
                ],
                min_version: Some("1.0.0".to_string()),
                optional: false,
            },
            CapabilityRequirement {
                category: CapabilityCategory::Security,
                operations: vec!["authenticate".to_string()],
                min_version: None,
                optional: false,
            },
        ],
        optional_capabilities: vec![
            CapabilityRequirement {
                category: CapabilityCategory::Intelligence,
                operations: vec!["deploy_agent".to_string()],
                min_version: None,
                optional: true,
            },
            CapabilityRequirement {
                category: CapabilityCategory::Custom("monitoring".to_string()),
                operations: vec!["collect_metrics".to_string()],
                min_version: None,
                optional: true,
            },
        ],
    };
    
    // Create manager with custom config
    let manager = create_biomeos_manager_with_config(custom_config).await?;
    
    println!("✅ Custom biomeOS manager created successfully!");
    
    // Show what capabilities are available
    let capabilities = manager.get_available_capabilities().await?;
    println!("📋 Available capabilities with custom config:");
    for (category, primals) in capabilities {
        println!("   {:?}: {} primals", category, primals.len());
    }
    
    Ok(())
}

/// Example of ecosystem monitoring
#[allow(dead_code)]
async fn demo_ecosystem_monitoring() -> BiomeResult<()> {
    println!("📊 Ecosystem monitoring demo...");
    
    let manager = create_biomeos_manager().await?;
    
    // Monitor ecosystem health over time
    for i in 1..=5 {
        println!("📈 Health check #{}", i);
        
        let ecosystem = manager.get_ecosystem_health().await?;
        println!("   Health: {:?}", ecosystem.health);
        println!("   Primals: {}", ecosystem.active_primals);
        
        // Refresh discovery periodically
        if i % 2 == 0 {
            manager.refresh_ecosystem().await?;
            println!("   🔄 Refreshed ecosystem discovery");
        }
        
        // Sleep between checks
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    
    println!("✅ Monitoring demo complete!");
    Ok(())
} 