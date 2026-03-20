// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Local gate discovery: runtime socket scan, primal health, models, load.

use anyhow::Result;
use serde_json::json;

use crate::atomic_client::AtomicClient;
use crate::model_cache::ModelCache;

use super::system;
use super::types::{BondType, GateInfo, PrimalStatus};

impl super::Plasmodium {
    /// Query local gate status
    ///
    /// Dynamically discovers all running primals by scanning the runtime
    /// directory for family-matching sockets, rather than hardcoding primal names.
    pub(super) async fn query_local_gate(&self) -> Result<GateInfo> {
        let mut primals = Vec::new();

        // Discover running primals from socket directory (capability-based discovery)
        let runtime_dir = biomeos_types::paths::SystemPaths::new_lazy()
            .runtime_dir()
            .to_path_buf();
        let suffix = format!("-{}.sock", self.family_id);

        if let Ok(mut entries) = tokio::fs::read_dir(&runtime_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(primal_name) = name.strip_suffix(&suffix) {
                    // Found a family-matching socket -- health check it
                    let socket_path = entry.path();
                    match AtomicClient::unix(&socket_path)
                        .call("health", json!({}))
                        .await
                    {
                        Ok(result) => {
                            primals.push(PrimalStatus {
                                name: primal_name.to_string(),
                                healthy: result
                                    .get("status")
                                    .and_then(|s| s.as_str())
                                    .is_some_and(|s| s == "healthy"),
                                version: result
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .map(std::string::ToString::to_string),
                            });
                        }
                        Err(_) => {
                            // Socket exists but unresponsive (stale or starting up)
                            primals.push(PrimalStatus {
                                name: primal_name.to_string(),
                                healthy: false,
                                version: None,
                            });
                        }
                    }
                }
            }
        }

        // Fallback: if socket directory scan found nothing, try known primals via env-based discovery
        if primals.is_empty() {
            for primal_name in biomeos_types::primal_names::CORE_PRIMALS {
                if let Ok(client) = AtomicClient::discover(primal_name).await {
                    let health = Self::check_primal_health(&client, primal_name).await;
                    primals.push(health);
                }
            }
        }

        // Get compute info
        let compute = system::query_local_compute(&self.local_gate_id).await;

        // Get model cache
        let models = Self::query_local_models().await;

        // Get system load
        let load = system::get_system_load();

        Ok(GateInfo {
            gate_id: self.local_gate_id.clone(),
            address: "local".to_string(),
            is_local: true,
            primals,
            compute,
            models,
            load,
            reachable: true,
            bond_type: BondType::Covalent,
        })
    }

    /// Check health of a single primal
    pub(super) async fn check_primal_health(client: &AtomicClient, name: &str) -> PrimalStatus {
        match client.call("health", json!({})).await {
            Ok(result) => PrimalStatus {
                name: name.to_string(),
                healthy: result
                    .get("status")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| s == "healthy"),
                version: result
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string),
            },
            Err(_) => PrimalStatus {
                name: name.to_string(),
                healthy: false,
                version: None,
            },
        }
    }

    /// Query local model cache
    pub(super) async fn query_local_models() -> Vec<String> {
        match ModelCache::new().await {
            Ok(cache) => cache
                .list_models()
                .iter()
                .map(|m| m.model_id.clone())
                .collect(),
            Err(_) => vec![],
        }
    }
}
