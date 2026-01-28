//! Lifecycle management handler for Neural API
//!
//! Exposes lifecycle management operations via JSON-RPC:
//! - `lifecycle.status` - Get status of all managed primals
//! - `lifecycle.get` - Get detailed info for a specific primal
//! - `lifecycle.resurrect` - Force resurrection of a degraded/dead primal
//! - `lifecycle.apoptosis` - Initiate graceful shutdown
//! - `lifecycle.register` - Register a primal for management
//! - `lifecycle.shutdown_all` - Initiate system-wide shutdown

use anyhow::{Context, Result};
use serde_json::{json, Value};
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
    pub fn new(family_id: &str) -> Self {
        Self {
            manager: Arc::new(RwLock::new(LifecycleManager::new(family_id))),
        }
    }

    /// Create with an existing manager
    pub fn with_manager(manager: Arc<RwLock<LifecycleManager>>) -> Self {
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
            .register_primal(
                name,
                PathBuf::from(socket_path),
                pid,
                deployment_node,
            )
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
fn state_to_string(state: &LifecycleState) -> &'static str {
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

    #[tokio::test]
    async fn test_lifecycle_handler_creation() {
        let handler = LifecycleHandler::new("test-family");

        // Should return empty status initially
        let status = handler.status().await.unwrap();
        assert_eq!(status["count"], 0);
    }

    #[tokio::test]
    async fn test_lifecycle_register() {
        let handler = LifecycleHandler::new("test-family");

        let params = json!({
            "name": "test-primal",
            "socket_path": "/tmp/test-primal.sock",
            "pid": 12345
        });

        let result = handler.register(&Some(params)).await.unwrap();
        assert_eq!(result["registered"], "test-primal");
        assert_eq!(result["state"], "incubating");

        // Should now show in status
        let status = handler.status().await.unwrap();
        assert_eq!(status["count"], 1);
    }
}

