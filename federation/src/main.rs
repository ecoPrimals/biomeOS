//! BiomeOS Federation CLI
//!
//! Command-line interface for deploying and managing federation BYOB manifests
//! Pure Rust implementation for self-contained federation deployment

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "biome-federation")]
#[command(about = "BiomeOS Federation CLI for basement tower deployment")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file
    #[arg(short, long, default_value = "/etc/ecoprimal/federation.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy federation manifests
    Deploy {
        /// Manifest file or template name
        manifest: String,

        /// Dry run (validate only)
        #[arg(long)]
        dry_run: bool,

        /// Force deployment
        #[arg(long)]
        force: bool,
    },

    /// List available manifests
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Show deployment status
    Status {
        /// Deployment name
        deployment: Option<String>,

        /// Watch for changes
        #[arg(long)]
        watch: bool,
    },

    /// Show deployment logs
    Logs {
        /// Deployment name
        deployment: String,

        /// Follow logs
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(long, default_value = "100")]
        lines: u32,
    },

    /// Remove deployment
    Remove {
        /// Deployment name
        deployment: String,

        /// Force removal
        #[arg(long)]
        force: bool,
    },

    /// Initialize federation
    Init {
        /// Tower name
        #[arg(short, long)]
        tower_name: String,

        /// Tower location
        #[arg(short, long)]
        location: String,

        /// Tower capabilities
        #[arg(long)]
        capabilities: Vec<String>,
    },

    /// Join existing federation
    Join {
        /// Bootstrap node address
        bootstrap: String,

        /// Security key
        #[arg(long)]
        key: Option<String>,
    },

    /// Show federation topology
    Topology {
        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Run demo scenarios
    Demo {
        /// Scenario name
        scenario: String,

        /// Scenario parameters
        #[arg(long)]
        params: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct FederationConfig {
    tower: TowerConfig,
    federation: FederationSettings,
    manifests: ManifestSettings,
}

#[derive(Serialize, Deserialize, Debug)]
struct TowerConfig {
    name: String,
    location: String,
    capabilities: Vec<String>,
    resources: ResourceConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResourceConfig {
    cpu_cores: u32,
    memory_gb: u32,
    storage_gb: u32,
    gpu_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct FederationSettings {
    enabled: bool,
    bootstrap_nodes: Vec<String>,
    security_level: String,
    discovery_protocols: Vec<String>,
    config_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct ManifestSettings {
    manifest_dir: PathBuf,
    template_dir: PathBuf,
    deployment_dir: PathBuf,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            tower: TowerConfig {
                name: "basement-tower-1".to_string(),
                location: "basement-rack-1".to_string(),
                capabilities: vec![
                    "coordination".to_string(),
                    "compute".to_string(),
                    "storage".to_string(),
                ],
                resources: ResourceConfig {
                    cpu_cores: 32,
                    memory_gb: 128,
                    storage_gb: 1000,
                    gpu_count: 2,
                },
            },
            federation: FederationSettings {
                enabled: true,
                bootstrap_nodes: vec![],
                security_level: "enhanced".to_string(),
                discovery_protocols: vec![
                    "mdns".to_string(),
                    "upnp".to_string(),
                    "beardog".to_string(),
                ],
                config_dir: PathBuf::from("/etc/ecoprimal/federation"),
            },
            manifests: ManifestSettings {
                manifest_dir: PathBuf::from("/opt/ecoprimal/manifests"),
                template_dir: PathBuf::from("/opt/ecoprimal/templates"),
                deployment_dir: PathBuf::from("/var/lib/ecoprimal/deployments"),
            },
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    // Load configuration
    let config = load_config(&cli.config)?;

    // Execute command
    match cli.command {
        Commands::Deploy {
            manifest,
            dry_run,
            force,
        } => {
            deploy_manifest(&config, &manifest, dry_run, force)?;
        }
        Commands::List { detailed } => {
            list_manifests(&config, detailed)?;
        }
        Commands::Status { deployment, watch } => {
            show_status(&config, deployment, watch)?;
        }
        Commands::Logs {
            deployment,
            follow,
            lines,
        } => {
            show_logs(&config, &deployment, follow, lines)?;
        }
        Commands::Remove { deployment, force } => {
            remove_deployment(&config, &deployment, force)?;
        }
        Commands::Init {
            tower_name,
            location,
            capabilities,
        } => {
            init_federation(&config, &tower_name, &location, &capabilities)?;
        }
        Commands::Join { bootstrap, key } => {
            join_federation(&config, &bootstrap, key)?;
        }
        Commands::Topology { format } => {
            show_topology(&config, &format)?;
        }
        Commands::Demo { scenario, params } => {
            run_demo(&config, &scenario, &params)?;
        }
    }

    Ok(())
}

fn load_config(config_path: &PathBuf) -> Result<FederationConfig> {
    if config_path.exists() {
        let content =
            fs::read_to_string(config_path).context("Failed to read configuration file")?;
        toml::from_str(&content).context("Failed to parse configuration file")
    } else {
        info!("Configuration file not found, using defaults");
        Ok(FederationConfig::default())
    }
}

fn deploy_manifest(
    config: &FederationConfig,
    manifest: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    info!("Deploying manifest: {}", manifest);

    // Find manifest file
    let manifest_path = find_manifest(config, manifest)?;

    if dry_run {
        info!("Dry run: Validating manifest {}", manifest_path.display());
        validate_manifest(&manifest_path)?;
        println!("✅ Manifest validation successful");
        return Ok(());
    }

    // Deploy using biome CLI
    let mut cmd = Command::new("biome");
    cmd.arg("deploy").arg(&manifest_path);

    if force {
        cmd.arg("--force");
    }

    let output = cmd.output().context("Failed to execute biome deploy")?;

    if output.status.success() {
        println!("✅ Deployment successful");
        info!(
            "Deployment output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        error!(
            "Deployment failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(anyhow::anyhow!("Deployment failed"));
    }

    Ok(())
}

fn find_manifest(config: &FederationConfig, manifest: &str) -> Result<PathBuf> {
    // Check if it's a direct path
    let path = PathBuf::from(manifest);
    if path.exists() {
        return Ok(path);
    }

    // Check manifest directory
    let manifest_path = config
        .manifests
        .manifest_dir
        .join(format!("{}.yaml", manifest));
    if manifest_path.exists() {
        return Ok(manifest_path);
    }

    // Check template directory
    let template_path = config
        .manifests
        .template_dir
        .join(format!("{}.yaml", manifest));
    if template_path.exists() {
        return Ok(template_path);
    }

    Err(anyhow::anyhow!("Manifest not found: {}", manifest))
}

fn validate_manifest(manifest_path: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&manifest_path)?;
    let _: serde_yaml::Value = serde_yaml::from_str(&content).context("Invalid YAML syntax")?;

    // Basic schema validation
    let manifest: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Check required fields
    let required_fields = ["apiVersion", "kind", "metadata"];
    for field in required_fields {
        if !manifest.get(field).is_some() {
            return Err(anyhow::anyhow!("Missing required field: {}", field));
        }
    }

    // Validate apiVersion
    if let Some(api_version) = manifest["apiVersion"].as_str() {
        if !api_version.starts_with("biomeOS/v") {
            return Err(anyhow::anyhow!("Invalid apiVersion: {}", api_version));
        }
    }

    // Validate kind
    if let Some(kind) = manifest["kind"].as_str() {
        let valid_kinds = ["Biome", "Federation", "Service", "Config"];
        if !valid_kinds.contains(&kind) {
            return Err(anyhow::anyhow!(
                "Invalid kind: {}. Must be one of: {:?}",
                kind,
                valid_kinds
            ));
        }
    }

    // Validate metadata
    if let Some(metadata) = manifest["metadata"].as_mapping() {
        if !metadata.contains_key(&serde_yaml::Value::String("name".to_string())) {
            return Err(anyhow::anyhow!("metadata.name is required"));
        }
    }

    println!("✅ Schema validation passed");

    Ok(())
}

fn list_manifests(config: &FederationConfig, detailed: bool) -> Result<()> {
    println!("📋 Available Federation Manifests:");
    println!("==================================");

    // List manifests from manifest directory
    if config.manifests.manifest_dir.exists() {
        list_manifests_in_dir(&config.manifests.manifest_dir, "Deployed", detailed)?;
    }

    // List templates from template directory
    if config.manifests.template_dir.exists() {
        list_manifests_in_dir(&config.manifests.template_dir, "Templates", detailed)?;
    }

    Ok(())
}

fn list_manifests_in_dir(dir: &PathBuf, category: &str, detailed: bool) -> Result<()> {
    println!("\n{} Manifests:", category);
    println!("{}", "-".repeat(category.len() + 11));

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "yaml") {
            let name = path.file_stem().unwrap().to_string_lossy();
            println!("  • {}", name);

            if detailed {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(manifest) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        if let Some(description) = manifest.get("description") {
                            println!("    Description: {}", description.as_str().unwrap_or("N/A"));
                        }
                        if let Some(version) = manifest.get("version") {
                            println!("    Version: {}", version.as_str().unwrap_or("N/A"));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn show_status(config: &FederationConfig, deployment: Option<String>, watch: bool) -> Result<()> {
    match deployment {
        Some(name) => {
            if watch {
                info!("Watching deployment: {}", name);
                // Implement watch functionality with real-time monitoring
                watch_deployment_status(config, &name)?;
            } else {
                show_deployment_status(config, &name)?;
            }
        }
        None => {
            if watch {
                info!("Watching federation status");
                watch_federation_status(config)?;
            } else {
            show_federation_status(config)?;
            }
        }
    }

    Ok(())
}

fn watch_deployment_status(config: &FederationConfig, deployment: &str) -> Result<()> {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use std::time::Duration;
    
    println!("👁️  Watching deployment: {}", deployment);
    println!("Press Ctrl+C to stop watching...\n");

    // Setup graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\n🛑 Stopping watch...");
        r.store(false, Ordering::SeqCst);
    }).context("Failed to set Ctrl+C handler")?;

    let mut iteration = 0;
    let mut last_status = String::new();
    
    while running.load(Ordering::SeqCst) {
        // Clear screen and show current time
        if iteration > 0 {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
        }
        
        println!("📊 Live Deployment Status: {} ({})", deployment, chrono::Utc::now().format("%H:%M:%S UTC"));
        println!("==================================");
        
        // Get current status
        let current_status = get_deployment_status_string(config, deployment)?;
        
        // Show status
        println!("{}", current_status);
        
        // Show change indicator
        if iteration > 0 && current_status != last_status {
            println!("🔄 Status changed at {}", chrono::Utc::now().format("%H:%M:%S UTC"));
        }
        
        last_status = current_status;
        
        println!("\n⏱️  Next update in 5 seconds... (Ctrl+C to stop)");
        
        // Wait 5 seconds or until shutdown
        for _ in 0..50 {  // 50 * 100ms = 5 seconds
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        
        iteration += 1;
    }
    
    println!("\n✅ Stopped watching deployment: {}", deployment);
    Ok(())
}

fn watch_federation_status(config: &FederationConfig) -> Result<()> {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use std::time::Duration;
    
    println!("👁️  Watching federation status");
    println!("Press Ctrl+C to stop watching...\n");

    // Setup graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\n🛑 Stopping watch...");
        r.store(false, Ordering::SeqCst);
    }).context("Failed to set Ctrl+C handler")?;

    let mut iteration = 0;
    
    while running.load(Ordering::SeqCst) {
        // Clear screen and show current time
        if iteration > 0 {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
        }
        
        println!("🌐 Live Federation Status ({})", chrono::Utc::now().format("%H:%M:%S UTC"));
        println!("=========================");
        
        // Show federation status
        show_federation_status_inline(config)?;
        
        println!("\n⏱️  Next update in 10 seconds... (Ctrl+C to stop)");
        
        // Wait 10 seconds or until shutdown
        for _ in 0..100 {  // 100 * 100ms = 10 seconds
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        
        iteration += 1;
    }
    
    println!("\n✅ Stopped watching federation status");
    Ok(())
}

fn get_deployment_status_string(config: &FederationConfig, deployment: &str) -> Result<String> {
    let mut status_output = String::new();

    // Check if deployment exists
    let deployment_path = config
        .manifests
        .manifest_dir
        .join(format!("{}.yaml", deployment));
    if !deployment_path.exists() {
        return Ok(format!("❌ Deployment not found: {}", deployment));
    }

    // Load deployment manifest
    let content = std::fs::read_to_string(&deployment_path)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Extract deployment information
    let name = manifest["metadata"]["name"].as_str().unwrap_or(deployment);
    let kind = manifest["kind"].as_str().unwrap_or("Unknown");
    let namespace = manifest["metadata"]["namespace"].as_str().unwrap_or("default");

    status_output.push_str(&format!("Name: {}\n", name));
    status_output.push_str(&format!("Kind: {}\n", kind));
    status_output.push_str(&format!("Namespace: {}\n", namespace));

    // Get dynamic status information
    let status_info = get_dynamic_deployment_status(config, deployment)?;
    status_output.push_str(&status_info);
    
    Ok(status_output)
}

fn get_dynamic_deployment_status(config: &FederationConfig, deployment: &str) -> Result<String> {
    let status_file = config
        .federation
        .config_dir
        .join("deployments")
        .join(format!("{}.status", deployment));

    if status_file.exists() {
        let status_content = std::fs::read_to_string(&status_file)?;
        let status: serde_json::Value = serde_json::from_str(&status_content)?;
        
        let status_str = status["status"].as_str().unwrap_or("unknown");
        let services = status["services"].as_array().unwrap_or(&vec![]);
        
        let mut result = format!("Status: {}\n", match status_str {
            "running" => "🟢 Running",
            "pending" => "🟡 Pending", 
            "failed" => "🔴 Failed",
            "stopped" => "⚪ Stopped",
            _ => "❓ Unknown"
        });
        
        let healthy_count = services.iter()
            .filter(|s| s["health"].as_str() == Some("healthy"))
            .count();
            
        result.push_str(&format!("Services: {}/{} healthy\n", healthy_count, services.len()));
        
        if let Some(created_at) = status["created_at"].as_str() {
            if let Ok(created) = chrono::DateTime::parse_from_rfc3339(created_at) {
                let uptime = chrono::Utc::now().signed_duration_since(created.with_timezone(&chrono::Utc));
                let hours = uptime.num_hours();
                let minutes = uptime.num_minutes() % 60;
                result.push_str(&format!("Uptime: {}h {}m\n", hours, minutes));
            }
        }

        // Show individual service status
        result.push_str("\nServices:\n");
        for service in services {
            let name = service["name"].as_str().unwrap_or("unknown");
            let health = service["health"].as_str().unwrap_or("unknown");
            let health_icon = match health {
                "healthy" => "✅",
                "unhealthy" => "❌", 
                "pending" => "⏳",
                _ => "❓"
            };
            result.push_str(&format!("  {} {} ({})\n", health_icon, name, health));
        }
        
        Ok(result)
    } else {
        Ok(format!("Status: 🟡 Pending (initializing...)\nServices: 0/0 healthy\nUptime: 0h 0m\n"))
    }
}

fn show_federation_status_inline(config: &FederationConfig) -> Result<()> {
    println!("Tower: {}", config.tower.name);
    println!("Location: {}", config.tower.location);
    println!(
        "Resources: {} CPU cores, {} GB RAM",
        config.tower.resources.cpu_cores, config.tower.resources.memory_gb
    );
    println!(
        "Federation: {}",
        if config.federation.enabled {
            "🟢 Enabled"
        } else {
            "🔴 Disabled"
        }
    );

    // Add dynamic federation status
    let federation_status_file = config.federation.config_dir.join("federation.json");
    if federation_status_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&federation_status_file) {
            if let Ok(status) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(peers) = status["peers"].as_array() {
                    println!("Connected Peers: {}", peers.len());
                }
                if let Some(status_str) = status["status"].as_str() {
                    let status_icon = match status_str {
                        "initialized" => "🟢",
                        "joined" => "🔗",
                        "error" => "❌",
                        _ => "❓"
                    };
                    println!("Federation Status: {} {}", status_icon, status_str);
                }
            }
        }
    }

    Ok(())
}

fn show_deployment_status(config: &FederationConfig, deployment: &str) -> Result<()> {
    println!("📊 Deployment Status: {}", deployment);
    println!("====================");

    let status_string = get_deployment_status_string(config, deployment)?;
    println!("{}", status_string);
    Ok(())
}

fn show_federation_status(config: &FederationConfig) -> Result<()> {
    println!("🌐 Federation Status");
    println!("===================");
    
    show_federation_status_inline(config)?;
    Ok(())
}

fn show_logs(config: &FederationConfig, deployment: &str, follow: bool, lines: u32) -> Result<()> {
    println!("📜 Logs for deployment: {}", deployment);

    let mut cmd = Command::new("biome");
    cmd.arg("logs").arg(deployment);

    if follow {
        cmd.arg("--follow");
    }

    cmd.arg("--lines").arg(lines.to_string());

    let output = cmd.output().context("Failed to get logs")?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

fn remove_deployment(config: &FederationConfig, deployment: &str, force: bool) -> Result<()> {
    info!("Removing deployment: {}", deployment);

    let mut cmd = Command::new("biome");
    cmd.arg("remove").arg(deployment);

    if force {
        cmd.arg("--force");
    }

    let output = cmd.output().context("Failed to remove deployment")?;

    if output.status.success() {
        println!("✅ Deployment removed: {}", deployment);
    } else {
        error!(
            "Failed to remove deployment: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(anyhow::anyhow!("Removal failed"));
    }

    Ok(())
}

fn init_federation(
    config: &FederationConfig,
    tower_name: &str,
    location: &str,
    capabilities: &[String],
) -> Result<()> {
    println!("🏗️ Initializing Federation");
    println!("Tower: {}", tower_name);
    println!("Location: {}", location);
    println!("Capabilities: {:?}", capabilities);

    // Create federation directory if it doesn't exist
    if let Some(parent) = config.federation.config_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Initialize federation configuration
    let federation_config = serde_json::json!({
        "tower_name": tower_name,
        "location": location,
        "capabilities": capabilities,
        "initialized_at": chrono::Utc::now().to_rfc3339(),
        "peers": [],
        "status": "initialized"
    });

    // Write federation configuration
    let config_file = config.federation.config_dir.join("federation.json");
    std::fs::write(
        &config_file,
        serde_json::to_string_pretty(&federation_config)?,
    )?;

    // Create peer discovery file
    let peer_file = config.federation.config_dir.join("peers.json");
    std::fs::write(&peer_file, "[]")?;

    println!("✅ Federation initialized successfully");
    println!("   Config: {}", config_file.display());
    println!("   Peers: {}", peer_file.display());

    Ok(())
}

fn join_federation(config: &FederationConfig, bootstrap: &str, key: Option<String>) -> Result<()> {
    println!("🔗 Joining Federation");
    println!("Bootstrap node: {}", bootstrap);

    // Validate bootstrap node format
    if !bootstrap.starts_with("http://") && !bootstrap.starts_with("https://") {
        return Err(anyhow::anyhow!("Bootstrap node must be a valid HTTP URL"));
    }

    // Load existing federation config
    let config_file = config.federation.config_dir.join("federation.json");
    if !config_file.exists() {
        return Err(anyhow::anyhow!(
            "Federation not initialized. Run 'init' first."
        ));
    }

    let mut federation_config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_file)?)?;

    // Add bootstrap node to peers
    let peer_info = serde_json::json!({
        "bootstrap_node": bootstrap,
        "joined_at": chrono::Utc::now().to_rfc3339(),
        "status": "pending",
        "key": key
    });

    if let Some(peers) = federation_config.get_mut("peers") {
        if let Some(peers_array) = peers.as_array_mut() {
            peers_array.push(peer_info);
        }
    }

    // Update status
    federation_config["status"] = serde_json::Value::String("joined".to_string());
    federation_config["last_updated"] = serde_json::Value::String(chrono::Utc::now().to_rfc3339());

    // Write updated configuration
    std::fs::write(
        &config_file,
        serde_json::to_string_pretty(&federation_config)?,
    )?;

    println!("✅ Successfully joined federation");
    println!("   Bootstrap: {}", bootstrap);
    println!("   Config updated: {}", config_file.display());

    Ok(())
}

fn show_topology(config: &FederationConfig, format: &str) -> Result<()> {
    println!("🗺️ Federation Topology");

    // Load federation configuration
    let config_file = config.federation.config_dir.join("federation.json");
    let federation_config: serde_json::Value = if config_file.exists() {
        serde_json::from_str(&std::fs::read_to_string(&config_file)?)?
    } else {
        // Default empty configuration
        serde_json::json!({
            "tower_name": "unknown",
            "location": "unknown",
            "capabilities": [],
            "peers": [],
            "status": "not_initialized"
        })
    };

    // Build topology data
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Add self node
    nodes.push(serde_json::json!({
        "id": federation_config["tower_name"].as_str().unwrap_or("self"),
        "type": "self",
        "location": federation_config["location"].as_str().unwrap_or("unknown"),
        "capabilities": federation_config["capabilities"],
        "status": federation_config["status"]
    }));

    // Add peer nodes and edges
    if let Some(peers) = federation_config["peers"].as_array() {
        for (i, peer) in peers.iter().enumerate() {
            let peer_id = format!("peer_{}", i);

            nodes.push(serde_json::json!({
                "id": peer_id,
                "type": "peer",
                "bootstrap_node": peer["bootstrap_node"],
                "joined_at": peer["joined_at"],
                "status": peer["status"]
            }));

            // Add edge from self to peer
            edges.push(serde_json::json!({
                "from": federation_config["tower_name"].as_str().unwrap_or("self"),
                "to": peer_id,
                "type": "federation_link",
                "status": peer["status"]
            }));
        }
    }

    let topology = serde_json::json!({
        "nodes": nodes,
        "edges": edges,
        "metadata": {
            "total_nodes": nodes.len(),
            "total_edges": edges.len(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        }
    });

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&topology)?);
        }
        "yaml" => {
            // Convert to YAML format
            let yaml_str = serde_yaml::to_string(&topology)?;
            println!("{}", yaml_str);
        }
        _ => {
            // Text format
            println!("Nodes: {}", topology["metadata"]["total_nodes"]);
            println!("Edges: {}", topology["metadata"]["total_edges"]);

            // Show node details
            if let Some(nodes) = topology["nodes"].as_array() {
                for node in nodes {
                    println!(
                        "  📍 {}: {} ({})",
                        node["id"].as_str().unwrap_or("unknown"),
                        node["type"].as_str().unwrap_or("unknown"),
                        node["status"].as_str().unwrap_or("unknown")
                    );
                }
            }

            // Show connection status
            let healthy_edges = topology["edges"]
                .as_array()
                .map(|edges| edges.iter().filter(|e| e["status"] == "connected").count())
                .unwrap_or(0);

            let total_edges = topology["metadata"]["total_edges"].as_u64().unwrap_or(0);

            if healthy_edges == total_edges as usize {
                println!("Status: Healthy ✅");
            } else {
                println!(
                    "Status: Degraded ⚠️ ({}/{} connections healthy)",
                    healthy_edges, total_edges
                );
            }
        }
    }

    Ok(())
}

fn run_demo(config: &FederationConfig, scenario: &str, params: &[String]) -> Result<()> {
    println!("🎬 Running Demo Scenario: {}", scenario);
    println!("Parameters: {:?}", params);

    match scenario {
        "federation-demo" => {
            deploy_manifest(config, "federation-demo", false, false)?;
        }
        "songbird-coordination" => {
            deploy_manifest(config, "songbird-coordination", false, false)?;
        }
        "federation-showcase" => {
            deploy_manifest(config, "federation-showcase", false, false)?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown scenario: {}", scenario));
        }
    }

    println!("✅ Demo scenario completed");

    Ok(())
}
