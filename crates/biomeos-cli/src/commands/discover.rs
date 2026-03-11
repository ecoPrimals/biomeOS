//! Discovery Command Handler
//!
//! Handles service discovery operations including capability-based discovery,
//! registry-based discovery, and detailed service information display.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use std::collections::HashMap;

use super::utils::{create_spinner, display_results, parse_capabilities};

/// Discovery methods supported by the CLI
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DiscoveryMethod {
    /// Discover services by their advertised capabilities
    #[value(name = "capability-based")]
    CapabilityBased,
    /// Discover services from a registry endpoint
    #[value(name = "registry-based")]
    RegistryBased,
    /// Discover services via DNS-based lookups
    #[value(name = "dns-based")]
    DnsBased,
    /// Discover services via multicast announcements
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
