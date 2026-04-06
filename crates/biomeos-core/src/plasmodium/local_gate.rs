// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Local gate discovery: runtime socket scan, primal health, models, load.
//!
//! Running primals are discovered only via [`crate::socket_discovery::SocketDiscovery`]:
//! all `*-{family}.sock` files under the biomeOS runtime directory are probed for health.
//! There is no fallback to a compiled-in primal name list—if nothing is listening, the
//! gate reports an empty primal list (graceful degradation).

use anyhow::Result;
use serde_json::json;

use crate::atomic_client::AtomicClient;
use crate::model_cache::ModelCache;
use crate::socket_discovery::SocketDiscovery;

use super::system;
use super::types::{BondType, GateInfo, PrimalStatus};

impl super::Plasmodium {
    /// Query local gate status
    ///
    /// Dynamically discovers all running primals by scanning the runtime
    /// directory for family-matching sockets via [`SocketDiscovery`], rather than
    /// iterating a compiled primal name list.
    pub(super) async fn query_local_gate(&self) -> Result<GateInfo> {
        let mut primals = Vec::new();

        let discovery = SocketDiscovery::new(&self.family_id);
        let socket_paths = discovery.list_family_scoped_unix_sockets();
        let suffix = format!("-{}.sock", self.family_id);

        for socket_path in socket_paths {
            let filename = socket_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let primal_name = filename
                .strip_suffix(&suffix)
                .unwrap_or(filename)
                .to_string();

            match AtomicClient::unix(&socket_path)
                .call("health", json!({}))
                .await
            {
                Ok(result) => {
                    primals.push(PrimalStatus {
                        name: primal_name,
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
                    primals.push(PrimalStatus {
                        name: primal_name,
                        healthy: false,
                        version: None,
                    });
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
