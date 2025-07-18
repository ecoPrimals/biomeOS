//! Service specifications and related types
//!
//! This module contains all service-related specifications, including
//! service types, endpoints, and load balancer configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub service_type: ServiceType,
    pub selector: HashMap<String, String>,
    pub ports: Vec<ServicePort>,
    pub load_balancer_config: Option<LoadBalancerConfig>,
    pub session_affinity: SessionAffinity,
}

/// Service port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub protocol: PortProtocol,
    pub port: u16,
    pub target_port: ServicePortTarget,
    pub node_port: Option<u16>,
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub load_balancer_ip: Option<String>,
    pub load_balancer_source_ranges: Vec<String>,
    pub external_traffic_policy: ExternalTrafficPolicy,
    pub health_check_node_port: Option<u16>,
}

/// Service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub namespace: Option<String>,
    pub endpoints: Vec<EndpointAddress>,
    pub ports: Vec<EndpointPort>,
    pub service_type: ServiceType,
}

/// Endpoint address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointAddress {
    pub ip: String,
    pub hostname: Option<String>,
    pub node_name: Option<String>,
    pub ready: bool,
}

/// Endpoint port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPort {
    pub name: Option<String>,
    pub port: u16,
    pub protocol: PortProtocol,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub discovery_mode: DiscoveryMode,
    pub health_check_enabled: bool,
    pub health_check_interval: u32,
    pub health_check_timeout: u32,
    pub health_check_path: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Service discovery modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMode {
    /// DNS-based discovery
    Dns,
    /// API-based discovery
    Api,
    /// Consul-based discovery
    Consul { consul_endpoint: String },
    /// Etcd-based discovery
    Etcd { etcd_endpoints: Vec<String> },
    /// Custom discovery
    Custom { discovery_type: String },
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    pub enabled: bool,
    pub mesh_type: MeshType,
    pub sidecar_injection: bool,
    pub traffic_policy: TrafficPolicy,
    pub security_policy: MeshSecurityPolicy,
}

/// Service mesh types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshType {
    Istio,
    Linkerd,
    Consul,
    Envoy,
    Custom { mesh_name: String },
}

/// Traffic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPolicy {
    pub load_balancer: LoadBalancerPolicy,
    pub connection_pool: ConnectionPoolSettings,
    pub circuit_breaker: CircuitBreakerSettings,
    pub retry: RetrySettings,
}

/// Load balancer policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerPolicy {
    RoundRobin,
    LeastConn,
    Random,
    PassThrough,
    Consistent,
}

/// Connection pool settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSettings {
    pub max_connections: Option<u32>,
    pub connect_timeout: Option<u32>,
    pub idle_timeout: Option<u32>,
    pub h2_settings: Option<H2Settings>,
}

/// HTTP/2 settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Settings {
    pub max_requests_per_connection: Option<u32>,
    pub max_concurrent_streams: Option<u32>,
    pub initial_stream_window_size: Option<u32>,
    pub initial_connection_window_size: Option<u32>,
}

/// Circuit breaker settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerSettings {
    pub consecutive_5xx_errors: Option<u32>,
    pub consecutive_gateway_errors: Option<u32>,
    pub interval: Option<u32>,
    pub base_ejection_time: Option<u32>,
    pub max_ejection_percent: Option<u32>,
}

/// Retry settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySettings {
    pub attempts: u32,
    pub per_try_timeout: Option<u32>,
    pub retry_on: Vec<String>,
    pub retry_remote_localities: bool,
}

/// Mesh security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSecurityPolicy {
    pub mode: SecurityMode,
    pub mutual_tls: MutualTlsSettings,
    pub authorization: AuthorizationSettings,
}

/// Security modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMode {
    Disabled,
    Permissive,
    Strict,
}

/// Mutual TLS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualTlsSettings {
    pub enabled: bool,
    pub mode: MutualTlsMode,
    pub ca_certificates: Option<String>,
    pub client_certificates: Option<String>,
    pub private_key: Option<String>,
}

/// Mutual TLS modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutualTlsMode {
    Disable,
    Permissive,
    Strict,
}

/// Authorization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationSettings {
    pub enabled: bool,
    pub rules: Vec<AuthorizationRule>,
}

/// Authorization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRule {
    pub from: Vec<AuthorizationSource>,
    pub to: Vec<AuthorizationTarget>,
    pub when: Vec<AuthorizationCondition>,
}

/// Authorization source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationSource {
    pub principals: Vec<String>,
    pub namespaces: Vec<String>,
    pub ip_blocks: Vec<String>,
}

/// Authorization target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationTarget {
    pub hosts: Vec<String>,
    pub ports: Vec<u16>,
    pub methods: Vec<String>,
    pub paths: Vec<String>,
}

/// Authorization condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCondition {
    pub key: String,
    pub values: Vec<String>,
}

// Default implementations
impl Default for ServiceSpec {
    fn default() -> Self {
        Self {
            name: "service".to_string(),
            service_type: ServiceType::default(),
            selector: HashMap::new(),
            ports: vec![],
            load_balancer_config: None,
            session_affinity: SessionAffinity::default(),
        }
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            load_balancer_ip: None,
            load_balancer_source_ranges: vec![],
            external_traffic_policy: ExternalTrafficPolicy::default(),
            health_check_node_port: None,
        }
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_mode: DiscoveryMode::Dns,
            health_check_enabled: true,
            health_check_interval: 30,
            health_check_timeout: 5,
            health_check_path: Some("/health".to_string()),
            metadata: HashMap::new(),
        }
    }
}

impl Default for ServiceMeshConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mesh_type: MeshType::Istio,
            sidecar_injection: true,
            traffic_policy: TrafficPolicy::default(),
            security_policy: MeshSecurityPolicy::default(),
        }
    }
}

impl Default for TrafficPolicy {
    fn default() -> Self {
        Self {
            load_balancer: LoadBalancerPolicy::RoundRobin,
            connection_pool: ConnectionPoolSettings::default(),
            circuit_breaker: CircuitBreakerSettings::default(),
            retry: RetrySettings::default(),
        }
    }
}

impl Default for ConnectionPoolSettings {
    fn default() -> Self {
        Self {
            max_connections: Some(1024),
            connect_timeout: Some(10),
            idle_timeout: Some(60),
            h2_settings: None,
        }
    }
}

impl Default for CircuitBreakerSettings {
    fn default() -> Self {
        Self {
            consecutive_5xx_errors: Some(5),
            consecutive_gateway_errors: Some(5),
            interval: Some(30),
            base_ejection_time: Some(30),
            max_ejection_percent: Some(50),
        }
    }
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            attempts: 3,
            per_try_timeout: Some(10),
            retry_on: vec!["gateway-error".to_string(), "connect-failure".to_string()],
            retry_remote_localities: false,
        }
    }
}

impl Default for MeshSecurityPolicy {
    fn default() -> Self {
        Self {
            mode: SecurityMode::Permissive,
            mutual_tls: MutualTlsSettings::default(),
            authorization: AuthorizationSettings::default(),
        }
    }
}

impl Default for MutualTlsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: MutualTlsMode::Permissive,
            ca_certificates: None,
            client_certificates: None,
            private_key: None,
        }
    }
}

impl Default for AuthorizationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            rules: vec![],
        }
    }
} 