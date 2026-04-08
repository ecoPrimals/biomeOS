// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Protocol Escalation JSON-RPC Handlers
//!
//! This module provides the JSON-RPC interface for protocol management,
//! enabling clients to query and control protocol escalation.
//!
//! # Methods
//!
//! - `protocol.status` - Get all connection protocol states
//! - `protocol.escalate` - Manually escalate a connection
//! - `protocol.fallback` - Manually fallback a connection
//! - `protocol.metrics` - Get metrics for a connection
//! - `graph.protocol_map` - Get full Living Graph snapshot

#![forbid(unsafe_code)]

use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::living_graph::{LivingGraph, PrimalProtocolState};
use crate::protocol_escalation::ProtocolEscalationManager;

/// Handler for protocol-related JSON-RPC methods
#[derive(Clone)]
pub struct ProtocolHandler {
    /// Living graph with runtime state
    graph: Arc<LivingGraph>,
    /// Protocol escalation manager
    escalation_manager: Arc<RwLock<ProtocolEscalationManager>>,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub const fn new(
        graph: Arc<LivingGraph>,
        escalation_manager: Arc<RwLock<ProtocolEscalationManager>>,
    ) -> Self {
        Self {
            graph,
            escalation_manager,
        }
    }

    /// Access the living graph (crate tests in `protocol_tests.rs`).
    #[cfg(test)]
    pub(crate) fn living_graph(&self) -> &Arc<LivingGraph> {
        &self.graph
    }

    /// Handle protocol.status
    ///
    /// Returns the protocol status for all connections.
    ///
    /// # Request
    /// ```json
    /// {"jsonrpc":"2.0","method":"protocol.status","params":{},"id":1}
    /// ```
    ///
    /// # Response
    /// ```json
    /// {
    ///   "connections": [...],
    ///   "summary": { "json_rpc": 2, "tarpc": 1, ... }
    /// }
    /// ```
    pub async fn status(&self) -> Result<Value> {
        debug!("📊 protocol.status called");
        let manager = self.escalation_manager.read().await;
        Ok(manager.get_status().await)
    }

    /// Handle protocol.escalate
    ///
    /// Manually escalate a connection from JSON-RPC to tarpc.
    ///
    /// # Request
    /// ```json
    /// {"jsonrpc":"2.0","method":"protocol.escalate","params":{"from":"songbird","to":"beardog"},"id":1}
    /// ```
    pub async fn escalate(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?;

        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?;

        info!("🚀 protocol.escalate: {} → {}", from, to);

        let manager = self.escalation_manager.read().await;
        let result = manager
            .escalate_connection(from, to)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(json!({
            "status": if result.success { "escalated" } else { "failed" },
            "from": result.from,
            "to": result.to,
            "previous_mode": format!("{:?}", result.previous_mode),
            "current_mode": format!("{:?}", result.current_mode),
            "tarpc_socket": result.tarpc_socket,
            "message": result.message,
        }))
    }

    /// Handle protocol.fallback
    ///
    /// Manually fallback a connection from tarpc to JSON-RPC (degraded mode).
    ///
    /// # Request
    /// ```json
    /// {"jsonrpc":"2.0","method":"protocol.fallback","params":{"from":"songbird","to":"beardog","reason":"manual"},"id":1}
    /// ```
    pub async fn fallback(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?;

        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?;

        let reason = params
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("manual");

        warn!(
            "⚠️ protocol.fallback: {} → {} (reason: {})",
            from, to, reason
        );

        let manager = self.escalation_manager.read().await;
        let result = manager
            .fallback_connection(from, to, reason)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(json!({
            "status": if result.success { "degraded" } else { "failed" },
            "from": result.from,
            "to": result.to,
            "previous_mode": format!("{:?}", result.previous_mode),
            "current_mode": format!("{:?}", result.current_mode),
            "message": result.message,
        }))
    }

    /// Handle protocol.metrics
    ///
    /// Get detailed metrics for a specific connection.
    ///
    /// # Request
    /// ```json
    /// {"jsonrpc":"2.0","method":"protocol.metrics","params":{"from":"songbird","to":"beardog"},"id":1}
    /// ```
    pub async fn metrics(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?;

        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?;

        debug!("📈 protocol.metrics: {} → {}", from, to);

        let manager = self.escalation_manager.read().await;

        manager
            .get_connection_metrics(from, to)
            .await
            .ok_or_else(|| anyhow!("Connection not found: {from} → {to}"))
    }

    /// Handle `graph.protocol_map`
    ///
    /// Get a full snapshot of the Living Graph including all nodes and edges.
    ///
    /// # Request
    /// ```json
    /// {"jsonrpc":"2.0","method":"graph.protocol_map","params":{},"id":1}
    /// ```
    pub async fn protocol_map(&self) -> Result<Value> {
        debug!("🗺️ graph.protocol_map called");

        let primals = self.graph.get_all_primal_states().await;
        let connections = self.graph.get_all_connections().await;

        let nodes: Vec<Value> = primals
            .iter()
            .map(|p| {
                json!({
                    "id": p.primal_id,
                    "json_rpc_socket": p.json_rpc_socket,
                    "tarpc_socket": p.tarpc_socket,
                    "current_mode": format!("{:?}", p.current_mode),
                    "health": format!("{:?}", p.health),
                    "capabilities": p.capabilities,
                    "tarpc_available": p.tarpc_available(),
                })
            })
            .collect();

        let edges: Vec<Value> = connections
            .iter()
            .map(|c| {
                json!({
                    "from": c.from,
                    "to": c.to,
                    "protocol": format!("{:?}", c.protocol),
                    "latency_us": c.metrics.avg_latency_us,
                    "requests": c.metrics.request_count,
                    "errors": c.metrics.error_count,
                    "escalation_attempts": c.escalation_attempts,
                    "fallback_count": c.fallback_count,
                })
            })
            .collect();

        Ok(json!({
            "family_id": self.graph.family_id(),
            "nodes": nodes,
            "edges": edges,
            "summary": {
                "primal_count": primals.len(),
                "connection_count": connections.len(),
            }
        }))
    }

    /// Handle `protocol.register_primal`
    ///
    /// Register a primal's protocol state in the Living Graph.
    ///
    /// # Request
    /// ```json
    /// {
    ///   "jsonrpc":"2.0",
    ///   "method":"protocol.register_primal",
    ///   "params":{
    ///     "primal_id": "beardog",
    ///     "json_rpc_socket": "/run/user/1000/biomeos/beardog-family.sock",
    ///     "tarpc_socket": "/run/user/1000/biomeos/beardog-family-tarpc.sock",
    ///     "capabilities": ["crypto", "identity"]
    ///   },
    ///   "id":1
    /// }
    /// ```
    pub async fn register_primal(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let primal_id = params
            .get("primal_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'primal_id' parameter"))?;

        let json_rpc_socket = params
            .get("json_rpc_socket")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'json_rpc_socket' parameter"))?;

        let tarpc_socket = params
            .get("tarpc_socket")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from);

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        info!(
            "📝 protocol.register_primal: {} (socket: {})",
            primal_id, json_rpc_socket
        );

        let mut state =
            PrimalProtocolState::new(primal_id, std::path::PathBuf::from(json_rpc_socket))
                .with_capabilities(capabilities);

        if let Some(tarpc) = tarpc_socket {
            state = state.with_tarpc_socket(tarpc);
        }

        self.graph.register_primal(state).await;

        Ok(json!({
            "status": "registered",
            "primal_id": primal_id,
        }))
    }

    /// Handle `protocol.register_connection`
    ///
    /// Register a connection between two primals.
    ///
    /// # Request
    /// ```json
    /// {
    ///   "jsonrpc":"2.0",
    ///   "method":"protocol.register_connection",
    ///   "params":{"from": "songbird", "to": "beardog"},
    ///   "id":1
    /// }
    /// ```
    pub async fn register_connection(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?;

        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?;

        info!("🔗 protocol.register_connection: {} → {}", from, to);

        self.graph.register_connection(from, to).await;

        Ok(json!({
            "status": "registered",
            "from": from,
            "to": to,
        }))
    }

    /// Handle `protocol.record_request`
    ///
    /// Record a request on a connection (for metrics).
    ///
    /// # Request
    /// ```json
    /// {
    ///   "jsonrpc":"2.0",
    ///   "method":"protocol.record_request",
    ///   "params":{"from": "songbird", "to": "beardog", "latency_us": 150, "success": true},
    ///   "id":1
    /// }
    /// ```
    pub async fn record_request(&self, params: &Option<Value>) -> Result<Value> {
        let params = params
            .as_ref()
            .ok_or_else(|| anyhow!("Missing parameters"))?;

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' parameter"))?;

        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' parameter"))?;

        let latency_us = params
            .get("latency_us")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Missing 'latency_us' parameter"))?;

        let success = params
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        self.graph
            .record_request(from, to, latency_us, success)
            .await;

        Ok(json!({
            "status": "recorded",
            "from": from,
            "to": to,
            "latency_us": latency_us,
            "success": success,
        }))
    }

    /// Handle `protocol.start_monitoring`
    ///
    /// Start the background auto-escalation monitoring loop.
    pub async fn start_monitoring(&self) -> Result<Value> {
        info!("🚀 Starting protocol escalation monitoring");

        let manager = self.escalation_manager.clone();
        tokio::spawn(async move {
            let m = manager.read().await;
            m.start_monitoring().await;
        });

        Ok(json!({
            "status": "started",
            "message": "Protocol escalation monitoring started",
        }))
    }

    /// Handle `protocol.stop_monitoring`
    ///
    /// Stop the background auto-escalation monitoring loop.
    pub async fn stop_monitoring(&self) -> Result<Value> {
        info!("🛑 Stopping protocol escalation monitoring");

        let manager = self.escalation_manager.read().await;
        manager.stop_monitoring().await;

        Ok(json!({
            "status": "stopped",
            "message": "Protocol escalation monitoring stopped",
        }))
    }
}
