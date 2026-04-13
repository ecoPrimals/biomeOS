// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Protocol Escalation Engine: JSON-RPC → tarpc Runtime Evolution
//!
//! This module implements the escalation engine that manages runtime
//! escalation of primal connections from JSON-RPC to tarpc.

#![forbid(unsafe_code)]

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::living_graph::{ConnectionState, LivingGraph, PrimalHealth, ProtocolMode};

use super::config::{EscalationConfig, EscalationResult};
use super::metrics;
use super::rpc;

/// Protocol Escalation Manager
pub struct ProtocolEscalationManager {
    /// Living graph with runtime state
    graph: Arc<LivingGraph>,
    /// Escalation configuration
    config: EscalationConfig,
    /// Running state
    pub(crate) running: Arc<RwLock<bool>>,
    /// Last check timestamp per connection (`tokio::time::Instant` for test determinism)
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
    pub const fn graph(&self) -> &Arc<LivingGraph> {
        &self.graph
    }

    /// Get the configuration
    pub const fn config(&self) -> &EscalationConfig {
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
            .ok_or_else(|| format!("Connection not found: {from} → {to}"))?;

        let previous_mode = conn.protocol;

        let tarpc_info = match rpc::query_tarpc_endpoint(&self.graph, to).await {
            Ok(info) => info,
            Err(e) => {
                return Ok(EscalationResult {
                    from: from.to_string(),
                    to: to.to_string(),
                    previous_mode,
                    current_mode: previous_mode,
                    tarpc_socket: None,
                    success: false,
                    message: format!("Failed to query tarpc endpoint: {e}"),
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

        if let Err(e) = rpc::notify_escalation(&self.graph, from, to, &tarpc_info).await {
            return Ok(EscalationResult {
                from: from.to_string(),
                to: to.to_string(),
                previous_mode,
                current_mode: previous_mode,
                tarpc_socket,
                success: false,
                message: format!("Failed to notify source primal: {e}"),
            });
        }

        self.graph
            .update_connection_protocol(from, to, ProtocolMode::Tarpc)
            .await;

        if let Err(e) = rpc::verify_tarpc_connection(&self.graph, from, to).await {
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
            message: format!("Successfully escalated {from} → {to} to tarpc"),
        })
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
            .ok_or_else(|| format!("Connection not found: {from} → {to}"))?;

        let previous_mode = conn.protocol;

        if let Err(e) = rpc::notify_fallback(&self.graph, from, to, reason).await {
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
            message: format!("Fell back to JSON-RPC: {reason}"),
        })
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
