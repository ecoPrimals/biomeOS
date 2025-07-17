//! # Universal Primal Interface System
//!
//! This module provides the universal, agnostic interface for any system to
//! participate as a "Primal" in the biomeOS ecosystem. This replaces hard-coded
//! Primal-specific implementations with a capability-based discovery system.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::{BiomeResult, HealthStatus};

/// Universal Primal Provider trait - any system can implement this
/// This replaces the hard-coded "toadstool", "songbird", "nestgate", etc. approach
#[async_trait]
pub trait UniversalPrimalProvider: Send + Sync {
    /// Dynamic primal identifier (not limited to 5 known names)
    fn primal_id(&self) -> &str;

    /// Self-declared primal type (could be anything)
    fn primal_type(&self) -> &str;

    /// Human-readable name and description
    fn metadata(&self) -> PrimalMetadata;

    /// Capabilities this primal provides to the ecosystem
    fn capabilities(&self) -> Vec<Capability>;

    /// Dependencies this primal requires (by capability, not name)
    fn dependencies(&self) -> Vec<CapabilityRequirement>;

    /// Network endpoints this primal exposes
    fn endpoints(&self) -> Vec<PrimalEndpoint>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> BiomeResult<()>;

    /// Start the primal services
    async fn start(&mut self) -> BiomeResult<()>;

    /// Stop the primal services gracefully
    async fn stop(&mut self) -> BiomeResult<()>;

    /// Check health status
    async fn health_check(&self) -> BiomeResult<HealthStatus>;

    /// Handle capability requests generically
    async fn handle_capability_request(
        &self,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse>;

    /// Handle coordination with other primals
    async fn coordinate_with_peer(
        &self,
        peer: &PrimalPeer,
        request: CoordinationRequest,
    ) -> BiomeResult<CoordinationResponse>;

    /// Get current metrics/telemetry
    async fn get_metrics(&self) -> BiomeResult<PrimalMetrics>;

    /// Handle events from the ecosystem
    async fn handle_ecosystem_event(&mut self, event: EcosystemEvent) -> BiomeResult<()>;
}

/// Primal metadata for discovery and coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Human-readable name
    pub name: String,
    /// Description of what this primal does
    pub description: String,
    /// Version information
    pub version: String,
    /// Maintainer/author information
    pub maintainer: Option<String>,
    /// Additional metadata tags
    pub tags: HashMap<String, String>,
}

/// Universal capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    /// Capability name (e.g., "compute.orchestration", "storage.zfs", "ai.inference")
    pub name: String,
    /// Version of this capability
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Category for organization
    pub category: CapabilityCategory,
    /// Parameters this capability accepts
    pub parameters: HashMap<String, ParameterSpec>,
    /// Performance characteristics
    pub performance: PerformanceSpec,
    /// Dependencies this capability requires
    pub dependencies: Vec<CapabilityRequirement>,
}

/// Capability categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityCategory {
    /// Compute and orchestration capabilities
    Compute,
    /// Storage and data management
    Storage,
    /// Networking and communication
    Networking,
    /// Security and authentication
    Security,
    /// AI and machine learning
    AI,
    /// Monitoring and observability
    Monitoring,
    /// Custom category
    Custom(String),
}

/// Parameter specification for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterSpec {
    /// Parameter type (string, number, boolean, array, object)
    pub param_type: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<serde_json::Value>,
    /// Human-readable description
    pub description: String,
    /// Validation constraints
    pub constraints: Vec<Constraint>,
}

/// Performance specifications for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceSpec {
    /// Expected latency range
    pub latency_ms: Option<(u64, u64)>,
    /// Throughput capacity
    pub throughput: Option<String>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Scaling characteristics
    pub scaling: ScalingSpec,
}

/// Resource requirements for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceRequirements {
    /// CPU requirements
    pub cpu: Option<String>,
    /// Memory requirements
    pub memory: Option<String>,
    /// Storage requirements
    pub storage: Option<String>,
    /// Network bandwidth requirements
    pub network: Option<String>,
    /// GPU requirements
    pub gpu: Option<String>,
}

/// Scaling specifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingSpec {
    /// Minimum instances
    pub min_instances: u32,
    /// Maximum instances
    pub max_instances: u32,
    /// Auto-scaling supported
    pub auto_scaling: bool,
    /// Scaling triggers
    pub triggers: Vec<String>,
}

/// Constraint for parameter validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Constraint {
    /// Constraint type (regex, range, enum, etc.)
    pub constraint_type: String,
    /// Constraint value
    pub value: serde_json::Value,
    /// Error message if constraint fails
    pub error_message: String,
}

/// Capability requirement (dependency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityRequirement {
    /// Required capability name
    pub capability: String,
    /// Minimum version required
    pub min_version: String,
    /// Maximum version supported
    pub max_version: Option<String>,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Constraints on the capability
    pub constraints: Vec<Constraint>,
    /// Fallback capability if unavailable
    pub fallback: Option<String>,
}

/// Network endpoint exposed by a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Endpoint name/identifier
    pub name: String,
    /// Protocol (http, grpc, websocket, tcp, etc.)
    pub protocol: String,
    /// Host or IP address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Path prefix for HTTP endpoints
    pub path: Option<String>,
    /// Whether this endpoint requires authentication
    pub requires_auth: bool,
    /// SSL/TLS configuration
    pub tls: Option<TlsConfig>,
    /// Health check endpoint
    pub health_check: Option<String>,
}

/// TLS configuration for endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether TLS is enabled
    pub enabled: bool,
    /// Certificate path
    pub cert_path: Option<String>,
    /// Private key path
    pub key_path: Option<String>,
    /// CA certificate path
    pub ca_path: Option<String>,
}

/// Capability request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    /// Unique request ID
    pub id: Uuid,
    /// Capability being requested
    pub capability: String,
    /// Operation to perform
    pub operation: String,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request context
    pub context: RequestContext,
    /// Priority level
    pub priority: RequestPriority,
    /// Timeout for the request
    pub timeout: Option<Duration>,
    /// Timestamp when request was created
    pub timestamp: DateTime<Utc>,
}

/// Response to a capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    /// Request ID this response is for
    pub request_id: Uuid,
    /// Whether the request was successful
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error information if failed
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp when response was created
    pub timestamp: DateTime<Utc>,
}

/// Request context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Requesting primal ID
    pub requester_id: String,
    /// Biome ID this request is for
    pub biome_id: Option<String>,
    /// User context
    pub user_context: Option<String>,
    /// Security context
    pub security_context: Option<String>,
    /// Tracing context
    pub trace_id: Option<String>,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    /// Critical system operations
    Critical,
    /// High priority operations
    High,
    /// Normal priority operations
    Normal,
    /// Low priority operations
    Low,
    /// Background operations
    Background,
}

/// Peer primal information for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalPeer {
    /// Peer primal ID
    pub id: String,
    /// Peer primal type
    pub primal_type: String,
    /// Peer capabilities
    pub capabilities: Vec<Capability>,
    /// Peer endpoints
    pub endpoints: Vec<PrimalEndpoint>,
    /// Peer metadata
    pub metadata: PrimalMetadata,
}

/// Coordination request between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequest {
    /// Request ID
    pub id: Uuid,
    /// Coordination type
    pub coordination_type: CoordinationType,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request context
    pub context: RequestContext,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Coordination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResponse {
    /// Request ID
    pub request_id: Uuid,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error information
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Types of coordination between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationType {
    /// Service discovery and registration
    ServiceDiscovery,
    /// Resource sharing and allocation
    ResourceSharing,
    /// Health monitoring and reporting
    HealthMonitoring,
    /// Security coordination
    SecurityCoordination,
    /// Data synchronization
    DataSync,
    /// Custom coordination type
    Custom(String),
}

/// Primal metrics and telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Primal ID
    pub primal_id: String,
    /// Metrics timestamp
    pub timestamp: DateTime<Utc>,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Resource utilization
    pub resources: ResourceMetrics,
    /// Health metrics
    pub health: HealthMetrics,
    /// Custom metrics
    pub custom: HashMap<String, serde_json::Value>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Request latency statistics
    pub latency_ms: LatencyStats,
    /// Throughput statistics
    pub throughput: ThroughputStats,
    /// Error rates
    pub error_rate: f64,
    /// Uptime percentage
    pub uptime: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU utilization percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Storage usage in bytes
    pub storage_bytes: u64,
    /// Network I/O statistics
    pub network_io: NetworkIoStats,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall health status
    pub status: HealthStatus,
    /// Health checks results
    pub checks: Vec<HealthCheck>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Average latency
    pub avg_ms: f64,
    /// 50th percentile
    pub p50_ms: f64,
    /// 95th percentile
    pub p95_ms: f64,
    /// 99th percentile
    pub p99_ms: f64,
    /// Maximum latency
    pub max_ms: f64,
}

/// Throughput statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Requests per second
    pub rps: f64,
    /// Bytes per second
    pub bps: f64,
    /// Operations per second
    pub ops: f64,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoStats {
    /// Bytes received
    pub bytes_rx: u64,
    /// Bytes transmitted
    pub bytes_tx: u64,
    /// Packets received
    pub packets_rx: u64,
    /// Packets transmitted
    pub packets_tx: u64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,
    /// Check status
    pub status: HealthStatus,
    /// Check message
    pub message: String,
    /// Check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Ecosystem event for primal coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type
    pub event_type: EcosystemEventType,
    /// Event source primal
    pub source: String,
    /// Event data
    pub data: serde_json::Value,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

/// Types of ecosystem events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemEventType {
    /// Primal joined ecosystem
    PrimalJoined,
    /// Primal left ecosystem
    PrimalLeft,
    /// Capability added
    CapabilityAdded,
    /// Capability removed
    CapabilityRemoved,
    /// Health status changed
    HealthChanged,
    /// Resource allocation changed
    ResourceChanged,
    /// Security event
    SecurityEvent,
    /// Custom event
    Custom(String),
}

/// Discovered primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal ID
    pub id: String,
    /// Primal type
    pub primal_type: String,
    /// Primal capabilities
    pub capabilities: Vec<Capability>,
    /// Primal endpoints
    pub endpoints: Vec<PrimalEndpoint>,
    /// Primal metadata
    pub metadata: PrimalMetadata,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Discovery source
    pub discovery_source: String,
}

/// Universal discovery service for finding primals
#[async_trait]
pub trait UniversalDiscoveryService: Send + Sync {
    /// Auto-discover primals on the network
    async fn auto_discover(&self) -> BiomeResult<Vec<DiscoveredPrimal>>;

    /// Discover primals by specific capabilities
    async fn discover_by_capabilities(
        &self,
        capabilities: &[String],
    ) -> BiomeResult<Vec<DiscoveredPrimal>>;

    /// Discover primals by category
    async fn discover_by_category(
        &self,
        category: CapabilityCategory,
    ) -> BiomeResult<Vec<DiscoveredPrimal>>;

    /// Register a primal for discovery
    async fn register_primal(&self, primal: &dyn UniversalPrimalProvider) -> BiomeResult<()>;

    /// Unregister a primal
    async fn unregister_primal(&self, primal_id: &str) -> BiomeResult<()>;

    /// Get all discovered primals
    async fn get_all_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>>;
}

/// Default implementation of discovery service
pub struct DefaultDiscoveryService {
    discovered_primals: HashMap<String, DiscoveredPrimal>,
    capability_index: HashMap<String, Vec<String>>,
}

impl DefaultDiscoveryService {
    pub fn new() -> Self {
        Self {
            discovered_primals: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }
}

impl Default for DefaultDiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UniversalDiscoveryService for DefaultDiscoveryService {
    async fn auto_discover(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        // TODO: Implement network discovery
        // This would scan the network for primals implementing the universal interface
        Ok(self.discovered_primals.values().cloned().collect())
    }

    async fn discover_by_capabilities(
        &self,
        capabilities: &[String],
    ) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let mut results = Vec::new();

        for capability in capabilities {
            if let Some(primal_ids) = self.capability_index.get(capability) {
                for primal_id in primal_ids {
                    if let Some(primal) = self.discovered_primals.get(primal_id) {
                        results.push(primal.clone());
                    }
                }
            }
        }

        Ok(results)
    }

    async fn discover_by_category(
        &self,
        category: CapabilityCategory,
    ) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let results = self
            .discovered_primals
            .values()
            .filter(|primal| {
                primal
                    .capabilities
                    .iter()
                    .any(|cap| cap.category == category)
            })
            .cloned()
            .collect();

        Ok(results)
    }

    async fn register_primal(&self, _primal: &dyn UniversalPrimalProvider) -> BiomeResult<()> {
        // TODO: Implement registration logic
        Ok(())
    }

    async fn unregister_primal(&self, _primal_id: &str) -> BiomeResult<()> {
        // TODO: Implement unregistration logic
        Ok(())
    }

    async fn get_all_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        Ok(self.discovered_primals.values().cloned().collect())
    }
}
