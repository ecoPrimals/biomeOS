//! # biomeOS System Manager
//!
//! Administrative interface for managing biomeOS systems.
//! Provides command-line and API interfaces for system administration.

use biomeos_system::{BiomeOSSystem, SystemConfig, SystemManager, SystemCommand};
use biomeos_manifest::BiomeManifest;
use clap::{Arg, Command, SubCommand};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command line arguments
    let matches = Command::new("biomeos-manager")
        .version(env!("CARGO_PKG_VERSION"))
        .about("biomeOS system management interface")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .subcommand(
            Command::new("status")
                .about("Show system status")
        )
        .subcommand(
            Command::new("start")
                .about("Start the system")
        )
        .subcommand(
            Command::new("stop")
                .about("Stop the system")
        )
        .subcommand(
            Command::new("restart")
                .about("Restart the system")
        )
        .subcommand(
            Command::new("health")
                .about("Run health check")
        )
        .subcommand(
            Command::new("update")
                .about("Update the system")
        )
        .subcommand(
            Command::new("maintenance")
                .about("Maintenance mode commands")
                .subcommand(
                    Command::new("enter")
                        .about("Enter maintenance mode")
                )
                .subcommand(
                    Command::new("exit")
                        .about("Exit maintenance mode")
                )
        )
        .subcommand(
            Command::new("package")
                .about("Package management")
                .subcommand(
                    Command::new("install")
                        .about("Install a package")
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help("Package name")
                        )
                        .arg(
                            Arg::new("version")
                                .short('v')
                                .long("version")
                                .value_name("VERSION")
                                .help("Package version")
                        )
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove a package")
                        .arg(
                            Arg::new("name")
                                .required(true)
                                .help("Package name")
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List installed packages")
                )
                .subcommand(
                    Command::new("search")
                        .about("Search for packages")
                        .arg(
                            Arg::new("query")
                                .required(true)
                                .help("Search query")
                        )
                )
        )
        .subcommand(
            Command::new("biome")
                .about("Biome management")
                .subcommand(
                    Command::new("deploy")
                        .about("Deploy a biome")
                        .arg(
                            Arg::new("manifest")
                                .required(true)
                                .help("Biome manifest file")
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List deployed biomes")
                )
        )
        .subcommand(
            Command::new("logs")
                .about("View system logs")
                .arg(
                    Arg::new("follow")
                        .short('f')
                        .long("follow")
                        .action(clap::ArgAction::SetTrue)
                        .help("Follow log output")
                )
                .arg(
                    Arg::new("lines")
                        .short('n')
                        .long("lines")
                        .value_name("NUM")
                        .help("Number of lines to show")
                        .default_value("50")
                )
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config");

    // Load configuration
    let config = if let Some(config_path) = config_path {
        load_config_from_file(config_path).await?
    } else {
        SystemConfig::default()
    };

    // Create biomeOS system
    let system = Arc::new(BiomeOSSystem::new(config)?);
    let manager = SystemManager::new(system.clone());

    // Handle subcommands
    match matches.subcommand() {
        Some(("status", _)) => {
            show_status(&system).await?;
        }
        Some(("start", _)) => {
            println!("🚀 Starting biomeOS system...");
            let result = manager.execute_command(SystemCommand::Start).await?;
            println!("✅ {}", result);
        }
        Some(("stop", _)) => {
            println!("🛑 Stopping biomeOS system...");
            let result = manager.execute_command(SystemCommand::Stop).await?;
            println!("✅ {}", result);
        }
        Some(("restart", _)) => {
            println!("🔄 Restarting biomeOS system...");
            let result = manager.execute_command(SystemCommand::Restart).await?;
            println!("✅ {}", result);
        }
        Some(("health", _)) => {
            println!("🏥 Running health check...");
            let result = manager.execute_command(SystemCommand::HealthCheck).await?;
            println!("✅ {}", result);
        }
        Some(("update", _)) => {
            println!("📦 Updating system...");
            let result = manager.execute_command(SystemCommand::Update).await?;
            println!("✅ {}", result);
        }
        Some(("maintenance", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("enter", _)) => {
                    println!("🔧 Entering maintenance mode...");
                    let result = manager.execute_command(SystemCommand::EnterMaintenance).await?;
                    println!("✅ {}", result);
                }
                Some(("exit", _)) => {
                    println!("▶️  Exiting maintenance mode...");
                    let result = manager.execute_command(SystemCommand::ExitMaintenance).await?;
                    println!("✅ {}", result);
                }
                _ => {
                    eprintln!("❌ Unknown maintenance subcommand");
                    std::process::exit(1);
                }
            }
        }
        Some(("package", sub_matches)) => {
            handle_package_commands(&manager, sub_matches).await?;
        }
        Some(("biome", sub_matches)) => {
            handle_biome_commands(&manager, sub_matches).await?;
        }
        Some(("logs", sub_matches)) => {
            handle_logs_command(sub_matches).await?;
        }
        _ => {
            // No subcommand, show status
            show_status(&system).await?;
        }
    }

    Ok(())
}

/// Show system status
async fn show_status(system: &BiomeOSSystem) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 biomeOS System Status");
    println!("========================");
    println!();

    match system.get_system_info().await {
        Ok(info) => {
            println!("📊 System Information:");
            println!("   Hostname: {}", info.hostname);
            println!("   Version: {}", info.version);
            println!("   Uptime: {}s", info.uptime.as_secs());
            println!("   Phase: {:?}", info.phase);
            println!("   Health: {:?}", info.health.overall_status);
            println!();

            // Show resource usage
            let resources = &info.health.resource_usage;
            println!("💻 Resource Usage:");
            println!("   CPU: {:.1}%", resources.cpu_percent);
            println!("   Memory: {} / {} GB", 
                resources.memory_usage_bytes / (1024 * 1024 * 1024),
                resources.memory_total_bytes / (1024 * 1024 * 1024)
            );
            println!("   Disk: {} / {} GB", 
                resources.disk_usage_bytes / (1024 * 1024 * 1024),
                resources.disk_total_bytes / (1024 * 1024 * 1024)
            );
            println!("   Load: {:.2}, {:.2}, {:.2}", 
                resources.load_averages.0,
                resources.load_averages.1,
                resources.load_averages.2
            );
            println!();

            // Show alerts
            if !info.health.alerts.is_empty() {
                println!("🚨 Active Alerts:");
                for alert in &info.health.alerts {
                    println!("   [{:?}] {}: {}", alert.level, alert.source, alert.message);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get system info: {}", e);
        }
    }

    Ok(())
}

/// Handle package management commands
async fn handle_package_commands(
    manager: &SystemManager,
    sub_matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match sub_matches.subcommand() {
        Some(("install", args)) => {
            let name = args.get_one::<String>("name").unwrap();
            let version = args.get_one::<String>("version").cloned();
            
            println!("📦 Installing package: {}", name);
            let result = manager.execute_command(SystemCommand::InstallPackage {
                name: name.clone(),
                version,
            }).await?;
            println!("✅ {}", result);
        }
        Some(("remove", args)) => {
            let name = args.get_one::<String>("name").unwrap();
            
            println!("🗑️  Removing package: {}", name);
            let result = manager.execute_command(SystemCommand::RemovePackage {
                name: name.clone(),
            }).await?;
            println!("✅ {}", result);
        }
        Some(("list", _)) => {
            println!("📦 Installed Packages:");
            let packages = manager.system.packages.get_installed_packages().await;
            for (name, package) in packages {
                println!("   {} ({})", name, package.version);
            }
        }
        Some(("search", args)) => {
            let query = args.get_one::<String>("query").unwrap();
            
            println!("🔍 Searching for: {}", query);
            let packages = manager.system.packages.search_packages(query).await;
            for package in packages {
                println!("   {} ({}) - {}", package.name, package.version, package.description);
            }
        }
        _ => {
            eprintln!("❌ Unknown package subcommand");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle biome management commands
async fn handle_biome_commands(
    manager: &SystemManager,
    sub_matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match sub_matches.subcommand() {
        Some(("deploy", args)) => {
            let manifest_path = args.get_one::<String>("manifest").unwrap();
            
            println!("🌱 Deploying biome from: {}", manifest_path);
            
            // Load manifest
            let manifest_content = tokio::fs::read_to_string(manifest_path).await?;
            let manifest: BiomeManifest = serde_yaml::from_str(&manifest_content)?;
            
            let result = manager.execute_command(SystemCommand::DeployBiome { manifest }).await?;
            println!("✅ {}", result);
        }
        Some(("list", _)) => {
            println!("🌱 Deployed Biomes:");
            // TODO: Implement biome listing
            println!("   (No biomes currently deployed)");
        }
        _ => {
            eprintln!("❌ Unknown biome subcommand");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle logs command
async fn handle_logs_command(sub_matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let follow = sub_matches.get_flag("follow");
    let lines = sub_matches.get_one::<String>("lines").unwrap().parse::<usize>()?;

    println!("📋 System Logs (last {} lines):", lines);
    
    // TODO: Implement log viewing
    if follow {
        println!("📋 Following logs... (Press Ctrl+C to exit)");
        // TODO: Implement log following
    }

    Ok(())
}

/// Load configuration from file
async fn load_config_from_file(path: &str) -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    let config: SystemConfig = serde_yaml::from_str(&content)?;
    Ok(config)
} 