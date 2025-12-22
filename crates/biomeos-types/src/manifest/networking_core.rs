//! Core Network Specifications
//!
//! This module contains the fundamental networking types including NetworkSpec,
//! NetworkMetadata, NetworkDriver, and basic network configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpec {
    /// Network metadata
    pub metadata: NetworkMetadata,

    /// Network driver
    pub driver: NetworkDriver,

    /// Network configuration
    pub config: NetworkConfig,

    /// Subnet configuration
    pub subnets: Vec<SubnetSpec>,

    /// Network policies
    pub policies: Vec<super::networking_policies::NetworkPolicySpec>,

    /// DNS configuration
    pub dns: Option<super::networking_services::NetworkDnsSpec>,

    /// IPAM configuration
    pub ipam: Option<super::networking_services::IpamSpec>,
}

/// Network metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetadata {
    /// Network name
    pub name: String,

    /// Network description
    pub description: Option<String>,

    /// Network labels
    pub labels: HashMap<String, String>,

    /// Network annotations
    pub annotations: HashMap<String, String>,
}

/// Network drivers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkDriver {
    Bridge,
    Host,
    Overlay,
    Macvlan,
    Ipvlan,
    None,
    Custom(String),
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable IPv6
    pub enable_ipv6: bool,

    /// Internal network
    pub internal: bool,

    /// Attachable
    pub attachable: bool,

    /// Ingress
    pub ingress: bool,

    /// Scope
    pub scope: NetworkScope,

    /// Options
    pub options: HashMap<String, String>,
}

/// Network scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkScope {
    Local,
    Global,
    Swarm,
}

/// Subnet specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetSpec {
    /// Subnet CIDR
    pub subnet: String,

    /// Gateway
    pub gateway: Option<String>,

    /// IP range
    pub ip_range: Option<String>,

    /// Auxiliary addresses
    pub aux_addresses: HashMap<String, String>,
}

/// Default implementations
impl Default for NetworkSpec {
    fn default() -> Self {
        Self {
            metadata: NetworkMetadata::default(),
            driver: NetworkDriver::Bridge,
            config: NetworkConfig::default(),
            subnets: Vec::new(),
            policies: Vec::new(),
            dns: None,
            ipam: None,
        }
    }
}

impl Default for NetworkMetadata {
    fn default() -> Self {
        Self {
            name: "default-network".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_ipv6: false,
            internal: false,
            attachable: true,
            ingress: false,
            scope: NetworkScope::Local,
            options: HashMap::new(),
        }
    }
} 