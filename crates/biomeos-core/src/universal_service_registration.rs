// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Universal Service Registration Standard Implementation
//!
//! Implements the universal API standards for ecosystem integration
//! as defined in handOff/ECOSYSTEM_API_STANDARDIZATION_GUIDE_UNIVERSAL.md

use std::collections::HashMap;
use biomeos_primal_sdk::{PrimalCapability, PrimalType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use biomeos_types::primal::HealthCheckConfig;

// Re-export the unified primal service trait from biomeos-types
pub use biomeos_types::{
    UniversalPrimalService,
    UniversalServiceRequest as UnifiedRequest,
    UniversalServiceResponse as UnifiedResponse,
    UniversalServiceRegistration as UnifiedRegistration,
    PrimalServiceMetadata,
    ServiceStatus as UnifiedServiceStatus,
};

/// Universal Service Registration - ALL PARTICIPANTS MUST IMPLEMENT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    /// Unique service identifier (UUID)
    pub service_id: Uuid,

    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Capabilities this service provides (using biomeos primal capabilities)
    pub capabilities: Vec<PrimalCapability>,

    /// Resource requirements and limits
    pub resources: ResourceSpec,

    /// API endpoints (dynamically discovered)
    pub endpoints: Vec<ServiceEndpoint>,

    /// Integration preferences
    pub integration: IntegrationPreferences,

    /// Extension points for custom data
    pub extensions: HashMap<String, serde_json::Value>,

    /// Registration timestamp
    pub registration_timestamp: DateTime<Utc>,

    /// Service version
    pub service_version: String,

    /// Instance identifier for multi-instance support
    pub instance_id: String,

    /// Priority level for load balancing
    pub priority: u8,

    /// Primal type information (using universal type)
    pub primal_type: PrimalType,
}

/// Service metadata for legacy services transitioning to unified system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Human-readable service name
    pub name: String,

    /// Service category (extensible)
    pub category: ServiceCategory,

    /// Version information
    pub version: String,

    /// Description and documentation
    pub description: String,

    /// Maintainer information
    pub maintainer: ContactInfo,

    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Open, extensible service categories - NO HARDCODED PRIMAL NAMES
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory {
    /// Computational services
    Compute {
        specialties: Vec<String>,
        resource_types: Vec<String>,
    },

    /// Storage and data services
    Storage {
        types: Vec<String>,
        persistence_levels: Vec<String>,
    },

    /// Security and identity services
    Security {
        domains: Vec<String>,
        trust_models: Vec<String>,
    },

    /// Network and communication services
    Network {
        layers: Vec<String>,
        protocols: Vec<String>,
    },

    /// Orchestration and coordination services
    Orchestration {
        scopes: Vec<String>,
        coordination_patterns: Vec<String>,
    },

    /// Artificial intelligence services
    Intelligence {
        modalities: Vec<String>,
        model_types: Vec<String>,
    },

    /// Monitoring and observability services
    Monitoring {
        metrics_types: Vec<String>,
        collection_methods: Vec<String>,
    },

    /// Gaming and entertainment services
    Gaming {
        game_types: Vec<String>,
        platforms: Vec<String>,
    },

    /// Blockchain and distributed ledger services
    Blockchain {
        consensus_mechanisms: Vec<String>,
        token_standards: Vec<String>,
    },

    /// IoT and device management services
    IoT {
        device_types: Vec<String>,
        communication_protocols: Vec<String>,
    },

    /// Community-defined custom categories (completely open)
    Custom {
        category: String,
        subcategories: Vec<String>,
        domain_specific_attributes: HashMap<String, serde_json::Value>,
    },
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: String,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub website: Option<String>,
}

/// Compute resources specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResources {
    pub cpu_cores: Option<f64>,
    pub memory_gb: Option<f64>,
    pub gpu_units: Option<f64>,
    pub storage_gb: Option<f64>,
    pub network_bandwidth_mbps: Option<f64>,
}

/// Data consistency levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Eventual,
    Strong,
    Weak,
    Sequential,
    Linearizable,
}

/// Data durability levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurabilityLevel {
    None,
    Cached,
    Persistent,
    Replicated,
    Distributed,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// Required resources
    pub requirements: ResourceRequirement,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Scaling configuration
    pub scaling: ScalingConfig,

    /// Cost information
    pub cost: Option<CostInfo>,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    pub cpu_cores_min: f64,
    pub memory_gb_min: f64,
    pub storage_gb_min: Option<f64>,
    pub network_bandwidth_min_mbps: Option<f64>,
    pub gpu_required: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores_max: Option<f64>,
    pub memory_gb_max: Option<f64>,
    pub storage_gb_max: Option<f64>,
    pub network_bandwidth_max_mbps: Option<f64>,
    pub cost_limit_per_hour_usd: Option<f64>,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub auto_scaling_enabled: bool,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    pub cost_model: CostModel,
    pub base_cost_per_hour_usd: f64,
    pub resource_multipliers: HashMap<String, f64>,
    pub volume_discounts: Vec<VolumeDiscount>,
}

/// Cost models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostModel {
    FixedRate,
    PayPerUse,
    Tiered,
    Custom { formula: String },
}

/// Volume discount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDiscount {
    pub threshold: u64,
    pub discount_percentage: f64,
}

/// Service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint type
    pub endpoint_type: EndpointType,

    /// URL or address
    pub address: String,

    /// Port number
    pub port: Option<u16>,

    /// Protocol details
    pub protocol: ProtocolSpec,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Security configuration
    pub security: EndpointSecurity,
}

/// Endpoint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointType {
    HTTP,
    HTTPS,
    WebSocket,
    GRPC,
    TCP,
    UDP,
    Custom { protocol: String },
}

/// Protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSpec {
    pub version: String,
    pub features: Vec<String>,
    pub extensions: HashMap<String, serde_json::Value>,
}

// Removed conflicting HealthCheckConfig - using unified type from biomeos-types::primal::HealthCheckConfig

/// Endpoint security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurity {
    pub authentication_required: bool,
    pub encryption_required: bool,
    pub allowed_origins: Vec<String>,
    pub rate_limiting: Option<RateLimitConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub burst_size: u64,
    pub rate_limit_key: RateLimitKey,
}

/// Rate limiting keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitKey {
    IPAddress,
    UserID,
    APIKey,
    Custom { field: String },
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    /// Preferred discovery methods
    pub discovery_methods: Vec<DiscoveryMethod>,

    /// Communication patterns
    pub communication_patterns: Vec<CommunicationPattern>,

    /// Load balancing preferences
    pub load_balancing: LoadBalancingConfig,

    /// Monitoring and observability
    pub monitoring: MonitoringConfig,

    /// Fault tolerance
    pub fault_tolerance: FaultToleranceConfig,
}

/// Discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Static,
    DNS,
    Consul,
    Kubernetes,
    Registry { url: String },
    Broadcast,
}

/// Communication patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationPattern {
    RequestResponse,
    PublishSubscribe,
    EventDriven,
    Streaming,
    Batch,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub algorithm: LoadBalancingAlgorithm,
    pub health_check_required: bool,
    pub sticky_sessions: bool,
    pub failover_enabled: bool,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    IPHash,
    Custom { algorithm: String },
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub logging_enabled: bool,
    pub tracing_enabled: bool,
    pub health_reporting_interval_seconds: u32,
    pub performance_monitoring: bool,
}

/// Fault tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultToleranceConfig {
    pub circuit_breaker_enabled: bool,
    pub retry_policy: RetryPolicy,
    pub timeout_seconds: u32,
    pub graceful_shutdown_seconds: u32,
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

// Legacy types have been removed - use UniversalServiceRequest/Response from biomeos-types instead
