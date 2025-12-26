//! Network Policy Specifications
//!
//! This module contains network policy types including NetworkPolicySpec,
//! ingress/egress rules, and network security policies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicySpec {
    /// Policy name
    pub name: String,

    /// Policy type
    pub policy_type: NetworkPolicyType,

    /// Pod selector
    pub pod_selector: HashMap<String, String>,

    /// Ingress rules
    pub ingress: Vec<NetworkIngressRuleSpec>,

    /// Egress rules
    pub egress: Vec<NetworkEgressRuleSpec>,
}

/// Network policy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyType {
    Ingress,
    Egress,
    Both,
}

/// Network ingress rule specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIngressRuleSpec {
    /// From selectors
    pub from: Vec<NetworkPolicyPeer>,

    /// Ports
    pub ports: Vec<NetworkPolicyPort>,
}

/// Network egress rule specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEgressRuleSpec {
    /// To selectors
    pub to: Vec<NetworkPolicyPeer>,

    /// Ports
    pub ports: Vec<NetworkPolicyPort>,
}

/// Network policy peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPeer {
    /// Pod selector
    pub pod_selector: Option<HashMap<String, String>>,

    /// Namespace selector
    pub namespace_selector: Option<HashMap<String, String>>,

    /// IP block
    pub ip_block: Option<IpBlockSpec>,
}

/// IP block specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlockSpec {
    /// CIDR
    pub cidr: String,

    /// Except
    pub except: Vec<String>,
}

/// Network policy port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPort {
    /// Protocol
    pub protocol: NetworkPolicyProtocol,

    /// Port
    pub port: Option<NetworkPolicyPortValue>,

    /// End port
    pub end_port: Option<u16>,
}

/// Network policy protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyProtocol {
    TCP,
    UDP,
    SCTP,
}

/// Network policy port value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyPortValue {
    Number(u16),
    Name(String),
}

/// Network policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyAction {
    Allow,
    Deny,
    Log,
}
