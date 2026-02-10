//! Plasmodium core types — pure data structures for collective coordination.

use serde::{Deserialize, Serialize};

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
    /// Primal name (discovered at runtime)
    pub name: String,
    /// Whether the primal responded to health check
    pub healthy: bool,
    /// Version if reported
    pub version: Option<String>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU model name
    pub name: String,
    /// VRAM in megabytes
    pub vram_mb: u64,
    /// Gate that hosts this GPU
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
    /// Available GPUs
    pub gpus: Vec<GpuInfo>,
    /// Total system RAM in gigabytes
    pub ram_gb: u64,
    /// CPU core count
    pub cpu_cores: usize,
}

/// Model availability across the collective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAvailability {
    /// Model identifier
    pub model_id: String,
    /// Size in bytes (0 if unknown)
    pub size_bytes: u64,
    /// Model format string
    pub format: String,
    /// Gates that have this model cached
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
    /// Total GPU count
    pub total_gpus: usize,
    /// All GPUs across gates
    pub gpus: Vec<GpuInfo>,
    /// Total RAM across gates in gigabytes
    pub total_ram_gb: u64,
    /// Total unique model count
    pub total_models: usize,
    /// Model availability details
    pub models: Vec<ModelAvailability>,
    /// Union of all capabilities
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

/// Peer discovered via mesh or `PLASMODIUM_PEERS` env var
#[derive(Debug, Clone)]
pub(crate) struct PeerInfo {
    /// Peer node identifier
    pub node_id: String,
    /// Peer address (host:port or ssh:user@host)
    pub address: String,
}
