//! Types and data structures for the universal biome coordinator
//!
//! This module contains all the core data structures, enums, and types used
//! by the universal biome coordinator system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::universal_manifest::{
    DeploymentStrategy, UniversalBiomeManifest, ValidationError,
};
use crate::universal_primal::{
    Capability, CapabilityRequirement, DiscoveredPrimal, PrimalEndpoint,
    UniversalDiscoveryService,
};
use crate::{BiomeResult, HealthStatus};

/// Universal biome coordinator that works with any Primal implementation
pub struct UniversalBiomeCoordinator {
    /// Discovery service for finding primals
    pub discovery_service: Arc<dyn UniversalDiscoveryService>,
    /// Capability router for routing requests
    pub capability_router: Arc<CapabilityRouter>,
    /// Requirement matcher for matching requirements to primals
    pub requirement_matcher: Arc<RequirementMatcher>,
    /// Active deployments
    pub active_deployments: Arc<RwLock<HashMap<String, EcosystemInstance>>>,
    /// Primal clients for communication
    pub primal_clients: Arc<RwLock<HashMap<String, Arc<dyn PrimalClient>>>>,
    /// Coordinator configuration
    pub config: CoordinatorConfig,
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
    pub capability_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Primal map: primal ID -> discovered primal
    pub primal_map: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    /// Routing strategy
    pub routing_strategy: RoutingStrategy,
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

/// Requirement matcher for matching requirements to primals
pub struct RequirementMatcher {
    /// Matching algorithms
    pub algorithms: Vec<Box<dyn MatchingAlgorithm>>,
    /// Matching configuration
    pub config: MatchingConfig,
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
    /// Primal that matched
    pub primal: DiscoveredPrimal,
    /// Match score
    pub score: f64,
    /// Match details
    pub details: MatchDetails,
}

/// Match details
#[derive(Debug, Clone)]
pub struct MatchDetails {
    /// Capability match scores
    pub capability_matches: HashMap<String, f64>,
    /// Version match score
    pub version_match: f64,
    /// Performance match score
    pub performance_match: f64,
    /// Resource match score
    pub resource_match: f64,
    /// Availability match score
    pub availability_match: f64,
}

/// Primal client trait for communication with primals
#[async_trait]
pub trait PrimalClient: Send + Sync {
    /// Get primal information
    async fn get_info(&self) -> BiomeResult<PrimalInfo>;
    
    /// Execute a capability request
    async fn execute_capability(
        &self,
        capability: &str,
        request: &CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse>;
    
    /// Execute a coordination request
    async fn execute_coordination(
        &self,
        request: &CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse>;
    
    /// Get primal health status
    async fn get_health(&self) -> BiomeResult<HealthStatus>;
}

/// Primal information
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    /// Primal ID
    pub id: String,
    /// Primal name
    pub name: String,
    /// Primal version
    pub version: String,
    /// Primal type
    pub primal_type: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Primal endpoints
    pub endpoints: Vec<PrimalEndpoint>,
    /// Primal metadata
    pub metadata: HashMap<String, String>,
}

/// HTTP primal client implementation
pub struct HttpPrimalClient {
    /// Base URL for the primal
    pub base_url: String,
    /// HTTP client
    pub client: reqwest::Client,
    /// Timeout for requests
    pub timeout: Duration,
}

// Import necessary types that might be referenced
use crate::universal_primal::{
    CapabilityRequest, CapabilityResponse, CoordinationRequest, CoordinationResponse,
}; 