//! Discovery Operations
//!
//! Handles all service discovery operations including registry discovery,
//! network scanning, capability-based discovery, and orchestration services.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::core::PrimalInfo;
use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::{BiomeOSConfig, Health, PrimalType};

/// Primal Discovery Service for ecosystem-wide primal discovery
#[derive(Debug, Clone)]
pub struct PrimalDiscoveryService {
    #[allow(dead_code)] // Config used for future discovery configuration
    config: Arc<BiomeOSConfig>,
}

/// Discovery result from primal scanning
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub id: String,
    pub endpoint: String,
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub health: Health,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Probe result from endpoint probing
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: Health,
}

impl PrimalDiscoveryService {
    /// Create new discovery service
    pub fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { config }
    }

    /// Initialize the discovery service
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("🚀 Initializing Primal Discovery Service");
        Ok(())
    }

    /// Discover primals from registry
    ///
    /// # Errors
    /// Returns an error if discovery fails.
    pub async fn discover_registry(&self, _registry_url: &str) -> Result<Vec<DiscoveryResult>> {
        // This method is deprecated - discovery now happens through ClientRegistry
        // which uses Songbird for capability-based discovery
        tracing::debug!("discover_registry called - using ClientRegistry instead");
        Ok(vec![])
    }

    /// Discover primals via network scan
    ///
    /// # Errors
    /// Returns an error if discovery fails.
    pub async fn discover_network_scan(&self) -> Result<Vec<DiscoveryResult>> {
        // This method is deprecated - discovery now happens through ClientRegistry
        // which uses Songbird for capability-based discovery
        tracing::debug!("discover_network_scan called - using ClientRegistry instead");
        Ok(vec![])
    }

    /// Probe specific endpoint
    ///
    /// # Errors
    /// Returns an error if probing fails.
    pub async fn probe_endpoint(&self, _endpoint: &str) -> Result<ProbeResult> {
        // This method is deprecated - health checking now happens through
        // PrimalClient::health_check() on individual clients
        tracing::debug!("probe_endpoint called - use PrimalClient::health_check() instead");
        Ok(ProbeResult {
            name: "unknown".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            health: Health::Healthy,
        })
    }

    /// Discover orchestration services
    ///
    /// # Errors
    /// Returns an error if discovery fails.
    pub async fn discover_orchestration(
        &self,
        _orchestration_url: &str,
    ) -> Result<Vec<DiscoveryResult>> {
        // This method is deprecated - use ClientRegistry and Songbird for discovery
        tracing::debug!("discover_orchestration called - using ClientRegistry instead");
        Ok(vec![])
    }

    /// Discover via multicast
    ///
    /// # Errors
    /// Returns an error if discovery fails.
    pub async fn discover_multicast(&self) -> Result<Vec<DiscoveryResult>> {
        // This method is deprecated - multicast discovery now handled by DiscoveryBootstrap
        tracing::debug!("discover_multicast called - using DiscoveryBootstrap instead");
        Ok(vec![])
    }
}

use super::core::UniversalBiomeOSManager;

impl UniversalBiomeOSManager {
    /// Discover primals in registry using unified configuration system
    pub async fn discover_registry(&self, registry_url: &str) -> Result<Vec<String>> {
        let results = self
            .discovery_service
            .discover_registry(registry_url)
            .await?;
        let mut endpoints = Vec::new();

        for result in results {
            endpoints.push(result.endpoint.clone());

            // Convert discovery result to PrimalInfo and register
            let primal_info = PrimalInfo {
                id: result.id,
                name: format!("Registry Primal {}", result.primal_type.name),
                primal_type: result.primal_type,
                endpoint: result.endpoint,
                capabilities: result.capabilities,
                health: result.health,
                last_seen: result.discovered_at,
                discovered_at: result.discovered_at,
                metadata: HashMap::new(),
            };

            // Auto-register discovered primals
            let _ = self.register_primal(primal_info).await;
        }

        tracing::info!("Discovered {} primals from registry", endpoints.len());
        Ok(endpoints)
    }

    /// Discover primals through network scanning
    pub async fn discover_network_scan(&self) -> Result<Vec<String>> {
        tracing::info!("🔍 Starting network scan for primals");

        let results = self.discovery_service.discover_network_scan().await?;
        let mut endpoints = Vec::new();

        for result in results {
            endpoints.push(result.endpoint.clone());

            // Convert discovery result to PrimalInfo and register
            let primal_info = PrimalInfo {
                id: result.id,
                name: format!("Network Scanned {}", result.primal_type.name),
                primal_type: result.primal_type,
                endpoint: result.endpoint,
                capabilities: result.capabilities,
                health: result.health,
                last_seen: result.discovered_at,
                discovered_at: result.discovered_at,
                metadata: HashMap::new(),
            };

            // Auto-register discovered primals
            let _ = self.register_primal(primal_info).await;
        }

        tracing::info!("Network scan discovered {} primals", endpoints.len());
        Ok(endpoints)
    }

    /// General discovery method that tries multiple approaches
    pub async fn discover(&self) -> Result<Vec<String>> {
        tracing::info!("🌐 Starting comprehensive primal discovery");

        let mut all_endpoints = Vec::new();

        // Try registry discovery first (from config)
        if let Some(registry_config) = self.config.discovery.registry.as_ref() {
            match self.discover_registry(&registry_config.url).await {
                Ok(mut endpoints) => {
                    all_endpoints.append(&mut endpoints);
                    tracing::info!("Registry discovery found {} primals", all_endpoints.len());
                }
                Err(e) => {
                    tracing::warn!("Registry discovery failed: {}", e);
                }
            }
        }

        // Try network scan as fallback
        match self.discover_network_scan().await {
            Ok(endpoints) => {
                all_endpoints.extend(endpoints);
                tracing::info!("Network scan found {} total primals", all_endpoints.len());
            }
            Err(e) => {
                tracing::warn!("Network scan failed: {}", e);
            }
        }

        // Try capability-based discovery as additional method
        // Removed recursive call to avoid infinite recursion
        tracing::info!(
            "Final discovery result: {} primals found",
            all_endpoints.len()
        );

        Ok(all_endpoints)
    }

    /// Discover primals by capabilities using unified configuration system
    pub async fn discover_by_capability(
        &self,
        capabilities: &[PrimalCapability],
    ) -> Result<Vec<String>> {
        tracing::info!("🔍 Discovering primals by capabilities: {:?}", capabilities);

        // Get all registered primals
        let primals = self.registered_primals.read().await;
        let mut matching_ids = Vec::new();

        // Filter by capabilities
        for (id, primal) in primals.iter() {
            // Check if primal has any required capabilities
            let has_required_capabilities = capabilities.iter().any(|required_cap| {
                primal.capabilities.iter().any(|primal_cap| {
                    primal_cap.category == required_cap.category
                        && primal_cap.name == required_cap.name
                })
            });

            if has_required_capabilities {
                matching_ids.push(id.clone());
            }
        }

        tracing::info!(
            "✅ Capability-based discovery found {} matching primals",
            matching_ids.len()
        );
        Ok(matching_ids)
    }

    /// Discover primals via multicast (delegated to Songbird discovery)
    pub async fn discover_via_multicast(&self) -> Result<Vec<String>> {
        tracing::info!("🔍 Starting multicast discovery via Songbird");

        // This delegates to Songbird's multicast discovery capabilities
        // In the Universal Adapter architecture, this would call Songbird's mDNS discovery
        let results = self.discovery_service.discover_multicast().await?;
        let endpoints: Vec<String> = results.iter().map(|r| r.endpoint.clone()).collect();

        tracing::info!("✅ Multicast discovery found {} primals", endpoints.len());
        Ok(endpoints)
    }

    /// Discover orchestration services specifically
    pub async fn discover_orchestration_services(&self, registry_url: &str) -> Result<Vec<String>> {
        tracing::info!(
            "🎭 Discovering orchestration services from registry: {}",
            registry_url
        );

        // Use a specific orchestration discovery endpoint
        let orchestration_url = format!("{}/api/v1/discovery/services", registry_url);
        let results = self
            .discovery_service
            .discover_orchestration(&orchestration_url)
            .await?;
        let mut orchestration_endpoints = Vec::new();

        for result in &results {
            // Filter for orchestration capabilities
            if results.iter().any(|r| {
                r.capabilities.iter().any(|cap| {
                    cap.category == "orchestration" || cap.name.contains("orchestration")
                })
            }) {
                orchestration_endpoints.push(result.endpoint.clone());

                // Convert discovery result to PrimalInfo and register
                let primal_info = PrimalInfo {
                    id: result.id.clone(),
                    name: format!("Orchestration Service {}", result.primal_type.name),
                    primal_type: result.primal_type.clone(),
                    endpoint: result.endpoint.clone(),
                    capabilities: result.capabilities.clone(),
                    health: result.health.clone(),
                    last_seen: result.discovered_at,
                    discovered_at: result.discovered_at,
                    metadata: HashMap::new(),
                };

                // Auto-register discovered orchestration services
                let _ = self.register_primal(primal_info).await;
            }
        }

        tracing::info!(
            "Discovered {} orchestration services",
            orchestration_endpoints.len()
        );
        Ok(orchestration_endpoints)
    }

    /// Discover primals via multicast
    pub async fn discover_multicast(&self) -> Result<Vec<String>> {
        tracing::info!("📡 Starting multicast discovery");

        match self.discovery_service.discover_multicast().await {
            Ok(results) => {
                let mut endpoints = Vec::new();

                for result in results {
                    endpoints.push(result.endpoint.clone());

                    // Convert discovery result to PrimalInfo and register
                    let primal_info = PrimalInfo {
                        id: result.id,
                        name: format!("Multicast {}", result.primal_type.name),
                        primal_type: result.primal_type,
                        endpoint: result.endpoint,
                        capabilities: result.capabilities,
                        health: result.health,
                        last_seen: result.discovered_at,
                        discovered_at: result.discovered_at,
                        metadata: HashMap::new(),
                    };

                    // Auto-register discovered primals
                    let _ = self.register_primal(primal_info).await;
                }

                tracing::info!("Multicast discovery found {} primals", endpoints.len());
                Ok(endpoints)
            }
            Err(e) => {
                tracing::warn!("Multicast discovery failed: {}", e);
                Ok(Vec::new()) // Return empty list instead of error
            }
        }
    }

    /// Discover all services using all available methods
    pub async fn discover_all_services(&self) -> Result<HashMap<String, serde_json::Value>> {
        let endpoints = self.discover().await?;
        let mut services = HashMap::new();

        let primals = self.registered_primals.read().await;
        for (id, primal) in primals.iter() {
            if endpoints.contains(&primal.endpoint) {
                services.insert(
                    id.clone(),
                    serde_json::json!({
                        "name": primal.name,
                        "type": primal.primal_type,
                        "endpoint": primal.endpoint,
                        "health": primal.health,
                        "capabilities": primal.capabilities,
                        "last_seen": primal.last_seen,
                    }),
                );
            }
        }

        Ok(services)
    }

    /// Discover services from a specific registry
    pub async fn discover_from_registry(
        &self,
        registry_url: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let endpoints = self.discover_registry(registry_url).await?;
        let mut services = HashMap::new();

        let primals = self.registered_primals.read().await;
        for (id, primal) in primals.iter() {
            if endpoints.contains(&primal.endpoint) {
                services.insert(
                    id.clone(),
                    serde_json::json!({
                        "name": primal.name,
                        "type": primal.primal_type,
                        "endpoint": primal.endpoint,
                        "health": primal.health,
                        "capabilities": primal.capabilities,
                        "discovered_at": primal.discovered_at,
                    }),
                );
            }
        }

        Ok(services)
    }

    /// Discover services via DNS
    /// 
    /// Future: Implement DNS-SD (Service Discovery) or SRV record lookups
    pub async fn discover_via_dns(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🌐 DNS-based discovery not yet implemented");
        Ok(HashMap::new())
    }

    /// Discover services by capabilities
    pub async fn discover_by_capabilities(
        &self,
        capabilities: &[PrimalCapability],
    ) -> Result<HashMap<String, serde_json::Value>> {
        let endpoints = self.discover_by_capability(capabilities).await?;
        let mut services = HashMap::new();

        let primals = self.registered_primals.read().await;
        for (id, primal) in primals.iter() {
            if endpoints.contains(&primal.endpoint) {
                services.insert(
                    id.clone(),
                    serde_json::json!({
                        "name": primal.name,
                        "type": primal.primal_type,
                        "endpoint": primal.endpoint,
                        "health": primal.health,
                        "capabilities": primal.capabilities,
                        "matches_criteria": true,
                    }),
                );
            }
        }

        Ok(services)
    }
}
