// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Protocol Escalation Engine: JSON-RPC → tarpc Runtime Evolution
//!
//! This module implements the escalation engine that manages runtime
//! escalation of primal connections from JSON-RPC to tarpc.

#![deny(unsafe_code)]

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::living_graph::{ConnectionState, LivingGraph, PrimalHealth, ProtocolMode};

use super::config::{EscalationConfig, EscalationResult, TarpcEndpoint};
use super::metrics;

/// Protocol Escalation Manager
pub struct ProtocolEscalationManager {
    /// Living graph with runtime state
    graph: Arc<LivingGraph>,
    /// Escalation configuration
    config: EscalationConfig,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Last check timestamp per connection
    last_check: RwLock<std::collections::HashMap<String, Instant>>,
}

impl ProtocolEscalationManager {
    /// Create a new protocol escalation manager
    pub fn new(graph: Arc<LivingGraph>, config: EscalationConfig) -> Self {
        Self {
            graph,
            config,
            running: Arc::new(RwLock::new(false)),
            last_check: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(graph: Arc<LivingGraph>) -> Self {
        Self::new(graph, EscalationConfig::default())
    }

    /// Get the living graph
    pub fn graph(&self) -> &Arc<LivingGraph> {
        &self.graph
    }

    /// Get the configuration
    pub fn config(&self) -> &EscalationConfig {
        &self.config
    }

    /// Start the background monitoring loop
    pub async fn start_monitoring(&self) {
        if !self.config.auto_escalate {
            info!("🔧 Auto-escalation disabled, skipping monitoring loop");
            return;
        }

        {
            let mut running = self.running.write().await;
            if *running {
                warn!("⚠️ Protocol escalation monitoring already running");
                return;
            }
            *running = true;
        }

        info!(
            "🚀 Starting protocol escalation monitoring (interval: {}s)",
            self.config.check_interval_secs
        );

        let check_interval = Duration::from_secs(self.config.check_interval_secs);
        let mut ticker = interval(check_interval);

        loop {
            ticker.tick().await;

            if !*self.running.read().await {
                info!("🛑 Protocol escalation monitoring stopped");
                break;
            }

            if let Err(e) = self.auto_escalate_check().await {
                error!("❌ Auto-escalation check failed: {}", e);
            }
        }
    }

    /// Stop the monitoring loop
    pub async fn stop_monitoring(&self) {
        *self.running.write().await = false;
        info!("🛑 Stopping protocol escalation monitoring");
    }

    /// Check all connections and escalate if needed
    pub async fn auto_escalate_check(&self) -> Result<(), String> {
        let candidates = self
            .graph
            .get_escalation_candidates(
                self.config.min_requests,
                self.config.latency_threshold_us as f64,
            )
            .await;

        debug!("🔍 Checking {} escalation candidates", candidates.len());

        for conn in candidates {
            if self.is_in_cooldown(&conn).await {
                debug!("⏳ Connection {} in cooldown, skipping", conn.id);
                continue;
            }

            let from_state = self.graph.get_primal_state(&conn.from).await;
            let to_state = self.graph.get_primal_state(&conn.to).await;

            let both_healthy = from_state
                .as_ref()
                .map(|s| s.health == PrimalHealth::Healthy)
                .unwrap_or(false)
                && to_state
                    .as_ref()
                    .map(|s| s.health == PrimalHealth::Healthy)
                    .unwrap_or(false);

            let target_has_tarpc = to_state
                .as_ref()
                .map(|s| s.tarpc_available())
                .unwrap_or(false);

            if both_healthy && target_has_tarpc {
                info!(
                    "🚀 Auto-escalating connection {} (latency: {:.1}μs, requests: {})",
                    conn.id, conn.metrics.avg_latency_us, conn.metrics.request_count
                );

                match self.escalate_connection(&conn.from, &conn.to).await {
                    Ok(result) => {
                        if result.success {
                            info!("✅ Escalation successful: {}", result.message);
                        } else {
                            warn!("⚠️ Escalation failed: {}", result.message);
                        }
                    }
                    Err(e) => {
                        error!("❌ Escalation error: {}", e);
                        self.record_cooldown(&conn.id.to_string()).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a connection is in cooldown
    pub(crate) async fn is_in_cooldown(&self, conn: &ConnectionState) -> bool {
        let cooldown = Duration::from_secs(self.config.escalation_cooldown_secs);
        let key = conn.id.to_string();

        if let Some(last) = self.last_check.read().await.get(&key) {
            last.elapsed() < cooldown
        } else {
            false
        }
    }

    /// Record cooldown for a connection
    pub(crate) async fn record_cooldown(&self, key: &str) {
        self.last_check
            .write()
            .await
            .insert(key.to_string(), Instant::now());
    }

    /// Attempt to escalate a connection to tarpc
    pub async fn escalate_connection(
        &self,
        from: &str,
        to: &str,
    ) -> Result<EscalationResult, String> {
        info!(
            "🚀 Attempting protocol escalation: {} → {} (JSON-RPC → tarpc)",
            from, to
        );

        let conn = self
            .graph
            .get_connection(from, to)
            .await
            .ok_or_else(|| format!("Connection not found: {} → {}", from, to))?;

        let previous_mode = conn.protocol;

        let tarpc_info = match self.query_tarpc_endpoint(to).await {
            Ok(info) => info,
            Err(e) => {
                return Ok(EscalationResult {
                    from: from.to_string(),
                    to: to.to_string(),
                    previous_mode,
                    current_mode: previous_mode,
                    tarpc_socket: None,
                    success: false,
                    message: format!("Failed to query tarpc endpoint: {}", e),
                });
            }
        };

        if !tarpc_info.available {
            return Ok(EscalationResult {
                from: from.to_string(),
                to: to.to_string(),
                previous_mode,
                current_mode: previous_mode,
                tarpc_socket: None,
                success: false,
                message: "Target primal does not support tarpc".to_string(),
            });
        }

        let tarpc_socket = tarpc_info.socket.clone();

        if let Err(e) = self.notify_escalation(from, to, &tarpc_info).await {
            return Ok(EscalationResult {
                from: from.to_string(),
                to: to.to_string(),
                previous_mode,
                current_mode: previous_mode,
                tarpc_socket,
                success: false,
                message: format!("Failed to notify source primal: {}", e),
            });
        }

        self.graph
            .update_connection_protocol(from, to, ProtocolMode::Tarpc)
            .await;

        if let Err(e) = self.verify_tarpc_connection(from, to).await {
            warn!(
                "⚠️ tarpc verification failed (will fall back on first real failure): {}",
                e
            );
        }

        info!("✅ Escalation complete: {} ═tarpc═► {}", from, to);

        Ok(EscalationResult {
            from: from.to_string(),
            to: to.to_string(),
            previous_mode,
            current_mode: ProtocolMode::Tarpc,
            tarpc_socket,
            success: true,
            message: format!("Successfully escalated {} → {} to tarpc", from, to),
        })
    }

    async fn query_tarpc_endpoint(&self, primal: &str) -> Result<TarpcEndpoint, String> {
        let state = self
            .graph
            .get_primal_state(primal)
            .await
            .ok_or_else(|| format!("Primal not found: {}", primal))?;

        if let Some(socket) = &state.tarpc_socket {
            return Ok(TarpcEndpoint {
                available: true,
                socket: Some(socket.clone()),
                services: state.capabilities.clone(),
            });
        }

        let request = json!({
            "jsonrpc": "2.0",
            "method": "rpc.tarpc_endpoint",
            "params": {},
            "id": self.graph.next_request_id(),
        });

        match self.send_json_rpc(&state.json_rpc_socket, &request).await {
            Ok(response) => {
                if let Some(result) = response.get("result") {
                    let endpoint: TarpcEndpoint = serde_json::from_value(result.clone())
                        .map_err(|e| format!("Invalid tarpc endpoint response: {}", e))?;
                    Ok(endpoint)
                } else if let Some(_error) = response.get("error") {
                    debug!("Primal {} doesn't support tarpc: {:?}", primal, _error);
                    Ok(TarpcEndpoint {
                        available: false,
                        socket: None,
                        services: vec![],
                    })
                } else {
                    Err("Invalid JSON-RPC response".to_string())
                }
            }
            Err(e) => {
                debug!("Failed to query {} for tarpc endpoint: {}", primal, e);
                Ok(TarpcEndpoint {
                    available: false,
                    socket: None,
                    services: vec![],
                })
            }
        }
    }

    async fn notify_escalation(
        &self,
        from: &str,
        to: &str,
        tarpc_info: &TarpcEndpoint,
    ) -> Result<(), String> {
        let from_state = self
            .graph
            .get_primal_state(from)
            .await
            .ok_or_else(|| format!("Source primal not found: {}", from))?;

        let request = json!({
            "jsonrpc": "2.0",
            "method": "rpc.escalate_to",
            "params": {
                "target": to,
                "tarpc_socket": tarpc_info.socket,
                "services": tarpc_info.services,
            },
            "id": self.graph.next_request_id(),
        });

        let response = self
            .send_json_rpc(&from_state.json_rpc_socket, &request)
            .await?;

        if response.get("error").is_some() {
            let error = response
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("Escalation notification failed: {}", error));
        }

        Ok(())
    }

    async fn verify_tarpc_connection(&self, _from: &str, _to: &str) -> Result<(), String> {
        debug!("Skipping tarpc verification (not yet implemented)");
        Ok(())
    }

    /// Fallback a connection to JSON-RPC
    pub async fn fallback_connection(
        &self,
        from: &str,
        to: &str,
        reason: &str,
    ) -> Result<EscalationResult, String> {
        warn!(
            "⚠️ Falling back to JSON-RPC: {} → {} (reason: {})",
            from, to, reason
        );

        let conn = self
            .graph
            .get_connection(from, to)
            .await
            .ok_or_else(|| format!("Connection not found: {} → {}", from, to))?;

        let previous_mode = conn.protocol;

        if let Err(e) = self.notify_fallback(from, to, reason).await {
            warn!("Failed to notify source primal of fallback: {}", e);
        }

        self.graph
            .update_connection_protocol(from, to, ProtocolMode::Degraded)
            .await;

        Ok(EscalationResult {
            from: from.to_string(),
            to: to.to_string(),
            previous_mode,
            current_mode: ProtocolMode::Degraded,
            tarpc_socket: None,
            success: true,
            message: format!("Fell back to JSON-RPC: {}", reason),
        })
    }

    async fn notify_fallback(&self, from: &str, to: &str, reason: &str) -> Result<(), String> {
        let from_state = self
            .graph
            .get_primal_state(from)
            .await
            .ok_or_else(|| format!("Source primal not found: {}", from))?;

        let request = json!({
            "jsonrpc": "2.0",
            "method": "rpc.fallback_to_json_rpc",
            "params": {
                "target": to,
                "reason": reason,
            },
            "id": self.graph.next_request_id(),
        });

        let response = self
            .send_json_rpc(&from_state.json_rpc_socket, &request)
            .await?;

        if response.get("error").is_some() {
            let error = response
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("Fallback notification failed: {}", error));
        }

        Ok(())
    }

    async fn send_json_rpc(&self, socket_path: &PathBuf, request: &Value) -> Result<Value, String> {
        let mut stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", socket_path.display(), e))?;

        let request_str = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        stream
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| format!("Failed to write request: {}", e))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();

        match tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(format!("Failed to read response: {}", e)),
            Err(_) => return Err("Response timeout (>5s)".to_string()),
        }

        serde_json::from_str(&response_line).map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get protocol status for all connections (for JSON-RPC API)
    pub async fn get_status(&self) -> Value {
        metrics::get_protocol_status(self.graph.as_ref(), &self.config).await
    }

    /// Get metrics for a specific connection (for JSON-RPC API)
    pub async fn get_connection_metrics(&self, from: &str, to: &str) -> Option<Value> {
        metrics::get_connection_metrics(self.graph.as_ref(), from, to).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_escalation_manager_creation() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph.clone());

        assert_eq!(manager.graph().family_id(), "test-family");
        assert!(manager.config().auto_escalate);
    }

    #[tokio::test]
    async fn test_cooldown_tracking() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;

        let config = EscalationConfig {
            escalation_cooldown_secs: 1,
            ..Default::default()
        };

        let manager = ProtocolEscalationManager::new(graph.clone(), config);

        let conn = graph.get_connection("a", "b").await.unwrap();

        assert!(!manager.is_in_cooldown(&conn).await);

        manager.record_cooldown(&conn.id.to_string()).await;

        assert!(manager.is_in_cooldown(&conn).await);

        tokio::time::sleep(Duration::from_millis(1100)).await;

        assert!(!manager.is_in_cooldown(&conn).await);
    }

    #[tokio::test]
    async fn test_stop_monitoring() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        manager.stop_monitoring().await;
        assert!(!*manager.running.read().await);
    }

    #[tokio::test]
    async fn test_start_monitoring_disabled() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let config = EscalationConfig {
            auto_escalate: false,
            ..Default::default()
        };
        let manager = ProtocolEscalationManager::new(graph, config);

        manager.start_monitoring().await;
    }

    #[tokio::test]
    async fn test_auto_escalate_check_no_candidates() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        let result = manager.auto_escalate_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_connection_not_found() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        let result = manager.fallback_connection("a", "b", "test reason").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection not found"));
    }

    #[tokio::test]
    async fn test_escalate_connection_not_found() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        let result = manager.escalate_connection("a", "b").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection not found"));
    }

    #[tokio::test]
    async fn test_multiple_cooldowns_different_connections() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;
        graph.register_connection("c", "d").await;

        let config = EscalationConfig {
            escalation_cooldown_secs: 60,
            ..Default::default()
        };
        let manager = ProtocolEscalationManager::new(graph.clone(), config);

        let conn_ab = graph.get_connection("a", "b").await.unwrap();
        let conn_cd = graph.get_connection("c", "d").await.unwrap();

        assert!(!manager.is_in_cooldown(&conn_ab).await);
        assert!(!manager.is_in_cooldown(&conn_cd).await);

        manager.record_cooldown(&conn_ab.id.to_string()).await;
        assert!(manager.is_in_cooldown(&conn_ab).await);
        assert!(!manager.is_in_cooldown(&conn_cd).await);

        manager.record_cooldown(&conn_cd.id.to_string()).await;
        assert!(manager.is_in_cooldown(&conn_ab).await);
        assert!(manager.is_in_cooldown(&conn_cd).await);
    }

    #[tokio::test]
    async fn test_cooldown_zero_duration() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;

        let config = EscalationConfig {
            escalation_cooldown_secs: 0,
            ..Default::default()
        };
        let manager = ProtocolEscalationManager::new(graph.clone(), config);

        let conn = graph.get_connection("a", "b").await.unwrap();
        manager.record_cooldown(&conn.id.to_string()).await;

        assert!(!manager.is_in_cooldown(&conn).await);
    }

    #[tokio::test]
    async fn test_auto_escalate_check_with_low_traffic_connections() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;
        graph
            .record_request("songbird", "beardog", 1000, true)
            .await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let result = manager.auto_escalate_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_graph_accessor() {
        let graph = Arc::new(LivingGraph::new("my-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);
        assert_eq!(manager.graph().family_id(), "my-family");
    }

    #[tokio::test]
    async fn test_config_accessor() {
        let config = EscalationConfig {
            min_requests: 77,
            latency_threshold_us: 333,
            auto_escalate: false,
            ..Default::default()
        };
        let graph = Arc::new(LivingGraph::new("test"));
        let manager = ProtocolEscalationManager::new(graph, config);
        assert_eq!(manager.config().min_requests, 77);
        assert_eq!(manager.config().latency_threshold_us, 333);
        assert!(!manager.config().auto_escalate);
    }

    #[tokio::test]
    async fn test_stop_then_check_running_flag() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        assert!(!*manager.running.read().await);

        manager.stop_monitoring().await;
        assert!(!*manager.running.read().await);
    }

    #[tokio::test]
    async fn test_fallback_existing_connection() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;
        graph
            .update_connection_protocol("songbird", "beardog", ProtocolMode::Tarpc)
            .await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let result = manager
            .fallback_connection("songbird", "beardog", "tarpc failure")
            .await;

        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.success);
        assert_eq!(r.current_mode, ProtocolMode::Degraded);
        assert!(r.message.contains("tarpc failure"));
    }

    #[tokio::test]
    async fn test_escalate_existing_connection_no_primal_state() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let result = manager.escalate_connection("songbird", "beardog").await;

        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.message.contains("Failed to query tarpc endpoint"));
    }
}
