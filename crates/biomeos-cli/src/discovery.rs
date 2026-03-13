// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Discovery utilities for CLI
//!
//! Specialized discovery functions: type filtering, comprehensive discovery,
//! location-based, and advanced filtering.

use anyhow::Result;
use biomeos_core::{universal_biomeos_manager::DiscoveryResult, UniversalBiomeOSManager};
use biomeos_types::{Health, PrimalCapability, PrimalType};
use uuid::Uuid;

/// Extended discovery utilities
pub struct DiscoveryUtils;

impl DiscoveryUtils {
    /// Convert String endpoints from discovery methods to full DiscoveryResult structures
    /// This is a helper function to bridge the new unified discovery API (returning Vec<String>)
    /// with the CLI's expectation of structured DiscoveryResult data
    async fn endpoints_to_discovery_results(
        manager: &UniversalBiomeOSManager,
        endpoints: Vec<String>,
    ) -> Result<Vec<DiscoveryResult>> {
        let mut results = Vec::new();

        for endpoint in endpoints {
            // Try to probe the endpoint to get detailed information
            match manager.probe_endpoint(&endpoint).await {
                Ok(_probe_result) => {
                    // Create a DiscoveryResult from the probe result
                    let discovery_result = DiscoveryResult {
                        id: Uuid::new_v4().to_string(),
                        primal_type: PrimalType::new("unknown", "Unknown Primal", "1.0.0"),
                        endpoint: endpoint.clone(),
                        capabilities: vec![PrimalCapability::new("basic", "basic", "1.0.0")],
                        health: Health::Healthy,
                        discovered_at: chrono::Utc::now(),
                    };
                    results.push(discovery_result);
                }
                Err(_) => {
                    // Even if probing fails, create a basic DiscoveryResult
                    let discovery_result = DiscoveryResult {
                        id: Uuid::new_v4().to_string(),
                        primal_type: PrimalType::new("unknown", "Unknown Primal", "1.0.0"),
                        endpoint: endpoint.clone(),
                        capabilities: vec![],
                        health: Health::Unknown {
                            reason: "Probe failed".to_string(),
                            last_known: None,
                        },
                        discovered_at: chrono::Utc::now(),
                    };
                    results.push(discovery_result);
                }
            }
        }

        Ok(results)
    }
    /// Discover services by type with filtering
    pub async fn discover_by_type(
        manager: &UniversalBiomeOSManager,
        service_type: &str,
    ) -> Result<Vec<DiscoveryResult>> {
        let all_endpoints = manager.discover_network_scan().await?;
        let all_services = Self::endpoints_to_discovery_results(manager, all_endpoints).await?;
        let filtered = filter_by_type(&all_services, service_type);
        Ok(filtered)
    }

    /// Perform comprehensive discovery scan
    pub async fn comprehensive_discovery(
        manager: &UniversalBiomeOSManager,
    ) -> Result<DiscoveryReport> {
        let start_time = std::time::Instant::now();

        // Discover all available services
        let all_endpoints = manager.discover_network_scan().await?;
        let all_services = Self::endpoints_to_discovery_results(manager, all_endpoints).await?;

        // Categorize services
        let by_type = categorize_by_type(&all_services);

        // Test connectivity to each service
        let mut healthy_services = 0;
        let mut unhealthy_services = 0;
        for service in &all_services {
            match manager.probe_endpoint(&service.endpoint).await {
                Ok(_) => healthy_services += 1,
                Err(_) => unhealthy_services += 1,
            }
        }

        let discovery_time = start_time.elapsed();

        Ok(DiscoveryReport {
            total_services: all_services.len(),
            services_by_type: by_type,
            healthy_services,
            unhealthy_services,
            discovery_time_ms: discovery_time.as_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Find services near a geographical location
    ///
    /// REMOVED: Mock implementation
    /// BiomeOS should NOT implement geolocation - that's Songbird's job
    ///
    /// Future: Delegate to Songbird via UniversalPrimalClient
    /// ```rust,ignore
    /// let songbird = manager.discover_primal("discovery").await?;
    /// songbird.query_services_by_location(latitude, longitude, radius_km).await
    /// ```
    pub async fn discover_by_location(
        _manager: &UniversalBiomeOSManager,
        _latitude: f64,
        _longitude: f64,
        _radius_km: f64,
    ) -> Result<Vec<DiscoveryResult>> {
        Err(anyhow::anyhow!(
            "Geolocation discovery requires Songbird primal. \
             BiomeOS delegates this functionality to Songbird."
        ))
    }

    /// Discover services with advanced filtering
    pub async fn discover_with_filter<F>(
        manager: &UniversalBiomeOSManager,
        filter: F,
    ) -> Result<Vec<DiscoveryResult>>
    where
        F: Fn(&DiscoveryResult) -> bool,
    {
        let all_endpoints = manager.discover_network_scan().await?;
        let all_services = Self::endpoints_to_discovery_results(manager, all_endpoints).await?;
        let filtered: Vec<DiscoveryResult> = all_services.into_iter().filter(filter).collect();
        Ok(filtered)
    }
    /// Discover services with retry logic
    pub async fn discover_with_retry(
        manager: &UniversalBiomeOSManager,
        _endpoint: &str,
        capabilities: &[PrimalCapability],
        max_retries: usize,
    ) -> Result<Vec<DiscoveryResult>> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match manager.discover_by_capability(capabilities).await {
                Ok(endpoints) => {
                    let results = Self::endpoints_to_discovery_results(manager, endpoints).await?;
                    return Ok(results);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_secs(
                            2_u64.pow(attempt as u32),
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Discovery failed after retries")))
    }
}

/// Report from a comprehensive discovery scan
#[derive(Debug)]
pub struct DiscoveryReport {
    /// Total number of services discovered
    pub total_services: usize,
    /// Services grouped by primal type category
    pub services_by_type: std::collections::HashMap<String, Vec<DiscoveryResult>>,
    /// Number of services responding to health checks
    pub healthy_services: usize,
    /// Number of services failing health checks
    pub unhealthy_services: usize,
    /// Time taken for the discovery scan in milliseconds
    pub discovery_time_ms: u64,
    /// When the discovery was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Filter discovery results by primal type category (testable pure function)
#[allow(dead_code)] // Used by tests
pub(crate) fn filter_by_type(
    services: &[DiscoveryResult],
    service_type: &str,
) -> Vec<DiscoveryResult> {
    services
        .iter()
        .filter(|s| s.primal_type.category.to_lowercase() == service_type.to_lowercase())
        .cloned()
        .collect()
}

/// Categorize services by primal type (testable pure function)
#[allow(dead_code)] // Used by tests
pub(crate) fn categorize_by_type(
    services: &[DiscoveryResult],
) -> std::collections::HashMap<String, Vec<DiscoveryResult>> {
    let mut by_type: std::collections::HashMap<String, Vec<DiscoveryResult>> =
        std::collections::HashMap::new();
    for service in services {
        let category = service.primal_type.category.clone();
        by_type.entry(category).or_default().push(service.clone());
    }
    by_type
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_by_location_returns_err() {
        let config = biomeos_types::BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config)
            .await
            .expect("manager creation should succeed");
        let result = DiscoveryUtils::discover_by_location(&manager, 0.0, 0.0, 100.0).await;
        assert!(result.is_err(), "Geolocation discovery should return error");
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("Songbird") || err_str.contains("Geolocation"),
            "Error should mention Songbird or geolocation: {}",
            err_str
        );
    }

    #[test]
    fn test_discovery_report_structure() {
        let report = DiscoveryReport {
            total_services: 3,
            services_by_type: std::collections::HashMap::new(),
            healthy_services: 2,
            unhealthy_services: 1,
            discovery_time_ms: 150,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(report.total_services, 3);
        assert_eq!(report.healthy_services, 2);
        assert_eq!(report.unhealthy_services, 1);
        assert_eq!(report.discovery_time_ms, 150);
    }

    #[test]
    fn test_filter_by_type() {
        let services = vec![
            DiscoveryResult {
                id: "1".to_string(),
                primal_type: PrimalType::new("discovery", "Discovery", "1.0"),
                endpoint: "http://a".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
            DiscoveryResult {
                id: "2".to_string(),
                primal_type: PrimalType::new("compute", "Compute", "1.0"),
                endpoint: "http://b".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
            DiscoveryResult {
                id: "3".to_string(),
                primal_type: PrimalType::new("discovery", "Discovery", "2.0"),
                endpoint: "http://c".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
        ];
        let filtered = filter_by_type(&services, "discovery");
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|s| s.primal_type.category == "discovery"));

        let filtered_case = filter_by_type(&services, "DISCOVERY");
        assert_eq!(filtered_case.len(), 2);
    }

    #[test]
    fn test_filter_by_type_empty() {
        let services: Vec<DiscoveryResult> = vec![];
        let filtered = filter_by_type(&services, "discovery");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_categorize_by_type() {
        let services = vec![
            DiscoveryResult {
                id: "1".to_string(),
                primal_type: PrimalType::new("a", "A", "1.0"),
                endpoint: "http://a".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
            DiscoveryResult {
                id: "2".to_string(),
                primal_type: PrimalType::new("b", "B", "1.0"),
                endpoint: "http://b".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
            DiscoveryResult {
                id: "3".to_string(),
                primal_type: PrimalType::new("a", "A2", "2.0"),
                endpoint: "http://c".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
                discovered_at: chrono::Utc::now(),
            },
        ];
        let by_type = categorize_by_type(&services);
        assert_eq!(by_type.len(), 2);
        assert_eq!(by_type.get("a").map(|v| v.len()), Some(2));
        assert_eq!(by_type.get("b").map(|v| v.len()), Some(1));
    }
}
