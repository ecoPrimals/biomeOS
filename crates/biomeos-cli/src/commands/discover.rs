//! Discovery Command Handler
//!
//! Handles service discovery operations including capability-based discovery,
//! registry-based discovery, and detailed service information display.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;

use super::utils::{create_spinner, display_results, parse_capabilities};

/// Discovery methods supported by the CLI
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DiscoveryMethod {
    #[value(name = "capability-based")]
    CapabilityBased,
    #[value(name = "registry-based")]
    RegistryBased,
    #[value(name = "dns-based")]
    DnsBased,
    #[value(name = "multicast")]
    Multicast,
}

/// Handle service discovery command
pub async fn handle_discover(
    endpoint: Option<String>,
    capabilities: Option<String>,
    method: DiscoveryMethod,
    registry: Option<String>,
    detailed: bool,
) -> Result<()> {
    let spinner = create_spinner("🔍 Discovering services...");

    let config = biomeos_types::BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config).await?;

    let discovery_result = match method {
        DiscoveryMethod::CapabilityBased => {
            if let Some(caps_str) = capabilities {
                let parsed_caps = parse_capabilities(&caps_str)?;
                manager.discover_by_capability(&parsed_caps).await?
            } else if let Some(target_endpoint) = endpoint {
                // Targeted discovery at specific endpoint
                manager
                    .probe_endpoint(&target_endpoint)
                    .await
                    .map(|_| vec![target_endpoint])?
            } else {
                manager.discover().await?
            }
        }
        DiscoveryMethod::Multicast => manager.discover_via_multicast().await?,
        DiscoveryMethod::RegistryBased => {
            if let Some(url) = registry {
                manager.discover_registry(&url).await?
            } else {
                return Err(anyhow::anyhow!(
                    "Registry URL required for registry discovery"
                ));
            }
        }
        DiscoveryMethod::DnsBased => {
            // Convert HashMap result to Vec<String> for consistency
            let dns_results = manager.discover_via_dns().await?;
            dns_results.keys().cloned().collect::<Vec<String>>()
        }
    };

    spinner.finish_with_message("✅ Discovery completed!");

    let mut result = HashMap::new();
    result.insert(
        "method".to_string(),
        serde_json::json!(format!("{:?}", method)),
    );
    result.insert("endpoints".to_string(), serde_json::json!(discovery_result));
    result.insert(
        "count".to_string(),
        serde_json::json!(discovery_result.len()),
    );
    result.insert(
        "timestamp".to_string(),
        serde_json::json!(chrono::Utc::now()),
    );

    // Fix the display_results call - use detailed parameter instead of json macro
    display_results("Discovery Results", &result, detailed).await?;

    Ok(())
}

/// Display discovery results with optional detailed information
#[allow(dead_code)]
async fn display_discovery_results(results: &HashMap<String, Value>, detailed: bool) -> Result<()> {
    if results.is_empty() {
        println!("🔍 No services discovered");
        return Ok(());
    }

    println!("🎯 Discovered {} service(s):", results.len());
    println!();

    for (service_id, service_info) in results {
        println!("🌟 Service: {}", service_id);

        if let Some(primal_type) = service_info.get("primal_type") {
            println!("   Type: {}", primal_type);
        }

        if let Some(capabilities) = service_info.get("capabilities").and_then(|c| c.as_array()) {
            println!(
                "   Capabilities: {}",
                capabilities
                    .iter()
                    .filter_map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if let Some(endpoint) = service_info.get("endpoint") {
            println!("   Endpoint: {}", endpoint);
        }

        if let Some(health) = service_info.get("health") {
            println!("   Health: {}", health);
        }

        if detailed {
            if let Some(metadata) = service_info.get("metadata") {
                println!("   Metadata: {}", serde_json::to_string_pretty(metadata)?);
            }

            if let Some(resource_usage) = service_info.get("resource_usage") {
                println!(
                    "   Resources: {}",
                    serde_json::to_string_pretty(resource_usage)?
                );
            }
        }

        println!();
    }

    // Summary statistics
    let healthy_count = results
        .values()
        .filter(|v| v.get("health").and_then(|h| h.as_str()) == Some("Healthy"))
        .count();

    println!(
        "📊 Summary: {}/{} services healthy",
        healthy_count,
        results.len()
    );

    Ok(())
}
