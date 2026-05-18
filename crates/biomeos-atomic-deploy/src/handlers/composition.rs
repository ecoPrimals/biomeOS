// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composition status and health handlers.
//!
//! Extracted from `lifecycle.rs` to keep handler files under 800 lines.
//! These methods report on the health and readiness of the composed primal
//! ecosystem, including pipeline status for content and compute.

use anyhow::Result;
use serde_json::{Value, json};
use tracing::warn;

use super::LifecycleHandler;
use super::spring_status::state_to_string;
use crate::lifecycle_manager::LifecycleState;
use biomeos_types::primal_names;

impl LifecycleHandler {
    /// Handle `composition.status` — adaptive daemon surface for pappusCast.
    ///
    /// Returns `{ active_users, primal_health, resource_pressure, pipelines }` so
    /// projectNUCLEUS can drive adaptive daemons without scraping JupyterHub.
    ///
    /// JSON-RPC method: `composition.status`
    pub async fn composition_status(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut primal_health = Vec::with_capacity(status.len());
        let mut active_count: u64 = 0;

        for (name, state) in &status {
            let info = manager.get_primal_info(name).await;

            let (latency_ms, failures, resurrection_count, capabilities) = if let Some(ref i) = info
            {
                let caps: Vec<String> = i
                    .deployment_node
                    .as_ref()
                    .map(|n| n.capabilities.clone())
                    .unwrap_or_default();
                (
                    i.metrics.last_health_latency_ms,
                    i.metrics.health_failures,
                    i.metrics.resurrection_count,
                    caps,
                )
            } else {
                (0, 0, 0, vec![])
            };

            let state_str = state_to_string(state);
            if state_str == "active" {
                active_count += 1;
            }

            primal_health.push(json!({
                "name": name,
                "state": state_str,
                "latency_ms": latency_ms,
                "failures": failures,
                "resurrection_count": resurrection_count,
                "capabilities": capabilities,
            }));
        }

        let resource_pressure = match biomeos_system::SystemInspector::get_resource_usage().await {
            Ok(metrics) => json!({
                "cpu": metrics.cpu_usage,
                "memory": metrics.memory_usage,
                "disk": metrics.disk_usage,
            }),
            Err(e) => {
                warn!("Resource metrics unavailable: {e}");
                json!({
                    "cpu": null,
                    "memory": null,
                    "disk": null,
                    "error": e.to_string(),
                })
            }
        };

        let topology = self
            .topology_version
            .load(std::sync::atomic::Ordering::Relaxed);

        let has_active = |name: &str| -> bool {
            primal_health
                .iter()
                .any(|p| p["name"].as_str() == Some(name) && p["state"].as_str() == Some("active"))
        };
        let content_pipeline = json!({
            "storage": has_active("nestgate"),
            "dag": has_active("rhizocrypt"),
            "ledger": has_active("loamspine"),
            "attribution": has_active("sweetgrass"),
            "ready": has_active("nestgate") && has_active("rhizocrypt")
                && has_active("loamspine") && has_active("sweetgrass"),
        });
        let compute_pipeline = json!({
            "dispatch": has_active("toadstool"),
            "shader": has_active("coralreef"),
            "gpu": has_active("barracuda"),
            "ready": has_active("toadstool"),
        });

        Ok(json!({
            "active_users": active_count,
            "primal_health": primal_health,
            "resource_pressure": resource_pressure,
            "total_primals": status.len(),
            "topology_version": topology,
            "pipelines": {
                "content": content_pipeline,
                "compute": compute_pipeline,
            },
        }))
    }

    /// Handle `composition.health` / `composition.tower_health` etc.
    ///
    /// Follows `COMPOSITION_HEALTH_STANDARD.md`: returns `healthy`, `deploy_graph`,
    /// and `subsystems` map. Aggregates lifecycle state into subsystem health.
    pub async fn composition_health(&self, _params: &Option<Value>) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut subsystems: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut deploy_graph = String::from("unknown");
        let mut all_healthy = true;

        let security_provider = std::env::var("BIOMEOS_SECURITY_PROVIDER")
            .unwrap_or_else(|_| primal_names::BEARDOG.to_string());
        let discovery_provider = std::env::var("BIOMEOS_NETWORK_PROVIDER")
            .unwrap_or_else(|_| primal_names::SONGBIRD.to_string());
        let compute_provider = std::env::var("BIOMEOS_COMPUTE_PROVIDER")
            .unwrap_or_else(|_| primal_names::TOADSTOOL.to_string());
        let storage_provider = std::env::var("BIOMEOS_STORAGE_PROVIDER")
            .unwrap_or_else(|_| primal_names::NESTGATE.to_string());

        let tower_primals: Vec<&str> = vec![&security_provider, &discovery_provider];
        let node_primals: Vec<&str> = vec![&compute_provider];
        let nest_primals: Vec<&str> = vec![&storage_provider];
        let mesh_primals: Vec<&str> = vec![&discovery_provider];

        let subsystem_status = |names: &[&str]| -> &'static str {
            let mut found_any = false;
            for (name, state) in &status {
                let lower = name.to_lowercase();
                if names.iter().any(|n| lower.contains(n)) {
                    found_any = true;
                    if !matches!(state, LifecycleState::Active { .. }) {
                        return "degraded";
                    }
                }
            }
            if found_any { "ok" } else { "unavailable" }
        };

        let tower = subsystem_status(&tower_primals);
        let node = subsystem_status(&node_primals);
        let nest = subsystem_status(&nest_primals);
        let mesh_process = subsystem_status(&mesh_primals);

        subsystems.insert("tower".to_string(), json!(tower));
        subsystems.insert("node".to_string(), json!(node));
        subsystems.insert("nest".to_string(), json!(nest));

        let mesh_detail = if mesh_process == "ok" {
            match self
                .probe_mesh_provider(&manager, &discovery_provider)
                .await
            {
                Ok(detail) => detail,
                Err(_) => json!({ "status": "ok", "detail": "process_alive" }),
            }
        } else {
            json!({ "status": mesh_process })
        };
        subsystems.insert("mesh".to_string(), mesh_detail.clone());

        let mesh_healthy = mesh_detail
            .get("status")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == "ok");
        if tower != "ok" || !mesh_healthy {
            all_healthy = false;
        }

        let active_count = status
            .iter()
            .filter(|(_, s)| matches!(s, LifecycleState::Active { .. }))
            .count();

        if active_count >= 5 {
            deploy_graph = "nucleus_complete".to_string();
        } else if active_count >= 3 {
            deploy_graph = "nucleus_simple".to_string();
        } else if active_count >= 2 {
            deploy_graph = "tower_atomic".to_string();
        }

        Ok(json!({
            "healthy": all_healthy,
            "deploy_graph": deploy_graph,
            "subsystems": subsystems,
            "capabilities_count": status.len(),
        }))
    }
}
