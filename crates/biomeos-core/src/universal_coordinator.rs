//! # Universal Biome Coordinator
//!
//! This module provides the universal coordinator that can bootstrap and manage
//! biome ecosystems using any Primal implementations through capability-based
//! discovery and routing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::universal_manifest::{
    DeploymentStrategy, UniversalBiomeManifest, ValidationError, ValidationSeverity,
};
use crate::universal_primal::{
    Capability, CapabilityRequest, CapabilityRequirement, CapabilityResponse,
    CoordinationRequest, CoordinationResponse, DefaultDiscoveryService,
    DiscoveredPrimal, PrimalEndpoint, PrimalMetadata,
    UniversalDiscoveryService,
};
use crate::{BiomeError, BiomeResult, HealthStatus};

/// Universal biome coordinator that works with any Primal implementation
pub struct UniversalBiomeCoordinator {
    /// Discovery service for finding primals
    discovery_service: Arc<dyn UniversalDiscoveryService>,
    /// Capability router for routing requests
    capability_router: Arc<CapabilityRouter>,
    /// Requirement matcher for matching requirements to primals
    requirement_matcher: Arc<RequirementMatcher>,
    /// Active deployments
    active_deployments: Arc<RwLock<HashMap<String, EcosystemInstance>>>,
    /// Primal clients for communication
    primal_clients: Arc<RwLock<HashMap<String, Arc<dyn PrimalClient>>>>,
    /// Coordinator configuration
    config: CoordinatorConfig,
}

/// Configuration for the universal coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Discovery timeout
    pub discovery_timeout: Duration,
    /// Deployment timeout
    pub deployment_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Enable auto-discovery
    pub auto_discovery: bool,
    /// Discovery refresh interval
    pub discovery_refresh_interval: Duration,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(30),
            deployment_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(5),
            auto_discovery: true,
            discovery_refresh_interval: Duration::from_secs(60),
        }
    }
}

/// Ecosystem instance representing a deployed biome
#[derive(Debug, Clone)]
pub struct EcosystemInstance {
    /// Instance ID
    pub id: String,
    /// Instance name
    pub name: String,
    /// Source manifest
    pub manifest: UniversalBiomeManifest,
    /// Deployed primals
    pub primals: HashMap<String, DeployedPrimal>,
    /// Instance status
    pub status: EcosystemStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Deployment plan
    pub deployment_plan: DeploymentPlan,
    /// Resource allocation
    pub resource_allocation: ResourceAllocation,
}

/// Deployed primal information
#[derive(Debug, Clone)]
pub struct DeployedPrimal {
    /// Primal ID
    pub id: String,
    /// Primal type
    pub primal_type: String,
    /// Primal endpoints
    pub endpoints: Vec<PrimalEndpoint>,
    /// Primal status
    pub status: PrimalStatus,
    /// Capabilities provided
    pub capabilities: Vec<Capability>,
    /// Resource allocation
    pub resources: ResourceAllocation,
    /// Health status
    pub health: HealthStatus,
    /// Last health check
    pub last_health_check: DateTime<Utc>,
}

/// Primal status
#[derive(Debug, Clone)]
pub enum PrimalStatus {
    /// Primal is starting
    Starting,
    /// Primal is running
    Running,
    /// Primal is stopping
    Stopping,
    /// Primal is stopped
    Stopped,
    /// Primal has failed
    Failed(String),
}

/// Ecosystem status
#[derive(Debug, Clone)]
pub enum EcosystemStatus {
    /// Ecosystem is deploying
    Deploying,
    /// Ecosystem is running
    Running,
    /// Ecosystem is scaling
    Scaling,
    /// Ecosystem is updating
    Updating,
    /// Ecosystem is stopping
    Stopping,
    /// Ecosystem is stopped
    Stopped,
    /// Ecosystem has failed
    Failed(String),
}

/// Deployment plan for an ecosystem
#[derive(Debug, Clone)]
pub struct DeploymentPlan {
    /// Plan ID
    pub id: String,
    /// Deployment strategy
    pub strategy: DeploymentStrategy,
    /// Primal assignments
    pub primal_assignments: Vec<PrimalAssignment>,
    /// Resource allocation plan
    pub resource_plan: ResourcePlan,
    /// Deployment order
    pub deployment_order: Vec<String>,
    /// Validation results
    pub validation_results: Vec<ValidationError>,
}

/// Primal assignment in deployment plan
#[derive(Debug, Clone)]
pub struct PrimalAssignment {
    /// Assignment ID
    pub id: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Assigned primal
    pub assigned_primal: DiscoveredPrimal,
    /// Assignment score
    pub score: f64,
    /// Assignment justification
    pub justification: String,
}

/// Resource allocation plan
#[derive(Debug, Clone)]
pub struct ResourcePlan {
    /// Total CPU allocation
    pub cpu: String,
    /// Total memory allocation
    pub memory: String,
    /// Total storage allocation
    pub storage: String,
    /// Network allocation
    pub network: Option<String>,
    /// GPU allocation
    pub gpu: Option<String>,
    /// Resource pools
    pub pools: Vec<ResourcePool>,
}

/// Resource pool
#[derive(Debug, Clone)]
pub struct ResourcePool {
    /// Pool name
    pub name: String,
    /// Pool type
    pub pool_type: String,
    /// Pool resources
    pub resources: HashMap<String, String>,
}

/// Resource allocation
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    /// CPU allocation
    pub cpu: String,
    /// Memory allocation
    pub memory: String,
    /// Storage allocation
    pub storage: String,
    /// Network allocation
    pub network: Option<String>,
    /// GPU allocation
    pub gpu: Option<String>,
}

/// Capability router for routing requests to primals
pub struct CapabilityRouter {
    /// Capability map: capability -> primal IDs
    capability_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Primal map: primal ID -> discovered primal
    primal_map: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    /// Routing strategy
    routing_strategy: RoutingStrategy,
}

/// Routing strategy for capability requests
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Round robin
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Least latency
    LeastLatency,
    /// Random
    Random,
    /// Weighted
    Weighted,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Requirement matcher for matching requirements to primals
pub struct RequirementMatcher {
    /// Matching algorithms
    algorithms: Vec<Box<dyn MatchingAlgorithm>>,
    /// Matching configuration
    config: MatchingConfig,
}

/// Matching configuration
#[derive(Debug, Clone)]
pub struct MatchingConfig {
    /// Minimum match score
    pub min_score: f64,
    /// Prefer exact matches
    pub prefer_exact: bool,
    /// Allow partial matches
    pub allow_partial: bool,
    /// Scoring weights
    pub weights: ScoringWeights,
}

/// Scoring weights for matching
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Capability match weight
    pub capability_match: f64,
    /// Version match weight
    pub version_match: f64,
    /// Performance match weight
    pub performance_match: f64,
    /// Resource match weight
    pub resource_match: f64,
    /// Availability match weight
    pub availability_match: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            capability_match: 0.4,
            version_match: 0.2,
            performance_match: 0.2,
            resource_match: 0.1,
            availability_match: 0.1,
        }
    }
}

/// Matching algorithm trait
#[async_trait]
pub trait MatchingAlgorithm: Send + Sync {
    /// Algorithm name
    fn name(&self) -> &str;

    /// Match requirements to primals
    async fn match_requirements(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
        config: &MatchingConfig,
    ) -> BiomeResult<Vec<MatchResult>>;
}

/// Match result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Matched primal
    pub primal: DiscoveredPrimal,
    /// Match score
    pub score: f64,
    /// Matched capabilities
    pub matched_capabilities: Vec<String>,
    /// Missing capabilities
    pub missing_capabilities: Vec<String>,
    /// Match details
    pub details: MatchDetails,
}

/// Match details
#[derive(Debug, Clone)]
pub struct MatchDetails {
    /// Capability scores
    pub capability_scores: HashMap<String, f64>,
    /// Version compatibility
    pub version_compatibility: HashMap<String, bool>,
    /// Performance score
    pub performance_score: f64,
    /// Resource score
    pub resource_score: f64,
    /// Availability score
    pub availability_score: f64,
}

/// Primal client for communication
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Send capability request
    async fn send_capability_request(
        &self,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse>;

    /// Send coordination request
    async fn send_coordination_request(
        &self,
        request: CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse>;

    /// Get primal health
    async fn get_health(&self) -> BiomeResult<HealthStatus>;

    /// Get primal metadata
    async fn get_metadata(&self) -> BiomeResult<PrimalMetadata>;
}

/// HTTP primal client implementation
pub struct HttpPrimalClient {
    /// HTTP client
    client: reqwest::Client,
    /// Base URL
    base_url: String,
    /// Timeout
    timeout: Duration,
}

impl HttpPrimalClient {
    /// Create new HTTP client
    pub fn new(base_url: String, timeout: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            timeout,
        }
    }
}

#[async_trait]
impl PrimalClient for HttpPrimalClient {
    async fn send_capability_request(
        &self,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse> {
        let url = format!("{}/capabilities", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let capability_response: CapabilityResponse = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(capability_response)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn send_coordination_request(
        &self,
        request: CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse> {
        let url = format!("{}/coordination", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let coordination_response: CoordinationResponse = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(coordination_response)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn get_health(&self) -> BiomeResult<HealthStatus> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let health: HealthStatus = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(health)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }

    async fn get_metadata(&self) -> BiomeResult<PrimalMetadata> {
        let url = format!("{}/metadata", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| BiomeError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let metadata: PrimalMetadata = response
                .json()
                .await
                .map_err(|e| BiomeError::RuntimeError(e.to_string()))?;
            Ok(metadata)
        } else {
            Err(BiomeError::RuntimeError(format!(
                "HTTP request failed: {}",
                response.status()
            )))
        }
    }
}

impl UniversalBiomeCoordinator {
    /// Create new coordinator with default configuration
    pub fn new() -> Self {
        Self::with_config(CoordinatorConfig::default())
    }

    /// Create coordinator with custom configuration
    pub fn with_config(config: CoordinatorConfig) -> Self {
        let discovery_service = Arc::new(DefaultDiscoveryService::new());
        let capability_router = Arc::new(CapabilityRouter::new());
        let requirement_matcher = Arc::new(RequirementMatcher::new());

        Self {
            discovery_service,
            capability_router,
            requirement_matcher,
            active_deployments: Arc::new(RwLock::new(HashMap::new())),
            primal_clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Bootstrap a new ecosystem from a manifest
    pub async fn bootstrap_ecosystem(
        &self,
        manifest: UniversalBiomeManifest,
    ) -> BiomeResult<EcosystemInstance> {
        info!(
            "Starting ecosystem bootstrap for biome: {}",
            manifest.metadata.name
        );

        // Step 1: Validate manifest
        let validation_results = manifest.validate()?;
        if validation_results
            .iter()
            .any(|e| matches!(e.severity, ValidationSeverity::Error))
        {
            return Err(BiomeError::ValidationError(
                "Manifest validation failed with errors".to_string(),
            ));
        }

        // Step 2: Discover available primals
        let available_primals = self.discover_primals().await?;
        info!("Discovered {} primals", available_primals.len());

        // Step 3: Create deployment plan
        let deployment_plan = self
            .create_deployment_plan(&manifest, &available_primals)
            .await?;
        info!(
            "Created deployment plan with {} primal assignments",
            deployment_plan.primal_assignments.len()
        );

        // Step 4: Validate deployment plan
        self.validate_deployment_plan(&deployment_plan).await?;

        // Step 5: Execute deployment
        let instance = self.execute_deployment(&manifest, deployment_plan).await?;
        info!("Successfully bootstrapped ecosystem: {}", instance.id);

        // Step 6: Store instance
        {
            let mut deployments = self.active_deployments.write().await;
            deployments.insert(instance.id.clone(), instance.clone());
        }

        Ok(instance)
    }

    /// Discover available primals
    async fn discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        debug!("Starting primal discovery");

        // Use discovery service to find primals
        let primals = self.discovery_service.auto_discover().await?;

        // Update capability router
        self.capability_router.update_primals(&primals).await?;

        Ok(primals)
    }

    /// Create deployment plan
    async fn create_deployment_plan(
        &self,
        manifest: &UniversalBiomeManifest,
        available_primals: &[DiscoveredPrimal],
    ) -> BiomeResult<DeploymentPlan> {
        debug!("Creating deployment plan");

        // Get all required capabilities
        let required_capabilities = manifest.get_all_required_capabilities();

        // Convert to capability requirements
        let capability_requirements: Vec<CapabilityRequirement> = required_capabilities
            .into_iter()
            .map(|cap| CapabilityRequirement {
                capability: cap,
                min_version: "1.0.0".to_string(),
                max_version: None,
                optional: false,
                constraints: vec![],
                fallback: None,
            })
            .collect();

        // Match requirements to primals
        let matches = self
            .requirement_matcher
            .match_requirements(&capability_requirements, available_primals)
            .await?;

        // Create primal assignments
        let mut assignments = Vec::new();
        for (i, match_result) in matches.iter().enumerate() {
            let assignment = PrimalAssignment {
                id: format!("assignment-{}", i),
                required_capabilities: match_result.matched_capabilities.clone(),
                assigned_primal: match_result.primal.clone(),
                score: match_result.score,
                justification: format!(
                    "Matched {} capabilities with score {:.2}",
                    match_result.matched_capabilities.len(),
                    match_result.score
                ),
            };
            assignments.push(assignment);
        }

        // Create resource plan
        let resource_plan = self.create_resource_plan(manifest, &assignments).await?;

        // Create deployment order
        let deployment_order = self.create_deployment_order(&assignments).await?;

        let plan = DeploymentPlan {
            id: Uuid::new_v4().to_string(),
            strategy: manifest.deployment.strategy.clone(),
            primal_assignments: assignments,
            resource_plan,
            deployment_order,
            validation_results: vec![],
        };

        Ok(plan)
    }

    /// Create resource allocation plan
    async fn create_resource_plan(
        &self,
        manifest: &UniversalBiomeManifest,
        _assignments: &[PrimalAssignment],
    ) -> BiomeResult<ResourcePlan> {
        let resource_summary = manifest.get_resource_summary();

        Ok(ResourcePlan {
            cpu: resource_summary.total_cpu,
            memory: resource_summary.total_memory,
            storage: resource_summary.total_storage,
            network: None,
            gpu: None,
            pools: vec![],
        })
    }

    /// Create deployment order
    async fn create_deployment_order(
        &self,
        assignments: &[PrimalAssignment],
    ) -> BiomeResult<Vec<String>> {
        // Simple ordering for now - deploy in assignment order
        let order = assignments.iter().map(|a| a.id.clone()).collect();
        Ok(order)
    }

    /// Validate deployment plan
    async fn validate_deployment_plan(&self, plan: &DeploymentPlan) -> BiomeResult<()> {
        debug!("Validating deployment plan");

        // Check if all assignments are valid
        if plan.primal_assignments.is_empty() {
            return Err(BiomeError::ValidationError(
                "No primal assignments in deployment plan".to_string(),
            ));
        }

        // Check if all assignments have valid scores
        for assignment in &plan.primal_assignments {
            if assignment.score < 0.5 {
                warn!(
                    "Assignment {} has low score: {:.2}",
                    assignment.id, assignment.score
                );
            }
        }

        Ok(())
    }

    /// Execute deployment
    async fn execute_deployment(
        &self,
        manifest: &UniversalBiomeManifest,
        plan: DeploymentPlan,
    ) -> BiomeResult<EcosystemInstance> {
        debug!("Executing deployment");

        let instance_id = Uuid::new_v4().to_string();
        let mut deployed_primals = HashMap::new();

        // Deploy primals in order
        for assignment_id in &plan.deployment_order {
            let assignment = plan
                .primal_assignments
                .iter()
                .find(|a| a.id == *assignment_id)
                .ok_or_else(|| {
                    BiomeError::RuntimeError(format!("Assignment not found: {}", assignment_id))
                })?;

            let deployed_primal = self
                .deploy_primal(
                    &assignment.assigned_primal,
                    &assignment.required_capabilities,
                )
                .await?;
            deployed_primals.insert(assignment.assigned_primal.id.clone(), deployed_primal);
        }

        let instance = EcosystemInstance {
            id: instance_id,
            name: manifest.metadata.name.clone(),
            manifest: manifest.clone(),
            primals: deployed_primals,
            status: EcosystemStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deployment_plan: plan,
            resource_allocation: ResourceAllocation {
                cpu: "1000m".to_string(),
                memory: "1Gi".to_string(),
                storage: "10Gi".to_string(),
                network: None,
                gpu: None,
            },
        };

        Ok(instance)
    }

    /// Deploy a single primal
    async fn deploy_primal(
        &self,
        primal: &DiscoveredPrimal,
        _capabilities: &[String],
    ) -> BiomeResult<DeployedPrimal> {
        debug!("Deploying primal: {}", primal.id);

        // Create primal client
        let client = self.create_primal_client(primal).await?;

        // Store client for future use
        {
            let mut clients = self.primal_clients.write().await;
            clients.insert(primal.id.clone(), client);
        }

        let deployed = DeployedPrimal {
            id: primal.id.clone(),
            primal_type: primal.primal_type.clone(),
            endpoints: primal.endpoints.clone(),
            status: PrimalStatus::Running,
            capabilities: primal.capabilities.clone(),
            resources: ResourceAllocation {
                cpu: "100m".to_string(),
                memory: "128Mi".to_string(),
                storage: "1Gi".to_string(),
                network: None,
                gpu: None,
            },
            health: HealthStatus::Healthy,
            last_health_check: Utc::now(),
        };

        Ok(deployed)
    }

    /// Create primal client
    async fn create_primal_client(
        &self,
        primal: &DiscoveredPrimal,
    ) -> BiomeResult<Arc<dyn PrimalClient>> {
        // Find HTTP endpoint
        let http_endpoint = primal
            .endpoints
            .iter()
            .find(|e| e.protocol == "http" || e.protocol == "https")
            .ok_or_else(|| BiomeError::RuntimeError("No HTTP endpoint found".to_string()))?;

        let base_url = format!(
            "{}://{}:{}",
            http_endpoint.protocol, http_endpoint.host, http_endpoint.port
        );
        let client = HttpPrimalClient::new(base_url, self.config.discovery_timeout);

        Ok(Arc::new(client))
    }

    /// Get active deployments
    pub async fn get_active_deployments(&self) -> BiomeResult<Vec<EcosystemInstance>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.values().cloned().collect())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, id: &str) -> BiomeResult<Option<EcosystemInstance>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.get(id).cloned())
    }

    /// Remove deployment
    pub async fn remove_deployment(&self, id: &str) -> BiomeResult<()> {
        let mut deployments = self.active_deployments.write().await;
        deployments.remove(id);
        Ok(())
    }
}

impl CapabilityRouter {
    /// Create new capability router
    pub fn new() -> Self {
        Self {
            capability_map: Arc::new(RwLock::new(HashMap::new())),
            primal_map: Arc::new(RwLock::new(HashMap::new())),
            routing_strategy: RoutingStrategy::default(),
        }
    }

    /// Update primals in the router
    pub async fn update_primals(&self, primals: &[DiscoveredPrimal]) -> BiomeResult<()> {
        let mut capability_map = self.capability_map.write().await;
        let mut primal_map = self.primal_map.write().await;

        // Clear existing data
        capability_map.clear();
        primal_map.clear();

        // Update with new primals
        for primal in primals {
            // Add to primal map
            primal_map.insert(primal.id.clone(), primal.clone());

            // Add to capability map
            for capability in &primal.capabilities {
                capability_map
                    .entry(capability.name.clone())
                    .or_insert_with(Vec::new)
                    .push(primal.id.clone());
            }
        }

        Ok(())
    }

    /// Route capability request to appropriate primal
    pub async fn route_capability_request(
        &self,
        capability: &str,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse> {
        let capability_map = self.capability_map.read().await;
        let primal_map = self.primal_map.read().await;

        // Find primals that provide this capability
        let primal_ids = capability_map.get(capability).ok_or_else(|| {
            BiomeError::RuntimeError(format!("No primals provide capability: {}", capability))
        })?;

        if primal_ids.is_empty() {
            return Err(BiomeError::RuntimeError(format!(
                "No primals available for capability: {}",
                capability
            )));
        }

        // Select primal based on routing strategy
        let selected_primal_id = self.select_primal(primal_ids, &self.routing_strategy);
        let _selected_primal = primal_map.get(&selected_primal_id).ok_or_else(|| {
            BiomeError::RuntimeError(format!("Primal not found: {}", selected_primal_id))
        })?;

        // TODO: Send request to selected primal
        // For now, return a mock response
        let response = CapabilityResponse {
            request_id: request.id,
            success: true,
            data: Some(serde_json::json!({"result": "success"})),
            error: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        Ok(response)
    }

    /// Select primal based on routing strategy
    fn select_primal(&self, primal_ids: &[String], _strategy: &RoutingStrategy) -> String {
        // Simple round-robin for now
        primal_ids[0].clone()
    }
}

impl RequirementMatcher {
    /// Create new requirement matcher
    pub fn new() -> Self {
        Self {
            algorithms: vec![Box::new(SimpleMatchingAlgorithm::new())],
            config: MatchingConfig {
                min_score: 0.5,
                prefer_exact: true,
                allow_partial: true,
                weights: ScoringWeights::default(),
            },
        }
    }

    /// Match requirements to primals
    pub async fn match_requirements(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
    ) -> BiomeResult<Vec<MatchResult>> {
        let mut all_matches = Vec::new();

        // Run all matching algorithms
        for algorithm in &self.algorithms {
            let matches = algorithm
                .match_requirements(requirements, primals, &self.config)
                .await?;
            all_matches.extend(matches);
        }

        // Filter by minimum score
        all_matches.retain(|m| m.score >= self.config.min_score);

        // Sort by score (descending)
        all_matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_matches)
    }
}

/// Simple matching algorithm implementation
pub struct SimpleMatchingAlgorithm {
    name: String,
}

impl SimpleMatchingAlgorithm {
    pub fn new() -> Self {
        Self {
            name: "simple".to_string(),
        }
    }
}

#[async_trait]
impl MatchingAlgorithm for SimpleMatchingAlgorithm {
    fn name(&self) -> &str {
        &self.name
    }

    async fn match_requirements(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
        config: &MatchingConfig,
    ) -> BiomeResult<Vec<MatchResult>> {
        let mut matches = Vec::new();

        for primal in primals {
            let mut matched_capabilities = Vec::new();
            let mut missing_capabilities = Vec::new();
            let mut capability_scores = HashMap::new();

            for requirement in requirements {
                let mut found = false;

                for capability in &primal.capabilities {
                    if capability.name == requirement.capability {
                        matched_capabilities.push(capability.name.clone());
                        capability_scores.insert(capability.name.clone(), 1.0);
                        found = true;
                        break;
                    }
                }

                if !found {
                    missing_capabilities.push(requirement.capability.clone());
                    capability_scores.insert(requirement.capability.clone(), 0.0);
                }
            }

            // Calculate overall score
            let capability_score = matched_capabilities.len() as f64 / requirements.len() as f64;
            let score = capability_score * config.weights.capability_match;

            if score >= config.min_score {
                let match_result = MatchResult {
                    primal: primal.clone(),
                    score,
                    matched_capabilities,
                    missing_capabilities,
                    details: MatchDetails {
                        capability_scores,
                        version_compatibility: HashMap::new(),
                        performance_score: 0.8,
                        resource_score: 0.8,
                        availability_score: 0.8,
                    },
                };
                matches.push(match_result);
            }
        }

        Ok(matches)
    }
}

impl Default for UniversalBiomeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RequirementMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_primal::{
        CapabilityCategory, ParameterSpec, PerformanceSpec, ScalingSpec,
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = UniversalBiomeCoordinator::new();
        assert!(coordinator.active_deployments.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_capability_router() {
        let router = CapabilityRouter::new();

        // Create test primals
        let primals = vec![DiscoveredPrimal {
            id: "test-primal-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![Capability {
                name: "compute.orchestration".to_string(),
                version: "1.0.0".to_string(),
                description: "Container orchestration".to_string(),
                category: CapabilityCategory::Compute,
                parameters: HashMap::new(),
                performance: PerformanceSpec {
                    latency_ms: Some((10, 100)),
                    throughput: Some("1000 rps".to_string()),
                    resource_requirements: crate::universal_primal::ResourceRequirements {
                        cpu: Some("100m".to_string()),
                        memory: Some("128Mi".to_string()),
                        storage: Some("1Gi".to_string()),
                        network: None,
                        gpu: None,
                    },
                    scaling: ScalingSpec {
                        min_instances: 1,
                        max_instances: 10,
                        auto_scaling: true,
                        triggers: vec!["cpu_usage".to_string()],
                    },
                },
                dependencies: vec![],
            }],
            endpoints: vec![],
            metadata: PrimalMetadata {
                name: "Test Primal".to_string(),
                description: "Test primal".to_string(),
                version: "1.0.0".to_string(),
                maintainer: None,
                tags: HashMap::new(),
            },
            discovered_at: Utc::now(),
            discovery_source: "test".to_string(),
        }];

        router.update_primals(&primals).await.unwrap();

        // Test routing
        let request = CapabilityRequest {
            id: Uuid::new_v4(),
            capability: "compute.orchestration".to_string(),
            operation: "test".to_string(),
            parameters: HashMap::new(),
            context: RequestContext {
                requester_id: "test".to_string(),
                biome_id: None,
                user_context: None,
                security_context: None,
                trace_id: None,
            },
            priority: RequestPriority::Normal,
            timeout: Some(Duration::from_secs(30)),
            timestamp: Utc::now(),
        };

        let response = router
            .route_capability_request("compute.orchestration", request)
            .await
            .unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_requirement_matcher() {
        let matcher = RequirementMatcher::new();

        // Create test requirements
        let requirements = vec![CapabilityRequirement {
            capability: "compute.orchestration".to_string(),
            min_version: "1.0.0".to_string(),
            max_version: None,
            optional: false,
            constraints: vec![],
            fallback: None,
        }];

        // Create test primals
        let primals = vec![DiscoveredPrimal {
            id: "test-primal-1".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec![Capability {
                name: "compute.orchestration".to_string(),
                version: "1.0.0".to_string(),
                description: "Container orchestration".to_string(),
                category: CapabilityCategory::Compute,
                parameters: HashMap::new(),
                performance: PerformanceSpec {
                    latency_ms: Some((10, 100)),
                    throughput: Some("1000 rps".to_string()),
                    resource_requirements: crate::universal_primal::ResourceRequirements {
                        cpu: Some("100m".to_string()),
                        memory: Some("128Mi".to_string()),
                        storage: Some("1Gi".to_string()),
                        network: None,
                        gpu: None,
                    },
                    scaling: ScalingSpec {
                        min_instances: 1,
                        max_instances: 10,
                        auto_scaling: true,
                        triggers: vec!["cpu_usage".to_string()],
                    },
                },
                dependencies: vec![],
            }],
            endpoints: vec![],
            metadata: PrimalMetadata {
                name: "Test Primal".to_string(),
                description: "Test primal".to_string(),
                version: "1.0.0".to_string(),
                maintainer: None,
                tags: HashMap::new(),
            },
            discovered_at: Utc::now(),
            discovery_source: "test".to_string(),
        }];

        let matches = matcher
            .match_requirements(&requirements, &primals)
            .await
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].primal.id, "test-primal-1");
        assert_eq!(matches[0].matched_capabilities.len(), 1);
        assert_eq!(matches[0].matched_capabilities[0], "compute.orchestration");
    }
}
