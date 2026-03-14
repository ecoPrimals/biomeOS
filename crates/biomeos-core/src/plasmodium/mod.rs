// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Plasmodium - Over-NUCLEUS Coordination Layer
//!
//! Named after *Physarum polycephalum* (slime mold), Plasmodium is the emergent
//! coordination layer that forms when 2+ NUCLEUS instances bond covalently.
//!
//! ## Design Principles
//!
//! - **No central brain**: Any gate can query the collective
//! - **Pulsing coordination**: Songbird mesh heartbeats carry state
//! - **Graceful degradation**: Gates join/leave dynamically
//! - **Capability-based routing**: Workloads flow to capabilities, not names
//!
//! ## Architecture
//!
//! ```text
//! biomeos plasmodium status
//!     |
//!     v
//! Query local Songbird: mesh.peers
//!     |
//!     v
//! For each peer: query NUCLEUS status via Songbird HTTP JSON-RPC gateway
//!   (POST /jsonrpc on port discovered via beacon, default 8080)
//!     |
//!     v
//! Aggregate into PlasmodiumState (the collective view)
//! ```
//!
//! See `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` for the full specification.

mod system;
pub mod types;

pub use types::*;

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;
use crate::model_cache::ModelCache;
use types::PeerInfo;

// ─── Plasmodium Query Engine ─────────────────────────────────────────────

/// Plasmodium collective query engine
///
/// Queries local and remote NUCLEUS instances to build a collective view.
pub struct Plasmodium {
    family_id: String,
    local_gate_id: String,
}

impl Default for Plasmodium {
    fn default() -> Self {
        Self::new()
    }
}

impl Plasmodium {
    /// Create a new Plasmodium query engine
    pub fn new() -> Self {
        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("NODE_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());

        let local_gate_id = std::env::var("GATE_ID")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| {
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string())
            });

        Self {
            family_id,
            local_gate_id,
        }
    }

    /// Query the full collective state
    ///
    /// 1. Gather local gate info
    /// 2. Query Songbird mesh for peers
    /// 3. For each peer, query their NUCLEUS status (TCP or SSH)
    /// 4. Aggregate into `PlasmodiumState`
    pub async fn query_collective(&self) -> Result<PlasmodiumState> {
        info!(
            "Querying plasmodium collective for family '{}'",
            self.family_id
        );

        let mut gates = Vec::new();

        // 1. Gather local gate info
        let local = self.query_local_gate().await?;
        gates.push(local);

        // 2. Query Songbird mesh for peers
        let peers = self.discover_peers().await;

        // 3. Query each peer (prefer Songbird HTTP JSON-RPC, fallback to SSH)
        for peer in peers {
            let result = if peer.address.starts_with("ssh:") {
                // SSH-based peer (legacy): ssh:user@host
                // Try Songbird HTTP JSON-RPC first by extracting the host
                let ssh_target = &peer.address[4..];
                let host = ssh_target.split('@').next_back().unwrap_or(ssh_target);
                // Evolved: No SSH fallback — use Songbird mesh RPC only
                self.query_remote_gate(host, &peer.node_id).await
            } else {
                // HTTP JSON-RPC query via Songbird gateway (covalent bond transport)
                self.query_remote_gate(&peer.address, &peer.node_id).await
            };

            match result {
                Ok(gate) => gates.push(gate),
                Err(e) => {
                    warn!("Could not reach peer {}: {}", peer.node_id, e);
                    // Add unreachable gate entry
                    gates.push(GateInfo {
                        gate_id: peer.node_id,
                        address: peer.address,
                        is_local: false,
                        primals: vec![],
                        compute: ComputeInfo::default(),
                        models: vec![],
                        load: 0.0,
                        reachable: false,
                        bond_type: BondType::Covalent,
                    });
                }
            }
        }

        // 4. Aggregate collective capabilities
        let collective = Self::aggregate_capabilities(&gates);

        Ok(PlasmodiumState {
            gates,
            snapshot_at: chrono::Utc::now().to_rfc3339(),
            family_id: self.family_id.clone(),
            collective,
        })
    }

    /// Query local gate status
    ///
    /// Dynamically discovers all running primals by scanning the runtime
    /// directory for family-matching sockets, rather than hardcoding primal names.
    async fn query_local_gate(&self) -> Result<GateInfo> {
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
                                    .map(|s| s == "healthy")
                                    .unwrap_or(false),
                                version: result
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
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
    async fn check_primal_health(client: &AtomicClient, name: &str) -> PrimalStatus {
        match client.call("health", json!({})).await {
            Ok(result) => PrimalStatus {
                name: name.to_string(),
                healthy: result
                    .get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s == "healthy")
                    .unwrap_or(false),
                version: result
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            },
            Err(_) => PrimalStatus {
                name: name.to_string(),
                healthy: false,
                version: None,
            },
        }
    }

    /// Query local model cache
    async fn query_local_models() -> Vec<String> {
        match ModelCache::new().await {
            Ok(cache) => cache
                .list_models()
                .iter()
                .map(|m| m.model_id.clone())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Discover peers via Songbird mesh + `PLASMODIUM_PEERS` env var
    async fn discover_peers(&self) -> Vec<PeerInfo> {
        let mut peers = Vec::new();

        let discovery_provider =
            std::env::var("DISCOVERY_PROVIDER").unwrap_or_else(|_| "songbird".to_string());
        if let Ok(client) = AtomicClient::discover(&discovery_provider).await {
            if let Ok(result) = client.call("mesh.peers", json!({})).await {
                let peers_array = result
                    .get("peers")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default();

                for peer_val in peers_array {
                    if let (Some(node_id), Some(address)) = (
                        peer_val.get("node_id").and_then(|n| n.as_str()),
                        peer_val.get("address").and_then(|a| a.as_str()),
                    ) {
                        peers.push(PeerInfo {
                            node_id: node_id.to_string(),
                            address: address.to_string(),
                        });
                    }
                }
            } else {
                debug!("mesh.peers call failed, falling back to env var");
            }
        } else {
            debug!(
                "Discovery provider '{discovery_provider}' not available, \
                 using PLASMODIUM_PEERS for peer discovery"
            );
        }

        // Always merge PLASMODIUM_PEERS env var (supplements Songbird mesh)
        // Format: node_id@host:port  or  node_id@ssh:user@host
        if let Ok(peer_list) = std::env::var("PLASMODIUM_PEERS") {
            for peer_str in peer_list.split(',') {
                let peer_str = peer_str.trim();
                if peer_str.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = peer_str.splitn(2, '@').collect();
                if parts.len() == 2 {
                    let node_id = parts[0].to_string();
                    let address = parts[1].to_string();
                    // Don't add duplicates
                    if !peers.iter().any(|p| p.node_id == node_id) {
                        peers.push(PeerInfo { node_id, address });
                    }
                } else {
                    // Just an IP/hostname
                    if !peers.iter().any(|p| p.node_id == parts[0]) {
                        peers.push(PeerInfo {
                            node_id: parts[0].to_string(),
                            address: parts[0].to_string(),
                        });
                    }
                }
            }
        }

        peers
    }

    /// Query a remote gate's NUCLEUS status via HTTP JSON-RPC gateway
    ///
    /// Uses HTTP POST to `/jsonrpc` on the remote discovery provider.
    /// The port is runtime-discovered from the `mesh.peers` response
    /// (beacon exchange), with env var and constants as fallbacks.
    async fn query_remote_gate(&self, address: &str, node_id: &str) -> Result<GateInfo> {
        let default_port: u16 = std::env::var("SONGBIRD_MESH_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(biomeos_types::constants::network::DEFAULT_HTTP_PORT);

        // Parse host:port from mesh.peers address (port comes from beacon discovery)
        let (host, port) = if let Some(idx) = address.rfind(':') {
            let h = &address[..idx];
            let p = address[idx + 1..].parse::<u16>().unwrap_or(default_port);
            (h.to_string(), p)
        } else {
            (address.to_string(), default_port)
        };

        // Use HTTP JSON-RPC gateway (covalent bond transport)
        let client = AtomicClient::http(&host, port);

        // Query health
        let health_result: Result<Value> = client.call("health", json!({})).await;
        let reachable = health_result.is_ok();

        if !reachable {
            anyhow::bail!("Gate {node_id} not reachable at {host}:{port}");
        }

        // Query remote primals
        let primals = Self::query_remote_primals(&client).await;

        Ok(GateInfo {
            gate_id: node_id.to_string(),
            address: address.to_string(),
            is_local: false,
            primals,
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        })
    }

    /// Query remote primals via Songbird TCP
    async fn query_remote_primals(client: &AtomicClient) -> Vec<PrimalStatus> {
        let mut primals = Vec::new();

        // Try lifecycle.status first (if neural API is running)
        if let Ok(result) = client.call("lifecycle.status", json!({})).await {
            if let Some(services) = result.get("services").and_then(|s| s.as_object()) {
                for (name, status) in services {
                    primals.push(PrimalStatus {
                        name: name.clone(),
                        healthy: status
                            .get("status")
                            .and_then(|s| s.as_str())
                            .map(|s| s == "healthy")
                            .unwrap_or(false),
                        version: status
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }

        if primals.is_empty() {
            let discovery_provider =
                std::env::var("DISCOVERY_PROVIDER").unwrap_or_else(|_| "songbird".to_string());
            primals.push(PrimalStatus {
                name: discovery_provider,
                healthy: true,
                version: None,
            });
        }

        primals
    }

    /// Aggregate capabilities across all gates
    fn aggregate_capabilities(gates: &[GateInfo]) -> CollectiveCapabilities {
        let mut all_gpus = Vec::new();
        let mut total_ram_gb = 0u64;
        let mut model_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut capability_set: HashSet<String> = HashSet::new();

        for gate in gates {
            if !gate.reachable {
                continue;
            }

            // GPUs
            for gpu in &gate.compute.gpus {
                all_gpus.push(gpu.clone());
            }

            // RAM
            total_ram_gb += gate.compute.ram_gb;

            // Models
            for model_id in &gate.models {
                model_map
                    .entry(model_id.clone())
                    .or_default()
                    .push(gate.gate_id.clone());
            }

            // Capabilities from primals (capability-based, not name-hardcoded)
            for primal in &gate.primals {
                if primal.healthy {
                    // Use the standard capability taxonomy to resolve primal capabilities
                    let caps =
                        biomeos_types::capability_taxonomy::capabilities_for_primal(&primal.name);
                    for cap in caps {
                        capability_set.insert(cap);
                    }
                }
            }
        }

        let models: Vec<ModelAvailability> = model_map
            .into_iter()
            .map(|(model_id, gates)| ModelAvailability {
                model_id,
                size_bytes: 0, // Would need model_cache lookup
                format: String::new(),
                gates,
            })
            .collect();

        let mut capabilities: Vec<String> = capability_set.into_iter().collect();
        capabilities.sort();

        CollectiveCapabilities {
            total_gpus: all_gpus.len(),
            gpus: all_gpus,
            total_ram_gb,
            total_models: models.len(),
            models,
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_type_display() {
        assert_eq!(BondType::Covalent.to_string(), "covalent");
        assert_eq!(BondType::Ionic.to_string(), "ionic");
    }

    #[test]
    fn test_aggregate_empty() {
        let caps = Plasmodium::aggregate_capabilities(&[]);
        assert_eq!(caps.total_gpus, 0);
        assert_eq!(caps.total_ram_gb, 0);
        assert!(caps.models.is_empty());
    }

    #[test]
    fn test_aggregate_with_gates() {
        let gates = vec![
            GateInfo {
                gate_id: "tower".to_string(),
                address: "local".to_string(),
                is_local: true,
                primals: vec![
                    PrimalStatus {
                        name: "beardog".to_string(),
                        healthy: true,
                        version: None,
                    },
                    PrimalStatus {
                        name: "songbird".to_string(),
                        healthy: true,
                        version: None,
                    },
                ],
                compute: ComputeInfo {
                    gpus: vec![GpuInfo {
                        name: "RTX 4070".to_string(),
                        vram_mb: 12288,
                        gate_id: "tower".to_string(),
                    }],
                    ram_gb: 32,
                    cpu_cores: 16,
                },
                models: vec!["TinyLlama/1.1B".to_string()],
                load: 0.1,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "gate2".to_string(),
                address: "192.168.1.132".to_string(),
                is_local: false,
                primals: vec![PrimalStatus {
                    name: "toadstool".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo {
                    gpus: vec![GpuInfo {
                        name: "RTX 3090".to_string(),
                        vram_mb: 24576,
                        gate_id: "gate2".to_string(),
                    }],
                    ram_gb: 256,
                    cpu_cores: 64,
                },
                models: vec!["TinyLlama/1.1B".to_string(), "Mistral-7B".to_string()],
                load: 0.05,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];

        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_gpus, 2);
        assert_eq!(caps.total_ram_gb, 32 + 256);
        assert_eq!(caps.total_models, 2); // unique
        assert!(caps.capabilities.contains(&"crypto".to_string()));
        assert!(caps.capabilities.contains(&"compute".to_string()));
    }

    #[test]
    fn test_system_ram() {
        // Just verify it doesn't panic
        let ram = system::get_system_ram_gb();
        assert!(ram > 0);
    }

    #[test]
    fn test_num_cpus() {
        assert!(system::num_cpus() > 0);
    }
}
