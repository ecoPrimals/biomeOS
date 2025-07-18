//! # Universal Primal Provider for biomeOS
//!
//! This module implements the advanced PrimalProvider architecture pioneered by Songbird,
//! providing multi-instance support, context-aware routing, and capability-based routing
//! for the biomeOS ecosystem.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{BiomeError, BiomeResult, HealthStatus, PrimalType};

/// Universal Primal Provider trait (aligned with Songbird's architecture)
#[async_trait]
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "biomeos", "toadstool", "nestgate")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category
    fn primal_type(&self) -> PrimalType;

    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<PrimalCapability>;

    /// Dependencies on other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Health check for this primal
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    async fn handle_primal_request(&self, request: PrimalRequest) -> BiomeResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> BiomeResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> BiomeResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}

/// Context for user/device-specific primal routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Session identifier
    pub session_id: String,
    /// Network location
    pub network_location: NetworkLocation,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Biome identifier
    pub biome_id: Option<String>,
    /// Team identifier
    pub team_id: Option<String>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: String,
    /// Subnet
    pub subnet: Option<String>,
    /// Network identifier
    pub network_id: Option<String>,
    /// Geographic location
    pub geo_location: Option<String>,
}

/// Security level requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security
    Basic,
    /// Standard security
    Standard,
    /// High security
    High,
    /// Critical security
    Critical,
}

/// Primal capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimalCapability {
    /// Capability type
    pub capability_type: CapabilityType,
    /// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Capability description
    pub description: String,
    /// Capability parameters
    pub parameters: HashMap<String, CapabilityParameter>,
    /// Capability status
    pub status: CapabilityStatus,
    /// Capability metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Eq for PrimalCapability {}

impl std::hash::Hash for PrimalCapability {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.capability_type.hash(state);
        self.name.hash(state);
        self.version.hash(state);
        self.description.hash(state);
        self.status.hash(state);
        // Skip parameters and metadata as they contain HashMap which doesn't implement Hash
    }
}

/// Types of capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Biome orchestration
    BiomeOrchestration,
    /// Team workspace management
    TeamWorkspace,
    /// Resource management
    ResourceManagement,
    /// Health monitoring
    HealthMonitoring,
    /// Service discovery
    ServiceDiscovery,
    /// Service orchestration
    ServiceOrchestration,
    /// Security and encryption
    SecurityEncryption,
    /// Primal coordination
    PrimalCoordination,
    /// Custom capability
    Custom(String),
}

/// Capability status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityStatus {
    /// Active capability
    Active,
    /// Inactive capability
    Inactive,
    /// Deprecated capability
    Deprecated,
    /// Experimental capability
    Experimental,
}

/// Capability parameter definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityParameter {
    /// Parameter type
    pub param_type: String,
    /// Whether required
    pub required: bool,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Description
    pub description: String,
}

/// Primal dependency specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Required primal type
    pub primal_type: PrimalType,
    /// Required capability
    pub capability: String,
    /// Minimum version
    pub min_version: String,
    /// Whether dependency is optional
    pub optional: bool,
}

/// Primal health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Health score (0.0-1.0)
    pub health_score: f64,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Health details
    pub details: HashMap<String, serde_json::Value>,
    /// Performance metrics
    pub metrics: HealthMetrics,
}

/// Health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Active connections
    pub active_connections: u64,
}

/// Primal endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// Admin endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
}

/// Universal request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request ID
    pub id: Uuid,
    /// Request type
    pub request_type: RequestType,
    /// Operation to perform
    pub operation: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request context
    pub context: PrimalContext,
    /// Request priority
    pub priority: Priority,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Universal response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response is for
    pub request_id: Uuid,
    /// Response type
    pub response_type: ResponseType,
    /// Response payload
    pub payload: serde_json::Value,
    /// Whether request was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    /// Deploy a biome
    Deploy,
    /// Get status
    Status,
    /// Scale resources
    Scale,
    /// Health check
    Health,
    /// Custom request
    Custom(String),
}

/// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    /// Deployment response
    Deployment,
    /// Status response
    Status,
    /// Health response
    Health,
    /// Error response
    Error,
    /// Custom response
    Custom(String),
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Dynamic port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Assigned port
    pub port: u16,
    /// Port type
    pub port_type: PortType,
    /// Whether port is secure
    pub secure: bool,
    /// Protocol
    pub protocol: String,
}

/// Port types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortType {
    Http,
    Https,
    Tcp,
    Udp,
    WebSocket,
    Grpc,
}

/// biomeOS Primal Provider implementation
pub struct BiomeOSPrimalProvider {
    /// Instance configuration
    instance_config: BiomeOSInstanceConfig,
    /// Current health status
    health_status: Arc<RwLock<PrimalHealth>>,
    /// Active capabilities
    capabilities: Vec<PrimalCapability>,
    /// Endpoints
    endpoints: PrimalEndpoints,
    /// Dynamic port info
    dynamic_ports: Arc<RwLock<HashMap<String, DynamicPortInfo>>>,
}

/// biomeOS instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSInstanceConfig {
    /// Instance ID
    pub instance_id: String,
    /// Context this instance serves
    pub context: PrimalContext,
    /// Base URL
    pub base_url: String,
    /// Team workspace settings
    pub team_workspace: TeamWorkspaceConfig,
    /// Resource management settings
    pub resource_management: ResourceManagementConfig,
}

/// Team workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkspaceConfig {
    /// Base directory for team workspaces
    pub base_dir: String,
    /// Default resource quotas
    pub default_quotas: HashMap<String, serde_json::Value>,
    /// Workspace isolation settings
    pub isolation: IsolationConfig,
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// CPU allocation strategy
    pub cpu_allocation: AllocationStrategy,
    /// Memory allocation strategy
    pub memory_allocation: AllocationStrategy,
    /// Storage allocation strategy
    pub storage_allocation: AllocationStrategy,
}

/// Isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Network isolation enabled
    pub network_isolation: bool,
    /// Filesystem isolation enabled
    pub filesystem_isolation: bool,
    /// Process isolation enabled
    pub process_isolation: bool,
}

/// Resource allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Fair share allocation
    FairShare,
    /// Priority-based allocation
    Priority,
    /// Guaranteed allocation
    Guaranteed,
    /// Best effort allocation
    BestEffort,
}

impl BiomeOSPrimalProvider {
    /// Create a new biomeOS primal provider
    pub fn new(config: BiomeOSInstanceConfig) -> Self {
        let capabilities = vec![
            PrimalCapability {
                name: "biome_orchestration".to_string(),
                version: "1.0.0".to_string(),
                capability_type: CapabilityType::BiomeOrchestration,
                description: "Biome orchestration and lifecycle management".to_string(),
                parameters: HashMap::new(),
                status: CapabilityStatus::Active,
                metadata: HashMap::new(),
            },
            PrimalCapability {
                name: "team_workspace".to_string(),
                version: "1.0.0".to_string(),
                capability_type: CapabilityType::TeamWorkspace,
                description: "Team workspace management and isolation".to_string(),
                parameters: HashMap::new(),
                status: CapabilityStatus::Active,
                metadata: HashMap::new(),
            },
            PrimalCapability {
                name: "resource_management".to_string(),
                version: "1.0.0".to_string(),
                capability_type: CapabilityType::ResourceManagement,
                description: "Resource allocation and management".to_string(),
                parameters: HashMap::new(),
                status: CapabilityStatus::Active,
                metadata: HashMap::new(),
            },
            PrimalCapability {
                name: "health_monitoring".to_string(),
                version: "1.0.0".to_string(),
                capability_type: CapabilityType::HealthMonitoring,
                description: "Health monitoring and analytics".to_string(),
                parameters: HashMap::new(),
                status: CapabilityStatus::Active,
                metadata: HashMap::new(),
            },
        ];

        let endpoints = PrimalEndpoints {
            primary: format!("{}/api/v1", config.base_url),
            health: format!("{}/api/v1/health", config.base_url),
            metrics: format!("{}/api/v1/metrics", config.base_url),
            admin: Some(format!("{}/api/v1/admin", config.base_url)),
            websocket: Some(format!("{}/ws", config.base_url)),
        };

        let health_status = Arc::new(RwLock::new(PrimalHealth {
            status: HealthStatus::Healthy,
            health_score: 1.0,
            last_check: Utc::now(),
            details: HashMap::new(),
            metrics: HealthMetrics {
                cpu_usage: 0.0,
                memory_mb: 0.0,
                response_time_ms: 0.0,
                error_rate: 0.0,
                active_connections: 0,
            },
        }));

        Self {
            instance_config: config,
            health_status,
            capabilities,
            endpoints,
            dynamic_ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl PrimalProvider for BiomeOSPrimalProvider {
    fn primal_id(&self) -> &str {
        "biomeos"
    }

    fn instance_id(&self) -> &str {
        &self.instance_config.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.instance_config.context
    }

    fn primal_type(&self) -> PrimalType {
        "biomeos".to_string()
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency {
                primal_type: "songbird".to_string(),
                capability: "service_discovery".to_string(),
                min_version: "1.0.0".to_string(),
                optional: false,
            },
            PrimalDependency {
                primal_type: "toadstool".to_string(),
                capability: "container_orchestration".to_string(),
                min_version: "1.0.0".to_string(),
                optional: false,
            },
            PrimalDependency {
                primal_type: "nestgate".to_string(),
                capability: "storage_management".to_string(),
                min_version: "1.0.0".to_string(),
                optional: false,
            },
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        let health = self.health_status.read().await;
        health.clone()
    }

    fn endpoints(&self) -> PrimalEndpoints {
        self.endpoints.clone()
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> BiomeResult<PrimalResponse> {
        info!("Handling primal request: {:?}", request.operation);

        let response_payload = match request.operation.as_str() {
            "deploy_biome" => self.handle_deploy_biome(request.payload).await?,
            "get_team_workspace" => self.handle_get_team_workspace(request.payload).await?,
            "manage_resources" => self.handle_manage_resources(request.payload).await?,
            "health_check" => {
                let health = self.health_check().await;
                serde_json::to_value(health).map_err(|e| {
                    BiomeError::RuntimeError(format!("Failed to serialize health: {}", e))
                })?
            }
            _ => {
                return Err(BiomeError::InvalidInput(format!(
                    "Unknown operation: {}",
                    request.operation
                )));
            }
        };

        Ok(PrimalResponse {
            request_id: request.id,
            response_type: ResponseType::Custom(request.operation.clone()),
            payload: response_payload,
            success: true,
            error: None,
            timestamp: Utc::now(),
        })
    }

    async fn initialize(&mut self, _config: serde_json::Value) -> BiomeResult<()> {
        info!("Initializing biomeOS primal provider");

        // Initialize health monitoring
        let mut health = self.health_status.write().await;
        health.status = HealthStatus::Healthy;
        health.last_check = Utc::now();

        Ok(())
    }

    async fn shutdown(&mut self) -> BiomeResult<()> {
        info!("Shutting down biomeOS primal provider");

        // Update health status
        let mut health = self.health_status.write().await;
        health.status = HealthStatus::Unhealthy;

        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if this instance can serve the given context
        self.instance_config.context.user_id == context.user_id
            && self.instance_config.context.team_id == context.team_id
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        // Return primary port info
        Some(DynamicPortInfo {
            port: 8080, // Default port
            port_type: PortType::Http,
            secure: false,
            protocol: "http".to_string(),
        })
    }
}

impl BiomeOSPrimalProvider {
    /// Handle deploy biome request
    async fn handle_deploy_biome(
        &self,
        _payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        debug!("Handling deploy biome request");

        // Implementation would go here
        Ok(serde_json::json!({
            "status": "deployed",
            "biome_id": "test-biome-id"
        }))
    }

    /// Handle get team workspace request
    async fn handle_get_team_workspace(
        &self,
        _payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        debug!("Handling get team workspace request");

        // Implementation would go here
        Ok(serde_json::json!({
            "workspace_id": "test-workspace-id",
            "status": "active"
        }))
    }

    /// Handle manage resources request
    async fn handle_manage_resources(
        &self,
        _payload: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        debug!("Handling manage resources request");

        // Implementation would go here
        Ok(serde_json::json!({
            "resources": {
                "cpu": "2.0",
                "memory": "4Gi",
                "storage": "20Gi"
            }
        }))
    }

    /// Get dynamic port information
    pub async fn get_dynamic_ports(&self) -> HashMap<String, DynamicPortInfo> {
        self.dynamic_ports.read().await.clone()
    }

    /// Add dynamic port information
    pub async fn add_dynamic_port(&self, service_id: String, port_info: DynamicPortInfo) {
        let mut ports = self.dynamic_ports.write().await;
        ports.insert(service_id, port_info);
    }

    /// Remove dynamic port information
    pub async fn remove_dynamic_port(&self, service_id: &str) -> Option<DynamicPortInfo> {
        let mut ports = self.dynamic_ports.write().await;
        ports.remove(service_id)
    }

    /// Get port info for a specific service
    pub async fn get_port_info(&self, service_id: &str) -> Option<DynamicPortInfo> {
        let ports = self.dynamic_ports.read().await;
        ports.get(service_id).cloned()
    }

    /// Update port info for a service
    pub async fn update_port_info(&self, service_id: &str, port_info: DynamicPortInfo) -> bool {
        let mut ports = self.dynamic_ports.write().await;
        if ports.contains_key(service_id) {
            ports.insert(service_id.to_string(), port_info);
            true
        } else {
            false
        }
    }
}

/// Universal Primal Registry for biomeOS
pub struct BiomeOSPrimalRegistry {
    /// Registered primal providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn PrimalProvider>>>>,
    /// Capability index
    capability_index: Arc<RwLock<HashMap<PrimalCapability, Vec<String>>>>,
    /// Context index
    context_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Default for BiomeOSPrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeOSPrimalRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            context_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a primal provider
    pub async fn register_provider(&self, provider: Arc<dyn PrimalProvider>) -> BiomeResult<()> {
        let instance_id = provider.instance_id().to_string();

        // Register provider
        let mut providers = self.providers.write().await;
        providers.insert(instance_id.clone(), provider.clone());

        // Update capability index
        let mut capability_index = self.capability_index.write().await;
        for capability in provider.capabilities() {
            capability_index
                .entry(capability)
                .or_insert_with(Vec::new)
                .push(instance_id.clone());
        }

        // Update context index
        let mut context_index = self.context_index.write().await;
        let context_key = format!(
            "{}:{}",
            provider.context().user_id,
            provider
                .context()
                .team_id
                .as_ref()
                .unwrap_or(&"default".to_string())
        );
        context_index
            .entry(context_key)
            .or_insert_with(Vec::new)
            .push(instance_id);

        info!("Registered primal provider: {}", provider.instance_id());
        Ok(())
    }

    /// Find providers by capability
    pub async fn find_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Vec<Arc<dyn PrimalProvider>> {
        let capability_index = self.capability_index.read().await;
        let providers = self.providers.read().await;

        capability_index
            .get(capability)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|instance_id| providers.get(instance_id))
            .cloned()
            .collect()
    }

    /// Find providers by context
    pub async fn find_by_context(&self, context: &PrimalContext) -> Vec<Arc<dyn PrimalProvider>> {
        let context_key = format!(
            "{}:{}",
            context.user_id,
            context.team_id.as_ref().unwrap_or(&"default".to_string())
        );

        let context_index = self.context_index.read().await;
        let providers = self.providers.read().await;

        context_index
            .get(&context_key)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|instance_id| providers.get(instance_id))
            .filter(|provider| provider.can_serve_context(context))
            .cloned()
            .collect()
    }


}
