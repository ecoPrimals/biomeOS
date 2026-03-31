// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Serde helpers for `bytes::Bytes` — serializes as base64 for JSON-RPC wire
/// compatibility while preserving zero-copy semantics in memory.
pub mod bytes_serde {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serialize `Bytes` as base64 string.
    pub fn serialize<S: Serializer>(b: &Bytes, s: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(b))
    }

    /// Deserialize `Bytes` from base64 string.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
        use base64::Engine;
        let encoded = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .map(Bytes::from)
            .map_err(serde::de::Error::custom)
    }
}

/// Serde helpers for `Option<Bytes>`.
pub mod option_bytes_serde {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serialize `Option<Bytes>` as optional base64 string.
    pub fn serialize<S: Serializer>(opt: &Option<Bytes>, s: S) -> Result<S::Ok, S::Error> {
        match opt {
            Some(b) => super::bytes_serde::serialize(b, s),
            None => s.serialize_none(),
        }
    }

    /// Deserialize `Option<Bytes>` from optional base64 string.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Bytes>, D::Error> {
        use base64::Engine;
        let opt: Option<String> = Option::deserialize(d)?;
        opt.map_or_else(
            || Ok(None),
            |encoded| {
                base64::engine::general_purpose::STANDARD
                    .decode(&encoded)
                    .map(|v| Some(Bytes::from(v)))
                    .map_err(serde::de::Error::custom)
            },
        )
    }
}

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

/// Security service - implemented by `BearDog`
///
/// Binary payloads use `bytes::Bytes` for zero-copy passing between layers.
#[tarpc::service]
pub trait SecurityRpc {
    /// Sign data with primal's key
    async fn sign(data: Bytes) -> SignatureResult;

    /// Verify a signature
    async fn verify(data: Bytes, signature: Bytes, public_key: Bytes) -> bool;

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
    /// Signature bytes (zero-copy via `bytes::Bytes`)
    #[serde(default, with = "option_bytes_serde")]
    pub signature: Option<Bytes>,
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
///
/// Returns [`ProtocolPreference`] based on the `IPC_PROTOCOL` environment variable.
/// Defaults to `Auto` if the variable is unset or contains an unrecognized value.
#[must_use]
pub fn protocol_from_env() -> ProtocolPreference {
    protocol_from_value(std::env::var(PROTOCOL_ENV_VAR).ok().as_deref())
}

/// Parse protocol preference from an optional value (same rules as [`protocol_from_env`]).
#[must_use]
pub fn protocol_from_value(val: Option<&str>) -> ProtocolPreference {
    match val {
        Some("jsonrpc" | "json-rpc") => ProtocolPreference::JsonRpcOnly,
        Some("tarpc") => ProtocolPreference::TarpcOnly,
        Some("prefer-jsonrpc") => ProtocolPreference::PreferJsonRpc,
        Some("prefer-tarpc") => ProtocolPreference::PreferTarpc,
        Some("auto" | _) | None => ProtocolPreference::Auto,
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

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
    fn test_health_metrics() {
        let metrics = HealthMetrics {
            healthy: true,
            cpu_usage: 25.5,
            memory_bytes: 1_000_000,
            active_connections: 10,
            total_requests: 1000,
            total_errors: 5,
            avg_latency_us: 100,
        };
        assert!(metrics.healthy);
        assert!((metrics.cpu_usage - 25.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_version_info() {
        let info = VersionInfo {
            version: "0.1.0".to_string(),
            git_commit: Some("abc123".to_string()),
            build_timestamp: Some("2026-01-01".to_string()),
            protocols: vec!["jsonrpc".to_string(), "tarpc".to_string()],
        };
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.git_commit.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_protocol_preference_default() {
        assert_eq!(
            ProtocolPreference::default(),
            ProtocolPreference::PreferJsonRpc
        );
    }

    #[test]
    fn test_protocol_from_env() {
        assert_eq!(
            protocol_from_value(Some("tarpc")),
            ProtocolPreference::TarpcOnly
        );
        assert_eq!(
            protocol_from_value(Some("jsonrpc")),
            ProtocolPreference::JsonRpcOnly
        );
        assert_eq!(
            protocol_from_value(Some("prefer-tarpc")),
            ProtocolPreference::PreferTarpc
        );
        assert_eq!(protocol_from_value(None), ProtocolPreference::Auto);
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

    #[test]
    fn test_service_registration() {
        let reg = ServiceRegistration {
            name: "songbird".to_string(),
            endpoint: "/tmp/songbird.sock".to_string(),
            capabilities: vec!["discovery".to_string()],
            protocols: vec!["jsonrpc".to_string()],
            family_id: Some("family-1".to_string()),
        };
        assert_eq!(reg.name, "songbird");
        assert_eq!(reg.family_id.as_deref(), Some("family-1"));
    }

    #[test]
    fn test_registration_result() {
        let ok = RegistrationResult {
            success: true,
            registration_id: Some("reg-123".to_string()),
            error: None,
        };
        assert!(ok.success);
        assert_eq!(ok.registration_id.as_deref(), Some("reg-123"));

        let err = RegistrationResult {
            success: false,
            registration_id: None,
            error: Some("Already registered".to_string()),
        };
        assert!(!err.success);
    }

    #[test]
    fn test_signature_result() {
        let ok = SignatureResult {
            success: true,
            signature: Some(Bytes::from_static(&[1, 2, 3])),
            error: None,
        };
        assert!(ok.success);
        assert_eq!(ok.signature.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_signature_result_serde_roundtrip() {
        let original = SignatureResult {
            success: true,
            signature: Some(Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef])),
            error: None,
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: SignatureResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.signature, original.signature);
    }

    #[test]
    fn test_lineage_result() {
        let result = LineageResult {
            verified: true,
            family_id: Some("fam-1".to_string()),
            generation: Some(2),
            error: None,
        };
        assert!(result.verified);
        assert_eq!(result.generation, Some(2));
    }

    #[test]
    fn test_protocol_info() {
        let info = ProtocolInfo {
            name: "tarpc".to_string(),
            available: true,
            version: "1.0".to_string(),
            endpoint: Some("/tmp/tarpc.sock".to_string()),
        };
        assert_eq!(info.name, "tarpc");
        assert!(info.available);
    }

    #[test]
    fn test_tarpc_types_serialization() {
        let status = HealthStatus {
            healthy: true,
            message: None,
            uptime_secs: 100,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let _: HealthStatus = serde_json::from_str(&json).expect("deserialize");
    }
}
