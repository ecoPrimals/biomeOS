// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Bridge network driver
    Bridge,
    /// Host network driver
    Host,
    /// Overlay network driver
    Overlay,
    /// Macvlan network driver
    Macvlan,
    /// IPvlan network driver
    Ipvlan,
    /// No network driver
    None,
    /// Custom network driver
    Custom(String),
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Config struct; refactoring would break API"
)]
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
    /// Local scope
    Local,
    /// Global scope
    Global,
    /// Docker Swarm scope
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

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_spec_default() {
        let spec = NetworkSpec::default();
        assert_eq!(spec.metadata.name, "default-network");
        assert_eq!(spec.subnets.len(), 0);
        assert_eq!(spec.policies.len(), 0);
        assert!(spec.dns.is_none());
        assert!(spec.ipam.is_none());
    }

    #[test]
    fn test_network_metadata_default() {
        let metadata = NetworkMetadata::default();
        assert_eq!(metadata.name, "default-network");
        assert!(metadata.description.is_none());
        assert!(metadata.labels.is_empty());
        assert!(metadata.annotations.is_empty());
    }

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(!config.enable_ipv6);
        assert!(!config.internal);
        assert!(config.attachable);
        assert!(!config.ingress);
        assert!(matches!(config.scope, NetworkScope::Local));
        assert!(config.options.is_empty());
    }

    #[test]
    fn test_network_driver_variants() {
        let drivers = vec![
            NetworkDriver::Bridge,
            NetworkDriver::Host,
            NetworkDriver::Overlay,
            NetworkDriver::Macvlan,
            NetworkDriver::Ipvlan,
            NetworkDriver::None,
            NetworkDriver::Custom("custom-driver".to_string()),
        ];

        for driver in drivers {
            let json = serde_json::to_string(&driver).unwrap();
            let _deserialized: NetworkDriver = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_network_scope_variants() {
        let scopes = vec![
            NetworkScope::Local,
            NetworkScope::Global,
            NetworkScope::Swarm,
        ];

        for scope in scopes {
            let json = serde_json::to_string(&scope).unwrap();
            let _deserialized: NetworkScope = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_subnet_spec_creation() {
        let mut aux_addresses = HashMap::new();
        aux_addresses.insert("router".to_string(), "192.168.1.1".to_string());

        let subnet = SubnetSpec {
            subnet: "192.168.1.0/24".to_string(),
            gateway: Some("192.168.1.1".to_string()),
            ip_range: Some("192.168.1.100-192.168.1.200".to_string()),
            aux_addresses,
        };

        assert_eq!(subnet.subnet, "192.168.1.0/24");
        assert_eq!(subnet.gateway, Some("192.168.1.1".to_string()));
        assert_eq!(subnet.aux_addresses.len(), 1);
    }

    #[test]
    fn test_network_spec_with_subnets() {
        let mut spec = NetworkSpec::default();

        spec.subnets.push(SubnetSpec {
            subnet: "10.0.0.0/24".to_string(),
            gateway: Some("10.0.0.1".to_string()),
            ip_range: None,
            aux_addresses: HashMap::new(),
        });

        assert_eq!(spec.subnets.len(), 1);
        assert_eq!(spec.subnets[0].subnet, "10.0.0.0/24");
    }

    #[test]
    fn test_network_metadata_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "production".to_string());
        labels.insert("tier".to_string(), "frontend".to_string());

        let metadata = NetworkMetadata {
            name: "prod-network".to_string(),
            description: Some("Production network".to_string()),
            labels,
            annotations: HashMap::new(),
        };

        assert_eq!(metadata.name, "prod-network");
        assert_eq!(metadata.labels.len(), 2);
        assert_eq!(
            metadata.labels.get("environment"),
            Some(&"production".to_string())
        );
    }

    #[test]
    fn test_network_config_ipv6() {
        let config = NetworkConfig {
            enable_ipv6: true,
            internal: false,
            attachable: true,
            ingress: false,
            scope: NetworkScope::Global,
            options: HashMap::new(),
        };

        assert!(config.enable_ipv6);
        assert!(matches!(config.scope, NetworkScope::Global));
    }

    #[test]
    fn test_network_serialization() {
        let spec = NetworkSpec::default();
        let json = serde_json::to_string(&spec).unwrap();

        let deserialized: NetworkSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.metadata.name, "default-network");
    }

    #[test]
    fn test_custom_network_driver() {
        let driver = NetworkDriver::Custom("cilium".to_string());

        match &driver {
            NetworkDriver::Custom(name) => assert_eq!(name, "cilium"),
            _ => panic!("Expected Custom driver"),
        }

        let json = serde_json::to_string(&driver).unwrap();
        assert!(json.contains("cilium"));
    }
}
