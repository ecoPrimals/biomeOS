// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Core Service Definitions
//!
//! This module contains the fundamental service types including `UniversalService`,
//! `ServiceMetadata`, `ServiceSpec`, and various service type classifications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::health::Health;
use crate::primal::{PrimalCapability, PrimalType, ResourceRequirements};

// Re-export from other service modules for convenience
pub use super::health::ServiceHealth;
pub use super::networking::ServiceNetworking;
pub use super::runtime::ServiceRuntime;
pub use super::security::ServiceSecurity;

/// Universal Service Definition
///
/// This represents any service in the biomeOS ecosystem, whether it's a
/// primal, application service, infrastructure component, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalService {
    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Service specification
    pub spec: ServiceSpec,

    /// Current service status
    pub status: ServiceStatus,

    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,

    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Unique service identifier
    pub id: Uuid,

    /// Service name
    pub name: String,

    /// Service namespace
    pub namespace: Option<String>,

    /// Service version
    pub version: String,

    /// Service description
    pub description: Option<String>,

    /// Service author/maintainer
    pub author: Option<String>,

    /// Service labels for selection
    pub labels: HashMap<String, String>,

    /// Service annotations for metadata
    pub annotations: HashMap<String, String>,

    /// Service tags for categorization
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Service owner reference
    pub owner_references: Vec<OwnerReference>,
}

/// Owner reference for garbage collection and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerReference {
    /// API version of the owner
    pub api_version: String,

    /// Kind of the owner
    pub kind: String,

    /// Name of the owner
    pub name: String,

    /// UID of the owner
    pub uid: Uuid,

    /// Whether this service is controlled by the owner
    pub controller: bool,

    /// Whether owner deletion should cascade
    pub block_owner_deletion: bool,
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Service type
    pub service_type: ServiceType,

    /// Primal type this service implements (if any)
    pub primal_type: Option<PrimalType>,

    /// Capabilities this service provides
    pub capabilities: Vec<PrimalCapability>,

    /// Service runtime configuration
    pub runtime: ServiceRuntime,

    /// Resource requirements and limits
    pub resources: ResourceRequirements,

    /// Network configuration
    pub networking: ServiceNetworking,

    /// Security configuration
    pub security: ServiceSecurity,

    /// Health monitoring configuration
    pub health: ServiceHealth,

    /// Scaling configuration
    pub scaling: ServiceScaling,

    /// Configuration management
    pub config: ServiceConfiguration,

    /// Lifecycle management
    pub lifecycle: ServiceLifecycle,
}

/// Service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    /// Primal service (core biomeOS component)
    Primal {
        /// Primal category
        category: String,
        /// Primal specialization
        specialization: Option<String>,
    },

    /// Application service
    Application {
        /// Application type
        app_type: String,
        /// Application framework
        framework: Option<String>,
    },

    /// Infrastructure service
    Infrastructure {
        /// Infrastructure component type
        component: String,
        /// Provider/vendor
        provider: Option<String>,
    },

    /// Database service
    Database {
        /// Database engine
        engine: String,
        /// Database version
        version: String,
    },

    /// Message queue service
    MessageQueue {
        /// Queue system
        system: String,
        /// Queue type
        queue_type: String,
    },

    /// Cache service
    Cache {
        /// Cache system
        system: String,
        /// Cache type
        cache_type: String,
    },

    /// Load balancer service
    LoadBalancer {
        /// Load balancer type
        lb_type: String,
        /// Algorithm
        algorithm: String,
    },

    /// API gateway service
    ApiGateway {
        /// Gateway type
        gateway_type: String,
        /// Features
        features: Vec<String>,
    },

    /// Monitoring service
    Monitoring {
        /// Monitoring system
        system: String,
        /// Component type
        component: String,
    },

    /// Security service
    Security {
        /// Security component
        component: String,
        /// Security domain
        domain: String,
    },

    /// Custom service type
    Custom {
        /// Type name
        type_name: String,
        /// Type attributes
        attributes: HashMap<String, String>,
    },
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Current service phase
    pub phase: ServicePhase,

    /// Service health status
    pub health: Health,

    /// Service conditions
    pub conditions: Vec<ServiceCondition>,

    /// Replica status
    pub replicas: ReplicaStatus,

    /// Observed generation
    pub observed_generation: u64,

    /// Last update time
    pub last_update_time: DateTime<Utc>,

    /// Status message
    pub message: Option<String>,

    /// Status reason
    pub reason: Option<String>,
}

/// Service phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePhase {
    /// Service is pending
    Pending,

    /// Service is starting
    Starting,

    /// Service is running
    Running,

    /// Service is stopping
    Stopping,

    /// Service has stopped
    Stopped,

    /// Service has failed
    Failed,

    /// Service is unknown state
    Unknown,
}

/// Service condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCondition {
    /// Condition type
    pub condition_type: String,

    /// Condition status
    pub status: ConditionStatus,

    /// Last transition time
    pub last_transition_time: DateTime<Utc>,

    /// Reason for the condition
    pub reason: Option<String>,

    /// Human readable message
    pub message: Option<String>,
}

/// Condition status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    /// Condition is met
    True,
    /// Condition is not met
    False,
    /// Condition status is unknown
    Unknown,
}

/// Replica status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    /// Desired number of replicas
    pub desired: u32,

    /// Current number of replicas
    pub current: u32,

    /// Number of ready replicas
    pub ready: u32,

    /// Number of available replicas
    pub available: u32,

    /// Number of unavailable replicas
    pub unavailable: u32,
}

/// Service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint name
    pub name: String,

    /// Endpoint URL or address
    pub address: String,

    /// Endpoint port
    pub port: u16,

    /// Endpoint protocol
    pub protocol: EndpointProtocol,

    /// Whether endpoint is ready
    pub ready: bool,

    /// Endpoint metadata
    pub metadata: HashMap<String, String>,
}

/// Endpoint protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointProtocol {
    /// Plain HTTP
    Http,
    /// HTTPS (TLS)
    Https,
    /// Raw TCP
    Tcp,
    /// UDP
    Udp,
    /// gRPC
    Grpc,
    /// WebSocket
    WebSocket,
    /// Custom protocol
    Custom(String),
}

/// Service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Dependency name
    pub name: String,

    /// Dependency type
    pub dependency_type: DependencyType,

    /// Dependency condition
    pub condition: DependencyCondition,

    /// Dependency timeout
    pub timeout: Option<u32>,

    /// Dependency status
    pub status: DependencyStatus,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency (required)
    Hard,

    /// Soft dependency (optional)
    Soft,

    /// Weak dependency (hint only)
    Weak,
}

/// Dependency conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyCondition {
    /// Service started
    Started,

    /// Service ready
    Ready,

    /// Service healthy
    Healthy,

    /// Custom condition
    Custom(String),
}

/// Dependency status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyStatus {
    /// Dependency satisfied
    Satisfied,

    /// Dependency pending
    Pending,

    /// Dependency failed
    Failed,

    /// Dependency timeout
    Timeout,
}

/// Service scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceScaling {
    /// Scaling type
    pub scaling_type: ScalingType,

    /// Minimum replicas
    pub min_replicas: u32,

    /// Maximum replicas
    pub max_replicas: u32,

    /// Scaling policies
    pub policies: Vec<ScalingPolicy>,

    /// Scaling metrics
    pub metrics: Vec<ScalingMetric>,
}

/// Scaling types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingType {
    /// Manual scaling
    Manual,

    /// Horizontal pod autoscaling
    Hpa,

    /// Vertical pod autoscaling
    Vpa,

    /// Custom scaling
    Custom(String),
}

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy name
    pub name: String,

    /// Scaling direction
    pub direction: ScalingDirection,

    /// Scaling amount
    pub amount: ScalingAmount,

    /// Policy cooldown
    pub cooldown: u32,
}

/// Scaling directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingDirection {
    /// Scale up (add replicas)
    Up,
    /// Scale down (remove replicas)
    Down,
    /// Scale in either direction
    Both,
}

/// Scaling amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAmount {
    /// Fixed number of replicas
    Fixed(u32),

    /// Percentage of current replicas
    Percent(u32),
}

/// Scaling metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetric {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: ScalingMetricType,

    /// Target value
    pub target_value: f64,

    /// Current value
    pub current_value: Option<f64>,
}

/// Scaling metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingMetricType {
    /// CPU utilization
    CpuUtilization,

    /// Memory utilization
    MemoryUtilization,

    /// Network utilization
    NetworkUtilization,

    /// Request rate
    RequestRate,

    /// Response time
    ResponseTime,

    /// Queue length
    QueueLength,

    /// Custom metric
    Custom {
        /// Metric source
        source: String,
        /// Query/selector
        query: String,
    },
}

/// Service configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    /// Configuration sources
    pub sources: Vec<ConfigSource>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Configuration files
    pub files: Vec<ConfigFile>,

    /// Feature flags
    pub features: HashMap<String, bool>,
}

/// Configuration sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Environment variables
    Environment,

    /// Configuration files
    Files,

    /// External configuration service
    External {
        /// Service URL
        url: String,
        /// Authentication
        auth: Option<String>,
    },

    /// Kubernetes `ConfigMap`
    ConfigMap(String),

    /// Kubernetes Secret
    Secret(String),
}

/// Configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// File path
    pub path: String,

    /// File format
    pub format: ConfigFormat,

    /// Whether file is required
    pub required: bool,

    /// File watch enabled
    pub watch: bool,
}

/// Configuration formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// Java properties format
    Properties,
    /// INI format
    Ini,
    /// Custom format
    Custom(String),
}

/// Service lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLifecycle {
    /// Startup hooks
    pub startup: Vec<LifecycleHook>,

    /// Shutdown hooks
    pub shutdown: Vec<LifecycleHook>,

    /// Restart policy
    pub restart_policy: RestartPolicy,

    /// Termination grace period (seconds)
    pub termination_grace_period: u32,
}

/// Lifecycle hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHook {
    /// Hook name
    pub name: String,

    /// Hook command
    pub command: Vec<String>,

    /// Hook timeout
    pub timeout: Option<u32>,

    /// Hook failure action
    pub on_failure: LifecycleFailureAction,
}

/// Restart policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Always restart
    Always,

    /// Restart on failure
    OnFailure,

    /// Never restart
    Never,

    /// Restart unless stopped
    UnlessStopped,
}

/// Lifecycle failure actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleFailureAction {
    /// Ignore failure
    Ignore,

    /// Abort operation
    Abort,

    /// Retry hook
    Retry,
}

/// Default implementation for `UniversalService`
impl Default for UniversalService {
    fn default() -> Self {
        Self {
            metadata: ServiceMetadata {
                id: Uuid::new_v4(),
                name: "default-service".to_string(),
                namespace: None,
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                owner_references: vec![],
            },
            spec: ServiceSpec {
                service_type: ServiceType::Application {
                    app_type: "generic".to_string(),
                    framework: None,
                },
                primal_type: None,
                capabilities: vec![],
                runtime: ServiceRuntime::default(),
                resources: ResourceRequirements {
                    cpu: Some(1),
                    memory: Some(100), // 100 MB
                    disk: Some(1000),  // 1000 MB = 1 GB
                    network: None,
                    gpu: None,
                    additional: Vec::new(),
                },
                networking: ServiceNetworking::default(),
                security: ServiceSecurity::default(),
                health: ServiceHealth::default(),
                scaling: ServiceScaling::default(),
                config: ServiceConfiguration::default(),
                lifecycle: ServiceLifecycle::default(),
            },
            status: ServiceStatus {
                phase: ServicePhase::Pending,
                health: Health::unknown("Service not started"),
                conditions: vec![],
                replicas: ReplicaStatus {
                    desired: 1,
                    current: 0,
                    ready: 0,
                    available: 0,
                    unavailable: 1,
                },
                observed_generation: 0,
                last_update_time: Utc::now(),
                message: None,
                reason: None,
            },
            endpoints: vec![],
            dependencies: vec![],
        }
    }
}

impl Default for ServiceScaling {
    fn default() -> Self {
        Self {
            scaling_type: ScalingType::Manual,
            min_replicas: 1,
            max_replicas: 1,
            policies: vec![],
            metrics: vec![],
        }
    }
}

impl Default for ServiceConfiguration {
    fn default() -> Self {
        Self {
            sources: vec![ConfigSource::Environment],
            environment: HashMap::new(),
            files: vec![],
            features: HashMap::new(),
        }
    }
}

impl Default for ServiceLifecycle {
    fn default() -> Self {
        Self {
            startup: vec![],
            shutdown: vec![],
            restart_policy: RestartPolicy::Always,
            termination_grace_period: 30,
        }
    }
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
