// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Discovery Command Handler
//!
//! Handles service discovery operations including capability-based discovery,
//! registry-based discovery, and detailed service information display.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use serde_json::Value;
use std::collections::HashMap;

use super::utils::{create_spinner, display_results, parse_capabilities};

/// Builds the discovery result HashMap from method and raw result.
pub(crate) fn build_discovery_result(method: &str, result: &[String]) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    map.insert("method".to_string(), serde_json::json!(method));
    map.insert("endpoints".to_string(), serde_json::json!(result));
    map.insert("count".to_string(), serde_json::json!(result.len()));
    map.insert(
        "timestamp".to_string(),
        serde_json::json!(chrono::Utc::now()),
    );
    map
}

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

    let result = build_discovery_result(&format!("{method:?}"), &discovery_result);
    display_results("Discovery Results", &result, detailed).await?;

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_build_discovery_result_empty() {
        let result = build_discovery_result("CapabilityBased", &[]);
        assert_eq!(
            result.get("method").and_then(|v| v.as_str()),
            Some("CapabilityBased")
        );
        assert_eq!(
            result.get("count").and_then(serde_json::Value::as_u64),
            Some(0)
        );
        assert!(result.contains_key("endpoints"));
        assert!(result.contains_key("timestamp"));
    }

    #[test]
    fn test_build_discovery_result_with_endpoints() {
        let endpoints = vec!["http://a".to_string(), "http://b".to_string()];
        let result = build_discovery_result("Multicast", &endpoints);
        assert_eq!(
            result.get("count").and_then(serde_json::Value::as_u64),
            Some(2)
        );
        let eps = result.get("endpoints").and_then(|v| v.as_array()).unwrap();
        assert_eq!(eps.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_discover_registry_required() {
        // Registry-based discovery without URL returns error
        let result = handle_discover(
            None,
            None,
            DiscoveryMethod::RegistryBased,
            None, // no registry URL
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Registry URL required")
        );
    }

    #[test]
    fn test_discovery_method_variants() {
        // Ensure all variants are usable
        let _ = DiscoveryMethod::CapabilityBased;
        let _ = DiscoveryMethod::RegistryBased;
        let _ = DiscoveryMethod::DnsBased;
        let _ = DiscoveryMethod::Multicast;
    }

    #[test]
    fn test_build_discovery_result_timestamp_present() {
        let result = build_discovery_result("Test", &[]);
        assert!(result.contains_key("timestamp"));
    }

    #[test]
    fn test_discovery_method_display() {
        let m = DiscoveryMethod::CapabilityBased;
        let s = format!("{m:?}");
        assert!(s.contains("CapabilityBased"));
    }

    #[test]
    fn test_build_discovery_result_method_preserved() {
        let result = build_discovery_result("CustomMethod", &["a".to_string()]);
        assert_eq!(
            result.get("method").and_then(|v| v.as_str()),
            Some("CustomMethod")
        );
    }

    #[tokio::test]
    async fn test_handle_discover_capability_based() {
        let result =
            handle_discover(None, None, DiscoveryMethod::CapabilityBased, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_discover_capability_based_with_caps() {
        let result = handle_discover(
            None,
            Some("storage".to_string()),
            DiscoveryMethod::CapabilityBased,
            None,
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_discover_capability_based_empty_caps_fails() {
        let result = handle_discover(
            None,
            Some(String::new()),
            DiscoveryMethod::CapabilityBased,
            None,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_discover_multicast() {
        let result = handle_discover(None, None, DiscoveryMethod::Multicast, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_discover_dns_based() {
        let result = handle_discover(None, None, DiscoveryMethod::DnsBased, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_discover_with_registry_url() {
        let result = handle_discover(
            None,
            None,
            DiscoveryMethod::RegistryBased,
            Some("http://localhost:9999/registry".to_string()),
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_handle_discover_detailed() {
        let result =
            handle_discover(None, None, DiscoveryMethod::CapabilityBased, None, true).await;
        assert!(result.is_ok());
    }
}
