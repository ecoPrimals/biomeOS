// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_type_display() {
        assert_eq!(format!("{}", BondType::Covalent), "covalent");
        assert_eq!(format!("{}", BondType::Ionic), "ionic");
        assert_eq!(format!("{}", BondType::Metallic), "metallic");
        assert_eq!(format!("{}", BondType::Weak), "weak");
    }

    #[test]
    fn test_bond_type_serialization() {
        for bond in [
            BondType::Covalent,
            BondType::Ionic,
            BondType::Metallic,
            BondType::Weak,
        ] {
            let json = serde_json::to_value(&bond).expect("serialize");
            let restored: BondType = serde_json::from_value(json).expect("deserialize");
            assert_eq!(bond, restored);
        }
    }

    #[test]
    fn test_gpu_info_display() {
        let gpu = GpuInfo {
            name: "RTX 4090".to_string(),
            vram_mb: 24 * 1024, // 24 GB
            gate_id: "gate-1".to_string(),
        };
        assert_eq!(format!("{}", gpu), "RTX 4090 (24 GB)");
    }

    #[test]
    fn test_compute_info_default() {
        let info = ComputeInfo::default();
        assert!(info.gpus.is_empty());
        assert_eq!(info.ram_gb, 0);
        assert_eq!(info.cpu_cores, 0);
    }

    #[test]
    fn test_primal_status_serialization() {
        let status = PrimalStatus {
            name: "squirrel".to_string(),
            healthy: true,
            version: Some("1.0.0".to_string()),
        };
        let json = serde_json::to_value(&status).expect("serialize");
        let restored: PrimalStatus = serde_json::from_value(json).expect("deserialize");
        assert_eq!(status.name, restored.name);
        assert_eq!(status.healthy, restored.healthy);
        assert_eq!(status.version, restored.version);
    }

    #[test]
    fn test_model_availability_serialization() {
        let avail = ModelAvailability {
            model_id: "TinyLlama/TinyLlama-1.1B".to_string(),
            size_bytes: 1_000_000_000,
            format: "gguf".to_string(),
            gates: vec!["gate-1".to_string(), "gate-2".to_string()],
        };
        let json = serde_json::to_value(&avail).expect("serialize");
        let restored: ModelAvailability = serde_json::from_value(json).expect("deserialize");
        assert_eq!(avail.model_id, restored.model_id);
        assert_eq!(avail.gates.len(), 2);
    }

    #[test]
    fn test_collective_capabilities_default() {
        let cap = CollectiveCapabilities::default();
        assert_eq!(cap.total_gpus, 0);
        assert_eq!(cap.total_ram_gb, 0);
        assert_eq!(cap.total_models, 0);
        assert!(cap.gpus.is_empty());
        assert!(cap.models.is_empty());
    }

    #[test]
    fn test_plasmodium_state_serialization() {
        let state = PlasmodiumState {
            gates: vec![],
            snapshot_at: "2024-01-01T00:00:00Z".to_string(),
            family_id: "family-1".to_string(),
            collective: CollectiveCapabilities::default(),
        };
        let json = serde_json::to_value(&state).expect("serialize");
        let restored: PlasmodiumState = serde_json::from_value(json).expect("deserialize");
        assert_eq!(state.family_id, restored.family_id);
        assert_eq!(state.gates.len(), restored.gates.len());
    }
}
