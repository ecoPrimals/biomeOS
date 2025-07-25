// Discovery utilities for CLI
// Specialized discovery functions implemented: type filtering, comprehensive discovery, location-based, advanced filtering

use anyhow::Result;
use biomeos_core::{universal_biomeos_manager::DiscoveryResult, UniversalBiomeOSManager};
use biomeos_primal_sdk::PrimalCapability;

/// Extended discovery utilities
pub struct DiscoveryUtils;

impl DiscoveryUtils {
    /// Discover services by type with filtering
    pub async fn discover_by_type(
        manager: &UniversalBiomeOSManager,
        service_type: &str,
    ) -> Result<Vec<DiscoveryResult>> {
        let all_services = manager.discover_network_scan().await?;
        let filtered: Vec<DiscoveryResult> = all_services
            .into_iter()
            .filter(|service| {
                service.primal_type.category.to_lowercase() == service_type.to_lowercase()
            })
            .collect();
        Ok(filtered)
    }

    /// Perform comprehensive discovery scan
    pub async fn comprehensive_discovery(
        manager: &UniversalBiomeOSManager,
    ) -> Result<DiscoveryReport> {
        let start_time = std::time::Instant::now();

        // Discover all available services
        let all_services = manager.discover_network_scan().await?;

        // Categorize services
        let mut by_type = std::collections::HashMap::new();
        for service in &all_services {
            let category = service.primal_type.category.clone();
            by_type
                .entry(category)
                .or_insert_with(Vec::new)
                .push(service.clone());
        }

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

    /// Find services near a geographical location (mock implementation)
    pub async fn discover_by_location(
        _manager: &UniversalBiomeOSManager,
        _latitude: f64,
        _longitude: f64,
        _radius_km: f64,
    ) -> Result<Vec<DiscoveryResult>> {
        // Mock implementation - in production would use geolocation data
        Ok(vec![])
    }

    /// Discover services with advanced filtering
    pub async fn discover_with_filter<F>(
        manager: &UniversalBiomeOSManager,
        filter: F,
    ) -> Result<Vec<DiscoveryResult>>
    where
        F: Fn(&DiscoveryResult) -> bool,
    {
        let all_services = manager.discover_network_scan().await?;
        let filtered: Vec<DiscoveryResult> = all_services.into_iter().filter(filter).collect();
        Ok(filtered)
    }
    /// Discover services with retry logic
    pub async fn discover_with_retry(
        manager: &UniversalBiomeOSManager,
        endpoint: &str,
        capabilities: &[PrimalCapability],
        max_retries: usize,
    ) -> Result<Vec<DiscoveryResult>> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match manager.discover_by_capability(capabilities).await {
                Ok(results) => return Ok(results),
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

#[derive(Debug)]
pub struct DiscoveryReport {
    pub total_services: usize,
    pub services_by_type: std::collections::HashMap<String, Vec<DiscoveryResult>>,
    pub healthy_services: usize,
    pub unhealthy_services: usize,
    pub discovery_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
