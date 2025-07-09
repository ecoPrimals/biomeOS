//! Universal Primal Adapter for biomeOS
//!
//! This adapter enables biomeOS to coordinate with any Primal (standard, custom, or forked)
//! using a universal federation pattern. It manages the entire ecosystem coordination
//! and provides seamless integration for new Primals.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Universal coordination configuration for any Primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordination {
    /// Whether this Primal is enabled for coordination
    pub enabled: bool,
    
    /// Network endpoint for coordination
    pub endpoint: Option<String>,
    
    /// Coordination capabilities this Primal provides
    pub capabilities: Vec<String>,
    
    /// API version supported by this Primal
    pub api_version: String,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Universal adapter for biomeOS federation coordination
pub struct BiomeOSUniversalAdapter {
    /// HTTP client for making requests
    client: Client,
    
    /// Configuration for all available Primals
    primal_configs: Arc<RwLock<HashMap<String, PrimalCoordination>>>,
    
    /// biomeOS federation identity
    federation_identity: FederationIdentity,
    
    /// Active coordination sessions
    active_sessions: Arc<RwLock<HashMap<String, CoordinationSession>>>,
}

/// biomeOS federation identity for universal coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationIdentity {
    pub federation_id: String,
    pub capabilities: Vec<String>,
    pub endpoints: HashMap<String, String>,
    pub supported_api_versions: Vec<String>,
    pub federation_info: FederationCapabilities,
}

/// Federation capabilities that biomeOS provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationCapabilities {
    pub max_biomes: u32,
    pub supported_runtimes: Vec<String>,
    pub resource_management: bool,
    pub multi_team_isolation: bool,
    pub cross_primal_coordination: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retries: u32,
}

/// Active coordination session
#[derive(Debug, Clone)]
pub struct CoordinationSession {
    pub session_id: String,
    pub primal_name: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Active,
    Idle,
    Failed,
    Terminated,
}

impl BiomeOSUniversalAdapter {
    /// Create a new universal adapter for biomeOS
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        let federation_identity = FederationIdentity {
            federation_id: format!("biomeos-federation-{}", Uuid::new_v4().simple()),
            capabilities: vec![
                "federation".to_string(),
                "orchestration".to_string(),
                "byob_coordination".to_string(),
                "multi_team_isolation".to_string(),
                "resource_management".to_string(),
                "service_discovery".to_string(),
                "manifest_processing".to_string(),
            ],
            endpoints: HashMap::new(), // Will be populated during initialization
            supported_api_versions: vec![
                "universal/v1".to_string(),
                "biomeOS/v1".to_string(),
                "federation/v1".to_string(),
            ],
            federation_info: FederationCapabilities {
                max_biomes: 1000,
                supported_runtimes: vec![
                    "container".to_string(),
                    "native".to_string(),
                    "wasm".to_string(),
                    "gpu".to_string(),
                    "python".to_string(),
                ],
                resource_management: true,
                multi_team_isolation: true,
                cross_primal_coordination: true,
            },
        };
        
        Self {
            client,
            primal_configs: Arc::new(RwLock::new(HashMap::new())),
            federation_identity,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize the universal adapter with default Primal configurations
    pub async fn initialize_with_defaults(&self) {
        let mut configs = self.primal_configs.write().await;
        
        // Standard Primals with universal coordination
        configs.insert("songbird".to_string(), PrimalCoordination {
            enabled: true,
            endpoint: Some("http://songbird:8080".to_string()),
            capabilities: vec![
                "orchestration".to_string(),
                "service_discovery".to_string(),
                "load_balancing".to_string(),
                "federation".to_string(),
            ],
            api_version: "universal/v1".to_string(),
            health_check: HealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
            },
        });
        
        configs.insert("nestgate".to_string(), PrimalCoordination {
            enabled: true,
            endpoint: Some("http://nestgate:8082".to_string()),
            capabilities: vec![
                "storage".to_string(),
                "data".to_string(),
                "provisioning".to_string(),
                "volume_management".to_string(),
            ],
            api_version: "universal/v1".to_string(),
            health_check: HealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
            },
        });
        
        configs.insert("toadstool".to_string(), PrimalCoordination {
            enabled: true,
            endpoint: Some("http://toadstool:8084".to_string()),
            capabilities: vec![
                "compute".to_string(),
                "execution".to_string(),
                "runtime_orchestration".to_string(),
                "multi_runtime_support".to_string(),
            ],
            api_version: "universal/v1".to_string(),
            health_check: HealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
            },
        });
        
        // Future Primals (can be enabled when ready)
        configs.insert("beardog".to_string(), PrimalCoordination {
            enabled: false, // Will be enabled when ready
            endpoint: Some("http://beardog:9000".to_string()),
            capabilities: vec![
                "security".to_string(),
                "authentication".to_string(),
                "encryption".to_string(),
                "audit".to_string(),
            ],
            api_version: "universal/v1".to_string(),
            health_check: HealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
            },
        });
        
        configs.insert("squirrel".to_string(), PrimalCoordination {
            enabled: false, // Will be enabled when MCP is ready
            endpoint: Some("http://squirrel:5000".to_string()),
            capabilities: vec![
                "ai".to_string(),
                "ml".to_string(),
                "agents".to_string(),
                "mcp".to_string(),
            ],
            api_version: "universal/v1".to_string(),
            health_check: HealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
            },
        });
        
        info!("Universal adapter initialized with {} Primal configurations", configs.len());
    }
    
    /// Universal coordination method that works with any Primal
    pub async fn coordinate_with_primal(&self, primal_name: &str, coordination_request: CoordinationRequest) -> Result<CoordinationResult, AdapterError> {
        let configs = self.primal_configs.read().await;
        let primal_config = configs.get(primal_name)
            .ok_or_else(|| AdapterError::PrimalNotFound(primal_name.to_string()))?;
            
        if !primal_config.enabled {
            info!("Primal {} coordination disabled - skipping", primal_name);
            return Ok(CoordinationResult::skipped(primal_name));
        }

        if let Some(endpoint) = &primal_config.endpoint {
            info!("Coordinating with {} at: {}", primal_name, endpoint);
            
            // Create coordination session
            let session = self.create_coordination_session(primal_name).await;
            
            // Use universal coordination based on capabilities
            let result = self.call_universal_primal_api(primal_name, endpoint, primal_config, coordination_request).await;
            
            // Update session status
            self.update_session_status(&session.session_id, &result).await;
            
            return result;
        }

        warn!("{} coordination endpoint not available - continuing without", primal_name);
        Ok(CoordinationResult::unavailable(primal_name))
    }
    
    /// Coordinate with all enabled Primals for a biome deployment
    pub async fn coordinate_biome_deployment(&self, biome_manifest: BiomeManifest) -> Vec<CoordinationResult> {
        let mut results = Vec::new();
        let configs = self.primal_configs.read().await;
        
        for (primal_name, config) in configs.iter() {
            if !config.enabled {
                continue;
            }
            
            // Create coordination request based on biome manifest
            let coordination_request = self.create_biome_coordination_request(primal_name, &biome_manifest);
            
            match self.coordinate_with_primal(primal_name, coordination_request).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Coordination with {} failed: {}", primal_name, e);
                    results.push(CoordinationResult::failed(primal_name, e.to_string()));
                }
            }
        }
        
        results
    }
    
    /// Universal API call that adapts to any Primal's interface
    async fn call_universal_primal_api(
        &self,
        primal_name: &str,
        endpoint: &str,
        config: &PrimalCoordination,
        request: CoordinationRequest,
    ) -> Result<CoordinationResult, AdapterError> {
        // Determine the appropriate API path based on capabilities
        let api_path = self.determine_api_path(primal_name, &config.capabilities, &request.request_type);
        let full_url = format!("{}{}", endpoint, api_path);
        
        // Create universal coordination payload
        let coordination_payload = self.create_universal_payload(primal_name, &config.capabilities, request);
        
        info!("Universal coordination with {} at {}", primal_name, full_url);
        
        let response = self.client
            .post(&full_url)
            .json(&coordination_payload)
            .send()
            .await
            .map_err(|e| AdapterError::NetworkError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            info!("Successfully coordinated with {} (universal adapter)", primal_name);
            
            // Parse response if available
            if let Ok(response_data) = response.json::<serde_json::Value>().await {
                return Ok(CoordinationResult::success(primal_name, Some(response_data)));
            }
            
            Ok(CoordinationResult::success(primal_name, None))
        } else {
            let error_msg = format!("{} coordination failed: {}", primal_name, response.status());
            warn!("{} (universal adapter)", error_msg);
            Ok(CoordinationResult::failed(primal_name, error_msg))
        }
    }
    
    /// Determine the appropriate API path based on Primal capabilities and request type
    fn determine_api_path(&self, primal_name: &str, capabilities: &[String], request_type: &CoordinationRequestType) -> String {
        match request_type {
            CoordinationRequestType::BiomeDeployment => {
                // Route based on primary capability
                for capability in capabilities {
                    match capability.as_str() {
                        "orchestration" | "coordination" => return "/api/v1/coordinate-biome".to_string(),
                        "storage" | "data" => return "/api/v1/provision-storage".to_string(),
                        "compute" | "execution" => return "/api/v1/deploy-workloads".to_string(),
                        "security" | "authentication" => return "/api/v1/secure-biome".to_string(),
                        "ai" | "ml" => return "/api/v1/optimize-biome".to_string(),
                        _ => continue,
                    }
                }
            }
            CoordinationRequestType::HealthCheck => return "/api/v1/health".to_string(),
            CoordinationRequestType::ResourceRequest => return "/api/v1/resources".to_string(),
            CoordinationRequestType::Custom(_) => return "/api/v1/coordinate".to_string(),
        }
        
        // Fallback to standard coordination endpoint
        "/api/v1/coordinate".to_string()
    }
    
    /// Create universal payload that any Primal can understand
    fn create_universal_payload(&self, primal_name: &str, capabilities: &[String], request: CoordinationRequest) -> serde_json::Value {
        serde_json::json!({
            "coordination_request": {
                "from": "biomeos",
                "to": primal_name,
                "federation_identity": self.federation_identity,
                "capabilities_requested": capabilities,
                "api_version": "universal/v1",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "request_type": request.request_type,
                "request_data": request.data
            },
            "federation_context": {
                "max_biomes": self.federation_identity.federation_info.max_biomes,
                "supported_runtimes": self.federation_identity.federation_info.supported_runtimes,
                "resource_management": self.federation_identity.federation_info.resource_management,
                "multi_team_isolation": self.federation_identity.federation_info.multi_team_isolation
            }
        })
    }
    
    /// Create biome-specific coordination request
    fn create_biome_coordination_request(&self, primal_name: &str, biome_manifest: &BiomeManifest) -> CoordinationRequest {
        CoordinationRequest {
            request_id: Uuid::new_v4().to_string(),
            request_type: CoordinationRequestType::BiomeDeployment,
            data: serde_json::json!({
                "biome_manifest": biome_manifest,
                "target_primal": primal_name,
                "coordination_mode": "universal"
            }),
            timestamp: Utc::now(),
        }
    }
    
    /// Create a new coordination session
    async fn create_coordination_session(&self, primal_name: &str) -> CoordinationSession {
        let session = CoordinationSession {
            session_id: Uuid::new_v4().to_string(),
            primal_name: primal_name.to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            status: SessionStatus::Active,
        };
        
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());
        
        session
    }
    
    /// Update session status based on coordination result
    async fn update_session_status(&self, session_id: &str, result: &Result<CoordinationResult, AdapterError>) {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
            session.status = match result {
                Ok(coord_result) => match coord_result.status {
                    CoordinationStatus::Success => SessionStatus::Active,
                    CoordinationStatus::Failed => SessionStatus::Failed,
                    _ => SessionStatus::Idle,
                },
                Err(_) => SessionStatus::Failed,
            };
        }
    }
    
    /// Add or update a Primal configuration dynamically
    pub async fn add_primal_config(&self, primal_name: String, config: PrimalCoordination) {
        let mut configs = self.primal_configs.write().await;
        configs.insert(primal_name.clone(), config);
        info!("Added/updated Primal configuration for: {}", primal_name);
    }
    
    /// Remove a Primal configuration
    pub async fn remove_primal_config(&self, primal_name: &str) {
        let mut configs = self.primal_configs.write().await;
        configs.remove(primal_name);
        info!("Removed Primal configuration for: {}", primal_name);
    }
    
    /// Get current Primal configurations
    pub async fn get_primal_configs(&self) -> HashMap<String, PrimalCoordination> {
        self.primal_configs.read().await.clone()
    }
    
    /// Get active coordination sessions
    pub async fn get_active_sessions(&self) -> HashMap<String, CoordinationSession> {
        self.active_sessions.read().await.clone()
    }
}

/// Coordination request for universal API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequest {
    pub request_id: String,
    pub request_type: CoordinationRequestType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationRequestType {
    BiomeDeployment,
    HealthCheck,
    ResourceRequest,
    Custom(String),
}

/// Result of coordination with a Primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub primal_name: String,
    pub status: CoordinationStatus,
    pub message: Option<String>,
    pub response_data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStatus {
    Success,
    Failed,
    Skipped,
    Unavailable,
}

impl CoordinationResult {
    pub fn success(primal_name: &str, response_data: Option<serde_json::Value>) -> Self {
        Self {
            primal_name: primal_name.to_string(),
            status: CoordinationStatus::Success,
            message: Some("Coordination successful".to_string()),
            response_data,
            timestamp: Utc::now(),
        }
    }
    
    pub fn failed(primal_name: &str, error: String) -> Self {
        Self {
            primal_name: primal_name.to_string(),
            status: CoordinationStatus::Failed,
            message: Some(error),
            response_data: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn skipped(primal_name: &str) -> Self {
        Self {
            primal_name: primal_name.to_string(),
            status: CoordinationStatus::Skipped,
            message: Some("Coordination disabled".to_string()),
            response_data: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn unavailable(primal_name: &str) -> Self {
        Self {
            primal_name: primal_name.to_string(),
            status: CoordinationStatus::Unavailable,
            message: Some("Endpoint not available".to_string()),
            response_data: None,
            timestamp: Utc::now(),
        }
    }
}

/// Errors that can occur during universal coordination
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Primal not found: {0}")]
    PrimalNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Coordination failed: {0}")]
    CoordinationFailed(String),
}

/// Biome manifest placeholder (should match actual biomeOS manifest structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    pub metadata: BiomeMetadata,
    pub services: HashMap<String, ServiceSpec>,
    pub networking: Option<NetworkingSpec>,
    pub security: Option<SecuritySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub primal: String,
    pub runtime: String,
    pub image: Option<String>,
    pub resources: ResourceSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu: f64,
    pub memory: u64,
    pub storage: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    pub load_balancing: Option<bool>,
    pub service_discovery: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub network_policies: Option<bool>,
    pub resource_quotas: Option<bool>,
}

/// Trait for implementing universal coordination in biomeOS components
#[async_trait]
pub trait UniversalFederationCoordination {
    /// Coordinate biome deployment across all Primals
    async fn coordinate_biome_deployment(&self, manifest: BiomeManifest) -> Result<Vec<CoordinationResult>, AdapterError>;
    
    /// Coordinate resource allocation with Primals
    async fn coordinate_resource_allocation(&self, requirements: ResourceRequirements) -> Result<Vec<CoordinationResult>, AdapterError>;
    
    /// Coordinate health checks across federation
    async fn coordinate_health_checks(&self) -> Result<Vec<CoordinationResult>, AdapterError>;
}

/// Resource requirements for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_bandwidth: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_adapter_creation() {
        let adapter = BiomeOSUniversalAdapter::new();
        assert!(adapter.federation_identity.capabilities.contains(&"federation".to_string()));
    }

    #[tokio::test]
    async fn test_primal_configuration() {
        let adapter = BiomeOSUniversalAdapter::new();
        adapter.initialize_with_defaults().await;
        
        let configs = adapter.get_primal_configs().await;
        assert!(configs.contains_key("songbird"));
        assert!(configs.contains_key("nestgate"));
        assert!(configs.contains_key("toadstool"));
    }

    #[test]
    fn test_api_path_determination() {
        let adapter = BiomeOSUniversalAdapter::new();
        
        let biome_path = adapter.determine_api_path(
            "toadstool", 
            &["compute".to_string()], 
            &CoordinationRequestType::BiomeDeployment
        );
        assert_eq!(biome_path, "/api/v1/deploy-workloads");
        
        let health_path = adapter.determine_api_path(
            "any", 
            &[], 
            &CoordinationRequestType::HealthCheck
        );
        assert_eq!(health_path, "/api/v1/health");
    }
} 