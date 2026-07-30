// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Living Graph: Runtime Protocol State for Primal Connections
//!
//! This module tracks the runtime protocol state of all primal connections,
//! enabling dynamic escalation from JSON-RPC (bootstrap) to tarpc (production).
//!
//! # Design Principles
//!
//! - **JSON-RPC First**: Bootstrap, configuration, and debugging always use JSON-RPC
//! - **tarpc for Performance**: Hot-paths escalate to tarpc for ~10x latency improvement
//! - **Graceful Degradation**: Automatic fallback to JSON-RPC if tarpc fails
//! - **Living Graph**: Runtime protocol state tracked per-connection, not per-primal
//!
//! # Architecture
//!
//! ```text
//! LivingGraph
//!     ├── protocol_state: HashMap<PrimalId, PrimalProtocolState>
//!     ├── connections: HashMap<ConnectionId, ConnectionState>
//!     └── metrics: ConnectionMetrics (per-connection)
//! ```

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::neural_graph::Graph as DeploymentGraph;

/// Unique identifier for a connection between two primals
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionId {
    /// Source primal name
    pub from: String,
    /// Target primal name
    pub to: String,
}

impl ConnectionId {
    /// Create a connection ID from source and target primal names.
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.from, self.to)
    }
}

/// Protocol mode for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolMode {
    /// Bootstrap/debug - JSON-RPC only (default)
    #[default]
    JsonRpc,
    /// Production - tarpc for hot-paths
    Tarpc,
    /// Hybrid - JSON-RPC for control, tarpc for data
    Hybrid,
    /// Degraded - fell back from tarpc to JSON-RPC
    Degraded,
}

impl std::fmt::Display for ProtocolMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonRpc => write!(f, "JSON-RPC"),
            Self::Tarpc => write!(f, "tarpc"),
            Self::Hybrid => write!(f, "Hybrid"),
            Self::Degraded => write!(f, "Degraded"),
        }
    }
}

/// Health status for a primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrimalHealth {
    /// Unknown health (not yet checked)
    #[default]
    Unknown,
    /// Healthy and responsive
    Healthy,
    /// Degraded performance
    Degraded,
    /// Unhealthy / not responding
    Unhealthy,
}

/// Protocol state for a single primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalProtocolState {
    /// Primal identifier
    pub primal_id: String,
    /// JSON-RPC socket path (always available)
    pub json_rpc_socket: PathBuf,
    /// tarpc socket path (optional, only if primal supports tarpc)
    pub tarpc_socket: Option<PathBuf>,
    /// Current protocol mode
    pub current_mode: ProtocolMode,
    /// Current health status
    pub health: PrimalHealth,
    /// Capabilities provided by this primal
    pub capabilities: Vec<String>,
    /// Last successful health check
    #[serde(skip)]
    pub last_health_check: Option<Instant>,
    /// Last escalation attempt
    #[serde(skip)]
    pub last_escalation_attempt: Option<Instant>,
    /// Number of consecutive tarpc failures (for fallback decision)
    pub tarpc_failure_count: u32,
}

impl PrimalProtocolState {
    /// Create a new primal protocol state (JSON-RPC mode by default)
    pub fn new(primal_id: impl Into<String>, json_rpc_socket: PathBuf) -> Self {
        Self {
            primal_id: primal_id.into(),
            json_rpc_socket,
            tarpc_socket: None,
            current_mode: ProtocolMode::JsonRpc,
            health: PrimalHealth::Unknown,
            capabilities: Vec::new(),
            last_health_check: None,
            last_escalation_attempt: None,
            tarpc_failure_count: 0,
        }
    }

    /// Set the tarpc socket path
    #[must_use]
    pub fn with_tarpc_socket(mut self, socket: PathBuf) -> Self {
        self.tarpc_socket = Some(socket);
        self
    }

    /// Set capabilities
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Check if tarpc is available for this primal
    #[must_use]
    pub const fn tarpc_available(&self) -> bool {
        self.tarpc_socket.is_some()
    }

    /// Record a successful health check
    pub fn record_health_check(&mut self, healthy: bool) {
        self.last_health_check = Some(Instant::now());
        self.health = if healthy {
            PrimalHealth::Healthy
        } else {
            PrimalHealth::Unhealthy
        };
    }

    /// Record a tarpc failure
    pub const fn record_tarpc_failure(&mut self) {
        self.tarpc_failure_count += 1;
    }

    /// Reset tarpc failure count (after successful call)
    pub const fn reset_tarpc_failures(&mut self) {
        self.tarpc_failure_count = 0;
    }
}

/// Metrics for a connection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Total number of requests
    pub request_count: u64,
    /// Total number of errors
    pub error_count: u64,
    /// Total latency in microseconds
    pub total_latency_us: u64,
    /// Average latency in microseconds
    pub avg_latency_us: f64,
    /// P50 latency estimate (rolling)
    pub p50_latency_us: u64,
    /// P95 latency estimate (rolling)
    pub p95_latency_us: u64,
    /// P99 latency estimate (rolling)
    pub p99_latency_us: u64,
    /// Maximum latency observed
    pub max_latency_us: u64,
}

impl ConnectionMetrics {
    /// Record a request with its latency
    pub fn record_request(&mut self, latency_us: u64, success: bool) {
        self.request_count += 1;
        if !success {
            self.error_count += 1;
        }
        self.total_latency_us += latency_us;
        self.avg_latency_us = self.total_latency_us as f64 / self.request_count as f64;
        self.max_latency_us = self.max_latency_us.max(latency_us);

        // Simple percentile estimation (exponential moving average)
        // In production, use a proper histogram (e.g., HdrHistogram)
        let alpha = 0.1;
        self.p50_latency_us =
            ((1.0 - alpha) * self.p50_latency_us as f64 + alpha * latency_us as f64) as u64;
        self.p95_latency_us = self.p95_latency_us.max(latency_us * 95 / 100);
        self.p99_latency_us = self.p99_latency_us.max(latency_us * 99 / 100);
    }

    /// Get error rate (0.0 - 1.0)
    #[must_use]
    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.request_count as f64
        }
    }
}

/// State of a connection between two primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    /// Connection identifier
    pub id: ConnectionId,
    /// Source primal
    pub from: String,
    /// Target primal
    pub to: String,
    /// Current protocol mode
    pub protocol: ProtocolMode,
    /// Connection metrics
    pub metrics: ConnectionMetrics,
    /// When the connection was established
    #[serde(skip)]
    pub established_at: Option<Instant>,
    /// Number of escalation attempts
    pub escalation_attempts: u32,
    /// Number of fallback events
    pub fallback_count: u32,
    /// Last escalation time
    #[serde(skip)]
    pub last_escalation: Option<Instant>,
}

impl ConnectionState {
    /// Create a new connection state
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        let from = from.into();
        let to = to.into();
        Self {
            id: ConnectionId::new(&from, &to),
            from,
            to,
            protocol: ProtocolMode::JsonRpc,
            metrics: ConnectionMetrics::default(),
            established_at: Some(Instant::now()),
            escalation_attempts: 0,
            fallback_count: 0,
            last_escalation: None,
        }
    }

    /// Record a request with latency
    pub fn record_request(&mut self, latency_us: u64, success: bool) {
        self.metrics.record_request(latency_us, success);
    }

    /// Escalate to tarpc
    pub fn escalate(&mut self) {
        self.protocol = ProtocolMode::Tarpc;
        self.escalation_attempts += 1;
        self.last_escalation = Some(Instant::now());
    }

    /// Fallback to JSON-RPC
    pub const fn fallback(&mut self) {
        self.protocol = ProtocolMode::Degraded;
        self.fallback_count += 1;
    }
}

/// Living deployment graph with runtime protocol state
pub struct LivingGraph {
    /// Static deployment definition (from TOML).
    /// Planned: wire up graph validation when graph-based deployment is active.
    _deployment: Option<DeploymentGraph>,
    /// Runtime state: primal → protocol state
    protocol_state: RwLock<HashMap<String, PrimalProtocolState>>,
    /// Active connections between primals
    connections: RwLock<HashMap<ConnectionId, ConnectionState>>,
    /// Family ID for this graph
    family_id: String,
    /// Request counter for generating unique IDs
    request_counter: AtomicU64,
}

impl LivingGraph {
    /// Create a new living graph
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            _deployment: None,
            protocol_state: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            family_id: family_id.into(),
            request_counter: AtomicU64::new(1),
        }
    }

    /// Create from a deployment graph
    pub fn from_deployment(family_id: impl Into<String>, deployment: DeploymentGraph) -> Self {
        Self {
            _deployment: Some(deployment),
            protocol_state: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            family_id: family_id.into(),
            request_counter: AtomicU64::new(1),
        }
    }

    /// Get family ID
    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    /// Register a primal with its protocol state
    pub async fn register_primal(&self, state: PrimalProtocolState) {
        let primal_id = state.primal_id.clone();
        info!(
            "📝 Registering primal {} (socket: {})",
            primal_id,
            state.json_rpc_socket.display()
        );
        self.protocol_state.write().await.insert(primal_id, state);
    }

    /// Get primal protocol state
    pub async fn get_primal_state(&self, primal_id: &str) -> Option<PrimalProtocolState> {
        self.protocol_state.read().await.get(primal_id).cloned()
    }

    /// Update primal protocol state
    pub async fn update_primal_state<F>(&self, primal_id: &str, update_fn: F)
    where
        F: FnOnce(&mut PrimalProtocolState),
    {
        if let Some(state) = self.protocol_state.write().await.get_mut(primal_id) {
            update_fn(state);
        }
    }

    /// Get all primal states
    pub async fn get_all_primal_states(&self) -> Vec<PrimalProtocolState> {
        self.protocol_state.read().await.values().cloned().collect()
    }

    /// Register or update a connection
    pub async fn register_connection(&self, from: &str, to: &str) {
        let id = ConnectionId::new(from, to);
        let mut connections = self.connections.write().await;

        if !connections.contains_key(&id) {
            info!("🔗 Registering connection: {} → {}", from, to);
            connections.insert(id.clone(), ConnectionState::new(from, to));
        }
    }

    /// Get connection state
    pub async fn get_connection(&self, from: &str, to: &str) -> Option<ConnectionState> {
        let id = ConnectionId::new(from, to);
        self.connections.read().await.get(&id).cloned()
    }

    /// Update connection protocol mode
    pub async fn update_connection_protocol(&self, from: &str, to: &str, mode: ProtocolMode) {
        let id = ConnectionId::new(from, to);
        if let Some(conn) = self.connections.write().await.get_mut(&id) {
            info!(
                "🔄 Updating connection {} protocol: {} → {}",
                id, conn.protocol, mode
            );
            conn.protocol = mode;
            if mode == ProtocolMode::Tarpc {
                conn.escalate();
            } else if mode == ProtocolMode::Degraded {
                conn.fallback();
            }
        }
    }

    /// Record a request on a connection
    pub async fn record_request(&self, from: &str, to: &str, latency_us: u64, success: bool) {
        let id = ConnectionId::new(from, to);
        if let Some(conn) = self.connections.write().await.get_mut(&id) {
            conn.record_request(latency_us, success);
            debug!(
                "📊 Connection {} request: {}μs, success={}, total={}",
                id, latency_us, success, conn.metrics.request_count
            );
        }
    }

    /// Get all connections
    pub async fn get_all_connections(&self) -> Vec<ConnectionState> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Get connections by protocol mode
    pub async fn get_connections_by_mode(&self, mode: ProtocolMode) -> Vec<ConnectionState> {
        self.connections
            .read()
            .await
            .values()
            .filter(|c| c.protocol == mode)
            .cloned()
            .collect()
    }

    /// Get protocol summary (counts by mode)
    pub async fn get_protocol_summary(&self) -> ProtocolSummary {
        let connections = self.connections.read().await;
        ProtocolSummary {
            json_rpc: connections
                .values()
                .filter(|c| c.protocol == ProtocolMode::JsonRpc)
                .count(),
            tarpc: connections
                .values()
                .filter(|c| c.protocol == ProtocolMode::Tarpc)
                .count(),
            hybrid: connections
                .values()
                .filter(|c| c.protocol == ProtocolMode::Hybrid)
                .count(),
            degraded: connections
                .values()
                .filter(|c| c.protocol == ProtocolMode::Degraded)
                .count(),
        }
    }

    /// Get next request ID
    pub fn next_request_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Check if a primal is registered
    pub async fn has_primal(&self, primal_id: &str) -> bool {
        self.protocol_state.read().await.contains_key(primal_id)
    }

    /// Get number of registered primals
    pub async fn primal_count(&self) -> usize {
        self.protocol_state.read().await.len()
    }

    /// Get number of connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Remove a primal and its connections
    pub async fn remove_primal(&self, primal_id: &str) {
        info!("🗑️ Removing primal {} from living graph", primal_id);

        // Remove from protocol state
        self.protocol_state.write().await.remove(primal_id);

        // Remove all connections involving this primal
        let mut connections = self.connections.write().await;
        connections.retain(|id, _| id.from != primal_id && id.to != primal_id);
    }

    /// Get primals that are healthy and tarpc-capable
    pub async fn get_tarpc_capable_primals(&self) -> Vec<PrimalProtocolState> {
        self.protocol_state
            .read()
            .await
            .values()
            .filter(|p| p.tarpc_available() && p.health == PrimalHealth::Healthy)
            .cloned()
            .collect()
    }

    /// Get JSON-RPC connections that could be escalated
    pub async fn get_escalation_candidates(
        &self,
        min_requests: u64,
        latency_threshold_us: f64,
    ) -> Vec<ConnectionState> {
        self.connections
            .read()
            .await
            .values()
            .filter(|c| {
                c.protocol == ProtocolMode::JsonRpc
                    && c.metrics.request_count >= min_requests
                    && c.metrics.avg_latency_us > latency_threshold_us
            })
            .cloned()
            .collect()
    }
}

/// Protocol summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolSummary {
    /// Connections using JSON-RPC only
    pub json_rpc: usize,
    /// Connections using tarpc
    pub tarpc: usize,
    /// Connections using hybrid (both)
    pub hybrid: usize,
    /// Connections in degraded fallback mode
    pub degraded: usize,
}

impl ProtocolSummary {
    /// Total number of connections
    #[must_use]
    pub const fn total(&self) -> usize {
        self.json_rpc + self.tarpc + self.hybrid + self.degraded
    }
}

#[cfg(test)]
#[path = "living_graph_tests.rs"]
mod tests;
