use anyhow::Result;
use biomeos_cli::{tui::BiomeOSDashboard, CliError, CliUtils};
use biomeos_core::universal_biomeos_manager::{DiscoveryResult, ProbeResult};
use biomeos_core::{SystemHealth, UniversalBiomeOSManager};
use biomeos_primal_sdk::PrimalCapability;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "biomeos")]
#[command(about = "🌱 BiomeOS Universal System Management CLI")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[arg(long, default_value = "info")]
    log_level: String,

    #[arg(long, value_enum, default_value = "pretty")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover services by capability or method
    Discover {
        /// Discovery endpoint to query
        #[arg(short, long)]
        endpoint: Option<String>,

        /// Required capabilities (comma-separated: domain/name,domain2/name2)
        #[arg(short, long)]
        capabilities: Option<String>,

        /// Discovery method to use
        #[arg(short, long, value_enum, default_value = "capability-based")]
        method: DiscoveryMethod,

        /// Registry URL for registry-based discovery
        #[arg(short, long)]
        registry: Option<String>,

        /// Show detailed service information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Monitor system and service health
    Health {
        /// Specific endpoint to check
        #[arg(short, long)]
        endpoint: Option<String>,

        /// Check overall system health
        #[arg(short, long)]
        system: bool,

        /// Include resource usage metrics
        #[arg(short, long)]
        resources: bool,

        /// Watch mode with refresh interval (seconds)
        #[arg(short, long)]
        watch: Option<u64>,
    },
    /// Real-time ecosystem monitoring
    Monitor {
        /// Update interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Services to monitor (comma-separated endpoints)
        #[arg(short, long)]
        services: Option<String>,

        /// Show detailed metrics
        #[arg(short, long)]
        detailed: bool,
    },
    /// Interactive TUI dashboard for real-time monitoring
    Dashboard {
        /// Update interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Auto-refresh data
        #[arg(short, long)]
        refresh: bool,
    },
    /// Probe specific service endpoints
    Probe {
        /// Service endpoint to probe
        endpoint: String,

        /// Show metadata information
        #[arg(short, long)]
        metadata: bool,

        /// Test specific capabilities
        #[arg(short, long)]
        capabilities: Option<String>,
    },
    /// Network scan for BiomeOS services
    Scan {
        /// Network range to scan (CIDR notation)
        #[arg(short, long)]
        range: Option<String>,

        /// Port range to scan
        #[arg(short, long)]
        ports: Option<String>,

        /// Scan timeout in seconds
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
    /// Show system and ecosystem status
    Status {
        /// Show detailed system information
        #[arg(short, long)]
        detailed: bool,

        /// Include service discovery status
        #[arg(short, long)]
        services: bool,

        /// Show resource usage
        #[arg(short, long)]
        resources: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum DiscoveryMethod {
    CapabilityBased,
    NetworkScan,
    Registry,
    ServiceDiscovery,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
    Yaml,
    Table,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    CliUtils::init_logging(&cli.log_level)?;
    CliUtils::print_header("🌱 BiomeOS Universal System Management");

    match cli.command {
        Commands::Discover {
            endpoint,
            capabilities,
            method,
            registry,
            detailed,
        } => {
            handle_discover(
                endpoint,
                capabilities,
                method,
                registry,
                detailed,
                &cli.output,
            )
            .await?;
        }
        Commands::Health {
            endpoint,
            system,
            resources,
            watch,
        } => {
            handle_health(endpoint, system, resources, watch, &cli.output).await?;
        }
        Commands::Monitor {
            interval,
            services,
            detailed,
        } => {
            handle_monitor(interval, services, detailed, &cli.output).await?;
        }
        Commands::Dashboard { interval, refresh } => {
            handle_dashboard(interval, refresh).await?;
        }
        Commands::Probe {
            endpoint,
            metadata,
            capabilities,
        } => {
            handle_probe(endpoint, metadata, capabilities, &cli.output).await?;
        }
        Commands::Scan {
            range,
            ports,
            timeout,
        } => {
            handle_scan(range, ports, timeout, &cli.output).await?;
        }
        Commands::Status {
            detailed,
            services,
            resources,
        } => {
            handle_status(detailed, services, resources, &cli.output).await?;
        }
    }

    Ok(())
}

async fn handle_discover(
    endpoint: Option<String>,
    capabilities: Option<String>,
    method: DiscoveryMethod,
    registry: Option<String>,
    detailed: bool,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();
    let spinner = create_spinner("🔍 Discovering services...");

    let results = match method {
        DiscoveryMethod::CapabilityBased => {
            let endpoint = endpoint.unwrap_or_else(|| "http://discovery:8080".to_string());
            let caps = parse_capabilities(&capabilities.unwrap_or_default())?;
            manager.discover_by_capability(&caps).await?
        }
        DiscoveryMethod::NetworkScan => manager.discover_network_scan().await?,
        DiscoveryMethod::Registry => {
            let registry_url = registry.unwrap_or_else(|| "http://registry:8080".to_string());
            manager.discover_registry(&registry_url).await?
        }
        DiscoveryMethod::ServiceDiscovery => {
            let endpoint = endpoint.unwrap_or_else(|| "http://discovery:8080".to_string());
            manager.discover_registry(&endpoint).await?
        }
    };

    spinner.finish_with_message("✅ Discovery completed");

    display_discovery_results(&results, detailed, output).await?;

    Ok(())
}

async fn handle_health(
    endpoint: Option<String>,
    system: bool,
    resources: bool,
    watch: Option<u64>,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();

    if let Some(endpoint) = endpoint {
        // Single endpoint health check
        perform_single_health_check(&manager, &endpoint, output).await?;
    } else if system {
        // System health check
        if let Some(watch_interval) = watch {
            // Watch mode
            health_watch_loop(watch_interval, resources, output).await?;
        } else {
            // Single check
            let health = manager.get_system_health().await;
            display_system_health(&health, resources, output).await?;
        }
    } else {
        CliUtils::print_error("Please specify either --endpoint or --system");
        return Ok(());
    }

    Ok(())
}

async fn handle_monitor(
    interval: u64,
    services: Option<String>,
    detailed: bool,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();

    CliUtils::print_info(&format!(
        "🔄 Starting monitoring with {}-second intervals",
        interval
    ));
    CliUtils::print_info("Press Ctrl+C to stop");

    monitor_loop(manager, interval, services, detailed, output).await?;

    Ok(())
}

async fn handle_dashboard(interval: u64, _refresh: bool) -> Result<()> {
    let manager = CliUtils::init_manager();

    CliUtils::print_info("🚀 Starting BiomeOS Interactive Dashboard");
    CliUtils::print_info("Use Tab/Shift+Tab to navigate tabs, Q to quit");

    // Create and run the TUI dashboard
    let mut dashboard = BiomeOSDashboard::new(manager);
    dashboard.run().await?;

    CliUtils::print_info("👋 Dashboard closed");

    Ok(())
}

async fn handle_probe(
    endpoint: String,
    metadata: bool,
    _capabilities: Option<String>,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();
    let spinner = create_spinner(&format!("🔍 Probing endpoint: {}", endpoint));

    let result = manager.probe_endpoint(&endpoint).await?;

    spinner.finish_with_message("✅ Probe completed");

    display_probe_result(&result, metadata, output).await?;

    Ok(())
}

async fn handle_scan(
    _range: Option<String>,
    _ports: Option<String>,
    _timeout: u64,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();

    CliUtils::print_info("🌐 Scanning network for BiomeOS services");

    let spinner = create_spinner("🔍 Scanning for BiomeOS services...");
    let results = manager.discover_network_scan().await?;
    spinner.finish_and_clear();
    println!("✅ Found {} services", results.len());

    display_discovery_results(&results, true, output).await?;

    Ok(())
}

async fn handle_status(
    detailed: bool,
    services: bool,
    resources: bool,
    output: &OutputFormat,
) -> Result<()> {
    let manager = CliUtils::init_manager();

    // System health
    let system_health = manager.get_system_health().await;
    display_system_health(&system_health, resources, output).await?;

    if services {
        // Service discovery status
        CliUtils::print_section("Service Discovery Status");
        let spinner = create_spinner("🔍 Discovering services...");
        let services = manager.discover_network_scan().await?;
        spinner.finish_and_clear();
        println!("✅ Discovered {} services", services.len());

        display_discovery_results(&services, detailed, output).await?;
    }

    Ok(())
}

// Helper functions

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

fn parse_capabilities(caps_str: &str) -> Result<Vec<PrimalCapability>> {
    if caps_str.is_empty() {
        return Ok(Vec::new());
    }

    let mut capabilities = Vec::new();
    for cap in caps_str.split(',') {
        let cap = cap.trim();
        if cap.contains('/') {
            let parts: Vec<&str> = cap.splitn(3, '/').collect();
            if parts.len() >= 2 {
                let domain = parts[0];
                let name_version: Vec<&str> = parts[1].splitn(2, ':').collect();
                let name = name_version[0];
                let version = name_version.get(1).unwrap_or(&"v1");
                capabilities.push(PrimalCapability::new(domain, name, version));
            }
        } else {
            capabilities.push(PrimalCapability::custom(cap, "CLI capability"));
        }
    }

    Ok(capabilities)
}

async fn display_discovery_results(
    results: &[DiscoveryResult],
    detailed: bool,
    output: &OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results)?);
        }
        OutputFormat::Yaml => {
            // YAML output would go here
            CliUtils::print_warning("YAML output not yet implemented, showing JSON");
            println!("{}", serde_json::to_string_pretty(results)?);
        }
        OutputFormat::Table | OutputFormat::Pretty => {
            if results.is_empty() {
                CliUtils::print_warning("No services discovered");
                return Ok(());
            }

            let mut table = CliUtils::create_table();
            table.set_header(vec![
                "Service ID",
                "Type",
                "Category",
                "Endpoint",
                "Health",
                "Capabilities",
            ]);

            for result in results {
                let capabilities_str = if detailed {
                    CliUtils::format_capabilities(&result.capabilities)
                } else {
                    format!("{} caps", result.capabilities.len())
                };

                table.add_row(vec![
                    result.id.clone(),
                    result.primal_type.name.clone(),
                    result.primal_type.category.clone(),
                    result.endpoint.clone(),
                    format!("{:?}", result.health),
                    capabilities_str,
                ]);
            }

            println!("\n{}", table);
            CliUtils::print_info(&format!("Found {} services", results.len()));
        }
    }

    Ok(())
}

async fn perform_single_health_check(
    manager: &UniversalBiomeOSManager,
    endpoint: &str,
    output: &OutputFormat,
) -> Result<()> {
    let spinner = create_spinner(&format!("🏥 Checking health: {}", endpoint));
    let result = manager.probe_endpoint(endpoint).await?;
    spinner.finish_with_message("✅ Health check completed");

    display_probe_result(&result, true, output).await?;

    Ok(())
}

async fn health_watch_loop(
    watch_interval: u64,
    resources: bool,
    output: &OutputFormat,
) -> Result<()> {
    CliUtils::print_info(&format!(
        "👀 Watching system health ({}s intervals)",
        watch_interval
    ));
    CliUtils::print_info("Press Ctrl+C to stop");

    let mut interval = tokio::time::interval(Duration::from_secs(watch_interval));

    loop {
        interval.tick().await;

        // Clear screen for watch mode
        print!("\x1B[2J\x1B[1;1H");

        CliUtils::print_header(&format!(
            "🌱 BiomeOS System Health - {}",
            chrono::Utc::now().format("%H:%M:%S UTC")
        ));

        // Create a new manager instance for each loop to avoid borrow issues
        let manager = CliUtils::init_manager();
        let health = manager.get_system_health().await;
        display_system_health(&health, resources, output).await?;

        CliUtils::print_info(&format!(
            "Next update in {}s (Ctrl+C to stop)",
            watch_interval
        ));
    }
}

async fn monitor_loop(
    manager: UniversalBiomeOSManager,
    interval: u64,
    _services: Option<String>,
    detailed: bool,
    output: &OutputFormat,
) -> Result<()> {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));

    loop {
        interval_timer.tick().await;

        // Clear screen for monitor mode
        print!("\x1B[2J\x1B[1;1H");

        CliUtils::print_header(&format!(
            "🔄 BiomeOS Ecosystem Monitor - {}",
            chrono::Utc::now().format("%H:%M:%S UTC")
        ));

        // System health
        let system_health = manager.get_system_health().await;
        display_system_health(&system_health, true, output).await?;

        // Service discovery
        CliUtils::print_section("Active Services");
        let services = manager.discover_network_scan().await?;
        display_discovery_results(&services, detailed, output).await?;

        CliUtils::print_info(&format!("Next update in {}s (Ctrl+C to stop)", interval));
    }
}

async fn display_system_health(
    health: &SystemHealth,
    show_resources: bool,
    output: &OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(health)?);
        }
        OutputFormat::Yaml => {
            CliUtils::print_warning("YAML output not yet implemented, showing JSON");
            println!("{}", serde_json::to_string_pretty(health)?);
        }
        OutputFormat::Table | OutputFormat::Pretty => {
            CliUtils::print_section("System Health");

            println!(
                "🎯 Overall Status: {}",
                CliUtils::format_system_health_status(health)
            );
            println!(
                "⏱️  Uptime: {}",
                CliUtils::format_duration(health.uptime.num_seconds() as u64)
            );

            if show_resources {
                CliUtils::print_section("Resource Usage");

                let cpu_color = if health.resource_usage.cpu_usage_percent > 80.0 {
                    "red"
                } else if health.resource_usage.cpu_usage_percent > 60.0 {
                    "yellow"
                } else {
                    "green"
                };
                let memory_color = if health.resource_usage.memory_usage_percent > 80.0 {
                    "red"
                } else if health.resource_usage.memory_usage_percent > 60.0 {
                    "yellow"
                } else {
                    "green"
                };
                let disk_color = if health.resource_usage.disk_usage_percent > 80.0 {
                    "red"
                } else if health.resource_usage.disk_usage_percent > 60.0 {
                    "yellow"
                } else {
                    "green"
                };

                println!(
                    "💾 CPU Usage: {}",
                    format!("{:.1}%", health.resource_usage.cpu_usage_percent).color(cpu_color)
                );
                println!(
                    "🧠 Memory Usage: {}",
                    format!("{:.1}%", health.resource_usage.memory_usage_percent)
                        .color(memory_color)
                );
                println!(
                    "💿 Disk Usage: {}",
                    format!("{:.1}%", health.resource_usage.disk_usage_percent).color(disk_color)
                );

                if health.resource_usage.network_usage_mbps > 0.0 {
                    println!(
                        "🌐 Network: {:.1} Mbps",
                        health.resource_usage.network_usage_mbps
                    );
                }
            }

            if !health.primal_health.is_empty() {
                CliUtils::print_section("Primal Health");
                for (primal_id, primal_health) in &health.primal_health {
                    println!("  {} {}", format!("{:?}", primal_health), primal_id);
                }
            }
        }
    }

    Ok(())
}

async fn display_probe_result(
    result: &ProbeResult,
    show_metadata: bool,
    output: &OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Yaml => {
            CliUtils::print_warning("YAML output not yet implemented, showing JSON");
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Table | OutputFormat::Pretty => {
            CliUtils::print_section("Probe Results");

            println!("🏷️  Name: {}", result.name);
            println!("📦 Capabilities: {:?}", result.capabilities);

            if show_metadata {
                println!(
                    "⚡ Capabilities: {}",
                    CliUtils::format_capabilities(&result.capabilities)
                );
            } else {
                println!("⚡ Capabilities: {} available", result.capabilities.len());
            }
        }
    }

    Ok(())
}
