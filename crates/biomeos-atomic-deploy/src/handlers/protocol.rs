// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

#![deny(unsafe_code)]

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
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
    pub fn new(
        graph: Arc<LivingGraph>,
        escalation_manager: Arc<RwLock<ProtocolEscalationManager>>,
    ) -> Self {
        Self {
            graph,
            escalation_manager,
        }
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
            .ok_or_else(|| anyhow!("Connection not found: {} → {}", from, to))
    }

    /// Handle graph.protocol_map
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

    /// Handle protocol.register_primal
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

    /// Handle protocol.register_connection
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

    /// Handle protocol.record_request
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

    /// Handle protocol.start_monitoring
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

    /// Handle protocol.stop_monitoring
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol_escalation::EscalationConfig;

    async fn create_test_handler() -> ProtocolHandler {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
            graph.clone(),
            EscalationConfig::default(),
        )));
        ProtocolHandler::new(graph, manager)
    }

    #[tokio::test]
    async fn test_status() {
        let handler = create_test_handler().await;

        handler.graph.register_connection("a", "b").await;

        let result = handler.status().await.unwrap();

        assert!(result.get("connections").is_some());
        assert!(result.get("summary").is_some());
        assert_eq!(result["summary"]["total"], 1);
    }

    #[tokio::test]
    async fn test_protocol_map() {
        let handler = create_test_handler().await;

        // Register a primal
        handler
            .graph
            .register_primal(PrimalProtocolState::new(
                "beardog",
                std::path::PathBuf::from("/tmp/beardog.sock"),
            ))
            .await;

        // Register a connection
        handler
            .graph
            .register_connection("songbird", "beardog")
            .await;

        let result = handler.protocol_map().await.unwrap();

        assert_eq!(result["family_id"], "test-family");
        assert_eq!(result["summary"]["primal_count"], 1);
        assert_eq!(result["summary"]["connection_count"], 1);
    }

    #[tokio::test]
    async fn test_register_primal() {
        let handler = create_test_handler().await;

        let params = Some(json!({
            "primal_id": "test-primal",
            "json_rpc_socket": "/tmp/test.sock",
            "capabilities": ["capability1", "capability2"]
        }));

        let result = handler.register_primal(&params).await.unwrap();

        assert_eq!(result["status"], "registered");
        assert_eq!(result["primal_id"], "test-primal");

        // Verify registration
        assert!(handler.graph.has_primal("test-primal").await);
    }

    #[tokio::test]
    async fn test_register_connection() {
        let handler = create_test_handler().await;

        let params = Some(json!({
            "from": "primal-a",
            "to": "primal-b"
        }));

        let result = handler.register_connection(&params).await.unwrap();

        assert_eq!(result["status"], "registered");
        assert_eq!(result["from"], "primal-a");
        assert_eq!(result["to"], "primal-b");

        // Verify registration
        assert!(handler
            .graph
            .get_connection("primal-a", "primal-b")
            .await
            .is_some());
    }

    #[tokio::test]
    async fn test_record_request() {
        let handler = create_test_handler().await;

        // First register the connection
        handler.graph.register_connection("a", "b").await;

        let params = Some(json!({
            "from": "a",
            "to": "b",
            "latency_us": 150,
            "success": true
        }));

        let result = handler.record_request(&params).await.unwrap();

        assert_eq!(result["status"], "recorded");
        assert_eq!(result["latency_us"], 150);

        // Verify metrics
        let conn = handler.graph.get_connection("a", "b").await.unwrap();
        assert_eq!(conn.metrics.request_count, 1);
    }

    #[tokio::test]
    async fn test_metrics() {
        let handler = create_test_handler().await;

        // Register and add requests
        handler.graph.register_connection("x", "y").await;

        for i in 0..10 {
            handler
                .graph
                .record_request("x", "y", 100 + i * 10, true)
                .await;
        }

        let params = Some(json!({
            "from": "x",
            "to": "y"
        }));

        let result = handler.metrics(&params).await.unwrap();

        assert!(result.get("connection").is_some());
        assert!(result.get("metrics").is_some());
        assert_eq!(result["metrics"]["request_count"], 10);
    }

    #[tokio::test]
    async fn test_missing_params() {
        let handler = create_test_handler().await;

        // Missing 'from'
        let params = Some(json!({ "to": "b" }));
        let result = handler.escalate(&params).await;
        assert!(result.is_err());

        // Missing 'to'
        let params = Some(json!({ "from": "a" }));
        let result = handler.escalate(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_escalate_none_params() {
        let handler = create_test_handler().await;
        let result = handler.escalate(&None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("missing"),
            "Error should mention missing params"
        );
    }

    #[tokio::test]
    async fn test_fallback_missing_params() {
        let handler = create_test_handler().await;

        let params = Some(json!({ "to": "b" }));
        let result = handler.fallback(&params).await;
        assert!(result.is_err());

        let params = Some(json!({ "from": "a" }));
        let result = handler.fallback(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_none_params() {
        let handler = create_test_handler().await;
        let result = handler.fallback(&None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fallback_with_reason() {
        let handler = create_test_handler().await;
        handler.graph.register_connection("src", "dst").await;

        let params = Some(json!({
            "from": "src",
            "to": "dst",
            "reason": "manual_test"
        }));

        let result = handler.fallback(&params).await.unwrap();
        assert!(result.get("status").is_some());
        assert_eq!(result["from"], "src");
        assert_eq!(result["to"], "dst");
    }

    #[tokio::test]
    async fn test_fallback_default_reason() {
        let handler = create_test_handler().await;
        handler.graph.register_connection("a", "b").await;

        let params = Some(json!({ "from": "a", "to": "b" }));
        let result = handler.fallback(&params).await.unwrap();
        assert_eq!(result["status"], "degraded");
    }

    #[tokio::test]
    async fn test_metrics_missing_params() {
        let handler = create_test_handler().await;

        let params = Some(json!({ "to": "b" }));
        let result = handler.metrics(&params).await;
        assert!(result.is_err());

        let params = Some(json!({ "from": "a" }));
        let result = handler.metrics(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metrics_none_params() {
        let handler = create_test_handler().await;
        let result = handler.metrics(&None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metrics_connection_not_found() {
        let handler = create_test_handler().await;

        let params = Some(json!({
            "from": "nonexistent",
            "to": "also-nonexistent"
        }));
        let result = handler.metrics(&params).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("not found"),
            "Error should mention connection not found"
        );
    }

    #[tokio::test]
    async fn test_register_primal_with_tarpc_socket() {
        let handler = create_test_handler().await;

        let params = Some(json!({
            "primal_id": "tarpc-primal",
            "json_rpc_socket": "/tmp/json.sock",
            "tarpc_socket": "/tmp/tarpc.sock",
            "capabilities": ["rpc"]
        }));

        let result = handler.register_primal(&params).await.unwrap();
        assert_eq!(result["status"], "registered");
        assert_eq!(result["primal_id"], "tarpc-primal");

        assert!(handler.graph.has_primal("tarpc-primal").await);
        let state = handler
            .graph
            .get_primal_state("tarpc-primal")
            .await
            .expect("primal state");
        assert!(state.tarpc_socket.is_some());
        assert_eq!(
            state.tarpc_socket.as_ref().unwrap().to_string_lossy(),
            "/tmp/tarpc.sock"
        );
    }

    #[tokio::test]
    async fn test_register_primal_missing_params() {
        let handler = create_test_handler().await;

        let result = handler.register_primal(&None).await;
        assert!(result.is_err());

        let params = Some(json!({}));
        let result = handler.register_primal(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_connection_missing_params() {
        let handler = create_test_handler().await;

        let result = handler.register_connection(&None).await;
        assert!(result.is_err());

        let params = Some(json!({ "from": "a" }));
        let result = handler.register_connection(&params).await;
        assert!(result.is_err());

        let params = Some(json!({ "to": "b" }));
        let result = handler.register_connection(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_request_success_false() {
        let handler = create_test_handler().await;
        handler.graph.register_connection("a", "b").await;

        let params = Some(json!({
            "from": "a",
            "to": "b",
            "latency_us": 200,
            "success": false
        }));

        let result = handler.record_request(&params).await.unwrap();
        assert_eq!(result["success"], false);

        let conn = handler.graph.get_connection("a", "b").await.unwrap();
        assert_eq!(conn.metrics.error_count, 1);
    }

    #[tokio::test]
    async fn test_record_request_default_success() {
        let handler = create_test_handler().await;
        handler.graph.register_connection("x", "y").await;

        let params = Some(json!({
            "from": "x",
            "to": "y",
            "latency_us": 100
        }));

        let result = handler.record_request(&params).await.unwrap();
        assert_eq!(result["success"], true);
    }

    #[tokio::test]
    async fn test_record_request_missing_params() {
        let handler = create_test_handler().await;

        let result = handler.record_request(&None).await;
        assert!(result.is_err());

        let params = Some(json!({ "from": "a", "to": "b" }));
        let result = handler.record_request(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_monitoring() {
        let handler = create_test_handler().await;
        let result = handler.start_monitoring().await.unwrap();
        assert_eq!(result["status"], "started");
        assert!(result["message"].as_str().unwrap().contains("started"));
    }

    #[tokio::test]
    async fn test_stop_monitoring() {
        let handler = create_test_handler().await;
        let result = handler.stop_monitoring().await.unwrap();
        assert_eq!(result["status"], "stopped");
        assert!(result["message"].as_str().unwrap().contains("stopped"));
    }

    #[tokio::test]
    async fn test_protocol_map_empty() {
        let handler = create_test_handler().await;
        let result = handler.protocol_map().await.unwrap();

        assert_eq!(result["family_id"], "test-family");
        assert_eq!(result["summary"]["primal_count"], 0);
        assert_eq!(result["summary"]["connection_count"], 0);
        assert!(result["nodes"].as_array().unwrap().is_empty());
        assert!(result["edges"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_protocol_map_node_structure() {
        let handler = create_test_handler().await;
        handler
            .graph
            .register_primal(PrimalProtocolState::new(
                "test-node",
                std::path::PathBuf::from("/tmp/test.sock"),
            ))
            .await;

        let result = handler.protocol_map().await.unwrap();
        let nodes = result["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["id"], "test-node");
        assert!(nodes[0].get("tarpc_available").is_some());
        assert!(nodes[0].get("current_mode").is_some());
    }

    #[tokio::test]
    async fn test_escalate_registered_connection() {
        let handler = create_test_handler().await;
        handler.graph.register_connection("client", "server").await;

        let params = Some(json!({
            "from": "client",
            "to": "server"
        }));

        let result = handler.escalate(&params).await.unwrap();
        assert!(result.get("status").is_some());
        assert_eq!(result["from"], "client");
        assert_eq!(result["to"], "server");
    }
}
