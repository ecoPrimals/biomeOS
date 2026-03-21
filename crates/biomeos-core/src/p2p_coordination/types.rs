// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Types for P2P coordination
//!
//! These types are **agnostic** - they work with any primal that provides
//! the required capabilities.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Lineage proof for genetic cryptography
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// Lineage identifier
    pub lineage_id: String,

    /// Depth in lineage tree
    pub depth: u32,

    /// Cryptographic proof (zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub proof: Bytes,

    /// Timestamp
    pub timestamp: SystemTime,
}

/// Request for a secure tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    /// Tunnel identifier
    pub id: String,

    /// Endpoint for node A
    pub endpoint_a: TransportEndpoint,

    /// Endpoint for node B
    pub endpoint_b: TransportEndpoint,

    /// Encryption key (for coordination, zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub encryption_key: Bytes,

    /// Tunnel created at
    pub created_at: SystemTime,
}

/// Transport endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEndpoint {
    /// Node identifier
    pub node_id: String,

    /// Network address
    pub address: String,

    /// Port
    pub port: u16,

    /// Protocol (tcp, udp, quic, etc.)
    pub protocol: String,

    /// Whether endpoint uses TLS
    pub secure: bool,
}

/// Tunnel health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelHealth {
    /// Encryption status
    pub encryption_status: HealthStatus,

    /// Forward secrecy enabled
    pub forward_secrecy: bool,

    /// Last key rotation
    pub last_key_rotation: Option<SystemTime>,

    /// Overall status
    pub status: HealthStatus,
}

/// Transport health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportHealth {
    /// Connection status
    pub connection_status: HealthStatus,

    /// Latency in milliseconds
    pub latency_ms: Option<u32>,

    /// Packet loss percentage
    pub packet_loss: Option<f32>,

    /// Overall status
    pub status: HealthStatus,
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Fully operational
    Healthy,

    /// Degraded but functional
    Degraded,

    /// Not functional
    Unhealthy,
}

/// Overall health combining multiple components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealth {
    /// Tunnel identifier
    pub tunnel_id: String,

    /// Security component health
    pub security_health: TunnelHealth,

    /// Transport component health
    pub transport_health: TransportHealth,

    /// Combined status
    pub status: HealthStatus,
}

/// Tunnel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    /// Tunnel identifier
    pub tunnel_id: String,

    /// Current status
    pub status: TunnelStatus,

    /// Endpoints
    pub endpoints: Vec<TransportEndpoint>,

    /// Established at
    pub established_at: SystemTime,
}

/// Tunnel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelStatus {
    /// Tunnel is active
    Active,

    /// Tunnel is establishing
    Establishing,

    /// Tunnel is degraded
    Degraded,

    /// Tunnel is closed
    Closed,
}

/// Broadcast encryption keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastKeys {
    /// Broadcast encryption key (zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub broadcast_key: Bytes,

    /// Lineage proof for filtering
    pub lineage_proof: LineageProof,

    /// Generated at
    pub generated_at: SystemTime,
}

/// Configuration for encrypted discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDiscoveryConfig {
    /// Encryption key (zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub encryption_key: Bytes,

    /// Lineage filter
    pub lineage_filter: LineageProof,

    /// Discovery mode
    pub mode: DiscoveryMode,
}

/// Discovery mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMode {
    /// Plaintext discovery (trusted LAN)
    Plaintext,

    /// Encrypted discovery (BirdSong mode)
    Encrypted,
}

/// Broadcast test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastTest {
    /// Whether broadcast was encrypted
    pub encrypted: bool,

    /// Test timestamp
    pub timestamp: SystemTime,

    /// Test successful
    pub success: bool,
}

/// Lineage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    /// Whether requester is ancestor of target
    pub is_ancestor: bool,

    /// Lineage depth
    pub depth: u32,

    /// Lineage proof
    pub proof: LineageProof,
}

/// Relay offer from a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayOffer {
    /// Relay node identifier
    pub relay_node: String,

    /// Relay endpoint
    pub relay_endpoint: TransportEndpoint,

    /// Offer expires at
    pub expires_at: SystemTime,

    /// Lineage verification
    pub lineage_verified: bool,
}

/// Active relay connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConnection {
    /// Connection identifier
    pub connection_id: String,

    /// Relay node
    pub relay_node: String,

    /// Established at
    pub established_at: SystemTime,

    /// Status
    pub status: RelayStatus,
}

/// Relay status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayStatus {
    /// Relay is active
    Active,

    /// Relay is establishing
    Establishing,

    /// Relay has failed
    Failed,
}

/// Relay information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayInfo {
    /// Relay node identifier
    pub relay_node: String,

    /// Requester node
    pub requester: String,

    /// Target node
    pub target: String,

    /// Current status
    pub status: RelayStatus,
}
