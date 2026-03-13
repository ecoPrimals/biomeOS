// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Service Networking Configurations
//!
//! This module contains networking-related types including service networking,
//! discovery, load balancing, and traffic management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::health::HealthCheckConfig;

/// Service networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNetworking {
    /// Network mode
    pub network_mode: NetworkMode,

    /// Service ports
    pub ports: Vec<ServicePort>,

    /// Service discovery
    pub discovery: ServiceDiscovery,

    /// Load balancing
    pub load_balancing: Option<LoadBalancingConfig>,

    /// Network policies
    pub policies: Vec<NetworkPolicyRef>,
}

/// Network modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMode {
    /// Bridge network
    Bridge,
    /// Host network
    Host,
    /// Container network
    Container(String),
    /// Custom network
    Custom(String),
    /// None (no networking)
    None,
}

/// Service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    /// Port name
    pub name: String,

    /// Port number
    pub port: u16,

    /// Target port
    pub target_port: Option<u16>,

    /// Protocol
    pub protocol: PortProtocol,

    /// Expose port externally
    pub expose: bool,

    /// External port (if exposed)
    pub external_port: Option<u16>,

    /// Load balancer configuration
    pub load_balancer: Option<PortLoadBalancer>,
}

/// Port protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortProtocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
    /// HTTP protocol
    Http,
    /// HTTPS protocol
    Https,
    /// gRPC protocol
    Grpc,
    /// WebSocket protocol
    WebSocket,
    /// Custom protocol
    Custom(String),
}

/// Port load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortLoadBalancer {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check
    pub health_check: Option<HealthCheckConfig>,

    /// Session affinity
    pub session_affinity: Option<SessionAffinity>,

    /// Load balancer timeouts
    pub timeouts: Option<LoadBalancerTimeouts>,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least active connections
    LeastConnections,
    /// Weighted round-robin distribution
    WeightedRoundRobin,
    /// Hash by client IP
    IpHash,
    /// Consistent hashing (minimises redistribution)
    ConsistentHash,
    /// Random selection
    Random,
    /// Custom algorithm
    Custom(String),
}

/// Session affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinity {
    /// Affinity type
    pub affinity_type: AffinityType,

    /// Affinity timeout (seconds)
    pub timeout: u32,

    /// Cookie configuration (for cookie affinity)
    pub cookie: Option<AffinityCookie>,
}

/// Affinity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffinityType {
    /// No affinity
    None,
    /// Sticky by client IP address
    ClientIp,
    /// Sticky by cookie
    Cookie,
    /// Sticky by header value
    Header(String),
}

/// Affinity cookie configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffinityCookie {
    /// Cookie name
    pub name: String,

    /// Cookie path
    pub path: Option<String>,

    /// Cookie domain
    pub domain: Option<String>,

    /// Cookie max age (seconds)
    pub max_age: Option<u32>,
}

/// Load balancer timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerTimeouts {
    /// Connection timeout (seconds)
    pub connect: u32,

    /// Request timeout (seconds)
    pub request: u32,

    /// Response timeout (seconds)
    pub response: u32,

    /// Idle timeout (seconds)
    pub idle: Option<u32>,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    /// Discovery enabled
    pub enabled: bool,

    /// Discovery methods
    pub methods: Vec<DiscoveryMethod>,

    /// Service registration
    pub registration: ServiceRegistration,

    /// Health check for discovery
    pub health_check: Option<HealthCheckConfig>,
}

/// Discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// DNS-based discovery
    Dns {
        /// DNS domain
        domain: String,
        /// TTL for DNS records
        ttl: u32,
    },

    /// Consul discovery
    Consul {
        /// Consul address
        address: String,
        /// Consul datacenter
        datacenter: Option<String>,
    },

    /// Etcd discovery
    Etcd {
        /// Etcd endpoints
        endpoints: Vec<String>,
        /// Key prefix
        prefix: String,
    },

    /// Kubernetes discovery
    Kubernetes {
        /// Namespace
        namespace: Option<String>,
        /// Service type
        service_type: KubernetesServiceType,
    },

    /// Custom discovery
    Custom {
        /// Discovery type
        discovery_type: String,
        /// Configuration
        config: HashMap<String, serde_json::Value>,
    },
}

/// Kubernetes service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KubernetesServiceType {
    /// Internal cluster IP
    ClusterIP,
    /// Exposed via node port
    NodePort,
    /// External load balancer
    LoadBalancer,
    /// DNS CNAME alias
    ExternalName,
}

/// Service registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Registration TTL (seconds)
    pub ttl: u32,

    /// Registration interval (seconds)
    pub interval: u32,

    /// Registration metadata
    pub metadata: HashMap<String, String>,

    /// Service tags
    pub tags: Vec<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancer type
    pub lb_type: LoadBalancerType,

    /// Target groups
    pub target_groups: Vec<TargetGroup>,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Load balancer settings
    pub settings: LoadBalancerSettings,
}

/// Load balancer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerType {
    /// Application load balancer (Layer 7)
    Application,
    /// Network load balancer (Layer 4)
    Network,
    /// Classic load balancer
    Classic,
    /// Custom load balancer
    Custom(String),
}

/// Target group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetGroup {
    /// Target group name
    pub name: String,

    /// Target protocol
    pub protocol: TargetProtocol,

    /// Target port
    pub port: u16,

    /// Health check path (for HTTP/HTTPS)
    pub health_check_path: Option<String>,

    /// Target instances
    pub targets: Vec<Target>,

    /// Target group attributes
    pub attributes: HashMap<String, String>,
}

/// Target protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetProtocol {
    /// HTTP protocol
    Http,
    /// HTTPS protocol
    Https,
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
    /// Combined TCP and UDP
    TcpUdp,
    /// TLS-encrypted protocol
    Tls,
}

/// Load balancer target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    /// Target ID
    pub id: String,

    /// Target address
    pub address: String,

    /// Target port
    pub port: u16,

    /// Target weight
    pub weight: Option<u32>,

    /// Target availability zone
    pub availability_zone: Option<String>,
}

/// Load balancer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSettings {
    /// Connection draining timeout (seconds)
    pub connection_draining_timeout: u32,

    /// Cross-zone load balancing
    pub cross_zone_load_balancing: bool,

    /// Access logs enabled
    pub access_logs_enabled: bool,

    /// Access logs bucket
    pub access_logs_bucket: Option<String>,

    /// Idle timeout (seconds)
    pub idle_timeout: u32,

    /// Deletion protection
    pub deletion_protection: bool,
}

/// Network policy reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyRef {
    /// Policy name
    pub name: String,

    /// Policy namespace
    pub namespace: Option<String>,
}

/// Traffic management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficManagement {
    /// Traffic splitting
    pub traffic_splitting: Option<TrafficSplitting>,

    /// Circuit breaker
    pub circuit_breaker: Option<CircuitBreaker>,

    /// Rate limiting
    pub rate_limiting: Option<RateLimiting>,

    /// Timeout configuration
    pub timeouts: Option<TrafficTimeouts>,

    /// Retry configuration
    pub retries: Option<RetryConfig>,
}

/// Traffic splitting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSplitting {
    /// Traffic routes
    pub routes: Vec<TrafficRoute>,

    /// Default route
    pub default_route: Option<String>,

    /// Splitting strategy
    pub strategy: SplittingStrategy,
}

/// Traffic route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRoute {
    /// Route name
    pub name: String,

    /// Target service
    pub target: String,

    /// Traffic weight (percentage)
    pub weight: u32,

    /// Route conditions
    pub conditions: Vec<RouteCondition>,
}

/// Route condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteCondition {
    /// Header condition
    Header {
        /// Header name
        name: String,
        /// Expected value
        value: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Path condition
    Path {
        /// Path pattern
        pattern: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Query parameter condition
    QueryParam {
        /// Parameter name
        name: String,
        /// Expected value
        value: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Custom condition
    Custom {
        /// Condition type identifier
        condition_type: String,
        /// Additional configuration
        config: HashMap<String, String>,
    },
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Exact equality
    Equals,
    /// Not equal
    NotEquals,
    /// Contains substring
    Contains,
    /// Does not contain substring
    NotContains,
    /// Starts with prefix
    StartsWith,
    /// Ends with suffix
    EndsWith,
    /// Matches regex pattern
    Matches,
}

/// Traffic splitting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplittingStrategy {
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Canary deployment
    Canary,
    /// Blue-green deployment
    BlueGreen,
    /// A/B testing
    AbTesting,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Failure threshold
    pub failure_threshold: u32,

    /// Recovery timeout (seconds)
    pub recovery_timeout: u32,

    /// Request volume threshold
    pub request_volume_threshold: u32,

    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,

    /// Sleep window (seconds)
    pub sleep_window: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiting {
    /// Rate limit rules
    pub rules: Vec<RateLimitRule>,

    /// Default rate limit
    pub default_limit: Option<RateLimit>,

    /// Rate limiting strategy
    pub strategy: RateLimitStrategy,
}

/// Rate limit rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// Rule name
    pub name: String,

    /// Rule conditions
    pub conditions: Vec<RateLimitCondition>,

    /// Rate limit
    pub limit: RateLimit,

    /// Rule priority
    pub priority: u32,
}

/// Rate limit condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitCondition {
    /// Client IP condition
    ClientIp(String),

    /// User ID condition
    UserId(String),

    /// API key condition
    ApiKey(String),

    /// Custom condition
    Custom {
        /// Condition type identifier
        condition_type: String,
        /// Condition value
        value: String,
    },
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per time window
    pub requests: u32,

    /// Time window (seconds)
    pub window: u32,

    /// Burst size
    pub burst: Option<u32>,
}

/// Rate limiting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token bucket
    TokenBucket,
    /// Leaky bucket
    LeakyBucket,
    /// Fixed window
    FixedWindow,
    /// Sliding window
    SlidingWindow,
}

/// Traffic timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficTimeouts {
    /// Request timeout (seconds)
    pub request: u32,

    /// Response timeout (seconds)
    pub response: u32,

    /// Connection timeout (seconds)
    pub connection: u32,

    /// Idle timeout (seconds)
    pub idle: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Retry timeout (seconds)
    pub timeout: u32,

    /// Retry conditions
    pub conditions: Vec<RetryCondition>,

    /// Backoff strategy
    pub backoff: BackoffStrategy,
}

/// Retry conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// HTTP status code
    HttpStatus(u16),

    /// Connection error
    ConnectionError,

    /// Timeout error
    TimeoutError,

    /// Custom condition
    Custom(String),
}

/// Backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed backoff
    Fixed {
        /// Delay in seconds
        delay: u32,
    },

    /// Exponential backoff
    Exponential {
        /// Base delay in seconds
        base_delay: u32,
        /// Maximum delay cap in seconds
        max_delay: u32,
        /// Multiplier per attempt
        multiplier: f64,
    },

    /// Linear backoff
    Linear {
        /// Base delay in seconds
        base_delay: u32,
        /// Linear increment per attempt
        increment: u32,
    },

    /// Random backoff
    Random {
        /// Minimum delay in seconds
        min_delay: u32,
        /// Maximum delay in seconds
        max_delay: u32,
    },
}

/// Default implementation for ServiceNetworking
impl Default for ServiceNetworking {
    fn default() -> Self {
        Self {
            network_mode: NetworkMode::Bridge,
            ports: vec![],
            discovery: ServiceDiscovery {
                enabled: true,
                methods: vec![DiscoveryMethod::Dns {
                    domain: "local".to_string(),
                    ttl: 300,
                }],
                registration: ServiceRegistration {
                    ttl: 300,
                    interval: 30,
                    metadata: HashMap::new(),
                    tags: vec![],
                },
                health_check: None,
            },
            load_balancing: None,
            policies: vec![],
        }
    }
}
