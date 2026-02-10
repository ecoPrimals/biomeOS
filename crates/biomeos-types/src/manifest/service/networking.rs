//! Service Networking Types
//!
//! This module contains networking-related service types including PortSpec,
//! LoadBalancerSpec, and related networking configurations.

use crate::health::HealthCheckConfig;
use serde::{Deserialize, Serialize};

/// Port specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port name
    pub name: String,

    /// Port number
    pub port: u16,

    /// Target port (for service meshes)
    pub target_port: Option<u16>,

    /// Protocol (TCP, UDP, HTTP, HTTPS, gRPC)
    pub protocol: PortProtocol,

    /// Whether to expose externally
    pub expose: bool,

    /// Load balancer configuration
    pub load_balancer: Option<LoadBalancerSpec>,

    /// Health check for this port
    pub health_check: Option<PortHealthCheckSpec>,
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

/// Load balancer specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSpec {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,

    /// Session affinity
    pub session_affinity: Option<SessionAffinity>,

    /// Timeouts
    pub timeouts: Option<LoadBalancerTimeouts>,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least active connections
    LeastConnections,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// IP-based hashing
    IpHash,
    /// Consistent hashing
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

    /// Cookie configuration
    pub cookie: Option<AffinityCookieSpec>,

    /// Timeout
    pub timeout: Option<u32>,
}

/// Affinity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffinityType {
    /// Client IP-based affinity
    ClientIP,
    /// Cookie-based affinity
    Cookie,
    /// Header-based affinity
    Header,
    /// No session affinity
    None,
}

/// Affinity cookie specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffinityCookieSpec {
    /// Cookie name
    pub name: String,

    /// Cookie path
    pub path: Option<String>,

    /// Cookie domain
    pub domain: Option<String>,
}

/// Load balancer timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerTimeouts {
    /// Connection timeout
    pub connect: Option<u32>,

    /// Request timeout
    pub request: Option<u32>,

    /// Response timeout
    pub response: Option<u32>,

    /// Idle timeout
    pub idle: Option<u32>,
}

/// Port health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortHealthCheckSpec {
    /// Health check path (for HTTP)
    pub path: Option<String>,

    /// Health check interval
    pub interval: u32,

    /// Health check timeout
    pub timeout: u32,

    /// Healthy threshold
    pub healthy_threshold: u32,

    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
}
