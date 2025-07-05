//! # biomeOS Init
//!
//! Main system initialization process for biomeOS.
//! This is the first process started by the system and is responsible
//! for bringing up all system components.

use biomeos_system::{BiomeOSSystem, SystemConfig};
use clap::{Arg, Command};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command line arguments
    let matches = Command::new("biomeos-init")
        .version(env!("CARGO_PKG_VERSION"))
        .about("biomeOS system initialization process")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Boot mode (normal, maintenance, recovery)")
                .default_value("normal")
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(clap::ArgAction::SetTrue)
                .help("Enable debug mode")
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config");
    let boot_mode = matches.get_one::<String>("mode").unwrap();
    let debug_mode = matches.get_flag("debug");

    println!("🌱 biomeOS Init v{}", env!("CARGO_PKG_VERSION"));
    println!("==================================");
    println!();

    if debug_mode {
        println!("🐛 Debug mode enabled");
    }

    println!("🚀 Boot mode: {}", boot_mode);
    println!();

    // Load configuration
    let config = if let Some(config_path) = config_path {
        println!("📄 Loading configuration from: {}", config_path);
        load_config_from_file(config_path).await?
    } else {
        println!("📄 Using default configuration");
        SystemConfig::default()
    };

    // Create biomeOS system
    let system = Arc::new(BiomeOSSystem::new(config)?);

    // Set up signal handlers
    let system_clone = system.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("\n🛑 Received shutdown signal, shutting down...");
        
        if let Err(e) = system_clone.shutdown().await {
            eprintln!("❌ Error during shutdown: {}", e);
        }
        
        std::process::exit(0);
    });

    // Initialize the system
    println!("🔧 Initializing biomeOS system...");
    if let Err(e) = system.initialize().await {
        eprintln!("❌ Failed to initialize system: {}", e);
        return Err(e.into());
    }
    println!("✅ System initialized");
    println!();

    // Start the system
    println!("▶️  Starting biomeOS system...");
    if let Err(e) = system.start().await {
        eprintln!("❌ Failed to start system: {}", e);
        return Err(e.into());
    }
    println!("✅ System started");
    println!();

    // Get system info
    match system.get_system_info().await {
        Ok(info) => {
            println!("📊 System Information:");
            println!("   Hostname: {}", info.hostname);
            println!("   Version: {}", info.version);
            println!("   Uptime: {}s", info.uptime.as_secs());
            println!("   Phase: {:?}", info.phase);
            println!("   Health: {:?}", info.health.overall_status);
            println!();
        }
        Err(e) => {
            eprintln!("⚠️  Failed to get system info: {}", e);
        }
    }

    println!("🎯 biomeOS is running!");
    println!("   Press Ctrl+C to shutdown");
    println!();

    // Main event loop
    loop {
        // Monitor system health
        match system.get_state().await.health.overall_status {
            biomeos_system::HealthStatus::Critical => {
                eprintln!("🚨 CRITICAL: System health is critical!");
                // TODO: Implement emergency procedures
            }
            biomeos_system::HealthStatus::Warning => {
                println!("⚠️  WARNING: System health is degraded");
            }
            _ => {
                // System is healthy, continue normal operation
            }
        }

        // Sleep for a bit before next check
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

/// Load configuration from file
async fn load_config_from_file(path: &str) -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: SystemConfig = serde_yaml::from_str(&content)?;
    Ok(config)
} 