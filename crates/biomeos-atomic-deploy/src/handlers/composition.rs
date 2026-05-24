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

        let has_active_domain = |domain: &str| -> bool {
            primal_health.iter().any(|p| {
                p["state"].as_str() == Some("active")
                    && p["capabilities"].as_array().is_some_and(|caps| {
                        caps.iter()
                            .any(|c| c.as_str().is_some_and(|s| s.starts_with(domain)))
                    })
            })
        };
        let content_pipeline = json!({
            "storage": has_active_domain("storage.") || has_active_domain("content."),
            "dag": has_active_domain("dag."),
            "ledger": has_active_domain("spine.") || has_active_domain("anchor."),
            "attribution": has_active_domain("braid.") || has_active_domain("provenance."),
            "ready": (has_active_domain("storage.") || has_active_domain("content."))
                && has_active_domain("dag.")
                && (has_active_domain("spine.") || has_active_domain("anchor."))
                && (has_active_domain("braid.") || has_active_domain("provenance.")),
        });
        let compute_pipeline = json!({
            "dispatch": has_active_domain("compute."),
            "shader": has_active_domain("shader.") || has_active_domain("compile."),
            "gpu": has_active_domain("math.") || has_active_domain("gpu."),
            "ready": has_active_domain("compute."),
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

        // Collect capability surfaces for all registered primals.
        let mut primal_caps: Vec<(Vec<String>, bool)> = Vec::with_capacity(status.len());
        for (name, state) in &status {
            let info = manager.get_primal_info(name).await;
            let caps: Vec<String> = info
                .as_ref()
                .and_then(|i| i.deployment_node.as_ref())
                .map(|n| n.capabilities.clone())
                .unwrap_or_default();
            let is_active = matches!(state, LifecycleState::Active { .. });
            primal_caps.push((caps, is_active));
        }

        // Capability domains that define each subsystem tier.
        // Primals are matched by their registered capabilities, not by name.
        const TOWER_DOMAINS: &[&str] =
            &["crypto.", "security.", "discovery.", "relay.", "defense."];
        const NODE_DOMAINS: &[&str] = &["compute.", "science.", "inference."];
        const NEST_DOMAINS: &[&str] = &["storage.", "content.", "dag.", "spine.", "braid."];
        const MESH_DOMAINS: &[&str] = &["discovery.", "relay.", "network."];

        let subsystem_status = |domains: &[&str]| -> &'static str {
            let mut found_any = false;
            for (caps, is_active) in &primal_caps {
                let matches_domain = caps
                    .iter()
                    .any(|c| domains.iter().any(|d| c.starts_with(d)));
                if matches_domain {
                    found_any = true;
                    if !is_active {
                        return "degraded";
                    }
                }
            }
            if found_any { "ok" } else { "unavailable" }
        };

        let tower = subsystem_status(TOWER_DOMAINS);
        let node = subsystem_status(NODE_DOMAINS);
        let nest = subsystem_status(NEST_DOMAINS);
        let mesh_process = subsystem_status(MESH_DOMAINS);

        subsystems.insert("tower".to_string(), json!(tower));
        subsystems.insert("node".to_string(), json!(node));
        subsystems.insert("nest".to_string(), json!(nest));

        // Discover mesh provider by capability rather than hardcoded name.
        let mesh_provider_name: Option<String> = {
            let status_names: Vec<&String> = status.keys().collect();
            primal_caps
                .iter()
                .zip(status_names.iter())
                .find(|((caps, is_active), _)| {
                    *is_active
                        && caps
                            .iter()
                            .any(|c| c.starts_with("discovery.") || c.starts_with("relay."))
                })
                .map(|(_, name)| (*name).clone())
        };

        let mesh_detail = if mesh_process == "ok" {
            let probe_target = mesh_provider_name.unwrap_or_default();
            match self.probe_mesh_provider(&manager, &probe_target).await {
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
