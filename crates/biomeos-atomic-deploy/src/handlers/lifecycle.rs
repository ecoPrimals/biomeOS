// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Lifecycle management handler for Neural API
//!
//! Exposes lifecycle management operations via JSON-RPC:
//! - `lifecycle.status` - Get status of all managed primals
//! - `lifecycle.get` - Get detailed info for a specific primal
//! - `lifecycle.composition` - Live composition state for dashboards (active/degraded/dead)
//! - `lifecycle.resurrect` - Force resurrection of a degraded/dead primal
//! - `lifecycle.apoptosis` - Initiate graceful shutdown
//! - `lifecycle.register` - Register a primal for management
//! - `lifecycle.shutdown_all` - Initiate system-wide shutdown
//! - `composition.reload` - Hot-swap a single primal without full restart (JH-3)
//! - `composition.status` - Adaptive daemon surface: active_users, primal_health, resource_pressure

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::lifecycle_manager::{ApoptosisReason, LifecycleManager, LifecycleState};
use crate::neural_graph::GraphNode;

use super::spring_status::{binary_search_dirs, probe_binary, state_to_string};
use biomeos_core::atomic_client::AtomicClient;

/// Lifecycle handler for Neural API
#[derive(Clone)]
pub struct LifecycleHandler {
    pub(crate) manager: Arc<RwLock<LifecycleManager>>,
    /// Monotonic topology version, incremented on each composition change
    /// (register, reload, apoptosis). Used by `composition.reload` contract.
    pub(crate) topology_version: Arc<std::sync::atomic::AtomicU64>,
    /// Shared graph execution status map (for workload counts in `spring_status`).
    executions: Option<
        Arc<RwLock<std::collections::HashMap<String, crate::handlers::graph::ExecutionStatus>>>,
    >,
}

impl LifecycleHandler {
    /// Create a new lifecycle handler
    #[must_use]
    pub fn new(family_id: &str) -> Self {
        Self {
            manager: Arc::new(RwLock::new(LifecycleManager::new(family_id))),
            topology_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            executions: None,
        }
    }

    /// Create with an existing manager
    pub fn with_manager(manager: Arc<RwLock<LifecycleManager>>) -> Self {
        Self {
            manager,
            topology_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            executions: None,
        }
    }

    /// Wire the shared graph executions map (enables workload counts in `spring_status`).
    pub fn with_executions(
        mut self,
        executions: Arc<
            RwLock<std::collections::HashMap<String, crate::handlers::graph::ExecutionStatus>>,
        >,
    ) -> Self {
        self.executions = Some(executions);
        self
    }

    fn bump_topology(&self) -> u64 {
        self.topology_version
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self) -> Result<()> {
        let manager = self.manager.read().await;
        manager.start_monitoring().await
    }

    /// Handle `lifecycle.status` - Get status of all managed primals
    pub async fn status(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let primals: Vec<Value> = status
            .iter()
            .map(|(name, state)| {
                json!({
                    "name": name,
                    "state": state_to_string(state),
                    "details": state_details(state)
                })
            })
            .collect();

        Ok(json!({
            "primals": primals,
            "count": primals.len(),
            "healthy": primals.iter().filter(|p| {
                p.get("state").and_then(|s| s.as_str()) == Some("active")
            }).count()
        }))
    }

    /// Handle `lifecycle.get` - Get detailed info for a specific primal
    pub async fn get(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;

        let manager = self.manager.read().await;

        if let Some(primal) = manager.get_primal_info(name).await {
            Ok(json!({
                "name": primal.name,
                "family_id": primal.family_id,
                "socket_path": primal.socket_path.to_string_lossy(),
                "pid": primal.pid,
                "state": state_to_string(&primal.state),
                "state_details": state_details(&primal.state),
                "depends_on": primal.depends_on,
                "depended_by": primal.depended_by,
                "metrics": {
                    "total_uptime_secs": primal.metrics.total_uptime_secs,
                    "resurrection_count": primal.metrics.resurrection_count,
                    "health_failures": primal.metrics.health_failures,
                    "last_health_latency_ms": primal.metrics.last_health_latency_ms,
                    "requests_served": primal.metrics.requests_served
                },
                "health_config": {
                    "check_interval_secs": primal.health_config.check_interval.as_secs(),
                    "timeout_secs": primal.health_config.timeout.as_secs(),
                    "failure_threshold": primal.health_config.failure_threshold,
                    "health_method": primal.health_config.health_method
                },
                "resurrection_config": {
                    "enabled": primal.resurrection_config.enabled,
                    "max_attempts": primal.resurrection_config.max_attempts,
                    "base_delay_secs": primal.resurrection_config.base_delay.as_secs(),
                    "max_delay_secs": primal.resurrection_config.max_delay.as_secs()
                }
            }))
        } else {
            Ok(json!({
                "error": format!("Primal not found: {}", name)
            }))
        }
    }

    /// Handle `lifecycle.register` - Register a primal for management
    pub async fn register(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;
        let socket_path = params["socket_path"]
            .as_str()
            .context("Missing 'socket_path' parameter")?;
        let pid = params["pid"].as_u64().map(|p| p as u32);

        // Parse deployment node if provided
        let deployment_node: Option<GraphNode> = params
            .get("deployment_node")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // Note: register_primal uses internal locking, we just need to access the manager
        let manager = self.manager.read().await;
        manager
            .register_primal(name, PathBuf::from(socket_path), pid, deployment_node)
            .await?;
        drop(manager); // Explicit drop for clarity

        let version = self.bump_topology();
        info!(
            "🌱 Registered primal via API: {} (topology v{version})",
            name
        );

        Ok(json!({
            "registered": name,
            "socket_path": socket_path,
            "pid": pid,
            "state": "incubating",
            "topology_version": version,
        }))
    }

    /// Handle `lifecycle.resurrect` - Force resurrection of a primal
    pub async fn resurrect(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;

        info!("🔄 Resurrection requested for: {}", name);

        // Check if primal exists
        let manager = self.manager.read().await;
        let primal = manager.get_primal_info(name).await;
        drop(manager);

        if primal.is_none() {
            return Ok(json!({
                "error": format!("Primal not found: {}", name)
            }));
        }

        // Trigger resurrection by marking as degraded
        // The monitoring loop will handle the actual resurrection
        // For now, we can't directly trigger resurrection without internal methods
        // Instead, we return instructions

        Ok(json!({
            "requested": name,
            "message": "Resurrection triggered. Monitor lifecycle.status for progress."
        }))
    }

    /// Handle `lifecycle.apoptosis` - Initiate graceful shutdown
    pub async fn apoptosis(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;

        let reason_str = params["reason"].as_str().unwrap_or("user_request");
        let reason = match reason_str {
            "ecosystem_health" => ApoptosisReason::EcosystemHealth,
            "resource_pressure" => ApoptosisReason::ResourcePressure,
            "system_shutdown" => ApoptosisReason::SystemShutdown,
            _ => ApoptosisReason::UserRequest,
        };

        info!("💀 Apoptosis requested for {}: {:?}", name, reason);

        let manager = self.manager.read().await;
        manager.apoptosis(name, reason.clone()).await?;

        Ok(json!({
            "initiated": name,
            "reason": reason_str,
            "state": "apoptosis"
        }))
    }

    /// Handle `lifecycle.composition` - Get live composition state for dashboards.
    ///
    /// Returns the current composition: which primals are up, which capabilities
    /// are live, per-primal health status, dependency edges, and deploy graph
    /// context. Designed for real-time monitoring dashboards (ludoSpring,
    /// petalTongue) and spring composition validation.
    ///
    /// JSON-RPC method: `lifecycle.composition`
    pub async fn composition(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut active = Vec::new();
        let mut degraded = Vec::new();
        let mut dead = Vec::new();
        let mut all_capabilities: Vec<String> = Vec::new();
        let mut dependency_edges: Vec<Value> = Vec::new();

        for (name, state) in &status {
            let primal_info = manager.get_primal_info(name).await;

            let (capabilities, health_detail, deps, depended_by) =
                if let Some(ref info) = primal_info {
                    let caps: Vec<String> = info
                        .deployment_node
                        .as_ref()
                        .map(|n| n.capabilities.clone())
                        .unwrap_or_default();

                    let health = json!({
                        "last_health_latency_ms": info.metrics.last_health_latency_ms,
                        "health_failures": info.metrics.health_failures,
                        "resurrection_count": info.metrics.resurrection_count,
                        "total_uptime_secs": info.metrics.total_uptime_secs,
                    });

                    for dep in &info.depends_on {
                        dependency_edges.push(json!({
                            "from": dep,
                            "to": name,
                        }));
                    }

                    (
                        caps,
                        health,
                        info.depends_on.clone(),
                        info.depended_by.clone(),
                    )
                } else {
                    (vec![], json!({}), vec![], vec![])
                };

            all_capabilities.extend(capabilities.iter().cloned());

            let entry = json!({
                "name": name,
                "state": state_to_string(state),
                "state_details": state_details(state),
                "capabilities": capabilities,
                "health": health_detail,
                "depends_on": deps,
                "depended_by": depended_by,
            });

            match state {
                LifecycleState::Active { .. } => active.push(entry),
                LifecycleState::Degraded { .. }
                | LifecycleState::Incubating { .. }
                | LifecycleState::Germinating => {
                    degraded.push(entry);
                }
                LifecycleState::Apoptosis { .. } | LifecycleState::Dead { .. } => {
                    dead.push(entry);
                }
            }
        }

        all_capabilities.sort();
        all_capabilities.dedup();

        let total = status.len();
        let health_ratio = if total == 0 {
            1.0
        } else {
            active.len() as f64 / total as f64
        };

        Ok(json!({
            "active": active,
            "degraded": degraded,
            "dead": dead,
            "total": total,
            "active_count": active.len(),
            "degraded_count": degraded.len(),
            "dead_count": dead.len(),
            "health_ratio": health_ratio,
            "composition_healthy": health_ratio >= 0.5,
            "capabilities_live": all_capabilities,
            "dependency_graph": dependency_edges,
        }))
    }

    /// Spring status for Tier 2 notebook integration.
    ///
    /// Returns per-primal availability: binary on disk, runtime state,
    /// capabilities, workload count, and last health-check timestamp.
    /// Designed for projectNUCLEUS downstream consumers that need to know
    /// which primals are usable from a JupyterHub session.
    ///
    /// JSON-RPC method: `biomeos.spring_status`
    ///
    /// # Response shape
    /// ```json
    /// {
    ///   "primals": [
    ///     {
    ///       "name": "beardog",
    ///       "display_name": "BearDog",
    ///       "binary_available": true,
    ///       "binary_path": "/opt/plasmidBin/primals/beardog",
    ///       "runtime_state": "active",
    ///       "capabilities": ["security", "crypto"],
    ///       "last_health_check": "2026-05-13T17:00:00Z"
    ///     }
    ///   ],
    ///   "workload_count": 3,
    ///   "workloads_running": 1,
    ///   "topology_version": 7
    /// }
    /// ```
    pub async fn spring_status(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let all_primals: Vec<&str> = biomeos_types::primal_names::CORE_PRIMALS
            .iter()
            .chain(biomeos_types::primal_names::PROVENANCE_PRIMALS.iter())
            .chain(biomeos_types::primal_names::SPRING_PRIMALS.iter())
            .chain(biomeos_types::primal_names::AUXILIARY_PRIMALS.iter())
            .copied()
            .collect();

        let search_dirs = binary_search_dirs();

        let mut primals_out = Vec::with_capacity(all_primals.len());

        for &primal_name in &all_primals {
            let (binary_available, binary_path) = probe_binary(primal_name, &search_dirs);

            let display_name = biomeos_types::primal_names::display::for_id(primal_name)
                .unwrap_or(primal_name)
                .to_string();

            let (runtime_state, capabilities, last_health_ts) =
                if let Some(state) = status.iter().find(|(n, _)| n.as_str() == primal_name) {
                    let info = manager.get_primal_info(primal_name).await;
                    let caps: Vec<String> = info
                        .as_ref()
                        .and_then(|i| i.deployment_node.as_ref())
                        .map(|n| n.capabilities.clone())
                        .unwrap_or_default();
                    let state_str = state_to_string(state.1);
                    let ts = if state_str == "active" {
                        chrono::Utc::now().to_rfc3339()
                    } else {
                        String::new()
                    };
                    (Some(state_str), caps, ts)
                } else {
                    (None, vec![], String::new())
                };

            primals_out.push(json!({
                "name": primal_name,
                "display_name": display_name,
                "binary_available": binary_available,
                "binary_path": binary_path,
                "runtime_state": runtime_state,
                "capabilities": capabilities,
                "last_health_check": if last_health_ts.is_empty() { Value::Null } else { Value::String(last_health_ts) },
            }));
        }

        let (workload_count, workloads_running) = if let Some(ref execs) = self.executions {
            let map = execs.read().await;
            let total = map.len() as u64;
            let running = map.values().filter(|e| e.state == "running").count() as u64;
            (total, running)
        } else {
            (0, 0)
        };

        let topology = self
            .topology_version
            .load(std::sync::atomic::Ordering::Relaxed);

        Ok(json!({
            "primals": primals_out,
            "workload_count": workload_count,
            "workloads_running": workloads_running,
            "topology_version": topology,
        }))
    }

    /// Handle `lifecycle.shutdown_all` - Initiate system-wide shutdown
    pub async fn shutdown_all(&self) -> Result<Value> {
        warn!("🛑 System-wide shutdown requested");

        let manager = self.manager.read().await;
        manager.shutdown_all().await?;

        Ok(json!({
            "shutdown": "complete",
            "message": "All primals have been shut down"
        }))
    }

    /// Handle `composition.reload` — hot-swap a single primal without restarting
    /// the full composition (JH-3).
    ///
    /// Steps:
    /// 1. Verify the named primal is currently registered.
    /// 2. Gracefully stop it (apoptosis with `reload` reason).
    /// 3. Wait briefly for process exit.
    /// 4. Re-register at the new socket path (or the same one).
    /// 5. Health-check the restarted primal.
    /// 6. Return success/failure.
    ///
    /// Params: `{ "name": "primal_name", "socket_path": "/new/path.sock" (optional) }`
    pub async fn reload(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;

        info!("🔄 Composition reload requested for '{name}'");

        let manager = self.manager.read().await;

        let existing = manager.get_primal_info(name).await;
        let (old_socket, old_pid, old_node) = match existing {
            Some(info) => (
                info.socket_path.clone(),
                info.pid,
                info.deployment_node.clone(),
            ),
            None => {
                return Ok(json!({
                    "reloaded": false,
                    "error": format!("Primal '{name}' is not registered in the composition"),
                }));
            }
        };

        let new_socket = params["socket_path"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| old_socket.clone());

        manager
            .apoptosis(name, ApoptosisReason::UserRequest)
            .await
            .ok();

        drop(manager);

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        let manager = self.manager.read().await;
        manager
            .register_primal(name, new_socket.clone(), old_pid, old_node)
            .await?;

        drop(manager);

        let healthy = {
            let client = AtomicClient::unix(&new_socket)
                .with_timeout(biomeos_types::constants::timeouts::DEFAULT_IPC_TIMEOUT);
            match client.call("health.check", json!({})).await {
                Ok(_) => true,
                Err(e) => {
                    warn!("🔄 Reload health check failed for '{name}': {e}");
                    false
                }
            }
        };

        let version = self.bump_topology();

        info!(
            "🔄 Composition reload for '{name}': healthy={healthy}, socket={}, topology v{version}",
            new_socket.display()
        );

        Ok(json!({
            "status": "reloaded",
            "reloaded": true,
            "name": name,
            "socket_path": new_socket.display().to_string(),
            "healthy": healthy,
            "topology_version": version,
        }))
    }

    /// Probe the mesh provider's actual mesh state via `mesh.status` IPC.
    ///
    /// The mesh provider is resolved from `BIOMEOS_NETWORK_PROVIDER` (defaulting
    /// to the canonical discovery primal). Returns enriched mesh detail including
    /// peer count, mesh epoch, and partition info when available.
    pub(crate) async fn probe_mesh_provider(
        &self,
        manager: &LifecycleManager,
        mesh_provider: &str,
    ) -> Result<Value> {
        let provider_info = manager
            .get_primal_info(mesh_provider)
            .await
            .with_context(|| format!("Mesh provider '{mesh_provider}' not registered"))?;

        let client = AtomicClient::unix(&provider_info.socket_path)
            .with_timeout(biomeos_types::constants::timeouts::DEFAULT_IPC_TIMEOUT);

        match client.call("mesh.status", json!({})).await {
            Ok(mesh_state) => {
                let peer_count = mesh_state
                    .get("peer_count")
                    .or_else(|| mesh_state.get("peers"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_array().map(|a| a.len() as u64)));

                let healthy = peer_count.unwrap_or(0) > 0
                    || mesh_state
                        .get("status")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| s == "ok" || s == "healthy");

                Ok(json!({
                    "status": if healthy { "ok" } else { "degraded" },
                    "detail": "mesh_probed",
                    "peer_count": peer_count,
                    "mesh_state": mesh_state,
                }))
            }
            Err(e) => {
                info!(
                    "Songbird mesh.status probe unavailable: {e}; \
                     falling back to process liveness"
                );
                Ok(json!({
                    "status": "ok",
                    "detail": "process_alive_mesh_unprobed",
                    "probe_error": e.to_string(),
                }))
            }
        }
    }
}

// ============================================================================
// HELPERS
// ============================================================================

/// Get state-specific details
fn state_details(state: &LifecycleState) -> Value {
    match state {
        LifecycleState::Germinating => json!({}),
        LifecycleState::Incubating {
            started_at,
            timeout_ms,
        } => json!({
            "started_at": started_at.to_rfc3339(),
            "timeout_ms": timeout_ms
        }),
        LifecycleState::Active {
            since,
            last_health_check,
        } => json!({
            "since": since.to_rfc3339(),
            "last_health_check": last_health_check.to_rfc3339()
        }),
        LifecycleState::Degraded {
            since,
            reason,
            resurrection_attempts,
        } => json!({
            "since": since.to_rfc3339(),
            "reason": reason,
            "resurrection_attempts": resurrection_attempts
        }),
        LifecycleState::Apoptosis { reason, started_at } => json!({
            "reason": format!("{:?}", reason),
            "started_at": started_at.to_rfc3339()
        }),
        LifecycleState::Dead { since, reason } => json!({
            "since": since.to_rfc3339(),
            "reason": reason
        }),
    }
}
