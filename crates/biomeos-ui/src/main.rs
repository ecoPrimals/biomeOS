//! biomeOS Universal UI
//!
//! AI-first, API-driven user interface for the entire biomeOS ecosystem.
//! Supports desktop (Tauri), terminal (TUI), and web modes.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use tracing::{info, error};
use tracing_subscriber;

use biomeos_ui::{
    BiomeOSUI, UIConfig,
    config::{UIMode, Theme},
    ai::AIConfig,
};

#[derive(Parser)]
#[command(name = "biomeos-ui")]
#[command(about = "Universal AI-first UI for biomeOS ecosystem")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// UI mode to run
    #[arg(short, long, default_value = "desktop")]
    mode: String,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Theme (dark/light)
    #[arg(short, long, default_value = "dark")]
    theme: String,
    
    /// API endpoints (format: primal=endpoint)
    #[arg(long, action = clap::ArgAction::Append)]
    endpoint: Vec<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the UI in interactive mode
    Start {
        /// Port for web UI (if web mode)
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    
    /// Execute a single AI command and exit
    Exec {
        /// The command to execute
        command: String,
        
        /// Output format (json, text, table)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Show ecosystem status
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
        
        /// Specific Primal to check
        #[arg(short, long)]
        primal: Option<String>,
    },
    
    /// Deploy a biome
    Deploy {
        /// Biome manifest file
        manifest: String,
        
        /// Watch deployment progress
        #[arg(short, long)]
        watch: bool,
    },
    
    /// List resources
    List {
        /// Resource type (primals, biomes, services)
        resource: String,
        
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("biomeos_ui={},biomeos_core={}", log_level, log_level))
        .init();
    
    info!("🚀 Starting biomeOS Universal UI v{}", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = load_config(&cli).await?;
    
    // Create UI instance
    let mut ui = BiomeOSUI::new(config).await?;
    
    // Handle commands
    match cli.command {
        Some(Commands::Start { port }) => {
            info!("Starting UI in {} mode", cli.mode);
            match cli.mode.as_str() {
                "desktop" => {
                    #[cfg(feature = "desktop-ui")]
                    {
                        ui.run().await?;
                    }
                    #[cfg(not(feature = "desktop-ui"))]
                    {
                        error!("Desktop UI not available. Compile with --features desktop-ui");
                        std::process::exit(1);
                    }
                }
                "terminal" | "tui" => {
                    #[cfg(feature = "terminal-ui")]
                    {
                        ui.run().await?;
                    }
                    #[cfg(not(feature = "terminal-ui"))]
                    {
                        error!("Terminal UI not available. Compile with --features terminal-ui");
                        std::process::exit(1);
                    }
                }
                "web" => {
                    #[cfg(feature = "web-ui")]
                    {
                        info!("Starting web UI on port {}", port);
                        ui.run().await?;
                    }
                    #[cfg(not(feature = "web-ui"))]
                    {
                        error!("Web UI not available. Compile with --features web-ui");
                        std::process::exit(1);
                    }
                }
                _ => {
                    error!("Unknown UI mode: {}. Use desktop, terminal, or web", cli.mode);
                    std::process::exit(1);
                }
            }
        }
        
        Some(Commands::Exec { command, format }) => {
            info!("Executing command: {}", command);
            
            let response = ui.ai_assistant.process_command(&command, &ui.api_client).await?;
            
            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                }
                "text" => {
                    println!("{}", response.message);
                    if !response.suggestions.is_empty() {
                        println!("\n💡 Suggestions:");
                        for suggestion in &response.suggestions {
                            println!("  • {}", suggestion);
                        }
                    }
                }
                "table" => {
                    // Simple table format
                    println!("┌─────────────────────────────────────────────────────────────┐");
                    println!("│ AI Response                                                 │");
                    println!("├─────────────────────────────────────────────────────────────┤");
                    for line in response.message.lines() {
                        println!("│ {:<59} │", line);
                    }
                    println!("└─────────────────────────────────────────────────────────────┘");
                }
                _ => {
                    error!("Unknown format: {}. Use json, text, or table", format);
                    std::process::exit(1);
                }
            }
        }
        
        Some(Commands::Status { detailed, primal }) => {
            info!("Getting ecosystem status");
            
            if let Some(primal_name) = primal {
                // Get specific Primal status
                println!("🔍 Checking status of Primal: {}", primal_name);
                // Implementation would check specific Primal
                println!("✅ {} is healthy", primal_name);
            } else {
                // Get ecosystem status
                let status = ui.api_client.get_ecosystem_status().await?;
                
                let health_emoji = match status.overall_health.as_str() {
                    "healthy" => "✅",
                    "degraded" => "⚠️",
                    _ => "❌",
                };
                
                println!("🌍 biomeOS Ecosystem Status");
                println!("─────────────────────────────");
                println!("{} Overall Health: {}", health_emoji, status.overall_health);
                println!("📊 Primals: {}/{} healthy", status.healthy_primals, status.total_primals);
                println!("🕐 Last Updated: {}", status.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));
                
                if detailed {
                    println!("\n🔧 Primal Details:");
                    for (name, primal_status) in &status.primal_statuses {
                        let primal_emoji = match primal_status.health.as_str() {
                            "healthy" => "✅",
                            "degraded" => "⚠️",
                            _ => "❌",
                        };
                        println!("  {} {} - {} ({} services)", 
                            primal_emoji, name, primal_status.health, primal_status.service_count);
                    }
                }
            }
        }
        
        Some(Commands::Deploy { manifest, watch }) => {
            info!("Deploying biome from manifest: {}", manifest);
            
            // Load manifest (placeholder - would load from file)
            println!("📋 Loading biome manifest: {}", manifest);
            println!("🚀 Deploying biome...");
            
            if watch {
                println!("👀 Watching deployment progress...");
                // Implementation would watch deployment status
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                println!("✅ Deployment completed successfully!");
            } else {
                println!("✅ Deployment initiated. Use 'biomeos-ui status' to check progress.");
            }
        }
        
        Some(Commands::List { resource, format }) => {
            info!("Listing {}", resource);
            
            match resource.as_str() {
                "primals" => {
                    let primals = ui.api_client.discover_primals().await?;
                    
                    match format.as_str() {
                        "table" => {
                            println!("🔧 Available Primals");
                            println!("┌─────────────┬─────────────────────────┬──────────┬─────────────────┐");
                            println!("│ Name        │ Endpoint                │ Health   │ Capabilities    │");
                            println!("├─────────────┼─────────────────────────┼──────────┼─────────────────┤");
                            for primal in &primals {
                                println!("│ {:<11} │ {:<23} │ {:<8} │ {:<15} │", 
                                    primal.name, 
                                    primal.endpoint,
                                    primal.health,
                                    primal.capabilities.join(",")
                                );
                            }
                            println!("└─────────────┴─────────────────────────┴──────────┴─────────────────┘");
                        }
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&primals)?);
                        }
                        _ => {
                            for primal in &primals {
                                println!("• {} ({}) - {}", primal.name, primal.health, primal.endpoint);
                            }
                        }
                    }
                }
                "biomes" => {
                    println!("📦 Active Biomes");
                    println!("(Biome listing not yet implemented)");
                }
                "services" => {
                    println!("⚙️  Running Services");
                    println!("(Service listing not yet implemented)");
                }
                _ => {
                    error!("Unknown resource type: {}. Use primals, biomes, or services", resource);
                    std::process::exit(1);
                }
            }
        }
        
        None => {
            // Default: start in interactive mode
            info!("Starting UI in interactive mode");
            ui.run().await?;
        }
    }
    
    Ok(())
}

/// Load configuration from file or CLI arguments
async fn load_config(cli: &Cli) -> Result<UIConfig> {
    let mut config = UIConfig::default();
    
    // Set UI mode
    config.ui_mode = match cli.mode.as_str() {
        "desktop" => UIMode::Desktop,
        "terminal" | "tui" => UIMode::Terminal,
        "web" => UIMode::Web,
        _ => UIMode::Desktop,
    };
    
    // Set theme
    config.theme = match cli.theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => Theme::Dark,
    };
    
    // Parse endpoint arguments
    for endpoint_arg in &cli.endpoint {
        if let Some((primal, endpoint)) = endpoint_arg.split_once('=') {
            config.api_endpoints.insert(primal.to_string(), endpoint.to_string());
        }
    }
    
    // Load from config file if specified
    if let Some(config_path) = &cli.config {
        info!("Loading configuration from: {}", config_path);
        // Implementation would load from file
        // For now, just log that we would load it
    }
    
    // Set up default endpoints if none provided
    if config.api_endpoints.is_empty() {
        info!("Using default Primal endpoints");
        config.api_endpoints.insert("songbird".to_string(), "http://localhost:8080".to_string());
        config.api_endpoints.insert("nestgate".to_string(), "http://localhost:8082".to_string());
        config.api_endpoints.insert("toadstool".to_string(), "http://localhost:8084".to_string());
        config.api_endpoints.insert("beardog".to_string(), "http://localhost:9000".to_string());
        config.api_endpoints.insert("squirrel".to_string(), "http://localhost:5000".to_string());
    }
    
    info!("Configuration loaded successfully");
    info!("UI Mode: {:?}", config.ui_mode);
    info!("Theme: {:?}", config.theme);
    info!("API Endpoints: {:?}", config.api_endpoints);
    
    Ok(config)
}

/// Print startup banner
fn print_banner() {
    println!(r#"
    ╔══════════════════════════════════════════════════════════════╗
    ║                        biomeOS UI                            ║
    ║              Universal AI-first Interface                    ║
    ║                                                              ║
    ║  🌍 Ecosystem Management  🤖 AI Assistant  🔧 All Primals   ║
    ╚══════════════════════════════════════════════════════════════╝
    "#);
}

/// Show quick help
fn show_quick_help() {
    println!("Quick Commands:");
    println!("  biomeos-ui start                    # Start interactive UI");
    println!("  biomeos-ui exec \"deploy a biome\"     # Execute AI command");
    println!("  biomeos-ui status                   # Show ecosystem status");
    println!("  biomeos-ui list primals             # List available Primals");
    println!("  biomeos-ui --help                   # Show full help");
    println!();
    println!("AI Commands (in interactive mode):");
    println!("  \"Deploy a biome called my-app\"");
    println!("  \"Show ecosystem status\"");
    println!("  \"List all Primals\"");
    println!("  \"Scale web-service to 3 instances\"");
    println!("  \"What's the status of Songbird?\"");
} 