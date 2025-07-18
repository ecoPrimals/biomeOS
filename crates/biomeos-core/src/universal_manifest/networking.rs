//! Networking Specifications Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    /// Network policies
    pub policies: Vec<NetworkPolicy>,
    /// DNS configuration
    pub dns: DnsConfig,
    /// Load balancing
    pub load_balancing: LoadBalancingConfig,
    /// Network topology
    pub topology: NetworkTopology,
}

/// Network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTopology {
    /// Flat network topology
    Flat,
    /// Hierarchical network topology
    Hierarchical,
    /// Mesh network topology
    Mesh,
    /// Star network topology
    Star,
    /// Ring network topology
    Ring,
    /// Bus network topology
    Bus,
    /// Hybrid network topology
    Hybrid,
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,
    /// Rules
    pub rules: Vec<String>,
}

/// DNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    /// DNS servers
    pub servers: Vec<String>,
    /// Search domains
    pub search_domains: Vec<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Algorithm
    pub algorithm: String,
    /// Health check
    pub health_check: String,
} 