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
///
/// ⚠️ DEPRECATED: This function uses the old graph_deployment module.
/// Please use `biomeos-atomic-deploy` instead.
async fn handle_graph_deploy(
    _niche_path: PathBuf,
    validate_only: bool,
    _graph_name: Option<String>,
) -> Result<()> {
    if validate_only {
        println!("⚠️  DEPRECATED: Niche/graph validation via CLI is deprecated.");
        println!("📖 Use biomeos-graph and biomeos-manifest APIs directly.");
        println!("💡 Example: cargo run --bin biomeos-api");
        return Ok(());
    }

    anyhow::bail!(
        "⚠️  DEPRECATED: Graph-based niche deployment via CLI is deprecated.\n\
         📖 Please use:\n\
         • biomeos-atomic-deploy for orchestrated deployments\n\
         • biomeos-api for graph execution via REST API"
    );
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

/// Handle direct graph deployment (no niche manifest)
///
/// ⚠️ DEPRECATED: This function uses the old graph_deployment module.
/// Please use `biomeos-atomic-deploy` and `launch_primal` instead.
///
/// Migration path:
/// - For graph validation: Use biomeos-graph APIs directly
/// - For deployment: Use biomeos-atomic-deploy crate
pub async fn handle_deploy_graph_direct(_graph_path: PathBuf, validate_only: bool) -> Result<()> {
    if validate_only {
        println!("⚠️  DEPRECATED: Graph validation via CLI is deprecated.");
        println!("📖 Use biomeos-graph APIs directly for validation.");
        println!("💡 Example: cargo run --bin biomeos-api");
        return Ok(());
    }

    anyhow::bail!(
        "⚠️  DEPRECATED: Direct graph deployment via CLI is deprecated.\n\
         📖 Please use:\n\
         • biomeos-atomic-deploy for orchestrated deployments\n\
         • launch_primal for individual primal launches\n\
         • biomeos-api for graph execution via REST API"
    );
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
