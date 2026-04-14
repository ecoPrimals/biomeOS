// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Service mesh, ingress, egress, and security policy types.

use super::routing::VirtualServiceSpec;
use super::traffic::TrafficPolicySpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// `HashiCorp` Consul Connect
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
