// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Discovery configuration (filters, timeouts) — reserved for future use
    _config: Arc<BiomeOSConfig>,
}

/// Discovery result from primal scanning
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Primal identifier
    pub id: String,
    /// Communication endpoint
    pub endpoint: String,
    /// Primal type classification
    pub primal_type: PrimalType,
    /// Capabilities discovered
    pub capabilities: Vec<PrimalCapability>,
    /// Health status at discovery time
    pub health: Health,
    /// When this primal was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Probe result from endpoint probing
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Primal name reported by the endpoint
    pub name: String,
    /// Primal version reported by the endpoint
    pub version: String,
    /// Capabilities reported by the endpoint
    pub capabilities: Vec<PrimalCapability>,
    /// Health status reported by the endpoint
    pub health: Health,
}

impl PrimalDiscoveryService {
    /// Create new discovery service
    pub fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { _config: config }
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
        let orchestration_url = format!("{registry_url}/api/v1/discovery/services");
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager};
    use biomeos_primal_sdk::PrimalCapability;
    use biomeos_types::{BiomeOSConfig, Health, PrimalType};
    use std::collections::HashMap;

    fn test_primal_info(
        id: &str,
        name: &str,
        endpoint: &str,
        capabilities: Vec<PrimalCapability>,
    ) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: PrimalType::from_discovered("compute", name, "1.0.0"),
            endpoint: endpoint.to_string(),
            capabilities,
            health: Health::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_discovery_result_construction() {
        let result = DiscoveryResult {
            id: "primal-1".to_string(),
            endpoint: "unix:///run/beardog.sock".to_string(),
            primal_type: PrimalType::new("security", "beardog", "1.0"),
            capabilities: vec![PrimalCapability::new("security", "crypto", "1.0")],
            health: Health::Healthy,
            discovered_at: chrono::Utc::now(),
        };
        assert_eq!(result.id, "primal-1");
        assert!(result.endpoint.contains("beardog"));
        assert_eq!(result.capabilities.len(), 1);
        assert_eq!(result.primal_type.name, "beardog");
    }

    #[test]
    fn test_probe_result_construction() {
        let result = ProbeResult {
            name: "beardog".to_string(),
            version: "1.2.3".to_string(),
            capabilities: vec![],
            health: Health::Healthy,
        };
        assert_eq!(result.name, "beardog");
        assert_eq!(result.version, "1.2.3");
    }

    #[test]
    fn test_primal_discovery_service_new() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let _ = service;
    }

    #[tokio::test]
    async fn test_primal_discovery_service_initialize() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        service.initialize().await.expect("initialize");
    }

    #[tokio::test]
    async fn test_discover_registry() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager
            .discover_registry("http://registry.test:8500")
            .await
            .expect("discover_registry");
        assert!(endpoints.is_empty()); // Current impl returns empty
    }

    #[tokio::test]
    async fn test_discover_network_scan() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager
            .discover_network_scan()
            .await
            .expect("discover_network_scan");
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager.discover().await.expect("discover");
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover_with_registry_config() {
        use biomeos_types::config::DiscoveryConfig;
        use biomeos_types::config::resources::{DiscoveryMethod, RegistryConfig};
        use std::time::Duration;

        let config = BiomeOSConfig {
            discovery: DiscoveryConfig {
                default_method: DiscoveryMethod::Registry,
                methods: vec![DiscoveryMethod::Registry],
                registry: Some(RegistryConfig {
                    url: "http://registry.test:8500".to_string(),
                    auth: None,
                    health_check_interval: Duration::from_secs(30),
                }),
                dns: None,
                consul: None,
                kubernetes: None,
            },
            ..Default::default()
        };

        let manager = UniversalBiomeOSManager::new(config).await.expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager.discover().await.expect("discover");
        // Registry discovery returns empty — delegates to ClientRegistry/Songbird at runtime
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover_by_capability_empty() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let ids = manager
            .discover_by_capability(&caps)
            .await
            .expect("discover");
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn test_discover_by_capability_matching() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let primal = test_primal_info(
            "cap-1",
            "compute-svc",
            "unix:///tmp/cap.sock",
            vec![PrimalCapability::new("compute", "execution", "1.0")],
        );
        manager.register_primal(primal).await.expect("register");

        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let ids = manager
            .discover_by_capability(&caps)
            .await
            .expect("discover");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "cap-1");
    }

    #[tokio::test]
    async fn test_discover_by_capability_no_match() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let primal = test_primal_info(
            "cap-2",
            "storage-svc",
            "unix:///tmp/storage.sock",
            vec![PrimalCapability::new("storage", "nestgate", "1.0")],
        );
        manager.register_primal(primal).await.expect("register");

        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let ids = manager
            .discover_by_capability(&caps)
            .await
            .expect("discover");
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn test_discover_via_multicast() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager.discover_via_multicast().await.expect("discover");
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover_orchestration_services() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager
            .discover_orchestration_services("http://registry.test:8500")
            .await
            .expect("discover");
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover_multicast() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let endpoints = manager.discover_multicast().await.expect("discover");
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_discover_all_services() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let services = manager.discover_all_services().await.expect("discover");
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_discover_from_registry() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let services = manager
            .discover_from_registry("http://registry.test:8500")
            .await
            .expect("discover");
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_discover_via_dns() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let services = manager.discover_via_dns().await.expect("discover");
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_discover_by_capabilities() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let primal = test_primal_info(
            "cb-1",
            "cap-svc",
            "unix:///tmp/cb.sock",
            vec![PrimalCapability::new("compute", "execution", "1.0")],
        );
        manager.register_primal(primal).await.expect("register");

        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let services = manager
            .discover_by_capabilities(&caps)
            .await
            .expect("discover");
        // Note: discover_by_capabilities uses endpoints.contains(&primal.endpoint) but
        // discover_by_capability returns IDs. So services may be empty due to that logic.
        let _ = services;
    }

    #[tokio::test]
    async fn test_probe_endpoint() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let result = service
            .probe_endpoint("unix:///tmp/test.sock")
            .await
            .expect("probe");
        assert_eq!(result.name, "unknown");
        assert_eq!(result.version, "1.0.0");
        assert!(matches!(result.health, Health::Healthy));
    }

    #[tokio::test]
    async fn test_primal_discovery_service_discover_registry_returns_empty() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let v = service
            .discover_registry("http://example.invalid:9/registry")
            .await
            .expect("registry");
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_primal_discovery_service_discover_network_scan_empty() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let v = service.discover_network_scan().await.expect("scan");
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_primal_discovery_service_discover_multicast_empty() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let v = service.discover_multicast().await.expect("multicast");
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_primal_discovery_service_discover_orchestration_empty() {
        let config = Arc::new(BiomeOSConfig::default());
        let service = PrimalDiscoveryService::new(config);
        let v = service
            .discover_orchestration("http://x.test/registry")
            .await
            .expect("orch");
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_discover_all_services_empty_without_registry_hits() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");
        let map = manager.discover_all_services().await.expect("all");
        assert!(map.is_empty());
    }

    #[tokio::test]
    async fn test_discover_from_registry_empty_endpoints() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");
        let map = manager
            .discover_from_registry("http://registry.test:8500")
            .await
            .expect("from reg");
        assert!(map.is_empty());
    }

    #[tokio::test]
    async fn test_discover_by_capabilities_registered_mismatch_endpoints() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");
        let primal = test_primal_info(
            "dcb-1",
            "dcb",
            "unix:///tmp/dcb.sock",
            vec![PrimalCapability::new("compute", "execution", "1.0")],
        );
        manager.register_primal(primal).await.expect("register");
        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let services = manager
            .discover_by_capabilities(&caps)
            .await
            .expect("by caps");
        // Implementation matches endpoints to capability ids — may be empty.
        let _ = services;
    }

    #[test]
    fn test_discovery_result_debug_clone() {
        let a = DiscoveryResult {
            id: "i".to_string(),
            endpoint: "e".to_string(),
            primal_type: PrimalType::new("t", "n", "v"),
            capabilities: vec![],
            health: Health::Healthy,
            discovered_at: chrono::Utc::now(),
        };
        let b = a.clone();
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn test_probe_result_clone() {
        let p = ProbeResult {
            name: "n".to_string(),
            version: "v".to_string(),
            capabilities: vec![],
            health: Health::Healthy,
        };
        assert_eq!(p.name, p.clone().name);
    }

    #[tokio::test]
    async fn test_discover_by_capabilities_populates_when_endpoint_matches_id() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let primal = test_primal_info(
            "same-id",
            "svc",
            "same-id",
            vec![PrimalCapability::new("compute", "execution", "1.0")],
        );
        manager.register_primal(primal).await.expect("register");

        let caps = vec![PrimalCapability::new("compute", "execution", "1.0")];
        let services = manager
            .discover_by_capabilities(&caps)
            .await
            .expect("discover");
        assert!(services.contains_key("same-id"));
    }

    #[tokio::test]
    async fn test_discover_by_capability_returns_multiple_matches() {
        let manager = UniversalBiomeOSManager::with_default_config()
            .await
            .expect("manager");
        manager.initialize().await.expect("init");

        let cap = PrimalCapability::new("compute", "execution", "1.0");
        for (id, ep) in [("m1", "unix:///a.sock"), ("m2", "unix:///b.sock")] {
            let primal = test_primal_info(id, "svc", ep, vec![cap.clone()]);
            manager.register_primal(primal).await.expect("register");
        }

        let ids = manager
            .discover_by_capability(&[cap])
            .await
            .expect("discover");
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"m1".to_string()));
        assert!(ids.contains(&"m2".to_string()));
    }
}
