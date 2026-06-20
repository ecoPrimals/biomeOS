// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Status and introspection handlers (`lifecycle.status`, `lifecycle.get`).

use anyhow::{Context, Result};
use serde_json::{Value, json};

use super::super::spring_status::state_to_string;
use super::LifecycleHandler;
use super::helpers::state_details;

impl LifecycleHandler {
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
}
