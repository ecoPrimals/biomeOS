// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Live composition dashboard handler (`lifecycle.composition`).

use anyhow::Result;
use serde_json::{Value, json};

use super::super::spring_status::state_to_string;
use super::LifecycleHandler;
use super::helpers::state_details;
use crate::lifecycle_manager::LifecycleState;

impl LifecycleHandler {
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
}
