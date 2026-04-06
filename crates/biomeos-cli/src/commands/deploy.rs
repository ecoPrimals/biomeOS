// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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

/// Maps deployment/creation status to (icon, message) for display.
pub(crate) fn status_to_display(status: &str) -> (&'static str, &'static str) {
    match status {
        "created" => ("✅", "Successfully created"),
        "planned" => ("📋", "Creation plan generated"),
        "updated" => ("🔄", "Service updated"),
        "error" => ("❌", "Creation failed"),
        _ => ("🔹", "Status unknown"),
    }
}

/// Builds display lines for deployment/creation result.
pub(crate) fn format_deployment_result(
    result: &HashMap<String, Value>,
    dry_run: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    let title = if dry_run {
        "Creation Plan"
    } else {
        "Creation Results"
    };
    lines.push(format!("📋 {title}:"));

    if let Some(service_name) = result.get("service_name") {
        lines.push(format!("🌟 Service: {service_name}"));
    }

    if let Some(service_id) = result.get("service_id") {
        lines.push(format!("🆔 ID: {service_id}"));
    }

    if let Some(status) = result.get("status") {
        let status_str = status.as_str().unwrap_or("");
        let (icon, message) = status_to_display(status_str);
        lines.push(format!("{icon} Status: {message}"));
    }

    if let Some(endpoint) = result.get("endpoint") {
        lines.push(format!("🌐 Endpoint: {endpoint}"));
    }

    if let Some(capabilities) = result.get("capabilities").and_then(|c| c.as_array()) {
        let caps_str = capabilities
            .iter()
            .filter_map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("⚡ Capabilities: {caps_str}"));
    }

    if dry_run {
        if let Some(plan) = result.get("execution_plan") {
            lines.push("\n📝 Execution Plan:".to_string());
            if let Ok(pretty) = serde_json::to_string_pretty(plan) {
                for line in pretty.lines() {
                    lines.push(format!("   {line}"));
                }
            }
        }
    }

    lines.push(String::new());
    lines
}

/// Handle deployment command
pub async fn handle_deploy(
    manifest: PathBuf,
    validate_only: bool,
    use_graph: bool,
    graph_name: Option<String>,
) -> Result<()> {
    // Graph-based deployment (Neural API)
    if use_graph {
        return handle_graph_deploy(manifest, validate_only, graph_name);
    }

    // Legacy deployment
    let action = if validate_only {
        "Validating"
    } else {
        "Deploying"
    };
    let spinner = create_spinner(&format!("🚀 {action} manifest..."));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

    // Read and parse the manifest
    let manifest_content = std::fs::read_to_string(&manifest)?;

    if validate_only {
        let validated_manifest = manager.validate_manifest(&manifest_content)?;
        spinner.finish_with_message("✅ Validation completed");
        println!(
            "🎉 Manifest '{}' is valid!",
            validated_manifest.metadata.name
        );
    } else {
        let validated_manifest = manager.validate_manifest(&manifest_content)?;
        let deployment_id = manager.deploy_manifest(&manifest_content)?;
        spinner.finish_with_message("✅ Deployment completed");
        println!(
            "🎉 Biome '{}' deployed successfully!",
            validated_manifest.metadata.name
        );
        println!("📋 Deployment ID: {deployment_id}");
    }

    Ok(())
}

/// Handle graph-based deployment (Neural API)
///
/// ⚠️ DEPRECATED: This function uses the old `graph_deployment` module.
/// Please use `biomeos-atomic-deploy` instead.
fn handle_graph_deploy(
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
    let spinner = create_spinner(&format!("🏗️  {action} service '{name}'..."));

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config)?;

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
        manager.plan_service_creation(&config_data_str)?
    } else {
        manager
            .create_service(&service_type, &name, config_path, dry_run)
            .await?
    };

    spinner.finish_with_message("✅ Service operation completed");

    display_create_result(&result, dry_run);

    Ok(())
}

/// Handle direct graph deployment (no niche manifest)
///
/// ⚠️ DEPRECATED: This function uses the old `graph_deployment` module.
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

/// Display service creation results (thin wrapper)
fn display_create_result(result: &HashMap<String, Value>, dry_run: bool) {
    let lines = format_deployment_result(result, dry_run);
    for line in lines {
        println!("{line}");
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::path::PathBuf;

    #[test]
    fn test_status_to_display_all_variants() {
        assert_eq!(status_to_display("created"), ("✅", "Successfully created"));
        assert_eq!(
            status_to_display("planned"),
            ("📋", "Creation plan generated")
        );
        assert_eq!(status_to_display("updated"), ("🔄", "Service updated"));
        assert_eq!(status_to_display("error"), ("❌", "Creation failed"));
        assert_eq!(status_to_display("unknown"), ("🔹", "Status unknown"));
        assert_eq!(status_to_display(""), ("🔹", "Status unknown"));
    }

    #[test]
    fn test_format_deployment_result_empty() {
        let result = HashMap::new();
        let lines = format_deployment_result(&result, false);
        assert!(lines[0].contains("Creation Results"));
        assert!(lines.last().is_some_and(String::is_empty));
    }

    #[test]
    fn test_format_deployment_result_with_status() {
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::String("created".to_string()));
        result.insert(
            "service_name".to_string(),
            Value::String("mysvc".to_string()),
        );
        let lines = format_deployment_result(&result, false);
        assert!(lines.iter().any(|l| l.contains("✅")));
        assert!(lines.iter().any(|l| l.contains("Successfully created")));
        assert!(lines.iter().any(|l| l.contains("mysvc")));
    }

    #[tokio::test]
    async fn test_handle_graph_deploy_validate_only() {
        // Deprecated path - validate_only returns Ok with deprecation message
        let result = handle_deploy(
            PathBuf::from("/nonexistent/niche.yaml"),
            true, // validate_only
            true, // use_graph
            Some("test-graph".to_string()),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_graph_deploy_deprecated_error() {
        // use_graph without validate_only should bail with deprecation
        let result =
            handle_deploy(PathBuf::from("/nonexistent/niche.yaml"), false, true, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("DEPRECATED"));
    }

    #[tokio::test]
    async fn test_handle_deploy_graph_direct_validate_only() {
        let result = handle_deploy_graph_direct(PathBuf::from("/nonexistent"), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_deploy_graph_direct_deprecated_error() {
        let result = handle_deploy_graph_direct(PathBuf::from("/nonexistent"), false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DEPRECATED"));
    }

    #[test]
    fn test_format_deployment_result_dry_run() {
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::String("planned".to_string()));
        result.insert(
            "execution_plan".to_string(),
            serde_json::json!({"steps": ["step1", "step2"]}),
        );
        let lines = format_deployment_result(&result, true);
        assert!(lines.iter().any(|l| l.contains("Creation Plan")));
        assert!(lines.iter().any(|l| l.contains("Execution Plan")));
        assert!(lines.iter().any(|l| l.contains("step1")));
    }

    #[test]
    fn test_format_deployment_result_with_capabilities() {
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::String("created".to_string()));
        result.insert(
            "capabilities".to_string(),
            serde_json::json!(["storage", "compute"]),
        );
        let lines = format_deployment_result(&result, false);
        assert!(lines.iter().any(|l| l.contains("storage")));
        assert!(lines.iter().any(|l| l.contains("compute")));
    }

    #[test]
    fn test_format_deployment_result_with_endpoint() {
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::String("created".to_string()));
        result.insert(
            "endpoint".to_string(),
            Value::String("http://localhost:9000".to_string()),
        );
        let lines = format_deployment_result(&result, false);
        assert!(lines.iter().any(|l| l.contains("localhost")));
    }

    #[test]
    fn test_format_deployment_result_with_service_id() {
        let mut result = HashMap::new();
        result.insert(
            "service_id".to_string(),
            Value::String("svc-123".to_string()),
        );
        let lines = format_deployment_result(&result, false);
        assert!(lines.iter().any(|l| l.contains("svc-123")));
    }

    #[test]
    fn test_status_to_display_updated() {
        assert_eq!(status_to_display("updated"), ("🔄", "Service updated"));
    }
}
