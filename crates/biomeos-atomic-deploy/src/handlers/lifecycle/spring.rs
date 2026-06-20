// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JupyterHub / Tier-2 spring status handler (`biomeos.spring_status`).

use anyhow::Result;
use serde_json::{Value, json};

use super::super::spring_status::{binary_search_dirs, probe_binary, state_to_string};
use super::LifecycleHandler;

impl LifecycleHandler {
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
}
