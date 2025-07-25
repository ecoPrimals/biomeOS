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
        "  Discovery hosts: {:?}",
        dev_config.primals.discovery.scan_hosts
    );
    println!(
        "  Discovery ports: {:?}",
        dev_config.primals.discovery.scan_ports
    );
    println!(
        "  Auto-discovery: {}",
        dev_config.primals.discovery.auto_discovery
    );

    let dev_manager = UniversalBiomeOSManager::new(dev_config);
    dev_manager.initialize().await?;
    println!("  ✅ Manager created with development configuration\n");

    // Demo 2: Production Configuration with Custom Hosts/Ports
    println!("🏭 Demo 2: Production Configuration");
    let prod_hosts = vec![
        "10.0.1.10".to_string(),
        "10.0.1.11".to_string(),
        "10.0.1.12".to_string(),
    ];
    let prod_ports = vec![9090, 9091, 9092];

    let prod_config =
        BiomeOSConfigBuilder::for_production(prod_hosts.clone(), prod_ports.clone()).build();

    println!("  Environment: {:?}", prod_config.system.environment);
    println!(
        "  Discovery hosts: {:?}",
        prod_config.primals.discovery.scan_hosts
    );
    println!(
        "  Discovery ports: {:?}",
        prod_config.primals.discovery.scan_ports
    );

    let prod_manager = UniversalBiomeOSManager::new(prod_config);
    prod_manager.initialize().await?;
    println!("  ✅ Manager created with production configuration\n");

    // Demo 3: Testing Configuration (Fast timeouts, static discovery)
    println!("🧪 Demo 3: Testing Configuration");
    let test_config = BiomeOSConfigBuilder::for_testing().build();

    println!("  Environment: {:?}", test_config.system.environment);
    println!(
        "  Discovery method: {:?}",
        test_config.primals.discovery.method
    );
    println!(
        "  Auto-discovery: {}",
        test_config.primals.discovery.auto_discovery
    );

    let test_manager = UniversalBiomeOSManager::new(test_config);
    test_manager.initialize().await?;
    println!("  ✅ Manager created with testing configuration\n");

    // Demo 4: Registry-based Configuration
    println!("📋 Demo 4: Registry-based Configuration");
    let registry_url = "http://consul.example.com:8500".to_string();
    let registry_config = BiomeOSConfigBuilder::with_registry(registry_url.clone()).build();

    println!("  Registry URL: {}", registry_url);
    println!(
        "  Discovery method: {:?}",
        registry_config.primals.discovery.method
    );

    let registry_manager = UniversalBiomeOSManager::new(registry_config);
    registry_manager.initialize().await?;
    println!("  ✅ Manager created with registry configuration\n");

    // Demo 5: Custom Configuration with Static Endpoints
    println!("⚙️ Demo 5: Custom Configuration with Static Endpoints");
    let custom_config = BiomeOSConfigBuilder::new()
        .with_discovery_hosts(vec![
            "api-server-1.company.com".to_string(),
            "api-server-2.company.com".to_string(),
        ])
        .with_discovery_ports(vec![8443, 9443])
        .with_static_endpoint(
            "toadstool".to_string(),
            "https://compute.company.com:8080".to_string(),
        )
        .with_static_endpoint(
            "songbird".to_string(),
            "https://mesh.company.com:3000".to_string(),
        )
        .with_static_endpoint(
            "nestgate".to_string(),
            "https://storage.company.com:8082".to_string(),
        )
        .with_auto_discovery(false) // Rely only on static endpoints
        .with_crypto_locks(true)
        .build();

    println!(
        "  Discovery hosts: {:?}",
        custom_config.primals.discovery.scan_hosts
    );
    println!(
        "  Discovery ports: {:?}",
        custom_config.primals.discovery.scan_ports
    );
    println!(
        "  Static endpoints: {:?}",
        custom_config.primals.discovery.static_endpoints
    );
    println!(
        "  Auto-discovery: {}",
        custom_config.primals.discovery.auto_discovery
    );
    println!(
        "  Crypto locks enabled: {}",
        custom_config.security.enable_crypto_locks
    );

    let custom_manager = UniversalBiomeOSManager::new(custom_config);
    custom_manager.initialize().await?;
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
        println!("  {} health: {:?}", name, health.overall_status);
    }

    println!("\n✨ Configuration builder demo completed successfully!");
    println!(
        "   🎯 Demonstrated: Environment-specific configs, flexible discovery, static endpoints"
    );
    println!("   🚫 Eliminated: Hardcoded localhost, fixed ports, inflexible defaults");

    Ok(())
}
