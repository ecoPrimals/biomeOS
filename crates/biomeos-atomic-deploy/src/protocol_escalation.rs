//! Protocol Escalation Manager: JSON-RPC → tarpc Runtime Evolution
//!
//! This module manages the runtime escalation of primal connections from
//! JSON-RPC (bootstrap/debug) to tarpc (production/performance).
//!
//! # Design Principles
//!
//! - **Metrics-Based**: Escalate based on measured latency and request volume
//! - **Graceful Degradation**: Automatic fallback on tarpc failure
//! - **Non-Intrusive**: Primals continue working during escalation
//! - **Configurable**: Thresholds tunable per-deployment
//!
//! # Escalation Flow
//!
//! ```text
//! 1. Monitor connection metrics (latency, request count)
//! 2. When threshold met: query target primal for tarpc endpoint
//! 3. Notify source primal to connect via tarpc
//! 4. Verify with test call
//! 5. Update Living Graph
//! ```

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
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

/// Configuration for protocol escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Minimum requests before considering escalation
    #[serde(default = "default_min_requests")]
    pub min_requests: u64,

    /// Latency threshold to trigger escalation (μs)
    #[serde(default = "default_latency_threshold")]
    pub latency_threshold_us: u64,

    /// Stable health duration before auto-escalate
    #[serde(default = "default_stable_health_duration")]
    pub stable_health_duration_secs: u64,

    /// tarpc failures before fallback
    #[serde(default = "default_tarpc_failure_threshold")]
    pub tarpc_failure_threshold: u32,

    /// Time between auto-escalation checks
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u64,

    /// Cooldown after failed escalation attempt
    #[serde(default = "default_escalation_cooldown")]
    pub escalation_cooldown_secs: u64,

    /// Enable auto-escalation
    #[serde(default = "default_auto_escalate")]
    pub auto_escalate: bool,
}

fn default_min_requests() -> u64 {
    100
}
fn default_latency_threshold() -> u64 {
    500
}
fn default_stable_health_duration() -> u64 {
    30
}
fn default_tarpc_failure_threshold() -> u32 {
    3
}
fn default_check_interval() -> u64 {
    10
}
fn default_escalation_cooldown() -> u64 {
    60
}
fn default_auto_escalate() -> bool {
    true
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            min_requests: default_min_requests(),
            latency_threshold_us: default_latency_threshold(),
            stable_health_duration_secs: default_stable_health_duration(),
            tarpc_failure_threshold: default_tarpc_failure_threshold(),
            check_interval_secs: default_check_interval(),
            escalation_cooldown_secs: default_escalation_cooldown(),
            auto_escalate: default_auto_escalate(),
        }
    }
}

/// Result of an escalation attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationResult {
    pub from: String,
    pub to: String,
    pub previous_mode: ProtocolMode,
    pub current_mode: ProtocolMode,
    pub tarpc_socket: Option<PathBuf>,
    pub success: bool,
    pub message: String,
}

/// tarpc endpoint information from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcEndpoint {
    pub available: bool,
    pub socket: Option<PathBuf>,
    pub services: Vec<String>,
}

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

        debug!(
            "🔍 Checking {} escalation candidates",
            candidates.len()
        );

        for conn in candidates {
            // Check cooldown
            if self.is_in_cooldown(&conn).await {
                debug!(
                    "⏳ Connection {} in cooldown, skipping",
                    conn.id
                );
                continue;
            }

            // Check if both primals are healthy
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

            // Check if target supports tarpc
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
    async fn is_in_cooldown(&self, conn: &ConnectionState) -> bool {
        let cooldown = Duration::from_secs(self.config.escalation_cooldown_secs);
        let key = conn.id.to_string();

        if let Some(last) = self.last_check.read().await.get(&key) {
            last.elapsed() < cooldown
        } else {
            false
        }
    }

    /// Record cooldown for a connection
    async fn record_cooldown(&self, key: &str) {
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

        // Get current connection state
        let conn = self
            .graph
            .get_connection(from, to)
            .await
            .ok_or_else(|| format!("Connection not found: {} → {}", from, to))?;

        let previous_mode = conn.protocol;

        // 1. Query target primal for tarpc endpoint
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

        // 2. Notify source primal to connect via tarpc
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

        // 3. Update living graph
        self.graph
            .update_connection_protocol(from, to, ProtocolMode::Tarpc)
            .await;

        // 4. Verify with test call (optional, fail-safe)
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

    /// Query a primal's tarpc endpoint via JSON-RPC
    async fn query_tarpc_endpoint(&self, primal: &str) -> Result<TarpcEndpoint, String> {
        let state = self
            .graph
            .get_primal_state(primal)
            .await
            .ok_or_else(|| format!("Primal not found: {}", primal))?;

        // If we already know the tarpc socket, use it
        if let Some(socket) = &state.tarpc_socket {
            return Ok(TarpcEndpoint {
                available: true,
                socket: Some(socket.clone()),
                services: state.capabilities.clone(),
            });
        }

        // Otherwise, query the primal via JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "method": "rpc.tarpc_endpoint",
            "params": {},
            "id": self.graph.next_request_id(),
        });

        match self
            .send_json_rpc(&state.json_rpc_socket, &request)
            .await
        {
            Ok(response) => {
                if let Some(result) = response.get("result") {
                    let endpoint: TarpcEndpoint = serde_json::from_value(result.clone())
                        .map_err(|e| format!("Invalid tarpc endpoint response: {}", e))?;
                    Ok(endpoint)
                } else if let Some(error) = response.get("error") {
                    // Primal doesn't support tarpc
                    debug!("Primal {} doesn't support tarpc: {:?}", primal, error);
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
                // Connection failed, assume no tarpc
                debug!("Failed to query {} for tarpc endpoint: {}", primal, e);
                Ok(TarpcEndpoint {
                    available: false,
                    socket: None,
                    services: vec![],
                })
            }
        }
    }

    /// Notify source primal to escalate to tarpc
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

    /// Verify tarpc connection with a test call
    async fn verify_tarpc_connection(&self, _from: &str, _to: &str) -> Result<(), String> {
        // In a real implementation, this would make a tarpc test call
        // For now, we trust the escalation notification
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

        // Notify source primal
        if let Err(e) = self.notify_fallback(from, to, reason).await {
            warn!("Failed to notify source primal of fallback: {}", e);
        }

        // Update graph
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

    /// Notify source primal to fall back to JSON-RPC
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

    /// Send a JSON-RPC request to a Unix socket
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

        // Read with timeout
        match tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(format!("Failed to read response: {}", e)),
            Err(_) => return Err("Response timeout (>5s)".to_string()),
        }

        serde_json::from_str(&response_line)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get protocol status for all connections (for JSON-RPC API)
    pub async fn get_status(&self) -> Value {
        let connections = self.graph.get_all_connections().await;
        let summary = self.graph.get_protocol_summary().await;

        let connection_status: Vec<Value> = connections
            .iter()
            .map(|c| {
                json!({
                    "from": c.from,
                    "to": c.to,
                    "protocol": format!("{:?}", c.protocol),
                    "requests": c.metrics.request_count,
                    "avg_latency_us": c.metrics.avg_latency_us,
                    "p99_latency_us": c.metrics.p99_latency_us,
                    "error_rate": c.metrics.error_rate(),
                    "escalation_attempts": c.escalation_attempts,
                    "fallback_count": c.fallback_count,
                })
            })
            .collect();

        json!({
            "connections": connection_status,
            "summary": {
                "json_rpc": summary.json_rpc,
                "tarpc": summary.tarpc,
                "hybrid": summary.hybrid,
                "degraded": summary.degraded,
                "total": summary.total(),
            },
            "config": {
                "auto_escalate": self.config.auto_escalate,
                "min_requests": self.config.min_requests,
                "latency_threshold_us": self.config.latency_threshold_us,
                "check_interval_secs": self.config.check_interval_secs,
            }
        })
    }

    /// Get metrics for a specific connection (for JSON-RPC API)
    pub async fn get_connection_metrics(&self, from: &str, to: &str) -> Option<Value> {
        let conn = self.graph.get_connection(from, to).await?;

        Some(json!({
            "connection": {
                "from": conn.from,
                "to": conn.to,
                "protocol": format!("{:?}", conn.protocol),
            },
            "metrics": {
                "request_count": conn.metrics.request_count,
                "error_count": conn.metrics.error_count,
                "total_latency_us": conn.metrics.total_latency_us,
                "avg_latency_us": conn.metrics.avg_latency_us,
                "p50_latency_us": conn.metrics.p50_latency_us,
                "p95_latency_us": conn.metrics.p95_latency_us,
                "p99_latency_us": conn.metrics.p99_latency_us,
                "max_latency_us": conn.metrics.max_latency_us,
                "error_rate": conn.metrics.error_rate(),
            },
            "history": {
                "escalation_attempts": conn.escalation_attempts,
                "fallback_count": conn.fallback_count,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_escalation_config_defaults() {
        let config = EscalationConfig::default();
        assert_eq!(config.min_requests, 100);
        assert_eq!(config.latency_threshold_us, 500);
        assert!(config.auto_escalate);
    }

    #[tokio::test]
    async fn test_escalation_manager_creation() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph.clone());

        assert_eq!(manager.graph().family_id(), "test-family");
        assert!(manager.config().auto_escalate);
    }

    #[tokio::test]
    async fn test_get_status() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;
        graph.register_connection("b", "c").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let status = manager.get_status().await;

        assert_eq!(status["summary"]["total"], 2);
        assert_eq!(status["summary"]["json_rpc"], 2);
    }

    #[tokio::test]
    async fn test_cooldown_tracking() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;

        let mut config = EscalationConfig::default();
        config.escalation_cooldown_secs = 1; // 1 second cooldown for test

        let manager = ProtocolEscalationManager::new(graph.clone(), config);

        let conn = graph.get_connection("a", "b").await.unwrap();

        // Not in cooldown initially
        assert!(!manager.is_in_cooldown(&conn).await);

        // Record cooldown
        manager.record_cooldown(&conn.id.to_string()).await;

        // Now in cooldown
        assert!(manager.is_in_cooldown(&conn).await);

        // Wait for cooldown to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // No longer in cooldown
        assert!(!manager.is_in_cooldown(&conn).await);
    }

    #[tokio::test]
    async fn test_escalation_result_serialization() {
        let result = EscalationResult {
            from: "songbird".to_string(),
            to: "beardog".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::Tarpc,
            tarpc_socket: Some(PathBuf::from("/tmp/beardog-tarpc.sock")),
            success: true,
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("songbird"));
        assert!(json.contains("tarpc"));
    }
}

