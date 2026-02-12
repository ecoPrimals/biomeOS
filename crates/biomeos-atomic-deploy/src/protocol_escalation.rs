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

        debug!("🔍 Checking {} escalation candidates", candidates.len());

        for conn in candidates {
            // Check cooldown
            if self.is_in_cooldown(&conn).await {
                debug!("⏳ Connection {} in cooldown, skipping", conn.id);
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

        match self.send_json_rpc(&state.json_rpc_socket, &request).await {
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

        serde_json::from_str(&response_line).map_err(|e| format!("Failed to parse response: {}", e))
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

        let config = EscalationConfig {
            escalation_cooldown_secs: 1, // 1 second cooldown for test
            ..Default::default()
        };

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

        let json = serde_json::to_string(&result).expect("serialize escalation result");
        assert!(json.contains("songbird"));
        assert!(json.contains("tarpc"));

        let parsed: EscalationResult =
            serde_json::from_str(&json).expect("parse escalation result");
        assert_eq!(parsed.from, "songbird");
        assert_eq!(parsed.to, "beardog");
        assert!(parsed.success);
    }

    #[test]
    fn test_escalation_config_serialization() {
        let config = EscalationConfig {
            min_requests: 200,
            latency_threshold_us: 1000,
            stable_health_duration_secs: 60,
            tarpc_failure_threshold: 5,
            check_interval_secs: 20,
            escalation_cooldown_secs: 120,
            auto_escalate: false,
        };

        let json = serde_json::to_string(&config).expect("serialize config");
        let parsed: EscalationConfig = serde_json::from_str(&json).expect("parse config");
        assert_eq!(parsed.min_requests, 200);
        assert_eq!(parsed.latency_threshold_us, 1000);
        assert!(!parsed.auto_escalate);
        assert_eq!(parsed.tarpc_failure_threshold, 5);
    }

    #[test]
    fn test_tarpc_endpoint_serialization() {
        let endpoint = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/tmp/beardog-tarpc.sock")),
            services: vec!["health".to_string(), "deploy".to_string()],
        };

        let json = serde_json::to_string(&endpoint).expect("serialize endpoint");
        let parsed: TarpcEndpoint = serde_json::from_str(&json).expect("parse endpoint");
        assert!(parsed.available);
        assert_eq!(parsed.services.len(), 2);
    }

    #[test]
    fn test_tarpc_endpoint_unavailable() {
        let endpoint = TarpcEndpoint {
            available: false,
            socket: None,
            services: vec![],
        };

        let json = serde_json::to_string(&endpoint).expect("serialize");
        let parsed: TarpcEndpoint = serde_json::from_str(&json).expect("parse");
        assert!(!parsed.available);
        assert!(parsed.socket.is_none());
        assert!(parsed.services.is_empty());
    }

    #[tokio::test]
    async fn test_escalation_result_failed() {
        let result = EscalationResult {
            from: "a".to_string(),
            to: "b".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::JsonRpc, // stayed the same
            tarpc_socket: None,
            success: false,
            message: "Target does not support tarpc".to_string(),
        };

        assert!(!result.success);
        assert!(result.tarpc_socket.is_none());
        assert_eq!(result.previous_mode, result.current_mode);
    }

    #[tokio::test]
    async fn test_stop_monitoring() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        // Stop monitoring sets running to false
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

        // Should return immediately since auto_escalate is false
        manager.start_monitoring().await;
        // If this returns, the test passes (not stuck in a loop)
    }

    #[tokio::test]
    async fn test_get_connection_metrics_existing() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let metrics = manager.get_connection_metrics("songbird", "beardog").await;

        assert!(metrics.is_some());
        let m = metrics.expect("metrics");
        assert_eq!(m["connection"]["from"], "songbird");
        assert_eq!(m["connection"]["to"], "beardog");
        assert!(m["metrics"]["request_count"].is_number());
    }

    #[tokio::test]
    async fn test_get_connection_metrics_nonexistent() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        let metrics = manager.get_connection_metrics("a", "b").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_get_status_empty_graph() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);
        let status = manager.get_status().await;

        assert_eq!(status["summary"]["total"], 0);
        assert_eq!(status["summary"]["json_rpc"], 0);
        assert_eq!(status["summary"]["tarpc"], 0);
        assert!(status["connections"].is_array());
        assert!(status["config"]["auto_escalate"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_get_status_with_config_info() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let config = EscalationConfig {
            min_requests: 50,
            latency_threshold_us: 250,
            check_interval_secs: 5,
            ..Default::default()
        };
        let manager = ProtocolEscalationManager::new(graph, config);
        let status = manager.get_status().await;

        assert_eq!(status["config"]["min_requests"], 50);
        assert_eq!(status["config"]["latency_threshold_us"], 250);
        assert_eq!(status["config"]["check_interval_secs"], 5);
    }

    #[tokio::test]
    async fn test_auto_escalate_check_no_candidates() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        let manager = ProtocolEscalationManager::with_defaults(graph);

        // Should succeed with no candidates
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
    async fn test_multiple_connections_status() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;
        graph.register_connection("b", "c").await;
        graph.register_connection("a", "c").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let status = manager.get_status().await;

        assert_eq!(status["summary"]["total"], 3);
        let connections = status["connections"].as_array().expect("array");
        assert_eq!(connections.len(), 3);
    }

    #[test]
    fn test_escalation_config_default_fn_values() {
        assert_eq!(default_min_requests(), 100);
        assert_eq!(default_latency_threshold(), 500);
        assert_eq!(default_stable_health_duration(), 30);
        assert_eq!(default_tarpc_failure_threshold(), 3);
        assert_eq!(default_check_interval(), 10);
        assert_eq!(default_escalation_cooldown(), 60);
        assert!(default_auto_escalate());
    }

    // --- New tests for comprehensive coverage ---

    #[test]
    fn test_config_deserialization_empty_json_uses_defaults() {
        // Empty JSON should use all serde defaults
        let config: EscalationConfig = serde_json::from_str("{}").expect("parse empty json");
        assert_eq!(config.min_requests, 100);
        assert_eq!(config.latency_threshold_us, 500);
        assert_eq!(config.stable_health_duration_secs, 30);
        assert_eq!(config.tarpc_failure_threshold, 3);
        assert_eq!(config.check_interval_secs, 10);
        assert_eq!(config.escalation_cooldown_secs, 60);
        assert!(config.auto_escalate);
    }

    #[test]
    fn test_config_deserialization_partial_json() {
        // Only override some fields, rest use defaults
        let json = r#"{"min_requests": 500, "auto_escalate": false}"#;
        let config: EscalationConfig = serde_json::from_str(json).expect("parse partial json");
        assert_eq!(config.min_requests, 500);
        assert!(!config.auto_escalate);
        // Defaults for the rest
        assert_eq!(config.latency_threshold_us, 500);
        assert_eq!(config.stable_health_duration_secs, 30);
        assert_eq!(config.tarpc_failure_threshold, 3);
        assert_eq!(config.check_interval_secs, 10);
        assert_eq!(config.escalation_cooldown_secs, 60);
    }

    #[test]
    fn test_config_clone() {
        let config = EscalationConfig {
            min_requests: 42,
            latency_threshold_us: 999,
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.min_requests, 42);
        assert_eq!(cloned.latency_threshold_us, 999);
        assert_eq!(cloned.check_interval_secs, config.check_interval_secs);
    }

    #[test]
    fn test_config_debug() {
        let config = EscalationConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("min_requests"));
        assert!(debug_str.contains("auto_escalate"));
    }

    #[test]
    fn test_escalation_result_degraded_mode() {
        let result = EscalationResult {
            from: "songbird".to_string(),
            to: "beardog".to_string(),
            previous_mode: ProtocolMode::Tarpc,
            current_mode: ProtocolMode::Degraded,
            tarpc_socket: None,
            success: true,
            message: "Fell back due to tarpc failure".to_string(),
        };
        assert!(result.success);
        assert_eq!(result.current_mode, ProtocolMode::Degraded);
        assert_eq!(result.previous_mode, ProtocolMode::Tarpc);
    }

    #[test]
    fn test_escalation_result_clone_and_debug() {
        let result = EscalationResult {
            from: "a".to_string(),
            to: "b".to_string(),
            previous_mode: ProtocolMode::JsonRpc,
            current_mode: ProtocolMode::Tarpc,
            tarpc_socket: Some(PathBuf::from("/tmp/test.sock")),
            success: true,
            message: "ok".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.from, result.from);
        assert_eq!(cloned.tarpc_socket, result.tarpc_socket);

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("EscalationResult"));
    }

    #[test]
    fn test_tarpc_endpoint_with_many_services() {
        let endpoint = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/run/user/1000/biomeos/beardog.sock")),
            services: vec![
                "health".to_string(),
                "deploy".to_string(),
                "crypto.encrypt".to_string(),
                "crypto.decrypt".to_string(),
                "birdsong.verify".to_string(),
            ],
        };
        assert_eq!(endpoint.services.len(), 5);
        assert!(endpoint.services.contains(&"crypto.encrypt".to_string()));

        // Roundtrip serialization
        let json = serde_json::to_string(&endpoint).unwrap();
        let parsed: TarpcEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.services.len(), 5);
        assert_eq!(parsed.socket, endpoint.socket);
    }

    #[test]
    fn test_tarpc_endpoint_debug_and_clone() {
        let endpoint = TarpcEndpoint {
            available: false,
            socket: None,
            services: vec![],
        };
        let cloned = endpoint.clone();
        assert_eq!(cloned.available, endpoint.available);
        let debug_str = format!("{:?}", endpoint);
        assert!(debug_str.contains("TarpcEndpoint"));
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

        // Neither in cooldown initially
        assert!(!manager.is_in_cooldown(&conn_ab).await);
        assert!(!manager.is_in_cooldown(&conn_cd).await);

        // Record cooldown for a→b only
        manager.record_cooldown(&conn_ab.id.to_string()).await;
        assert!(manager.is_in_cooldown(&conn_ab).await);
        assert!(!manager.is_in_cooldown(&conn_cd).await);

        // Record cooldown for c→d too
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

        // With 0-second cooldown, should NOT be in cooldown (elapsed >= 0 is always true,
        // but the check is `elapsed < cooldown` which is `elapsed < 0` — always false)
        assert!(!manager.is_in_cooldown(&conn).await);
    }

    #[tokio::test]
    async fn test_status_connection_details() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let status = manager.get_status().await;

        let connections = status["connections"].as_array().expect("array");
        assert_eq!(connections.len(), 1);

        let conn = &connections[0];
        assert_eq!(conn["from"], "songbird");
        assert_eq!(conn["to"], "beardog");
        assert_eq!(conn["protocol"], "JsonRpc");
        assert_eq!(conn["requests"], 0);
        assert_eq!(conn["escalation_attempts"], 0);
        assert_eq!(conn["fallback_count"], 0);
    }

    #[tokio::test]
    async fn test_connection_metrics_detailed_fields() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;
        // Record some requests to populate metrics
        graph.record_request("songbird", "beardog", 100, true).await;
        graph.record_request("songbird", "beardog", 200, true).await;
        graph
            .record_request("songbird", "beardog", 300, false)
            .await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let metrics = manager
            .get_connection_metrics("songbird", "beardog")
            .await
            .expect("metrics should exist");

        assert_eq!(metrics["metrics"]["request_count"], 3);
        assert_eq!(metrics["metrics"]["error_count"], 1);
        assert!(metrics["metrics"]["avg_latency_us"].as_f64().unwrap() > 0.0);
        assert!(metrics["metrics"]["max_latency_us"].as_u64().unwrap() >= 300);
        assert_eq!(metrics["history"]["escalation_attempts"], 0);
        assert_eq!(metrics["history"]["fallback_count"], 0);
    }

    #[tokio::test]
    async fn test_auto_escalate_check_with_low_traffic_connections() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;
        // Record only a few requests — well below threshold
        graph
            .record_request("songbird", "beardog", 1000, true)
            .await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        // Should succeed — connections exist but don't meet escalation criteria
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

        // Initially not running
        assert!(!*manager.running.read().await);

        // Stop sets it to false (idempotent)
        manager.stop_monitoring().await;
        assert!(!*manager.running.read().await);
    }

    #[test]
    fn test_escalation_result_roundtrip_all_modes() {
        for mode in [
            ProtocolMode::JsonRpc,
            ProtocolMode::Tarpc,
            ProtocolMode::Hybrid,
            ProtocolMode::Degraded,
        ] {
            let result = EscalationResult {
                from: "a".to_string(),
                to: "b".to_string(),
                previous_mode: ProtocolMode::JsonRpc,
                current_mode: mode,
                tarpc_socket: None,
                success: true,
                message: format!("mode: {:?}", mode),
            };
            let json = serde_json::to_string(&result).unwrap();
            let parsed: EscalationResult = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.current_mode, mode);
        }
    }

    #[tokio::test]
    async fn test_status_after_protocol_update() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("a", "b").await;
        graph
            .update_connection_protocol("a", "b", ProtocolMode::Tarpc)
            .await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let status = manager.get_status().await;

        assert_eq!(status["summary"]["tarpc"], 1);
        assert_eq!(status["summary"]["json_rpc"], 0);
        assert_eq!(status["summary"]["total"], 1);
    }

    #[tokio::test]
    async fn test_fallback_existing_connection() {
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;
        // First escalate to tarpc
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
        // Connection exists but no primal state registered — escalation
        // will fail because query_tarpc_endpoint can't find the primal
        let graph = Arc::new(LivingGraph::new("test-family"));
        graph.register_connection("songbird", "beardog").await;

        let manager = ProtocolEscalationManager::with_defaults(graph);
        let result = manager.escalate_connection("songbird", "beardog").await;

        // Should return Ok with success=false (primal not found for tarpc query)
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(!r.success);
        assert!(r.message.contains("Failed to query tarpc endpoint"));
    }
}
