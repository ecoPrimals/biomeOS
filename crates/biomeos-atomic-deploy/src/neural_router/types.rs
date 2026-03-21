// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural Router types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Discovered primal with socket and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "beardog", "songbird"). Uses Arc&lt;str&gt; for zero-copy cloning.
    pub name: Arc<str>,

    /// Unix socket path
    pub socket_path: PathBuf,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,

    /// Health status
    pub healthy: bool,

    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Atomic type composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomicType {
    /// Tower Atomic: BearDog + Songbird (secure communications)
    Tower,

    /// Nest Atomic: Tower + NestGate (secure storage)
    Nest,

    /// Node Atomic: Tower + ToadStool (secure compute)
    Node,
}

/// Result of capability discovery
#[derive(Debug, Clone)]
pub struct DiscoveredAtomic {
    /// Capability that was discovered. Uses Arc&lt;str&gt; for zero-copy cloning.
    pub capability: Arc<str>,

    /// Primals that provide this capability
    pub primals: Vec<DiscoveredPrimal>,

    /// Atomic type (if applicable)
    pub atomic_type: Option<AtomicType>,

    /// Primary primal to route to
    pub primary_socket: PathBuf,
}

/// Metrics for a routing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Unique request ID. Uses Arc&lt;str&gt; for zero-copy cloning.
    pub request_id: Arc<str>,

    /// Capability requested
    pub capability: Arc<str>,

    /// Method called
    pub method: Arc<str>,

    /// Primals involved in routing
    pub routed_through: Vec<Arc<str>>,

    /// Total latency in milliseconds
    pub latency_ms: u64,

    /// Success status
    pub success: bool,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Capability registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredCapability {
    /// Capability name (e.g., "http.request", "crypto.sign"). Uses Arc&lt;str&gt; for zero-copy cloning.
    pub capability: Arc<str>,

    /// Primal that provides it
    pub primal_name: Arc<str>,

    /// Socket path
    pub socket_path: PathBuf,

    /// When it was registered
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Source of registration (graph, primal_announcement, manual)
    pub source: Arc<str>,
}
