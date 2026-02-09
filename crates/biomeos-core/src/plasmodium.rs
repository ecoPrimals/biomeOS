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
//! For each peer: query NUCLEUS status via Songbird TCP
//!     |
//!     v
//! Aggregate into PlasmodiumState (the collective view)
//! ```
//!
//! See `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` for the full specification.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;
use crate::model_cache::ModelCache;

// ─── Core Types ──────────────────────────────────────────────────────────

/// Bond type between gates (from NUCLEUS_BONDING_MODEL)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BondType {
    /// Shared family seed, genetic trust, zero metering
    Covalent,
    /// Contract-based, metered
    Ionic,
    /// Electron sea, sub-specialization
    Metallic,
    /// Minimal interaction, pre-trust
    Weak,
}

impl std::fmt::Display for BondType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondType::Covalent => write!(f, "covalent"),
            BondType::Ionic => write!(f, "ionic"),
            BondType::Metallic => write!(f, "metallic"),
            BondType::Weak => write!(f, "weak"),
        }
    }
}

/// Status of a primal on a gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub name: String,
    pub healthy: bool,
    pub version: Option<String>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub gate_id: String,
}

impl std::fmt::Display for GpuInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} GB)", self.name, self.vram_mb / 1024)
    }
}

/// Compute information for a gate
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeInfo {
    pub gpus: Vec<GpuInfo>,
    pub ram_gb: u64,
    pub cpu_cores: usize,
}

/// Model availability across the collective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAvailability {
    pub model_id: String,
    pub size_bytes: u64,
    pub format: String,
    pub gates: Vec<String>,
}

/// Per-gate status within the collective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateInfo {
    /// Gate identifier (hostname)
    pub gate_id: String,

    /// Network address
    pub address: String,

    /// Is this the local gate?
    pub is_local: bool,

    /// Running primals
    pub primals: Vec<PrimalStatus>,

    /// Compute capabilities
    pub compute: ComputeInfo,

    /// Cached model IDs
    pub models: Vec<String>,

    /// Current load (0.0 - 1.0)
    pub load: f64,

    /// Whether this gate is reachable
    pub reachable: bool,

    /// Bond type
    pub bond_type: BondType,
}

/// Collective capabilities (union across all gates)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectiveCapabilities {
    pub total_gpus: usize,
    pub gpus: Vec<GpuInfo>,
    pub total_ram_gb: u64,
    pub total_models: usize,
    pub models: Vec<ModelAvailability>,
    pub capabilities: Vec<String>,
}

/// The aggregate snapshot of the collective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasmodiumState {
    /// All gates in the collective
    pub gates: Vec<GateInfo>,

    /// Timestamp of this snapshot
    pub snapshot_at: String,

    /// Family ID binding this plasmodium
    pub family_id: String,

    /// Aggregated capabilities
    pub collective: CollectiveCapabilities,
}

// ─── Plasmodium Query Engine ─────────────────────────────────────────────

/// Plasmodium collective query engine
///
/// Queries local and remote NUCLEUS instances to build a collective view.
pub struct Plasmodium {
    family_id: String,
    local_gate_id: String,
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
    /// 3. For each peer, query their NUCLEUS status
    /// 4. Aggregate into PlasmodiumState
    pub async fn query_collective(&self) -> Result<PlasmodiumState> {
        info!("Querying plasmodium collective for family '{}'", self.family_id);

        let mut gates = Vec::new();

        // 1. Gather local gate info
        let local = self.query_local_gate().await?;
        gates.push(local);

        // 2. Query Songbird mesh for peers
        let peers = self.discover_peers().await;

        // 3. Query each peer
        for peer in peers {
            match self.query_remote_gate(&peer.address, &peer.node_id).await {
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
    async fn query_local_gate(&self) -> Result<GateInfo> {
        let mut primals = Vec::new();

        // Check BearDog
        if let Ok(client) = AtomicClient::discover("beardog").await {
            let health = self.check_primal_health(&client, "beardog").await;
            primals.push(health);
        }

        // Check Songbird
        if let Ok(client) = AtomicClient::discover("songbird").await {
            let health = self.check_primal_health(&client, "songbird").await;
            primals.push(health);
        }

        // Check Toadstool
        if let Ok(client) = AtomicClient::discover("toadstool").await {
            let health = self.check_primal_health(&client, "toadstool").await;
            primals.push(health);
        }

        // Check NestGate
        if let Ok(client) = AtomicClient::discover("nestgate").await {
            let health = self.check_primal_health(&client, "nestgate").await;
            primals.push(health);
        }

        // Check Squirrel
        if let Ok(client) = AtomicClient::discover("squirrel").await {
            let health = self.check_primal_health(&client, "squirrel").await;
            primals.push(health);
        }

        // Get compute info
        let compute = self.query_local_compute().await;

        // Get model cache
        let models = self.query_local_models().await;

        // Get system load
        let load = Self::get_system_load();

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
    async fn check_primal_health(&self, client: &AtomicClient, name: &str) -> PrimalStatus {
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

    /// Query local compute capabilities (GPU, RAM)
    async fn query_local_compute(&self) -> ComputeInfo {
        let mut gpus = Vec::new();

        // Try nvidia-smi for NVIDIA GPUs
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split(", ").collect();
                    if parts.len() >= 2 {
                        gpus.push(GpuInfo {
                            name: parts[0].trim().to_string(),
                            vram_mb: parts[1].trim().parse().unwrap_or(0),
                            gate_id: self.local_gate_id.clone(),
                        });
                    }
                }
            }
        }

        // Get RAM
        let ram_gb = Self::get_system_ram_gb();

        // Get CPU cores
        let cpu_cores = num_cpus();

        ComputeInfo {
            gpus,
            ram_gb,
            cpu_cores,
        }
    }

    /// Query local model cache
    async fn query_local_models(&self) -> Vec<String> {
        match ModelCache::new().await {
            Ok(cache) => cache
                .list_models()
                .iter()
                .map(|m| m.model_id.clone())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Discover peers via Songbird mesh
    async fn discover_peers(&self) -> Vec<PeerInfo> {
        let client = match AtomicClient::discover("songbird").await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Query mesh.peers
        let result = match client.call("mesh.peers", json!({})).await {
            Ok(r) => r,
            Err(e) => {
                debug!("mesh.peers failed: {}", e);
                return vec![];
            }
        };

        let peers_array = result
            .get("peers")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();

        let mut peers = Vec::new();
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

        // Also try known gates from environment or config
        if peers.is_empty() {
            // Fallback: check PLASMODIUM_PEERS env var for manual peer list
            if let Ok(peer_list) = std::env::var("PLASMODIUM_PEERS") {
                for peer_str in peer_list.split(',') {
                    let parts: Vec<&str> = peer_str.trim().split('@').collect();
                    if parts.len() == 2 {
                        peers.push(PeerInfo {
                            node_id: parts[0].to_string(),
                            address: parts[1].to_string(),
                        });
                    } else if parts.len() == 1 {
                        // Just an IP/hostname
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

    /// Query a remote gate's NUCLEUS status via Songbird TCP
    async fn query_remote_gate(&self, address: &str, node_id: &str) -> Result<GateInfo> {
        // Parse host:port (default Songbird TCP port is 3492)
        let (host, port) = if let Some(idx) = address.rfind(':') {
            let h = &address[..idx];
            let p = address[idx + 1..].parse::<u16>().unwrap_or(3492);
            (h.to_string(), p)
        } else {
            (address.to_string(), 3492)
        };

        let client = AtomicClient::tcp(&host, port);

        // Query health
        let health_result: Result<Value> = client.call("health", json!({})).await;
        let reachable = health_result.is_ok();

        if !reachable {
            anyhow::bail!("Gate {} not reachable at {}:{}", node_id, host, port);
        }

        // Query remote primals
        let primals = self.query_remote_primals(&client).await;

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
    async fn query_remote_primals(&self, client: &AtomicClient) -> Vec<PrimalStatus> {
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

        // Fallback: just check if Songbird is healthy (we already connected)
        if primals.is_empty() {
            primals.push(PrimalStatus {
                name: "songbird".to_string(),
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

            // Capabilities from primals
            for primal in &gate.primals {
                if primal.healthy {
                    match primal.name.as_str() {
                        "beardog" => {
                            capability_set.insert("crypto".to_string());
                            capability_set.insert("security".to_string());
                        }
                        "songbird" => {
                            capability_set.insert("discovery".to_string());
                            capability_set.insert("network".to_string());
                        }
                        "toadstool" => {
                            capability_set.insert("compute".to_string());
                        }
                        "nestgate" => {
                            capability_set.insert("storage".to_string());
                        }
                        "squirrel" => {
                            capability_set.insert("ai".to_string());
                        }
                        _ => {}
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

    // ─── System info helpers ─────────────────────────────────────────

    fn get_system_load() -> f64 {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| {
                s.split_whitespace()
                    .next()
                    .and_then(|load| load.parse::<f64>().ok())
            })
            .map(|load_1m| {
                let cores = num_cpus() as f64;
                if cores > 0.0 {
                    (load_1m / cores).min(1.0)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }

    fn get_system_ram_gb() -> u64 {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| {
                        l.split_whitespace()
                            .nth(1)
                            .and_then(|kb| kb.parse::<u64>().ok())
                    })
            })
            .map(|kb| kb / 1_048_576) // KB to GB
            .unwrap_or(0)
    }
}

/// Peer discovered via mesh
#[derive(Debug, Clone)]
struct PeerInfo {
    node_id: String,
    address: String,
}

/// Helper: get CPU core count without external dependency
fn num_cpus() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.lines().filter(|l| l.starts_with("processor")).count())
        .unwrap_or(1)
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
                models: vec![
                    "TinyLlama/1.1B".to_string(),
                    "Mistral-7B".to_string(),
                ],
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
        let ram = Plasmodium::get_system_ram_gb();
        assert!(ram > 0);
    }

    #[test]
    fn test_num_cpus() {
        assert!(num_cpus() > 0);
    }
}
