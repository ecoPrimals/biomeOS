//! Universal Primal Adapter for biomeOS
//!
//! This adapter enables biomeOS to coordinate with any Primal (standard, custom, or forked)
//! using Songbird's advanced universal adapter architecture. It provides multi-instance support,
//! context-aware routing, and capability-based coordination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

// Import from biomeos-core
use biomeos_core::{
    universal_primal_provider::{
        AllocationStrategy, HealthMetrics, IsolationConfig, ResourceManagementConfig,
        TeamWorkspaceConfig,
    },
    BiomeError, BiomeOSInstanceConfig, BiomeOSPrimalProvider, BiomeOSPrimalRegistry, BiomeResult,
    DynamicPortInfo, NetworkLocation, PrimalCapability, PrimalContext, PrimalHealth,
    PrimalRequest, PrimalResponse, Priority, RequestType, SecurityLevel,
};

/// Enhanced Universal Adapter for biomeOS (Songbird-compatible)
pub struct BiomeOSUniversalAdapter {
    /// HTTP client for making requests
    client: Client,

    /// Primal provider registry
    primal_registry: Arc<BiomeOSPrimalRegistry>,

    /// biomeOS provider instance
    biomeos_provider: Arc<BiomeOSPrimalProvider>,

    /// Federation configuration
    federation_config: FederationConfig,

    /// Active coordination sessions
    active_sessions: Arc<RwLock<HashMap<String, CoordinationSession>>>,

    /// Context-aware routing table
    context_routing: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

/// Federation configuration for biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Federation identity
    pub identity: FederationIdentity,
    /// Known primals configuration
    pub primals: HashMap<String, PrimalConfig>,
    /// Discovery settings
    pub discovery: DiscoveryConfig,
    /// Security settings
    pub security: SecurityConfig,
}

/// Federation identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationIdentity {
    pub federation_id: String,
    pub capabilities: Vec<String>,
    pub endpoints: HashMap<String, String>,
    pub supported_api_versions: Vec<String>,
    pub federation_info: FederationCapabilities,
}

/// Federation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationCapabilities {
    pub max_biomes: u32,
    pub supported_runtimes: Vec<String>,
    pub resource_management: bool,
    pub multi_team_isolation: bool,
    pub cross_primal_coordination: bool,
    pub context_aware_routing: bool,
    pub multi_instance_support: bool,
}

/// Enhanced primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Whether this Primal is enabled for coordination
    pub enabled: bool,

    /// Network endpoint for coordination
    pub endpoint: Option<String>,

    /// Coordination capabilities this Primal provides
    pub capabilities: Vec<PrimalCapability>,

    /// API version supported by this Primal
    pub api_version: String,

    /// Context constraints for this primal
    pub context_constraints: Vec<ContextConstraint>,

    /// Multi-instance configuration
    pub multi_instance: MultiInstanceConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Context constraint for primal routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConstraint {
    /// Field to constrain
    pub field: String,
    /// Constraint operator
    pub operator: ConstraintOperator,
    /// Value to compare against
    pub value: String,
}

/// Constraint operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
}

/// Multi-instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceConfig {
    /// Whether multi-instance is enabled
    pub enabled: bool,
    /// Maximum instances per user
    pub max_instances_per_user: u32,
    /// Maximum instances per team
    pub max_instances_per_team: u32,
    /// Instance creation strategy
    pub creation_strategy: InstanceCreationStrategy,
}

/// Instance creation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceCreationStrategy {
    /// Create on demand
    OnDemand,
    /// Pre-create instances
    PreCreate,
    /// Share instances
    Shared,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Discovery method
    pub method: DiscoveryMethod,
    /// Discovery interval
    pub interval_seconds: u64,
    /// Discovery timeout
    pub timeout_seconds: u64,
    /// Auto-discovery enabled
    pub auto_discovery: bool,
}

/// Discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// DNS-based discovery
    Dns,
    /// mDNS discovery
    Mdns,
    /// Static configuration
    Static,
    /// Consul-based discovery
    Consul,
    /// Kubernetes-based discovery
    Kubernetes,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// TLS enabled
    pub tls_enabled: bool,
    /// Authentication method
    pub auth_method: AuthMethod,
    /// Authorization enabled
    pub authorization_enabled: bool,
    /// Token validation
    pub token_validation: TokenValidation,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// API key authentication
    ApiKey,
    /// JWT authentication
    Jwt,
    /// mTLS authentication
    Mtls,
}

/// Token validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    /// Token issuer
    pub issuer: String,
    /// Token audience
    pub audience: String,
    /// Token algorithm
    pub algorithm: String,
    /// Token expiration
    pub expiration_seconds: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retries: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

/// Active coordination session
#[derive(Debug, Clone)]
pub struct CoordinationSession {
    pub session_id: String,
    pub primal_name: String,
    pub context: PrimalContext,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: SessionStatus,
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Active,
    Idle,
    Failed,
    Terminated,
}

impl BiomeOSUniversalAdapter {
    /// Create a new universal adapter with advanced features
    pub async fn new(federation_config: FederationConfig) -> BiomeResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                BiomeError::RuntimeError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Create primal registry
        let primal_registry = Arc::new(BiomeOSPrimalRegistry::new());

        // Create biomeOS provider instance
        let biomeos_config = BiomeOSInstanceConfig {
            instance_id: format!("biomeos-{}", Uuid::new_v4().simple()),
            context: PrimalContext {
                user_id: "system".to_string(),
                device_id: "biomeos-orchestrator".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: SecurityLevel::Standard,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
            base_url: "http://localhost:8080".to_string(),
            team_workspace: TeamWorkspaceConfig {
                base_dir: "/tmp/biomeos/workspaces".to_string(),
                default_quotas: HashMap::new(),
                isolation: IsolationConfig {
                    network_isolation: true,
                    filesystem_isolation: true,
                    process_isolation: true,
                },
            },
            resource_management: ResourceManagementConfig {
                cpu_allocation: AllocationStrategy::FairShare,
                memory_allocation: AllocationStrategy::FairShare,
                storage_allocation: AllocationStrategy::FairShare,
            },
        };

        let biomeos_provider = Arc::new(BiomeOSPrimalProvider::new(biomeos_config));

        // Register biomeOS provider
        primal_registry
            .register_provider(biomeos_provider.clone())
            .await?;

        Ok(Self {
            client,
            primal_registry,
            biomeos_provider,
            federation_config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            context_routing: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Auto-discover primals using multiple methods
    pub async fn auto_discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        info!("Starting auto-discovery of primals with advanced features");

        let mut discovered = Vec::new();

        match self.federation_config.discovery.method {
            DiscoveryMethod::Static => {
                discovered.extend(self.discover_static_primals().await?);
            }
            DiscoveryMethod::Dns => {
                discovered.extend(self.discover_dns_primals().await?);
            }
            DiscoveryMethod::Mdns => {
                discovered.extend(self.discover_mdns_primals().await?);
            }
            DiscoveryMethod::Consul => {
                discovered.extend(self.discover_consul_primals().await?);
            }
            DiscoveryMethod::Kubernetes => {
                discovered.extend(self.discover_kubernetes_primals().await?);
            }
        }

        info!(
            "Auto-discovery completed. Found {} primals",
            discovered.len()
        );
        Ok(discovered)
    }

    /// Discover primals from static configuration
    async fn discover_static_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let mut discovered = Vec::new();

        for (primal_name, primal_config) in &self.federation_config.primals {
            if !primal_config.enabled {
                continue;
            }

            if let Some(endpoint) = &primal_config.endpoint {
                let health = self.check_primal_health(endpoint).await?;

                discovered.push(DiscoveredPrimal {
                    id: primal_name.clone(),
                    instance_id: format!("{}-{}", primal_name, Uuid::new_v4().simple()),
                    primal_type: primal_name.clone(),
                    capabilities: primal_config.capabilities.clone(),
                    endpoint: endpoint.clone(),
                    health,
                    context: PrimalContext {
                        user_id: "system".to_string(),
                        device_id: "auto-discovered".to_string(),
                        session_id: Uuid::new_v4().to_string(),
                        network_location: NetworkLocation {
                            ip_address: "127.0.0.1".to_string(),
                            subnet: None,
                            network_id: None,
                            geo_location: None,
                        },
                        security_level: SecurityLevel::Standard,
                        biome_id: None,
                        team_id: None,
                        metadata: HashMap::new(),
                    },
                    port_info: None,
                });
            }
        }

        Ok(discovered)
    }

    /// Discover primals via DNS
    async fn discover_dns_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        // Implementation for DNS-based discovery
        info!("DNS-based discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Discover primals via mDNS
    async fn discover_mdns_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        // Implementation for mDNS-based discovery
        info!("mDNS-based discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Discover primals via Consul
    async fn discover_consul_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        // Implementation for Consul-based discovery
        info!("Consul-based discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Discover primals via Kubernetes
    async fn discover_kubernetes_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        // Implementation for Kubernetes-based discovery
        info!("Kubernetes-based discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Check primal health
    async fn check_primal_health(&self, endpoint: &str) -> BiomeResult<PrimalHealth> {
        let health_endpoint = format!("{}/api/v1/health", endpoint);

        match self.client.get(&health_endpoint).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let _health_data: serde_json::Value = response.json().await.map_err(|e| {
                        BiomeError::RuntimeError(format!("Failed to parse health response: {}", e))
                    })?;

                    Ok(PrimalHealth {
                        status: biomeos_core::HealthStatus::Healthy,
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
                    })
                } else {
                    Ok(PrimalHealth {
                        status: biomeos_core::HealthStatus::Unhealthy,
                        health_score: 0.0,
                        last_check: Utc::now(),
                        details: HashMap::new(),
                        metrics: HealthMetrics {
                            cpu_usage: 0.0,
                            memory_mb: 0.0,
                            response_time_ms: 0.0,
                            error_rate: 0.0,
                            active_connections: 0,
                        },
                    })
                }
            }
            Err(e) => {
                warn!("Health check failed for {}: {}", endpoint, e);
                Ok(PrimalHealth {
                    status: biomeos_core::HealthStatus::Unhealthy,
                    health_score: 0.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: HealthMetrics {
                        cpu_usage: 0.0,
                        memory_mb: 0.0,
                        response_time_ms: 0.0,
                        error_rate: 0.0,
                        active_connections: 0,
                    },
                })
            }
        }
    }

    /// Route request to appropriate primal based on context and capabilities
    pub async fn route_request(&self, request: PrimalRequest) -> BiomeResult<PrimalResponse> {
        // Find appropriate primal providers
        let providers = self.primal_registry.find_by_context(&request.context).await;

        if providers.is_empty() {
            return Err(BiomeError::PrimalNotFound(format!(
                "No primal providers found for context: {:?}",
                request.context
            )));
        }

        // Route to the first available provider (could be enhanced with load balancing)
        let provider = &providers[0];
        provider.handle_primal_request(request).await
    }

    /// Get federation status
    pub async fn get_federation_status(&self) -> FederationStatus {
        let sessions = self.active_sessions.read().await;

        FederationStatus {
            federation_id: self.federation_config.identity.federation_id.clone(),
            active_sessions: sessions.len(),
            total_primals: self.federation_config.primals.len(),
            healthy_primals: 0, // Would need to check each primal
            last_discovery: Utc::now(),
        }
    }
}

/// Discovered primal information
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    pub id: String,
    pub instance_id: String,
    pub primal_type: String,
    pub capabilities: Vec<PrimalCapability>,
    pub endpoint: String,
    pub health: PrimalHealth,
    pub context: PrimalContext,
    pub port_info: Option<DynamicPortInfo>,
}

/// Federation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    pub federation_id: String,
    pub active_sessions: usize,
    pub total_primals: usize,
    pub healthy_primals: usize,
    pub last_discovery: DateTime<Utc>,
}

/// Universal coordination interface
#[async_trait]
pub trait UniversalCoordination {
    /// Deploy a biome across multiple primals
    async fn deploy_biome(
        &self,
        manifest: serde_json::Value,
        context: PrimalContext,
    ) -> BiomeResult<String>;

    /// Get deployment status
    async fn get_deployment_status(&self, deployment_id: &str) -> BiomeResult<DeploymentStatus>;

    /// Scale deployment resources
    async fn scale_deployment(
        &self,
        deployment_id: &str,
        scale_config: ScaleConfig,
    ) -> BiomeResult<()>;

    /// Remove deployment
    async fn remove_deployment(&self, deployment_id: &str) -> BiomeResult<()>;
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub deployment_id: String,
    pub status: String,
    pub health: biomeos_core::HealthStatus,
    pub primal_statuses: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConfig {
    pub replicas: Option<u32>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub auto_scale: bool,
}

#[async_trait]
impl UniversalCoordination for BiomeOSUniversalAdapter {
    async fn deploy_biome(
        &self,
        manifest: serde_json::Value,
        context: PrimalContext,
    ) -> BiomeResult<String> {
        let deployment_id = Uuid::new_v4().to_string();

        // Create deployment request
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: RequestType::Deploy,
            operation: "deploy_biome".to_string(),
            payload: manifest,
            context,
            priority: Priority::Normal,
            timestamp: Utc::now(),
        };

        // Route to appropriate primal
        let _response = self.route_request(request).await?;

        Ok(deployment_id)
    }

    async fn get_deployment_status(&self, deployment_id: &str) -> BiomeResult<DeploymentStatus> {
        Ok(DeploymentStatus {
            deployment_id: deployment_id.to_string(),
            status: "running".to_string(),
            health: biomeos_core::HealthStatus::Healthy,
            primal_statuses: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn scale_deployment(
        &self,
        _deployment_id: &str,
        _scale_config: ScaleConfig,
    ) -> BiomeResult<()> {
        // Implementation would go here
        Ok(())
    }

    async fn remove_deployment(&self, _deployment_id: &str) -> BiomeResult<()> {
        // Implementation would go here
        Ok(())
    }
}
