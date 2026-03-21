// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Network Services Specifications
//!
//! This module contains network service types including DNS, IPAM,
//! Service Mesh, and advanced networking configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network DNS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDnsSpec {
    /// Nameservers
    pub nameservers: Vec<String>,

    /// Search domains
    pub search: Vec<String>,

    /// Options
    pub options: Vec<DnsOptionSpec>,
}

/// DNS option specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOptionSpec {
    /// Name
    pub name: String,

    /// Value
    pub value: Option<String>,
}

/// IPAM (IP Address Management) specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpamSpec {
    /// IPAM driver
    pub driver: String,

    /// Configuration
    pub config: Vec<IpamConfigSpec>,

    /// Options
    pub options: HashMap<String, String>,
}

/// IPAM configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpamConfigSpec {
    /// Subnet
    pub subnet: String,

    /// IP range
    pub ip_range: Option<String>,

    /// Gateway
    pub gateway: Option<String>,

    /// Auxiliary addresses
    pub aux_addresses: HashMap<String, String>,
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshSpec {
    /// Mesh type
    pub mesh_type: ServiceMeshType,

    /// Mesh configuration
    pub config: ServiceMeshConfig,

    /// Traffic policies
    pub traffic_policies: Vec<TrafficPolicySpec>,

    /// Security policies
    pub security_policies: Vec<MeshSecurityPolicySpec>,
}

/// Service mesh types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshType {
    /// Istio service mesh
    Istio,
    /// Linkerd service mesh
    Linkerd,
    /// HashiCorp Consul Connect
    Consul,
    /// Envoy proxy
    Envoy,
    /// Custom service mesh
    Custom(String),
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Enable mTLS
    pub mtls_enabled: bool,

    /// Telemetry configuration
    pub telemetry: Option<MeshTelemetrySpec>,

    /// Ingress configuration
    pub ingress: Option<MeshIngressSpec>,

    /// Egress configuration
    pub egress: Option<MeshEgressSpec>,
}

/// Mesh telemetry specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTelemetrySpec {
    /// Enable tracing
    pub tracing_enabled: bool,

    /// Enable metrics
    pub metrics_enabled: bool,

    /// Enable access logs
    pub access_logs_enabled: bool,

    /// Sampling rate
    pub sampling_rate: Option<f64>,
}

/// Mesh ingress specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshIngressSpec {
    /// Gateway configuration
    pub gateways: Vec<GatewaySpec>,

    /// Virtual services
    pub virtual_services: Vec<VirtualServiceSpec>,
}

/// Gateway specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySpec {
    /// Gateway name
    pub name: String,

    /// Selector
    pub selector: HashMap<String, String>,

    /// Servers
    pub servers: Vec<ServerSpec>,
}

/// Server specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSpec {
    /// Port
    pub port: PortSpec,

    /// Hosts
    pub hosts: Vec<String>,

    /// TLS configuration
    pub tls: Option<TlsSpec>,
}

/// Port specification for gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Number
    pub number: u16,

    /// Name
    pub name: String,

    /// Protocol
    pub protocol: String,
}

/// TLS specification for gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSpec {
    /// Mode
    pub mode: TlsMode,

    /// Credential name
    pub credential_name: Option<String>,

    /// Server certificate
    pub server_certificate: Option<String>,

    /// Private key
    pub private_key: Option<String>,
}

/// TLS modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsMode {
    /// Pass through TLS to the backend
    Passthrough,
    /// Simple TLS termination
    Simple,
    /// Mutual TLS (mTLS)
    Mutual,
    /// Auto passthrough with SNI
    AutoPassthrough,
}

/// Load balancer specification for traffic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSpec {
    /// Simple load balancer
    pub simple: Option<LoadBalancerAlgorithm>,

    /// Consistent hash
    pub consistent_hash: Option<ConsistentHashSpec>,
}

/// Load balancer algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections
    LeastConn,
    /// Random selection
    Random,
    /// Pass through without balancing
    Passthrough,
}

/// Traffic policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPolicySpec {
    /// Load balancer
    pub load_balancer: Option<LoadBalancerSpec>,

    /// Connection pool
    pub connection_pool: Option<ConnectionPoolSpec>,

    /// Outlier detection
    pub outlier_detection: Option<OutlierDetectionSpec>,

    /// TLS
    pub tls: Option<ClientTlsSettings>,
}

/// Mesh security policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSecurityPolicySpec {
    /// Policy name
    pub name: String,

    /// Namespace
    pub namespace: Option<String>,

    /// Action
    pub action: SecurityAction,

    /// Rules
    pub rules: Vec<SecurityRuleSpec>,
}

/// Security actions for mesh policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Audit the action (log only)
    Audit,
}

/// Security rule specification for mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRuleSpec {
    /// Source
    pub from: Vec<Source>,

    /// Operation
    pub to: Vec<Operation>,

    /// When conditions
    pub when: Vec<Condition>,
}

/// Source specification for security rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Principals
    pub principals: Vec<String>,

    /// Namespaces
    pub namespaces: Vec<String>,

    /// IP blocks
    pub ip_blocks: Vec<String>,
}

/// Operation specification for security rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Hosts
    pub hosts: Vec<String>,

    /// Ports
    pub ports: Vec<String>,

    /// Methods
    pub methods: Vec<String>,

    /// Paths
    pub paths: Vec<String>,
}

/// Condition specification for security rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Key
    pub key: String,

    /// Values
    pub values: Vec<String>,
}

/// Virtual service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServiceSpec {
    /// Service name
    pub name: String,

    /// Hosts
    pub hosts: Vec<String>,

    /// Gateways
    pub gateways: Vec<String>,

    /// HTTP routes
    pub http: Vec<HttpRouteSpec>,

    /// TCP routes
    pub tcp: Vec<TcpRouteSpec>,

    /// TLS routes
    pub tls: Vec<TlsRouteSpec>,
}

/// Mesh egress specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshEgressSpec {
    /// Service entries
    pub service_entries: Vec<ServiceEntrySpec>,

    /// Destination rules
    pub destination_rules: Vec<DestinationRuleSpec>,
}

/// Service entry specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntrySpec {
    /// Service name
    pub name: String,

    /// Hosts
    pub hosts: Vec<String>,

    /// Ports
    pub ports: Vec<ServiceEntryPort>,

    /// Location
    pub location: ServiceLocation,

    /// Resolution
    pub resolution: ServiceResolution,
}

/// Service entry port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntryPort {
    /// Number
    pub number: u16,

    /// Name
    pub name: String,

    /// Protocol
    pub protocol: String,
}

/// Service location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceLocation {
    /// Service is external to the mesh
    MeshExternal,
    /// Service is internal to the mesh
    MeshInternal,
}

/// Service resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceResolution {
    /// No resolution
    None,
    /// Static IP resolution
    Static,
    /// DNS-based resolution
    DNS,
}

/// Destination rule specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationRuleSpec {
    /// Rule name
    pub name: String,

    /// Host
    pub host: String,

    /// Traffic policy
    pub traffic_policy: Option<TrafficPolicySpec>,

    /// Subsets
    pub subsets: Vec<SubsetSpec>,
}

/// Subset specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsetSpec {
    /// Name
    pub name: String,

    /// Labels
    pub labels: HashMap<String, String>,

    /// Traffic policy
    pub traffic_policy: Option<TrafficPolicySpec>,
}

/// Connection pool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSpec {
    /// TCP settings
    pub tcp: Option<TcpSettingsSpec>,

    /// HTTP settings
    pub http: Option<HttpSettingsSpec>,
}

/// TCP settings specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSettingsSpec {
    /// Max connections
    pub max_connections: Option<u32>,

    /// Connect timeout
    pub connect_timeout: Option<u32>,

    /// TCP no delay
    pub tcp_no_delay: Option<bool>,
}

/// HTTP settings specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSettingsSpec {
    /// HTTP1 max pending requests
    pub http1_max_pending_requests: Option<u32>,

    /// HTTP2 max requests
    pub http2_max_requests: Option<u32>,

    /// Max requests per connection
    pub max_requests_per_connection: Option<u32>,

    /// Max retries
    pub max_retries: Option<u32>,

    /// Idle timeout
    pub idle_timeout: Option<u32>,

    /// H2 upgrade policy
    pub h2_upgrade_policy: Option<H2UpgradePolicy>,
}

/// H2 upgrade policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2UpgradePolicy {
    /// Use default upgrade behavior
    Default,
    /// Do not upgrade to HTTP/2
    DoNotUpgrade,
    /// Upgrade to HTTP/2
    Upgrade,
}

/// Outlier detection specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionSpec {
    /// Consecutive errors
    pub consecutive_errors: Option<u32>,

    /// Interval
    pub interval: Option<u32>,

    /// Base ejection time
    pub base_ejection_time: Option<u32>,

    /// Max ejection percent
    pub max_ejection_percent: Option<u32>,

    /// Min health percent
    pub min_health_percent: Option<u32>,
}

/// Client TLS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTlsSettings {
    /// Mode
    pub mode: ClientTlsMode,

    /// Client certificate
    pub client_certificate: Option<String>,

    /// Private key
    pub private_key: Option<String>,

    /// CA certificates
    pub ca_certificates: Option<String>,

    /// Subject alternative names
    pub subject_alternative_names: Vec<String>,

    /// SNI
    pub sni: Option<String>,
}

/// Client TLS modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientTlsMode {
    /// TLS disabled
    Disable,
    /// Simple TLS (server verification only)
    Simple,
    /// Mutual TLS
    Mutual,
    /// Istio-managed mutual TLS
    IstioMutual,
}

// HTTP routing types for VirtualService
/// HTTP route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<HttpMatchCondition>,

    /// Route destinations
    pub route: Vec<HttpRouteDestination>,

    /// Redirect
    pub redirect: Option<HttpRedirect>,

    /// Rewrite
    pub rewrite: Option<HttpRewrite>,

    /// Timeout
    pub timeout: Option<u32>,

    /// Retries
    pub retries: Option<HttpRetry>,
}

/// HTTP match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMatchCondition {
    /// URI
    pub uri: Option<StringMatch>,

    /// Scheme
    pub scheme: Option<StringMatch>,

    /// Method
    pub method: Option<StringMatch>,

    /// Authority
    pub authority: Option<StringMatch>,

    /// Headers
    pub headers: HashMap<String, StringMatch>,

    /// Query parameters
    pub query_params: HashMap<String, StringMatch>,
}

/// String match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringMatch {
    /// Exact string match
    Exact(String),
    /// String prefix match
    Prefix(String),
    /// Regular expression match
    Regex(String),
}

/// HTTP route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,

    /// Headers
    pub headers: Option<HeadersSpec>,
}

/// Destination specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationSpec {
    /// Host
    pub host: String,

    /// Subset
    pub subset: Option<String>,

    /// Port
    pub port: Option<PortSelector>,
}

/// Port selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortSelector {
    /// Port number
    Number(u16),
    /// Port name
    Name(String),
}

/// Headers specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadersSpec {
    /// Request headers
    pub request: Option<HeaderOperations>,

    /// Response headers
    pub response: Option<HeaderOperations>,
}

/// Header operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderOperations {
    /// Set headers
    pub set: HashMap<String, String>,

    /// Add headers
    pub add: HashMap<String, String>,

    /// Remove headers
    pub remove: Vec<String>,
}

/// HTTP redirect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRedirect {
    /// URI
    pub uri: Option<String>,

    /// Authority
    pub authority: Option<String>,

    /// Redirect code
    pub redirect_code: Option<u16>,
}

/// HTTP rewrite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRewrite {
    /// URI
    pub uri: Option<String>,

    /// Authority
    pub authority: Option<String>,
}

/// HTTP retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRetry {
    /// Attempts
    pub attempts: u32,

    /// Per try timeout
    pub per_try_timeout: Option<u32>,

    /// Retry on
    pub retry_on: Option<String>,
}

/// TCP route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<TcpMatchCondition>,

    /// Route destinations
    pub route: Vec<TcpRouteDestination>,
}

/// TCP match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpMatchCondition {
    /// Destination subnets
    pub destination_subnets: Vec<String>,

    /// Port
    pub port: Option<u16>,

    /// Source labels
    pub source_labels: HashMap<String, String>,

    /// Gateways
    pub gateways: Vec<String>,
}

/// TCP route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,
}

/// TLS route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<TlsMatchCondition>,

    /// Route destinations
    pub route: Vec<TlsRouteDestination>,
}

/// TLS match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsMatchCondition {
    /// SNI hosts
    pub sni_hosts: Vec<String>,

    /// Destination subnets
    pub destination_subnets: Vec<String>,

    /// Port
    pub port: Option<u16>,

    /// Source labels
    pub source_labels: HashMap<String, String>,

    /// Gateways
    pub gateways: Vec<String>,
}

/// TLS route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,
}

/// Consistent hash specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentHashSpec {
    /// HTTP header name
    pub http_header_name: Option<String>,

    /// HTTP cookie
    pub http_cookie: Option<HttpCookieSpec>,

    /// Use source IP
    pub use_source_ip: Option<bool>,

    /// Ring hash
    pub ring_hash: Option<RingHashSpec>,
}

/// HTTP cookie specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCookieSpec {
    /// Name
    pub name: String,

    /// Path
    pub path: Option<String>,

    /// TTL
    pub ttl: Option<u32>,
}

/// Ring hash specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingHashSpec {
    /// Minimum ring size
    pub minimum_ring_size: Option<u64>,

    /// Maximum ring size
    pub maximum_ring_size: Option<u64>,
}

#[cfg(test)]
#[path = "networking_services_tests.rs"]
mod tests;
