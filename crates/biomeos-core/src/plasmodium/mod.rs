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
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use biomeos_types::primal_names;

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
                    .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string())
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
    async fn check_primal_health(client: &AtomicClient, name: &str) -> PrimalStatus {
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

        let discovery_provider = std::env::var("DISCOVERY_PROVIDER")
            .unwrap_or_else(|_| biomeos_types::primal_names::SONGBIRD.to_string());
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
        if let Ok(result) = client.call("lifecycle.status", json!({})).await
            && let Some(services) = result.get("services").and_then(|s| s.as_object())
        {
            for (name, status) in services {
                primals.push(PrimalStatus {
                    name: name.clone(),
                    healthy: status
                        .get("status")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| s == "healthy"),
                    version: status
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(std::string::ToString::to_string),
                });
            }
        }

        if primals.is_empty() {
            let discovery_provider = std::env::var("DISCOVERY_PROVIDER")
                .unwrap_or_else(|_| primal_names::SONGBIRD.to_string());
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_type_display() {
        assert_eq!(BondType::Covalent.to_string(), "covalent");
        assert_eq!(BondType::Ionic.to_string(), "ionic");
        assert_eq!(BondType::Metallic.to_string(), "metallic");
        assert_eq!(BondType::Weak.to_string(), "weak");
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

    #[test]
    fn test_aggregate_unreachable_gates_excluded() {
        let gates = vec![
            GateInfo {
                gate_id: "reachable".to_string(),
                address: "local".to_string(),
                is_local: true,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo {
                    gpus: vec![],
                    ram_gb: 16,
                    cpu_cores: 8,
                },
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "unreachable".to_string(),
                address: "192.168.1.99".to_string(),
                is_local: false,
                primals: vec![],
                compute: ComputeInfo {
                    gpus: vec![GpuInfo {
                        name: "RTX 4090".to_string(),
                        vram_mb: 24576,
                        gate_id: "unreachable".to_string(),
                    }],
                    ram_gb: 64,
                    cpu_cores: 32,
                },
                models: vec!["BigModel".to_string()],
                load: 0.0,
                reachable: false,
                bond_type: BondType::Covalent,
            },
        ];

        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_gpus, 0, "unreachable gate GPUs excluded");
        assert_eq!(caps.total_ram_gb, 16, "only reachable gate RAM");
        assert_eq!(caps.models.len(), 0, "unreachable models excluded");
    }

    #[test]
    fn test_aggregate_ionic_bond_type() {
        let gates = vec![GateInfo {
            gate_id: "ionic".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Ionic,
        }];

        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert!(caps.capabilities.is_empty());
    }

    #[test]
    fn test_plasmodium_default() {
        let _p = Plasmodium::default();
        // Just verify construction succeeds
    }

    #[test]
    fn test_plasmodium_new() {
        let _p = Plasmodium::new();
        // Just verify construction succeeds
    }

    #[test]
    fn test_aggregate_capabilities_sorted() {
        let gates = vec![
            GateInfo {
                gate_id: "z-gate".to_string(),
                address: "local".to_string(),
                is_local: true,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "a-gate".to_string(),
                address: "local".to_string(),
                is_local: true,
                primals: vec![PrimalStatus {
                    name: "songbird".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert!(!caps.capabilities.is_empty());
        let mut sorted = caps.capabilities.clone();
        sorted.sort();
        assert_eq!(caps.capabilities, sorted, "capabilities should be sorted");
    }

    #[test]
    fn test_aggregate_capabilities_unhealthy_primal_excluded() {
        let gates = vec![GateInfo {
            gate_id: "gate".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![PrimalStatus {
                name: "beardog".to_string(),
                healthy: false,
                version: None,
            }],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        }];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert!(
            caps.capabilities.is_empty(),
            "unhealthy primals don't contribute"
        );
    }

    #[test]
    fn test_aggregate_capabilities_model_availability() {
        let gates = vec![GateInfo {
            gate_id: "gate1".to_string(),
            address: "local".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec!["model-a".to_string(), "model-b".to_string()],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        }];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_models, 2);
        assert_eq!(caps.models.len(), 2);
    }

    #[test]
    fn test_aggregate_capabilities_same_model_multiple_gates() {
        let gates = vec![
            GateInfo {
                gate_id: "gate1".to_string(),
                address: "local".to_string(),
                is_local: true,
                primals: vec![],
                compute: ComputeInfo::default(),
                models: vec!["shared-model".to_string()],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "gate2".to_string(),
                address: "remote".to_string(),
                is_local: false,
                primals: vec![],
                compute: ComputeInfo::default(),
                models: vec!["shared-model".to_string()],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_models, 1);
        assert_eq!(caps.models[0].gates.len(), 2);
    }

    #[tokio::test]
    async fn test_query_collective_no_peers() {
        let p = Plasmodium::new();
        let result = p.query_collective().await;
        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(!state.gates.is_empty(), "at least local gate");
        assert!(!state.family_id.is_empty());
        assert!(!state.snapshot_at.is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_query_collective_merges_plasmodium_peers_env() {
        use biomeos_test_utils::TestEnvGuard;

        let _guard = TestEnvGuard::set(
            "PLASMODIUM_PEERS",
            "peer-a@127.0.0.1:59997,peer-b@host-only",
        );
        let p = Plasmodium::new();
        let result = p.query_collective().await;
        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(
            state
                .gates
                .iter()
                .any(|g| g.gate_id == "peer-a" || g.gate_id == "peer-b"),
            "expected env-listed peers to appear in collective state: {:?}",
            state.gates.iter().map(|g| &g.gate_id).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_plasmodium_peers_bare_hostname_branch() {
        use biomeos_test_utils::TestEnvGuard;
        let _guard = TestEnvGuard::set("PLASMODIUM_PEERS", "bare-hostname-only-token-unique-8821");
        let p = Plasmodium::new();
        let state = p.query_collective().await.expect("collective");
        assert!(
            state
                .gates
                .iter()
                .any(|g| g.gate_id == "bare-hostname-only-token-unique-8821"),
            "bare entry should use same token for id and address"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_plasmodium_peers_skips_empty_segments() {
        use biomeos_test_utils::TestEnvGuard;
        let _guard =
            TestEnvGuard::set("PLASMODIUM_PEERS", " ,  dup@127.0.0.1:1 , dup@127.0.0.1:2 ");
        let p = Plasmodium::new();
        let state = p.query_collective().await.expect("collective");
        assert!(
            state.gates.iter().filter(|g| g.gate_id == "dup").count() <= 1,
            "duplicate node ids from env should be deduped"
        );
    }

    #[test]
    fn test_collective_capabilities_default_empty() {
        let c = CollectiveCapabilities {
            total_gpus: 0,
            gpus: vec![],
            total_ram_gb: 0,
            total_models: 0,
            models: vec![],
            capabilities: vec![],
        };
        assert!(c.capabilities.is_empty());
    }

    #[test]
    fn test_gate_info_reachable_field() {
        let g = GateInfo {
            gate_id: "g".to_string(),
            address: "a".to_string(),
            is_local: false,
            primals: vec![],
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: false,
            bond_type: BondType::Covalent,
        };
        assert!(!g.reachable);
    }

    #[test]
    fn test_aggregate_gpu_dedup_by_gate() {
        let gates = vec![
            GateInfo {
                gate_id: "g1".to_string(),
                address: "l".to_string(),
                is_local: true,
                primals: vec![],
                compute: ComputeInfo {
                    gpus: vec![GpuInfo {
                        name: "GPU".to_string(),
                        vram_mb: 1000,
                        gate_id: "g1".to_string(),
                    }],
                    ram_gb: 8,
                    cpu_cores: 4,
                },
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "g2".to_string(),
                address: "r".to_string(),
                is_local: false,
                primals: vec![],
                compute: ComputeInfo {
                    gpus: vec![GpuInfo {
                        name: "GPU2".to_string(),
                        vram_mb: 2000,
                        gate_id: "g2".to_string(),
                    }],
                    ram_gb: 16,
                    cpu_cores: 8,
                },
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_gpus, 2);
        assert_eq!(caps.gpus.len(), 2);
    }

    #[test]
    fn test_primal_status_version_some() {
        let p = PrimalStatus {
            name: "x".to_string(),
            healthy: true,
            version: Some("2.0".to_string()),
        };
        assert_eq!(p.version.as_deref(), Some("2.0"));
    }

    #[test]
    fn test_compute_info_default_ram() {
        let c = ComputeInfo::default();
        assert_eq!(c.ram_gb, 0);
        assert!(c.gpus.is_empty());
    }

    #[test]
    fn test_model_availability_struct() {
        let m = ModelAvailability {
            model_id: "m".to_string(),
            size_bytes: 0,
            format: String::new(),
            gates: vec!["a".to_string()],
        };
        assert_eq!(m.gates.len(), 1);
    }

    #[test]
    fn test_plasmodium_state_fields() {
        let s = PlasmodiumState {
            gates: vec![],
            snapshot_at: "t".to_string(),
            family_id: "fam".to_string(),
            collective: CollectiveCapabilities {
                total_gpus: 0,
                gpus: vec![],
                total_ram_gb: 0,
                total_models: 0,
                models: vec![],
                capabilities: vec![],
            },
        };
        assert_eq!(s.family_id, "fam");
    }

    #[test]
    fn test_aggregate_capabilities_multiple_gates_same_primal_name() {
        let gates = vec![
            GateInfo {
                gate_id: "g1".to_string(),
                address: "l".to_string(),
                is_local: true,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "g2".to_string(),
                address: "r".to_string(),
                is_local: false,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: Some("2".to_string()),
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert!(
            caps.capabilities.contains(&"crypto".to_string()),
            "beardog should map to at least crypto: {:?}",
            caps.capabilities
        );
    }

    #[test]
    fn test_aggregate_capabilities_models_merge_duplicate_ids() {
        let gates = vec![
            GateInfo {
                gate_id: "a".to_string(),
                address: "l".to_string(),
                is_local: true,
                primals: vec![],
                compute: ComputeInfo::default(),
                models: vec!["m1".to_string()],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "b".to_string(),
                address: "r".to_string(),
                is_local: false,
                primals: vec![],
                compute: ComputeInfo::default(),
                models: vec!["m1".to_string()],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_models, 1);
        assert_eq!(caps.models[0].gates.len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_query_collective_family_id_from_family_id_env() {
        use biomeos_test_utils::TestEnvGuard;
        let _g = TestEnvGuard::set("FAMILY_ID", "plasmo-env-family-42");
        let p = Plasmodium::new();
        let state = p.query_collective().await.expect("collective");
        assert_eq!(state.family_id, "plasmo-env-family-42");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_query_collective_node_family_id_fallback_env() {
        use biomeos_test_utils::TestEnvGuard;
        let _g1 = TestEnvGuard::remove("FAMILY_ID");
        let _g2 = TestEnvGuard::set("NODE_FAMILY_ID", "node-fam-99");
        let p = Plasmodium::new();
        let state = p.query_collective().await.expect("collective");
        assert_eq!(state.family_id, "node-fam-99");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_query_collective_gate_id_from_gate_id_env() {
        use biomeos_test_utils::TestEnvGuard;
        let _g = TestEnvGuard::set("GATE_ID", "gate-env-unique-771");
        let p = Plasmodium::new();
        let state = p.query_collective().await.expect("collective");
        let local = state.gates.iter().find(|g| g.is_local).expect("local gate");
        assert_eq!(local.gate_id, "gate-env-unique-771");
    }

    #[test]
    fn test_aggregate_capabilities_duplicate_capability_names_sorted() {
        let gates = vec![
            GateInfo {
                gate_id: "g1".to_string(),
                address: "l".to_string(),
                is_local: true,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Covalent,
            },
            GateInfo {
                gate_id: "g2".to_string(),
                address: "r".to_string(),
                is_local: false,
                primals: vec![PrimalStatus {
                    name: "beardog".to_string(),
                    healthy: true,
                    version: None,
                }],
                compute: ComputeInfo::default(),
                models: vec![],
                load: 0.0,
                reachable: true,
                bond_type: BondType::Weak,
            },
        ];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        let mut sorted = caps.capabilities.clone();
        sorted.sort();
        assert_eq!(caps.capabilities, sorted);
    }

    #[test]
    fn test_aggregate_capabilities_zero_ram_multiple_gpus() {
        let gates = vec![GateInfo {
            gate_id: "gpu-only".to_string(),
            address: "l".to_string(),
            is_local: true,
            primals: vec![],
            compute: ComputeInfo {
                gpus: vec![
                    GpuInfo {
                        name: "A".to_string(),
                        vram_mb: 1000,
                        gate_id: "gpu-only".to_string(),
                    },
                    GpuInfo {
                        name: "B".to_string(),
                        vram_mb: 2000,
                        gate_id: "gpu-only".to_string(),
                    },
                ],
                ram_gb: 0,
                cpu_cores: 0,
            },
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Metallic,
        }];
        let caps = Plasmodium::aggregate_capabilities(&gates);
        assert_eq!(caps.total_gpus, 2);
        assert_eq!(caps.total_ram_gb, 0);
    }

    #[test]
    fn test_plasmodium_state_snapshot_rfc3339() {
        let s = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2025-01-01T00:00:00+00:00".to_string(),
            family_id: "f".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        assert!(s.snapshot_at.contains('T'));
    }
}
