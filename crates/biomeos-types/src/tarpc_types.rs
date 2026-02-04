//! tarpc shared types and service definitions
//!
//! This module defines the tarpc service traits and types for high-performance
//! binary RPC communication between primals.
//!
//! ## Architecture
//!
//! biomeOS uses a dual-protocol approach:
//! - **JSON-RPC**: Initial flexible protocol for all primals
//! - **tarpc**: High-performance binary protocol for established, high-frequency paths
//!
//! Primals start with JSON-RPC and can be escalated to tarpc based on:
//! - Communication frequency
//! - Latency requirements
//! - Stability of the protocol
//!
//! ## Protocol Escalation
//!
//! ```text
//! JSON-RPC (flexible, debuggable)
//!     ↓ (based on metrics)
//! tarpc (fast, binary, type-safe)
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Core tarpc Service Traits
// ============================================================================

/// Core health service - all primals implement this
#[tarpc::service]
pub trait HealthRpc {
    /// Check if the primal is healthy
    async fn health_check() -> HealthStatus;

    /// Get detailed health metrics
    async fn health_metrics() -> HealthMetrics;

    /// Get primal version information
    async fn version() -> VersionInfo;
}

/// Discovery service - implemented by Songbird
#[tarpc::service]
pub trait DiscoveryRpc {
    /// Discover primals providing a capability
    async fn discover(capability: String) -> Vec<ServiceInfo>;

    /// List all known services
    async fn discover_all() -> Vec<ServiceInfo>;

    /// Register a service
    async fn register(registration: ServiceRegistration) -> RegistrationResult;

    /// Unregister a service
    async fn unregister(primal_id: String) -> bool;

    /// List supported protocols
    async fn protocols() -> Vec<ProtocolInfo>;
}

/// Security service - implemented by BearDog
#[tarpc::service]
pub trait SecurityRpc {
    /// Sign data with primal's key
    async fn sign(data: Vec<u8>) -> SignatureResult;

    /// Verify a signature
    async fn verify(data: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> bool;

    /// Get JWT secret for service
    async fn get_jwt_secret(service_name: String) -> JwtSecretResult;

    /// Verify family lineage
    async fn verify_lineage(primal_id: String) -> LineageResult;
}

// ============================================================================
// Shared Types
// ============================================================================

/// Health status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the primal is healthy
    pub healthy: bool,
    /// Optional status message
    pub message: Option<String>,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Detailed health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall health status
    pub healthy: bool,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Request count since start
    pub total_requests: u64,
    /// Error count since start
    pub total_errors: u64,
    /// Average latency in microseconds
    pub avg_latency_us: u64,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Semantic version
    pub version: String,
    /// Git commit hash
    pub git_commit: Option<String>,
    /// Build timestamp
    pub build_timestamp: Option<String>,
    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Service information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Primal name
    pub name: String,
    /// Socket or endpoint path
    pub endpoint: String,
    /// Capabilities provided
    pub capabilities: Vec<String>,
    /// Health status
    pub healthy: bool,
    /// Supported protocols (jsonrpc, tarpc, http)
    pub protocols: Vec<String>,
    /// Last health check timestamp
    pub last_seen: i64,
}

/// Service registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Primal name
    pub name: String,
    /// Socket or endpoint path
    pub endpoint: String,
    /// Capabilities to register
    pub capabilities: Vec<String>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Family ID for lineage verification
    pub family_id: Option<String>,
}

/// Registration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// Whether registration succeeded
    pub success: bool,
    /// Registration ID
    pub registration_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Protocol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    /// Protocol name (jsonrpc, tarpc, http)
    pub name: String,
    /// Whether protocol is available
    pub available: bool,
    /// Protocol version
    pub version: String,
    /// Endpoint for this protocol
    pub endpoint: Option<String>,
}

/// Signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureResult {
    /// Whether signing succeeded
    pub success: bool,
    /// Signature bytes
    pub signature: Option<Vec<u8>>,
    /// Error message if failed
    pub error: Option<String>,
}

/// JWT secret result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSecretResult {
    /// Whether retrieval succeeded
    pub success: bool,
    /// JWT secret
    pub secret: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Lineage verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageResult {
    /// Whether verification succeeded
    pub verified: bool,
    /// Family ID
    pub family_id: Option<String>,
    /// Generation number
    pub generation: Option<u32>,
    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// Protocol Negotiation
// ============================================================================

/// Protocol preference for escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProtocolPreference {
    /// Use JSON-RPC only
    JsonRpcOnly,
    /// Prefer JSON-RPC, fallback to tarpc
    #[default]
    PreferJsonRpc,
    /// Prefer tarpc, fallback to JSON-RPC
    PreferTarpc,
    /// Use tarpc only
    TarpcOnly,
    /// Auto-detect based on capabilities
    Auto,
}

/// Environment variable for protocol preference
pub const PROTOCOL_ENV_VAR: &str = "IPC_PROTOCOL";

/// Parse protocol preference from environment
pub fn protocol_from_env() -> ProtocolPreference {
    match std::env::var(PROTOCOL_ENV_VAR).as_deref() {
        Ok("jsonrpc") | Ok("json-rpc") => ProtocolPreference::JsonRpcOnly,
        Ok("tarpc") => ProtocolPreference::TarpcOnly,
        Ok("prefer-jsonrpc") => ProtocolPreference::PreferJsonRpc,
        Ok("prefer-tarpc") => ProtocolPreference::PreferTarpc,
        Ok("auto") => ProtocolPreference::Auto,
        _ => ProtocolPreference::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        let status = HealthStatus {
            healthy: true,
            message: Some("Running".to_string()),
            uptime_secs: 3600,
        };
        assert!(status.healthy);
    }

    #[test]
    fn test_protocol_preference_default() {
        assert_eq!(
            ProtocolPreference::default(),
            ProtocolPreference::PreferJsonRpc
        );
    }

    #[test]
    fn test_service_info() {
        let info = ServiceInfo {
            name: "beardog".to_string(),
            endpoint: "/tmp/beardog.sock".to_string(),
            capabilities: vec!["security".to_string()],
            healthy: true,
            protocols: vec!["jsonrpc".to_string(), "tarpc".to_string()],
            last_seen: 0,
        };
        assert_eq!(info.name, "beardog");
        assert!(info.protocols.contains(&"tarpc".to_string()));
    }
}
