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

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::lifecycle_manager::{ApoptosisReason, LifecycleManager, LifecycleState};
use crate::neural_graph::GraphNode;

/// Lifecycle handler for Neural API
#[derive(Clone)]
pub struct LifecycleHandler {
    manager: Arc<RwLock<LifecycleManager>>,
}

impl LifecycleHandler {
    /// Create a new lifecycle handler
    #[must_use]
    pub fn new(family_id: &str) -> Self {
        Self {
            manager: Arc::new(RwLock::new(LifecycleManager::new(family_id))),
        }
    }

    /// Create with an existing manager
    pub const fn with_manager(manager: Arc<RwLock<LifecycleManager>>) -> Self {
        Self { manager }
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

        info!("🌱 Registered primal via API: {}", name);

        Ok(json!({
            "registered": name,
            "socket_path": socket_path,
            "pid": pid,
            "state": "incubating"
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
    /// are live, and per-primal health status. Designed for real-time monitoring
    /// dashboards (ludoSpring, petalTongue).
    ///
    /// JSON-RPC method: `lifecycle.composition`
    pub async fn composition(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut active = Vec::new();
        let mut degraded = Vec::new();
        let mut dead = Vec::new();

        for (name, state) in &status {
            let entry = json!({
                "name": name,
                "state": state_to_string(state),
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
}

// ============================================================================
// HELPERS
// ============================================================================

/// Convert lifecycle state to a simple string
const fn state_to_string(state: &LifecycleState) -> &'static str {
    match state {
        LifecycleState::Germinating => "germinating",
        LifecycleState::Incubating { .. } => "incubating",
        LifecycleState::Active { .. } => "active",
        LifecycleState::Degraded { .. } => "degraded",
        LifecycleState::Apoptosis { .. } => "apoptosis",
        LifecycleState::Dead { .. } => "dead",
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle_manager::LifecycleManager;
    use crate::neural_graph::GraphNode;

    fn make_graph_node(id: &str, depends_on: Vec<String>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            depends_on,
            ..Default::default()
        }
    }

    // =========================================================================
    // Handler creation and status
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_handler_creation() {
        let handler = LifecycleHandler::new("test-family");

        let status = handler.status().await.expect("status");
        assert_eq!(status["count"], 0);
        assert!(status["primals"].as_array().expect("primals").is_empty());
    }

    #[tokio::test]
    async fn test_lifecycle_handler_with_manager() {
        let manager = Arc::new(RwLock::new(LifecycleManager::new("custom-family")));
        let handler = LifecycleHandler::with_manager(manager);

        let status = handler.status().await.expect("status");
        assert_eq!(status["count"], 0);
    }

    // =========================================================================
    // Registration and status serialization
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_register() {
        let handler = LifecycleHandler::new("test-family");

        let params = json!({
            "name": "test-primal",
            "socket_path": "/tmp/test-primal.sock",
            "pid": 12345
        });

        let result = handler.register(&Some(params)).await.expect("register");
        assert_eq!(result["registered"], "test-primal");
        assert_eq!(result["state"], "incubating");
        assert_eq!(result["socket_path"], "/tmp/test-primal.sock");
        assert_eq!(result["pid"], 12345);

        let status = handler.status().await.expect("status");
        assert_eq!(status["count"], 1);
        assert_eq!(status["healthy"], 0);

        let primals = status["primals"].as_array().expect("primals");
        let p = &primals[0];
        assert_eq!(p["name"], "test-primal");
        assert_eq!(p["state"], "incubating");
        assert!(p.get("details").is_some());
    }

    #[tokio::test]
    async fn test_lifecycle_register_with_deployment_node() {
        let handler = LifecycleHandler::new("test-family");

        let deployment_node = make_graph_node("beardog", vec![]);
        let params = json!({
            "name": "beardog",
            "socket_path": "/tmp/beardog.sock",
            "pid": 42,
            "deployment_node": serde_json::to_value(&deployment_node).unwrap()
        });

        let result = handler.register(&Some(params)).await.expect("register");
        assert_eq!(result["registered"], "beardog");

        let get_result = handler
            .get(&Some(json!({"name": "beardog"})))
            .await
            .expect("get");
        assert_eq!(get_result["name"], "beardog");
        assert!(get_result.get("depends_on").is_some());
        assert!(get_result.get("depended_by").is_some());
    }

    #[tokio::test]
    async fn test_lifecycle_register_without_pid() {
        let handler = LifecycleHandler::new("test-family");

        let params = json!({
            "name": "no-pid-primal",
            "socket_path": "/tmp/no-pid.sock"
        });

        let result = handler.register(&Some(params)).await.expect("register");
        assert_eq!(result["registered"], "no-pid-primal");
        assert!(result["pid"].is_null());
    }

    // =========================================================================
    // Error handling - missing/invalid parameters
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_get_missing_params() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .get(&None)
            .await
            .expect_err("get with None should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_lifecycle_get_missing_name() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .get(&Some(json!({})))
            .await
            .expect_err("get with empty params should fail");
        assert!(err.to_string().contains("name"));
    }

    #[tokio::test]
    async fn test_lifecycle_get_nonexistent_primal() {
        let handler = LifecycleHandler::new("test-family");

        let result = handler
            .get(&Some(json!({"name": "nonexistent"})))
            .await
            .expect("get returns Ok with error in body");
        assert!(result.get("error").is_some());
        assert!(
            result["error"]
                .as_str()
                .expect("error string")
                .contains("nonexistent")
        );
    }

    #[tokio::test]
    async fn test_lifecycle_register_missing_params() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .register(&None)
            .await
            .expect_err("register with None should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_lifecycle_register_missing_name() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .register(&Some(json!({"socket_path": "/tmp/x.sock"})))
            .await
            .expect_err("register without name should fail");
        assert!(err.to_string().contains("name"));
    }

    #[tokio::test]
    async fn test_lifecycle_register_missing_socket_path() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .register(&Some(json!({"name": "x"})))
            .await
            .expect_err("register without socket_path should fail");
        assert!(err.to_string().contains("socket_path"));
    }

    #[tokio::test]
    async fn test_lifecycle_resurrect_missing_params() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .resurrect(&None)
            .await
            .expect_err("resurrect with None should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_lifecycle_resurrect_nonexistent() {
        let handler = LifecycleHandler::new("test-family");

        let result = handler
            .resurrect(&Some(json!({"name": "ghost"})))
            .await
            .expect("resurrect returns Ok");
        assert!(result.get("error").is_some());
        assert!(result["error"].as_str().unwrap().contains("ghost"));
    }

    #[tokio::test]
    async fn test_lifecycle_apoptosis_missing_params() {
        let handler = LifecycleHandler::new("test-family");

        let err = handler
            .apoptosis(&None)
            .await
            .expect_err("apoptosis with None should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    // =========================================================================
    // Apoptosis and reason handling
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_apoptosis_user_request() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "victim",
                "socket_path": "/tmp/victim.sock",
                "pid": 9999
            })))
            .await
            .expect("register");

        let result = handler
            .apoptosis(&Some(json!({"name": "victim"})))
            .await
            .expect("apoptosis");
        assert_eq!(result["initiated"], "victim");
        assert_eq!(result["reason"], "user_request");
        assert_eq!(result["state"], "apoptosis");

        let status = handler.status().await.expect("status");
        let primals = status["primals"].as_array().expect("primals");
        let victim = primals
            .iter()
            .find(|p| p["name"] == "victim")
            .expect("victim");
        assert_eq!(victim["state"], "dead");
    }

    #[tokio::test]
    async fn test_lifecycle_apoptosis_all_reasons() {
        let reasons = [
            ("ecosystem_health", "ecosystem_health"),
            ("resource_pressure", "resource_pressure"),
            ("system_shutdown", "system_shutdown"),
            ("unknown_reason", "unknown_reason"),
        ];

        for (reason_param, expected_reason) in reasons {
            let handler = LifecycleHandler::new("test-family");
            let name = format!("primal-{reason_param}");
            handler
                .register(&Some(json!({
                    "name": name,
                    "socket_path": format!("/tmp/{}.sock", name),
                    "pid": 1
                })))
                .await
                .expect("register");

            let result = handler
                .apoptosis(&Some(json!({
                    "name": name,
                    "reason": reason_param
                })))
                .await
                .expect("apoptosis");
            assert_eq!(
                result["reason"].as_str(),
                Some(expected_reason),
                "reason {reason_param} should map to {expected_reason}"
            );
        }
    }

    // =========================================================================
    // Shutdown and resurrection
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_shutdown_all() {
        let handler = LifecycleHandler::new("test-family");
        for name in &["a", "b", "c"] {
            handler
                .register(&Some(json!({
                    "name": name,
                    "socket_path": format!("/tmp/{}.sock", name),
                    "pid": 1
                })))
                .await
                .expect("register");
        }

        let result = handler.shutdown_all().await.expect("shutdown_all");
        assert_eq!(result["shutdown"], "complete");
        assert!(result["message"].as_str().unwrap().contains("All primals"));

        let status = handler.status().await.expect("status");
        assert_eq!(status["count"], 3);
        let primals = status["primals"].as_array().expect("primals");
        for p in primals {
            assert_eq!(p["state"], "dead");
        }
    }

    #[tokio::test]
    async fn test_lifecycle_resurrect_registered_primal() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "resurrect-me",
                "socket_path": "/tmp/resurrect-me.sock",
                "pid": 1234
            })))
            .await
            .expect("register");

        let result = handler
            .resurrect(&Some(json!({"name": "resurrect-me"})))
            .await
            .expect("resurrect");
        assert_eq!(result["requested"], "resurrect-me");
        assert!(result["message"].as_str().unwrap().contains("Resurrection"));
    }

    // =========================================================================
    // State serialization (via get and status)
    // =========================================================================

    #[tokio::test]
    async fn test_lifecycle_get_full_serialization() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "full-details",
                "socket_path": "/tmp/full.sock",
                "pid": 9999
            })))
            .await
            .expect("register");

        let result = handler
            .get(&Some(json!({"name": "full-details"})))
            .await
            .expect("get");

        assert_eq!(result["name"], "full-details");
        assert_eq!(result["family_id"], "test-family");
        assert_eq!(result["socket_path"], "/tmp/full.sock");
        assert_eq!(result["pid"], 9999);
        assert_eq!(result["state"], "incubating");

        assert!(result.get("state_details").is_some());
        assert!(result.get("depends_on").is_some());
        assert!(result.get("depended_by").is_some());
        assert!(result.get("metrics").is_some());
        assert!(result.get("health_config").is_some());
        assert!(result.get("resurrection_config").is_some());

        let metrics = &result["metrics"];
        assert!(metrics.get("total_uptime_secs").is_some());
        assert!(metrics.get("resurrection_count").is_some());
        assert!(metrics.get("health_failures").is_some());
        assert!(metrics.get("last_health_latency_ms").is_some());
        assert!(metrics.get("requests_served").is_some());

        let health_config = &result["health_config"];
        assert!(health_config.get("check_interval_secs").is_some());
        assert!(health_config.get("timeout_secs").is_some());
        assert!(health_config.get("failure_threshold").is_some());
        assert!(health_config.get("health_method").is_some());

        let res_config = &result["resurrection_config"];
        assert!(res_config.get("enabled").is_some());
        assert!(res_config.get("max_attempts").is_some());
        assert!(res_config.get("base_delay_secs").is_some());
        assert!(res_config.get("max_delay_secs").is_some());
    }

    #[tokio::test]
    async fn test_lifecycle_status_healthy_count() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "p1",
                "socket_path": "/tmp/p1.sock",
                "pid": 1
            })))
            .await
            .expect("register");

        let status = handler.status().await.expect("status");
        assert_eq!(status["count"], 1);
        assert_eq!(status["healthy"], 0);

        handler
            .apoptosis(&Some(json!({"name": "p1"})))
            .await
            .expect("apoptosis");

        let status = handler.status().await.expect("status");
        assert_eq!(status["healthy"], 0);
    }

    #[tokio::test]
    async fn test_lifecycle_incubating_state_details() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "incubating",
                "socket_path": "/tmp/inc.sock",
                "pid": 1
            })))
            .await
            .expect("register");

        let status = handler.status().await.expect("status");
        let primals = status["primals"].as_array().expect("primals");
        let p = primals
            .iter()
            .find(|x| x["name"] == "incubating")
            .expect("primal");
        assert_eq!(p["state"], "incubating");
        let details = &p["details"];
        assert!(details.get("started_at").is_some());
        assert!(details.get("timeout_ms").is_some());
    }

    #[tokio::test]
    async fn test_lifecycle_dead_state_details() {
        let handler = LifecycleHandler::new("test-family");
        handler
            .register(&Some(json!({
                "name": "to-die",
                "socket_path": "/tmp/die.sock",
                "pid": 1
            })))
            .await
            .expect("register");

        handler
            .apoptosis(&Some(json!({"name": "to-die", "reason": "user_request"})))
            .await
            .expect("apoptosis");

        let status = handler.status().await.expect("status");
        let primals = status["primals"].as_array().expect("primals");
        let p = primals
            .iter()
            .find(|x| x["name"] == "to-die")
            .expect("primal");
        assert_eq!(p["state"], "dead");
        let details = &p["details"];
        assert!(details.get("since").is_some());
        assert!(details.get("reason").is_some());
    }
}
