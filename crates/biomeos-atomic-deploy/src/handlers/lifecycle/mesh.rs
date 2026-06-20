// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Mesh provider probing for composition health reporting.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use tracing::info;

use super::LifecycleHandler;
use crate::lifecycle_manager::LifecycleManager;
use biomeos_core::atomic_client::AtomicClient;

impl LifecycleHandler {
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
