//! Universal Adapter for biomeOS
//!
//! This adapter implements the universal adapter pattern by delegating core functionality
//! to mature primal services rather than reimplementing them:
//! - Toadstool: Universal parser, validator, and executor
//! - Songbird: Universal discovery, coordination, and routing
//! - BiomeOS: Thin coordination layer between the two

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

// Import from biomeos-core
use biomeos_core::{BiomeError, BiomeResult};

/// Universal adapter that coordinates between Toadstool and Songbird
/// Note: This doesn't implement Clone/Serialize because it contains HTTP clients
pub struct BiomeOSUniversalAdapter {
    /// HTTP client for making requests
    client: Client,
    /// Toadstool client for parsing and execution
    toadstool_client: ToadstoolClient,
    /// Songbird client for discovery and coordination
    songbird_client: SongbirdClient,
    /// Capability registry (thin layer over Songbird)
    capability_registry: CapabilityRegistry,
    /// Health monitor (aggregates from both services)
    health_monitor: UniversalHealthMonitor,
}

impl std::fmt::Debug for BiomeOSUniversalAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BiomeOSUniversalAdapter")
            .field("toadstool_client", &self.toadstool_client)
            .field("songbird_client", &self.songbird_client)
            .field("capability_registry", &self.capability_registry)
            .field("health_monitor", &"UniversalHealthMonitor")
            .finish()
    }
}

/// Toadstool client for manifest parsing and execution
#[derive(Debug, Clone)]
pub struct ToadstoolClient {
    base_url: String,
    client: Client,
}

/// Songbird client for discovery and coordination
#[derive(Debug, Clone)]
pub struct SongbirdClient {
    base_url: String,
    client: Client,
}

/// Capability registry that maps requirements to discovered primals
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    capabilities: Arc<RwLock<HashMap<String, Vec<DiscoveredPrimal>>>>,
    songbird_client: SongbirdClient,
}

/// Universal health monitor that aggregates health from all services
#[derive(Debug)]
pub struct UniversalHealthMonitor {
    toadstool_client: ToadstoolClient,
    songbird_client: SongbirdClient,
    health_status: Arc<RwLock<SystemHealth>>,
}

/// Discovered primal information from Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    pub id: String,
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: PrimalHealth,
    pub metadata: HashMap<String, String>,
}

/// Primal health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub response_time_ms: u64,
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub toadstool_status: ServiceStatus,
    pub songbird_status: ServiceStatus,
    pub discovered_primals: Vec<DiscoveredPrimal>,
    pub last_updated: DateTime<Utc>,
}

/// Individual service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub available: bool,
    pub response_time_ms: u64,
    pub last_error: Option<String>,
}

/// Biome deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDeployment {
    pub id: String,
    pub name: String,
    pub status: String,
    pub deployed_services: Vec<DeployedService>,
    pub used_primals: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Deployed service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedService {
    pub name: String,
    pub primal: String,
    pub endpoint: String,
    pub status: String,
}

// Implement the universal adapter
impl BiomeOSUniversalAdapter {
    /// Create a new universal adapter
    pub async fn new() -> BiomeResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BiomeError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        // Initialize clients - these should discover endpoints via service discovery
        let toadstool_client = ToadstoolClient::new().await?;
        let songbird_client = SongbirdClient::new().await?;

        let capability_registry = CapabilityRegistry::new(songbird_client.clone()).await?;
        let health_monitor =
            UniversalHealthMonitor::new(toadstool_client.clone(), songbird_client.clone());

        Ok(Self {
            client,
            toadstool_client,
            songbird_client,
            capability_registry,
            health_monitor,
        })
    }

    /// Process a biome manifest using the universal adapter pattern
    pub async fn process_biome_manifest(
        &self,
        manifest_path: &str,
    ) -> BiomeResult<BiomeDeployment> {
        let span = tracing::info_span!("process_biome_manifest", manifest = manifest_path);
        let _enter = span.enter();

        info!("Starting biome deployment with universal adapter pattern");

        // Phase 1: Delegate parsing to Toadstool's proven parser
        info!("Phase 1: Delegating manifest parsing to Toadstool");
        let parsed_manifest = self
            .toadstool_client
            .parse_manifest(manifest_path)
            .await
            .map_err(|e| BiomeError::ConfigError(format!("Toadstool parsing failed: {}", e)))?;

        // Phase 2: Delegate discovery to Songbird's discovery system
        info!("Phase 2: Delegating primal discovery to Songbird");
        let available_primals =
            self.songbird_client.discover_primals().await.map_err(|e| {
                BiomeError::ConfigError(format!("Songbird discovery failed: {}", e))
            })?;

        // Phase 3: Match capabilities (thin coordination layer)
        info!("Phase 3: Matching capabilities to discovered primals");
        let resolved_primals = self
            .capability_registry
            .resolve_capabilities(&parsed_manifest, &available_primals)
            .await?;

        // Phase 4: Delegate execution to Toadstool's execution engine
        info!("Phase 4: Delegating execution to Toadstool");
        let deployment = self
            .toadstool_client
            .execute_manifest(parsed_manifest, resolved_primals)
            .await
            .map_err(|e| BiomeError::RuntimeError(format!("Toadstool execution failed: {}", e)))?;

        // Phase 5: Register with Songbird for coordination
        info!("Phase 5: Registering deployment with Songbird for coordination");
        self.songbird_client
            .register_deployment(&deployment)
            .await
            .map_err(|e| {
                BiomeError::RuntimeError(format!("Songbird registration failed: {}", e))
            })?;

        info!("Biome deployment completed successfully: {}", deployment.id);
        Ok(deployment)
    }

    /// Get system health by aggregating from both services
    pub async fn get_system_health(&self) -> BiomeResult<SystemHealth> {
        self.health_monitor.get_system_health().await
    }

    /// Discover available primals via Songbird
    pub async fn discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        self.songbird_client
            .discover_primals()
            .await
            .map_err(|e| BiomeError::RuntimeError(format!("Failed to discover primals: {}", e)))
    }
}

// Implement ToadstoolClient
impl ToadstoolClient {
    /// Create new Toadstool client with service discovery
    pub async fn new() -> BiomeResult<Self> {
        // In a real implementation, this would use service discovery to find Toadstool
        // For now, use environment variables or default endpoints
        let base_url = std::env::var("TOADSTOOL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8084".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| {
                BiomeError::ConfigError(format!("Failed to create Toadstool client: {}", e))
            })?;

        Ok(Self { base_url, client })
    }

    /// Parse manifest using Toadstool's proven parser
    pub async fn parse_manifest(&self, manifest_path: &str) -> Result<ParsedManifest, String> {
        let url = format!("{}/api/v1/manifest/parse", self.base_url);

        let request_body = serde_json::json!({
            "manifest_path": manifest_path,
            "validation_level": "strict"
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Toadstool parsing failed: {}", response.status()));
        }

        let parse_result: ParseResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !parse_result.success {
            return Err(format!(
                "Manifest validation failed: {:?}",
                parse_result.errors
            ));
        }

        parse_result
            .parsed_manifest
            .ok_or_else(|| "No parsed manifest in response".to_string())
    }

    /// Execute manifest using Toadstool's execution engine
    pub async fn execute_manifest(
        &self,
        manifest: ParsedManifest,
        resolved_primals: Vec<ResolvedPrimal>,
    ) -> Result<BiomeDeployment, String> {
        let url = format!("{}/api/v1/manifest/execute", self.base_url);

        let request_body = serde_json::json!({
            "manifest": manifest,
            "primals": resolved_primals,
            "execution_mode": "async"
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Toadstool execution failed: {}", response.status()));
        }

        let execution_result: ExecutionResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if !execution_result.success {
            return Err(format!(
                "Execution failed: {}",
                execution_result.error.unwrap_or_default()
            ));
        }

        execution_result
            .deployment
            .ok_or_else(|| "No deployment in response".to_string())
    }
}

// Implement SongbirdClient
impl SongbirdClient {
    /// Create new Songbird client with service discovery
    pub async fn new() -> BiomeResult<Self> {
        // In a real implementation, this would use mDNS or other discovery
        let base_url = std::env::var("SONGBIRD_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                BiomeError::ConfigError(format!("Failed to create Songbird client: {}", e))
            })?;

        Ok(Self { base_url, client })
    }

    /// Discover primals using Songbird's discovery system
    pub async fn discover_primals(&self) -> Result<Vec<DiscoveredPrimal>, String> {
        let url = format!("{}/api/v1/discovery/primals", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Discovery failed: {}", response.status()));
        }

        let discovery_result: DiscoveryResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(discovery_result.primals)
    }

    /// Register deployment with Songbird for coordination
    pub async fn register_deployment(&self, deployment: &BiomeDeployment) -> Result<(), String> {
        let url = format!("{}/api/v1/registry/deployments", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(deployment)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Registration failed: {}", response.status()));
        }

        Ok(())
    }
}

// Implement CapabilityRegistry
impl CapabilityRegistry {
    /// Create new capability registry
    pub async fn new(songbird_client: SongbirdClient) -> BiomeResult<Self> {
        let capabilities = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            capabilities,
            songbird_client,
        })
    }

    /// Resolve capabilities to specific primals
    pub async fn resolve_capabilities(
        &self,
        manifest: &ParsedManifest,
        available_primals: &[DiscoveredPrimal],
    ) -> BiomeResult<Vec<ResolvedPrimal>> {
        let mut resolved = Vec::new();

        // Match each required capability to available primals
        for (name, spec) in &manifest.primals {
            let capability = spec.capability_required.as_str();

            // Find primals that provide this capability
            let matching_primals: Vec<_> = available_primals
                .iter()
                .filter(|p| p.capabilities.contains(&capability.to_string()))
                .collect();

            if matching_primals.is_empty() {
                return Err(BiomeError::ConfigError(format!(
                    "No primals available for capability: {}",
                    capability
                )));
            }

            // Select best primal based on preferences
            let selected_primal =
                self.select_best_primal(&matching_primals, &spec.provider_preference)?;

            resolved.push(ResolvedPrimal {
                name: name.clone(),
                capability: capability.to_string(),
                primal: selected_primal.clone(),
                spec: spec.clone(),
            });
        }

        Ok(resolved)
    }

    /// Select the best primal based on preferences and health
    fn select_best_primal(
        &self,
        matching_primals: &[&DiscoveredPrimal],
        preferences: &[String],
    ) -> BiomeResult<DiscoveredPrimal> {
        // First, try preferences
        for preference in preferences {
            if let Some(primal) = matching_primals
                .iter()
                .find(|p| p.primal_type == *preference)
            {
                return Ok((*primal).clone());
            }
        }

        // Fallback to healthiest available primal
        matching_primals
            .iter()
            .min_by_key(|p| p.health.response_time_ms)
            .map(|p| (*p).clone())
            .ok_or_else(|| BiomeError::ConfigError("No suitable primal found".to_string()))
    }
}

// Implement UniversalHealthMonitor
impl UniversalHealthMonitor {
    /// Create new health monitor
    pub fn new(toadstool_client: ToadstoolClient, songbird_client: SongbirdClient) -> Self {
        let health_status = Arc::new(RwLock::new(SystemHealth {
            toadstool_status: ServiceStatus {
                available: false,
                response_time_ms: 0,
                last_error: None,
            },
            songbird_status: ServiceStatus {
                available: false,
                response_time_ms: 0,
                last_error: None,
            },
            discovered_primals: Vec::new(),
            last_updated: Utc::now(),
        }));

        Self {
            toadstool_client,
            songbird_client,
            health_status,
        }
    }

    /// Get current system health
    pub async fn get_system_health(&self) -> BiomeResult<SystemHealth> {
        // Update health status from both services
        let (toadstool_status, songbird_status, discovered_primals) = tokio::try_join!(
            self.check_toadstool_health(),
            self.check_songbird_health(),
            self.get_discovered_primals()
        )?;

        let health = SystemHealth {
            toadstool_status,
            songbird_status,
            discovered_primals,
            last_updated: Utc::now(),
        };

        // Update cached health
        *self.health_status.write().await = health.clone();

        Ok(health)
    }

    /// Check Toadstool health
    async fn check_toadstool_health(&self) -> BiomeResult<ServiceStatus> {
        let start = std::time::Instant::now();
        let url = format!("{}/health", self.toadstool_client.base_url);

        match self.toadstool_client.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => Ok(ServiceStatus {
                available: true,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: None,
            }),
            Ok(response) => Ok(ServiceStatus {
                available: false,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(format!("HTTP {}", response.status())),
            }),
            Err(e) => Ok(ServiceStatus {
                available: false,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(e.to_string()),
            }),
        }
    }

    /// Check Songbird health
    async fn check_songbird_health(&self) -> BiomeResult<ServiceStatus> {
        let start = std::time::Instant::now();
        let url = format!("{}/health", self.songbird_client.base_url);

        match self.songbird_client.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => Ok(ServiceStatus {
                available: true,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: None,
            }),
            Ok(response) => Ok(ServiceStatus {
                available: false,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(format!("HTTP {}", response.status())),
            }),
            Err(e) => Ok(ServiceStatus {
                available: false,
                response_time_ms: start.elapsed().as_millis() as u64,
                last_error: Some(e.to_string()),
            }),
        }
    }

    /// Get discovered primals from Songbird
    async fn get_discovered_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        self.songbird_client.discover_primals().await.map_err(|e| {
            BiomeError::RuntimeError(format!("Failed to get discovered primals: {}", e))
        })
    }
}

// Supporting types for API communication

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ManifestMetadata,
    pub primals: HashMap<String, PrimalSpec>,
    pub services: Vec<ServiceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSpec {
    pub capability_required: String,
    pub provider_preference: Vec<String>,
    pub version: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub runtime: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPrimal {
    pub name: String,
    pub capability: String,
    pub primal: DiscoveredPrimal,
    pub spec: PrimalSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParseResult {
    pub success: bool,
    pub parsed_manifest: Option<ParsedManifest>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionResult {
    pub success: bool,
    pub deployment: Option<BiomeDeployment>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryResult {
    pub primals: Vec<DiscoveredPrimal>,
    pub total_discovered: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_adapter_creation() {
        // Test that we can create the adapter (will fail without services running)
        // This test validates the structure, not the actual network calls
        let result = BiomeOSUniversalAdapter::new().await;

        // In a real environment with services, this should succeed
        // In test environment, it may fail due to missing services
        match result {
            Ok(_) => println!("Universal adapter created successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }

    #[test]
    fn test_primal_health_serialization() {
        let health = PrimalHealth {
            status: "healthy".to_string(),
            last_seen: Utc::now(),
            response_time_ms: 42,
        };

        let serialized = serde_json::to_string(&health).unwrap();
        let _deserialized: PrimalHealth = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_discovered_primal_serialization() {
        let primal = DiscoveredPrimal {
            id: "test-primal".to_string(),
            primal_type: "test".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test_capability".to_string()],
            health: PrimalHealth {
                status: "healthy".to_string(),
                last_seen: Utc::now(),
                response_time_ms: 42,
            },
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&primal).unwrap();
        let _deserialized: DiscoveredPrimal = serde_json::from_str(&serialized).unwrap();
    }
}
