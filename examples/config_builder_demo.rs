//! Configuration Builder Demo
//!
//! Demonstrates the flexible BiomeOS configuration system that replaces
//! hardcoded values with deployment-specific and environment-specific settings.

use anyhow::Result;
use biomeos_core::{BiomeOSConfigBuilder, UniversalBiomeOSManager};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛠️ BiomeOS Configuration Builder Demo");
    println!("=====================================");
    println!("Demonstrating flexible, non-hardcoded configuration management\n");

    // Demo 1: Local Development Configuration
    println!("📝 Demo 1: Local Development Configuration");
    let dev_config = BiomeOSConfigBuilder::for_local_development().build();

    println!("  Environment: {:?}", dev_config.system.environment);
    println!(
        "  Discovery methods: {:?}",
        dev_config.discovery.methods
    );
    println!(
        "  Network binding: {}",
        dev_config.network.bind_address
    );
    println!(
        "  Organization scale: {:?}",
        dev_config.system.organization_scale
    );

    let dev_manager = UniversalBiomeOSManager::new(dev_config).await?;
    println!("  ✅ Manager created with development configuration\n");

    // Demo 2: Production Configuration
    println!("🏭 Demo 2: Production Configuration");
    let prod_config = BiomeOSConfigBuilder::for_production().build();

    println!("  Environment: {:?}", prod_config.system.environment);
    println!(
        "  Discovery methods: {:?}",
        prod_config.discovery.methods
    );
    println!(
        "  Organization scale: {:?}",
        prod_config.system.organization_scale
    );
    println!(
        "  Network binding: {}",
        prod_config.network.bind_address
    );

    let prod_manager = UniversalBiomeOSManager::new(prod_config).await?;
    println!("  ✅ Manager created with production configuration\n");

    // Demo 3: Testing Configuration (Fast timeouts, static discovery)
    println!("🧪 Demo 3: Testing Configuration");
    let test_config = BiomeOSConfigBuilder::for_testing().build();

    println!("  Environment: {:?}", test_config.system.environment);
    println!(
        "  Discovery methods: {:?}",
        test_config.discovery.methods
    );
    println!(
        "  Organization scale: {:?}",
        test_config.system.organization_scale
    );

    let test_manager = UniversalBiomeOSManager::new(test_config).await?;
    println!("  ✅ Manager created with testing configuration\n");

    // Demo 4: Registry-based Configuration
    println!("📋 Demo 4: Registry-based Configuration");
    let registry_url = "http://consul.example.com:8500";
    let registry_config = BiomeOSConfigBuilder::new()
        .with_registry_discovery(registry_url, None)
        .build();

    println!("  Registry URL: {}", registry_url);
    println!(
        "  Discovery methods: {:?}",
        registry_config.discovery.methods
    );

    let registry_manager = UniversalBiomeOSManager::new(registry_config).await?;
    println!("  ✅ Manager created with registry configuration\n");

    // Demo 5: Custom Configuration with DNS Discovery
    println!("⚙️ Demo 5: Custom Configuration with DNS Discovery");
    let custom_config = BiomeOSConfigBuilder::new()
        .with_dns_discovery(vec![
            "api-server-1.company.com".to_string(),
            "api-server-2.company.com".to_string(),
        ])
        .build();

    println!(
        "  Discovery methods: {:?}",
        custom_config.discovery.methods
    );
    println!(
        "  DNS servers: {:?}",
        custom_config.discovery.dns.as_ref().map(|d| &d.servers)
    );
    println!(
        "  Environment: {:?}",
        custom_config.system.environment
    );
    println!(
        "  Network binding: {}",
        custom_config.network.bind_address
    );

    let custom_manager = UniversalBiomeOSManager::new(custom_config).await?;
    println!("  ✅ Manager created with custom configuration\n");

    // Demo 6: Show the difference from hardcoded approach
    println!("❌ What we REPLACED (hardcoded approach):");
    println!("  Old: Always localhost:8080, 8081, 8082");
    println!("  Old: Fixed development environment");
    println!("  Old: No environment-specific settings");
    println!("  Old: Hardcoded in Default::default()");

    println!("\n✅ What we PROVIDE NOW (flexible approach):");
    println!("  New: Environment-specific host and port configuration");
    println!("  New: Builder pattern for deployment-specific settings");
    println!("  New: Static endpoint configuration for known services");
    println!("  New: Configurable discovery methods (scan, registry, static)");
    println!("  New: Production-ready security and timeout settings");

    // Demo 7: Show system health with different configurations
    println!("\n🏥 Demo 7: System Health with Different Configurations");

    // Test health with each configuration
    let configs = vec![
        ("Development", dev_manager),
        ("Production", prod_manager),
        ("Testing", test_manager),
        ("Registry", registry_manager),
        ("Custom", custom_manager),
    ];

    for (name, manager) in configs {
        let health = manager.get_system_health().await;
        println!("  {} health: {:?}", name, health.health);
    }

    println!("\n✨ Configuration builder demo completed successfully!");
    println!(
        "   🎯 Demonstrated: Environment-specific configs, flexible discovery, static endpoints"
    );
    println!("   🚫 Eliminated: Hardcoded localhost, fixed ports, inflexible defaults");

    Ok(())
}
