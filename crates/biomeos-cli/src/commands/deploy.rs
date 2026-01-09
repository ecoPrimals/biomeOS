//! Deploy and Create Command Handlers
//!
//! Handles deployment operations including manifest deployment,
//! service creation, and related deployment tasks.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use super::utils::create_spinner;

/// Handle deployment command
pub async fn handle_deploy(
    manifest: PathBuf,
    validate_only: bool,
    use_graph: bool,
    graph_name: Option<String>,
) -> Result<()> {
    // Graph-based deployment (Neural API)
    if use_graph {
        return handle_graph_deploy(manifest, validate_only, graph_name).await;
    }
    
    // Legacy deployment
    let action = if validate_only {
        "Validating"
    } else {
        "Deploying"
    };
    let spinner = create_spinner(&format!("🚀 {} manifest...", action));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Read and parse the manifest
    let manifest_content = std::fs::read_to_string(&manifest)?;

    if validate_only {
        let validated_manifest = manager.validate_manifest(&manifest_content).await?;
        spinner.finish_with_message("✅ Validation completed");
        println!(
            "🎉 Manifest '{}' is valid!",
            validated_manifest.metadata.name
        );
    } else {
        let validated_manifest = manager.validate_manifest(&manifest_content).await?;
        let deployment_id = manager.deploy_manifest(&manifest_content).await?;
        spinner.finish_with_message("✅ Deployment completed");
        println!(
            "🎉 Biome '{}' deployed successfully!",
            validated_manifest.metadata.name
        );
        println!("📋 Deployment ID: {}", deployment_id);
    }

    Ok(())
}

/// Handle graph-based deployment (Neural API)
async fn handle_graph_deploy(
    niche_path: PathBuf,
    validate_only: bool,
    graph_name: Option<String>,
) -> Result<()> {
    use biomeos_core::graph_deployment::GraphDeploymentCoordinator;
    
    let action = if validate_only {
        "Validating"
    } else {
        "Deploying"
    };
    
    let graph_desc = graph_name.as_ref()
        .map(|n| format!(" with graph '{}'", n))
        .unwrap_or_default();
    
    let spinner = create_spinner(&format!("🧠 {} niche via Neural API{}...", action, graph_desc));
    
    // Create coordinator
    let coordinator = GraphDeploymentCoordinator::new();
    
    // Discover primals first
    let discovered = coordinator.registry().discover_primals().await?;
    println!("🔍 Discovered {} primals", discovered.len());
    for (primal_id, capabilities) in &discovered {
        println!("  • {} → {:?}", primal_id, capabilities);
    }
    
    if validate_only {
        // Just validate the niche and graph structure
        use biomeos_manifest::niche::NicheManifest;
        use biomeos_graph::{GraphParser, GraphValidator};
        
        let manifest = NicheManifest::from_file(&niche_path)?;
        
        let graph_ref = if let Some(name) = &graph_name {
            manifest.get_graph(name)
                .ok_or_else(|| anyhow::anyhow!("Graph '{}' not found in niche", name))?
        } else {
            manifest.get_default_graph()
                .ok_or_else(|| anyhow::anyhow!("Niche has no default graph"))?
        };
        
        let graph = GraphParser::parse_file(std::path::Path::new(&graph_ref.path))?;
        GraphValidator::validate(&graph)?;
        
        spinner.finish_with_message("✅ Validation completed");
        println!("🎉 Niche '{}' is valid!", manifest.niche.name);
        println!("📊 Graph '{}': {} nodes, {} edges", 
            graph.name, graph.nodes.len(), graph.edges.len());
    } else {
        // Execute deployment
        let result = if let Some(name) = graph_name {
            coordinator.deploy_niche_with_graph(&niche_path, &name).await?
        } else {
            coordinator.deploy_niche(&niche_path).await?
        };
        
        spinner.finish_with_message("✅ Deployment completed");
        
        if result.success {
            println!("🎉 Niche deployed successfully via Neural API!");
            println!("📊 Execution metrics:");
            println!("  • Nodes executed: {}", result.metrics.len());
            
            let mut total_duration_ms = 0u64;
            for metric in &result.metrics {
                let status = if metric.success { "✅" } else { "❌" };
                println!("  {} {} → {}ms", status, metric.node_id, metric.duration_ms);
                total_duration_ms += metric.duration_ms;
                
                if let Some(error) = &metric.error {
                    println!("     Error: {}", error);
                }
            }
            
            println!("  • Total time: {}ms", total_duration_ms);
        } else {
            let failed_nodes: Vec<_> = result.metrics.iter()
                .filter(|m| !m.success)
                .collect();
            
            anyhow::bail!(
                "Deployment failed: {} node(s) failed execution",
                failed_nodes.len()
            );
        }
    }
    
    Ok(())
}

/// Handle service creation command
pub async fn handle_create(
    service_type: String,
    name: String,
    config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<()> {
    let action = if dry_run { "Planning" } else { "Creating" };
    let spinner = create_spinner(&format!("🏗️  {} service '{}'...", action, name));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    // Load configuration if provided
    let config_data_str = if let Some(config_path) = &config_path {
        std::fs::read_to_string(config_path)?
    } else {
        serde_json::to_string(&serde_json::json!({
            "name": name,
            "type": service_type,
            "version": "latest"
        }))?
    };

    let result = if dry_run {
        manager.plan_service_creation(&config_data_str).await?
    } else {
        manager
            .create_service(&service_type, &name, config_path, dry_run)
            .await?
    };

    spinner.finish_with_message("✅ Service operation completed");

    display_create_result(&result, dry_run).await?;

    Ok(())
}

/// Display deployment results
#[allow(dead_code)]
async fn display_deploy_result(result: &HashMap<String, Value>, validate_only: bool) -> Result<()> {
    let title = if validate_only {
        "Validation Results"
    } else {
        "Deployment Results"
    };

    if let Some(status) = result.get("status") {
        match status.as_str() {
            Some("success") => println!("✅ {}: Success", title),
            Some("warning") => println!("⚠️  {}: Completed with warnings", title),
            Some("error") => println!("❌ {}: Failed", title),
            _ => println!("📋 {}: {}", title, status),
        }
    }

    if let Some(services) = result.get("services").and_then(|s| s.as_array()) {
        println!("\n🎯 Services processed: {}", services.len());

        for service in services {
            if let Some(name) = service.get("name") {
                let status = service
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let icon = match status {
                    "deployed" | "created" => "✅",
                    "updated" => "🔄",
                    "warning" => "⚠️",
                    "error" | "failed" => "❌",
                    _ => "🔹",
                };
                println!("  {} {}: {}", icon, name, status);

                if let Some(message) = service.get("message") {
                    println!("     {}", message);
                }
            }
        }
    }

    if let Some(warnings) = result.get("warnings").and_then(|w| w.as_array()) {
        if !warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for warning in warnings {
                println!("  • {}", warning);
            }
        }
    }

    if let Some(errors) = result.get("errors").and_then(|e| e.as_array()) {
        if !errors.is_empty() {
            println!("\n❌ Errors:");
            for error in errors {
                println!("  • {}", error);
            }
        }
    }

    // Show resource usage if available
    if let Some(resources) = result.get("resource_usage") {
        println!("\n📊 Resource Usage:");
        if let Some(cpu) = resources.get("cpu_cores") {
            println!("  CPU: {} cores", cpu);
        }
        if let Some(memory) = resources.get("memory_gb") {
            println!("  Memory: {} GB", memory);
        }
        if let Some(storage) = resources.get("storage_gb") {
            println!("  Storage: {} GB", storage);
        }
    }

    println!();
    Ok(())
}

/// Display service creation results
async fn display_create_result(result: &HashMap<String, Value>, dry_run: bool) -> Result<()> {
    let title = if dry_run {
        "Creation Plan"
    } else {
        "Creation Results"
    };

    println!("📋 {}:", title);

    if let Some(service_name) = result.get("service_name") {
        println!("🌟 Service: {}", service_name);
    }

    if let Some(service_id) = result.get("service_id") {
        println!("🆔 ID: {}", service_id);
    }

    if let Some(status) = result.get("status") {
        let (icon, message) = match status.as_str() {
            Some("created") => ("✅", "Successfully created"),
            Some("planned") => ("📋", "Creation plan generated"),
            Some("updated") => ("🔄", "Service updated"),
            Some("error") => ("❌", "Creation failed"),
            _ => ("🔹", "Status unknown"),
        };
        println!("{} Status: {}", icon, message);
    }

    if let Some(endpoint) = result.get("endpoint") {
        println!("🌐 Endpoint: {}", endpoint);
    }

    if let Some(capabilities) = result.get("capabilities").and_then(|c| c.as_array()) {
        println!(
            "⚡ Capabilities: {}",
            capabilities
                .iter()
                .filter_map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if dry_run {
        if let Some(plan) = result.get("execution_plan") {
            println!("\n📝 Execution Plan:");
            if let Ok(pretty) = serde_json::to_string_pretty(plan) {
                for line in pretty.lines() {
                    println!("   {}", line);
                }
            }
        }
    }

    println!();
    Ok(())
}
