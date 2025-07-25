//! Primal Discovery Service
//!
//! Provides comprehensive primal discovery capabilities using multiple methods
//! including static configuration, network scanning, and registry-based discovery.

use crate::config::BiomeOSConfig;
use anyhow::Result;
use biomeos_primal_sdk::{PrimalCapability, PrimalHealth, PrimalType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Discovery result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub id: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: PrimalHealth,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Result of probing an endpoint for primal discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub endpoint: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: PrimalHealth,
}

/// Primal Discovery Service for finding and registering primals
#[derive(Debug)]
pub struct PrimalDiscoveryService {
    config: Arc<BiomeOSConfig>,
}

impl PrimalDiscoveryService {
    /// Create new discovery service with configuration
    pub fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { config }
    }

    /// Discover primals using static configuration
    pub async fn discover_static(&self) -> Result<Vec<DiscoveryResult>> {
        info!("Discovering static primals from configuration");
        let mut results = Vec::new();

        // Use references to avoid cloning the entire HashMap
        for (name, endpoint) in &self.config.primals.discovery.static_endpoints {
            match self.probe_endpoint(endpoint).await {
                Ok(probe_result) => {
                    let discovery_result = DiscoveryResult {
                        id: name.clone(), // Only clone when necessary for owned data
                        primal_type: PrimalType::new(
                            "unknown",
                            &probe_result.name,
                            &probe_result.version,
                        ),
                        endpoint: endpoint.clone(), // Only clone when necessary for owned data
                        capabilities: probe_result.capabilities,
                        health: probe_result.health,
                        discovered_at: chrono::Utc::now(),
                    };
                    results.push(discovery_result);
                    info!("Discovered static primal: {}", name);
                }
                Err(e) => {
                    warn!("Failed to probe static endpoint {}: {}", endpoint, e);
                }
            }
        }

        Ok(results)
    }

    /// Discover primals via network scan
    pub async fn discover_network_scan(&self) -> Result<Vec<DiscoveryResult>> {
        info!("Discovering primals via network scan");
        let mut results = Vec::new();

        // Use config timeouts for network operations
        let timeout =
            std::time::Duration::from_millis(self.config.primals.timeouts.discovery_timeout_ms);

        // Use references to avoid cloning vectors
        let scan_hosts = &self.config.primals.discovery.scan_hosts;
        let scan_ports = &self.config.primals.discovery.scan_ports;

        for host in scan_hosts {
            for port in scan_ports {
                let endpoint = format!("http://{}:{}", host, port);

                // Create HTTP client with configured timeout
                let client = reqwest::Client::builder().timeout(timeout).build()?;

                // Try to probe the endpoint
                if let Ok(response) = client.get(format!("{}/health", &endpoint)).send().await {
                    if response.status().is_success() {
                        match self.probe_endpoint(&endpoint).await {
                            Ok(probe_result) => {
                                let discovery_result = DiscoveryResult {
                                    id: format!("discovered-{}-{}", host.replace(":", "-"), port),
                                    primal_type: PrimalType::new(
                                        "network",
                                        &probe_result.name,
                                        &probe_result.version,
                                    ),
                                    endpoint: endpoint.clone(), // Only clone when necessary for owned data
                                    capabilities: probe_result.capabilities,
                                    health: probe_result.health,
                                    discovered_at: chrono::Utc::now(),
                                };
                                results.push(discovery_result);
                                info!("Discovered network primal at {}", endpoint);
                            }
                            Err(e) => {
                                debug!("Failed to probe network endpoint {}: {}", endpoint, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Discover primals from registry
    pub async fn discover_registry(&self, registry_url: &str) -> Result<Vec<DiscoveryResult>> {
        info!("Discovering primals from registry: {}", registry_url);
        let mut results = Vec::new();

        let timeout =
            std::time::Duration::from_millis(self.config.primals.timeouts.discovery_timeout_ms);
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        match client.get(registry_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // In a real implementation, this would parse the registry response
                    // For now, return sample data to demonstrate registry discovery
                    let sample_primals = vec![
                        ("toadstool-registry", "http://localhost:8084"),
                        ("songbird-registry", "http://localhost:8081"),
                        ("nestgate-registry", "http://localhost:8082"),
                    ];

                    for (name, endpoint) in sample_primals {
                        match self.probe_endpoint(endpoint).await {
                            Ok(probe_result) => {
                                let discovery_result = DiscoveryResult {
                                    id: name.to_string(), // Use to_string() only when necessary
                                    primal_type: PrimalType::new(
                                        "registry",
                                        &probe_result.name,
                                        &probe_result.version,
                                    ),
                                    endpoint: endpoint.to_string(), // Use to_string() only when necessary
                                    capabilities: probe_result.capabilities,
                                    health: probe_result.health,
                                    discovered_at: chrono::Utc::now(),
                                };
                                results.push(discovery_result);
                                info!("Discovered registry primal: {}", name);
                            }
                            Err(e) => {
                                warn!("Failed to probe registry endpoint {}: {}", endpoint, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to connect to registry {}: {}", registry_url, e);
            }
        }

        Ok(results)
    }

    /// Probe a specific endpoint for primal information
    pub async fn probe_endpoint(&self, endpoint: &str) -> Result<ProbeResult> {
        debug!("Probing endpoint: {}", endpoint);

        let timeout =
            std::time::Duration::from_millis(self.config.primals.timeouts.default_timeout_ms);
        let client = reqwest::Client::builder().timeout(timeout).build()?;

        // Try multiple common health endpoints
        let health_endpoints = [
            format!("{}/health", endpoint),
            format!("{}/api/v1/health", endpoint),
            format!("{}/api/health", endpoint),
        ];

        for health_endpoint in &health_endpoints {
            match client.get(health_endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    // Parse health response
                    if let Ok(health_data) = response.json::<serde_json::Value>().await {
                        return Ok(ProbeResult {
                            endpoint: endpoint.to_string(), // Use to_string() only when necessary
                            health: PrimalHealth::Healthy,
                            capabilities: self.extract_capabilities_from_response(&health_data),
                            name: health_data
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            version: health_data
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("1.0.0")
                                .to_string(),
                        });
                    }
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }

        // If no health endpoint responds, try basic connectivity
        match client.get(endpoint).send().await {
            Ok(_) => Ok(ProbeResult {
                endpoint: endpoint.to_string(), // Use to_string() only when necessary
                health: PrimalHealth::Degraded,
                capabilities: vec![],
                name: "unknown".to_string(),
                version: "1.0.0".to_string(),
            }),
            Err(e) => Err(anyhow::anyhow!(
                "Failed to probe endpoint {}: {}",
                endpoint,
                e
            )),
        }
    }

    /// Extract capabilities from health response
    fn extract_capabilities_from_response(
        &self,
        response: &serde_json::Value,
    ) -> Vec<PrimalCapability> {
        let mut capabilities = Vec::new();

        // Look for capabilities in common response fields
        if let Some(caps) = response.get("capabilities").and_then(|v| v.as_array()) {
            for cap in caps {
                if let Some(cap_str) = cap.as_str() {
                    match cap_str {
                        "compute" | "wasm" | "docker" => {
                            capabilities.push(PrimalCapability::new("compute", cap_str, "1.0.0"));
                        }
                        "storage" | "zfs" | "file" => {
                            capabilities.push(PrimalCapability::new("storage", cap_str, "1.0.0"));
                        }
                        "orchestration" | "service" => {
                            capabilities.push(PrimalCapability::new(
                                "orchestration",
                                cap_str,
                                "1.0.0",
                            ));
                        }
                        "security" | "encryption" => {
                            capabilities.push(PrimalCapability::new("security", cap_str, "1.0.0"));
                        }
                        "monitoring" | "metrics" => {
                            capabilities.push(PrimalCapability::new(
                                "monitoring",
                                cap_str,
                                "1.0.0",
                            ));
                        }
                        "networking" | "mesh" => {
                            capabilities.push(PrimalCapability::new(
                                "networking",
                                cap_str,
                                "1.0.0",
                            ));
                        }
                        _ => {
                            capabilities.push(PrimalCapability::custom(
                                cap_str,
                                format!("{} capability", cap_str),
                            ));
                        }
                    }
                }
            }
        }

        // Fallback: infer capabilities from service name/type
        if capabilities.is_empty() {
            if let Some(name) = response.get("name").and_then(|v| v.as_str()) {
                match name.to_lowercase().as_str() {
                    name if name.contains("toadstool") => {
                        capabilities.push(PrimalCapability::new("compute", "toadstool", "1.0.0"));
                    }
                    name if name.contains("songbird") => {
                        capabilities.push(PrimalCapability::new(
                            "orchestration",
                            "songbird",
                            "1.0.0",
                        ));
                    }
                    name if name.contains("nestgate") => {
                        capabilities.push(PrimalCapability::new("storage", "nestgate", "1.0.0"));
                    }
                    name if name.contains("beardog") => {
                        capabilities.push(PrimalCapability::new("security", "beardog", "1.0.0"));
                    }
                    _ => {
                        capabilities
                            .push(PrimalCapability::custom("basic", "Basic primal capability"));
                    }
                }
            }
        }

        capabilities
    }
}
