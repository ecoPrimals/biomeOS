// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS atomic type definitions — Tower, Node, Nest compositions.
//!
//! Absorbed from `groundSpring/metalForge/forge/src/atomic.rs` and abstracted
//! for the biomeOS shared type system. All springs share this vocabulary for
//! describing atomic compositions and capability requirements.
//!
//! | Atomic | Required Capabilities | Provides |
//! |--------|----------------------|----------|
//! | **Tower** | `SecureIpc` | Encrypted IPC foundation |
//! | **Node** | Tower + `ComputeDispatch` | + GPU compute dispatch |
//! | **Nest** | Tower + `DataStorage` | + Data storage & provenance |
//! | **Full NUCLEUS** | All capabilities | Complete ecosystem |
//!
//! Each atomic declares the capabilities it provides and discovers
//! providers at runtime via biomeOS `topology.metrics`. No hardcoded
//! primal names — only capability semantics.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Primal health status within an atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalHealth {
    /// Provider is responding to health checks.
    Healthy,
    /// Provider is present but degraded (slow, partial capability).
    Degraded,
    /// Provider is not responding.
    Unavailable,
    /// Capability is not required for this atomic type.
    NotRequired,
}

/// Capabilities provided by a NUCLEUS atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomicCapability {
    /// Encrypted inter-primal communication (IPC foundation).
    SecureIpc,
    /// GPU/CPU compute dispatch.
    ComputeDispatch,
    /// NPU inference (int8 quantized).
    NpuInference,
    /// Data storage and provenance.
    DataStorage,
    /// Live data pipelines (NCBI, NOAA, IRIS).
    LiveData,
    /// AI/ML inference.
    AiInference,
    /// Cross-substrate pipeline orchestration.
    PipelineOrchestration,
}

/// Atomic composition tier within the NUCLEUS hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomicTier {
    /// Security + discovery foundation.
    Tower,
    /// Tower + compute dispatch.
    Node,
    /// Tower + data storage.
    Nest,
    /// All capabilities — complete ecosystem.
    Full,
}

impl AtomicTier {
    /// The capabilities required for this tier.
    #[must_use]
    pub fn required_capabilities(&self) -> Vec<AtomicCapability> {
        match self {
            Self::Tower => vec![AtomicCapability::SecureIpc],
            Self::Node => vec![
                AtomicCapability::SecureIpc,
                AtomicCapability::ComputeDispatch,
            ],
            Self::Nest => vec![AtomicCapability::SecureIpc, AtomicCapability::DataStorage],
            Self::Full => vec![
                AtomicCapability::SecureIpc,
                AtomicCapability::ComputeDispatch,
                AtomicCapability::DataStorage,
                AtomicCapability::AiInference,
            ],
        }
    }
}

/// Runtime-discovered capability provider health map.
///
/// Keys are capability identifiers (e.g. `"crypto"`, `"discovery"`),
/// values are the health status of the provider for that capability.
/// Populated at runtime via `topology.metrics`, never hardcoded.
pub type ProviderHealthMap = BTreeMap<String, PrimalHealth>;

/// Tower Atomic — secure IPC foundation.
///
/// The foundational atomic that all others build upon. Provides
/// secure inter-primal communication. Discovered at runtime via
/// capability probing, not by naming specific primals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerAtomic {
    /// Node identifier (e.g. "eastgate", "biomegate").
    pub node_id: String,
    /// Runtime-discovered capability providers and their health.
    pub providers: ProviderHealthMap,
    /// biomeOS Neural API socket path (discovered at runtime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<String>,
}

/// Node Atomic — Tower + compute dispatch.
///
/// Extends Tower with compute capabilities. Springs that need GPU/CPU
/// dispatch build on this tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// Compute dispatch provider health.
    pub compute: PrimalHealth,
}

/// Nest Atomic — Tower + data storage.
///
/// Extends Tower with data capabilities. Provides storage,
/// provenance, and live data pipeline access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// Data storage provider health.
    pub storage: PrimalHealth,
    /// Available data capabilities.
    pub data_capabilities: Vec<AtomicCapability>,
}

/// Full NUCLEUS — all capabilities for complete ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullNucleus {
    /// Node atomic (Tower + compute).
    pub node: NodeAtomic,
    /// Data storage provider health.
    pub storage: PrimalHealth,
    /// AI/ML inference provider health.
    pub inference: PrimalHealth,
}

impl TowerAtomic {
    /// Create a Tower atomic for a given node.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            providers: ProviderHealthMap::new(),
            socket_path: None,
        }
    }

    /// Set health for a capability provider discovered at runtime.
    pub fn set_provider_health(&mut self, capability: &str, health: PrimalHealth) {
        self.providers.insert(capability.to_string(), health);
    }

    /// Check if the Tower has healthy secure IPC (all required providers up).
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self.providers.is_empty()
            && self
                .providers
                .values()
                .all(|h| matches!(h, PrimalHealth::Healthy))
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        if self.is_healthy() {
            vec![AtomicCapability::SecureIpc]
        } else {
            Vec::new()
        }
    }
}

impl NodeAtomic {
    /// Create a Node atomic.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            tower: TowerAtomic::new(node_id),
            compute: PrimalHealth::Unavailable,
        }
    }

    /// Check if compute dispatch is available.
    #[must_use]
    pub const fn can_compute(&self) -> bool {
        matches!(self.compute, PrimalHealth::Healthy | PrimalHealth::Degraded)
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.tower.capabilities();
        if self.can_compute() {
            caps.push(AtomicCapability::ComputeDispatch);
        }
        caps
    }
}

impl NestAtomic {
    /// Create a Nest atomic.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            tower: TowerAtomic::new(node_id),
            storage: PrimalHealth::Unavailable,
            data_capabilities: Vec::new(),
        }
    }

    /// Check if data storage is available.
    #[must_use]
    pub const fn can_store(&self) -> bool {
        matches!(self.storage, PrimalHealth::Healthy)
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.tower.capabilities();
        if self.can_store() {
            caps.push(AtomicCapability::DataStorage);
            for dc in &self.data_capabilities {
                if !caps.contains(dc) {
                    caps.push(*dc);
                }
            }
        }
        caps
    }
}

impl FullNucleus {
    /// Check if all capabilities are healthy.
    #[must_use]
    pub fn is_fully_healthy(&self) -> bool {
        self.node.tower.is_healthy()
            && self.node.can_compute()
            && matches!(self.storage, PrimalHealth::Healthy)
            && matches!(self.inference, PrimalHealth::Healthy)
    }

    /// List all capabilities of the full NUCLEUS.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.node.capabilities();
        if matches!(self.storage, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::DataStorage);
            caps.push(AtomicCapability::LiveData);
        }
        if matches!(self.inference, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::AiInference);
        }
        caps
    }

    /// The sovereign degradation level — what's available when parts fail.
    #[must_use]
    pub fn degradation_level(&self) -> &'static str {
        if self.is_fully_healthy() {
            "Full NUCLEUS"
        } else if self.node.can_compute() && matches!(self.storage, PrimalHealth::Healthy) {
            "Node + Nest (no AI)"
        } else if self.node.can_compute() {
            "Node only (no storage)"
        } else if self.node.tower.is_healthy() {
            "Tower only (no compute)"
        } else {
            "Sovereign (local only)"
        }
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_tower(node_id: &str) -> TowerAtomic {
        let mut tower = TowerAtomic::new(node_id);
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("discovery", PrimalHealth::Healthy);
        tower
    }

    #[test]
    fn tower_unhealthy_by_default() {
        let tower = TowerAtomic::new("eastgate");
        assert!(!tower.is_healthy());
        assert!(tower.capabilities().is_empty());
    }

    #[test]
    fn tower_healthy_when_providers_respond() {
        let tower = healthy_tower("eastgate");
        assert!(tower.is_healthy());
        assert!(tower.capabilities().contains(&AtomicCapability::SecureIpc));
    }

    #[test]
    fn tower_unhealthy_if_any_provider_down() {
        let mut tower = TowerAtomic::new("eastgate");
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("discovery", PrimalHealth::Unavailable);
        assert!(!tower.is_healthy());
    }

    #[test]
    fn node_has_compute_when_provider_healthy() {
        let mut node = NodeAtomic::new("eastgate");
        node.compute = PrimalHealth::Healthy;
        assert!(node.can_compute());
        assert!(
            node.capabilities()
                .contains(&AtomicCapability::ComputeDispatch)
        );
    }

    #[test]
    fn node_degraded_still_computes() {
        let mut node = NodeAtomic::new("eastgate");
        node.compute = PrimalHealth::Degraded;
        assert!(node.can_compute());
    }

    #[test]
    fn nest_can_store_when_provider_healthy() {
        let mut nest = NestAtomic::new("westgate");
        nest.storage = PrimalHealth::Healthy;
        assert!(nest.can_store());
        assert!(nest.capabilities().contains(&AtomicCapability::DataStorage));
    }

    #[test]
    fn nest_cannot_store_when_unavailable() {
        let nest = NestAtomic::new("westgate");
        assert!(!nest.can_store());
        assert!(!nest.capabilities().contains(&AtomicCapability::DataStorage));
    }

    #[test]
    fn full_nucleus_degradation_levels() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::new("gate-primary"),
            storage: PrimalHealth::Unavailable,
            inference: PrimalHealth::Unavailable,
        };

        assert_eq!(nucleus.degradation_level(), "Sovereign (local only)");

        nucleus.node.tower = healthy_tower("gate-primary");
        assert_eq!(nucleus.degradation_level(), "Tower only (no compute)");

        nucleus.node.compute = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node only (no storage)");

        nucleus.storage = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node + Nest (no AI)");

        nucleus.inference = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Full NUCLEUS");
        assert!(nucleus.is_fully_healthy());
    }

    #[test]
    fn full_nucleus_capabilities() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::new("gate-primary"),
            storage: PrimalHealth::Healthy,
            inference: PrimalHealth::Healthy,
        };
        nucleus.node.tower = healthy_tower("gate-primary");
        nucleus.node.compute = PrimalHealth::Healthy;

        let caps = nucleus.capabilities();
        assert!(caps.contains(&AtomicCapability::SecureIpc));
        assert!(caps.contains(&AtomicCapability::ComputeDispatch));
        assert!(caps.contains(&AtomicCapability::DataStorage));
        assert!(caps.contains(&AtomicCapability::AiInference));
    }

    #[test]
    fn atomic_tier_required_capabilities() {
        assert_eq!(
            AtomicTier::Tower.required_capabilities(),
            vec![AtomicCapability::SecureIpc]
        );
        assert_eq!(
            AtomicTier::Node.required_capabilities(),
            vec![
                AtomicCapability::SecureIpc,
                AtomicCapability::ComputeDispatch
            ]
        );
        assert_eq!(
            AtomicTier::Nest.required_capabilities(),
            vec![AtomicCapability::SecureIpc, AtomicCapability::DataStorage]
        );
        assert_eq!(AtomicTier::Full.required_capabilities().len(), 4);
    }

    #[test]
    fn serde_roundtrip_tower() {
        let tower = healthy_tower("testgate");
        let json = serde_json::to_string(&tower).unwrap();
        let parsed: TowerAtomic = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.node_id, "testgate");
        assert!(parsed.is_healthy());
    }

    #[test]
    fn serde_roundtrip_primal_health() {
        let json = serde_json::to_string(&PrimalHealth::Degraded).unwrap();
        assert_eq!(json, "\"degraded\"");
        let parsed: PrimalHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PrimalHealth::Degraded);
    }

    #[test]
    fn serde_roundtrip_atomic_capability() {
        let json = serde_json::to_string(&AtomicCapability::ComputeDispatch).unwrap();
        assert_eq!(json, "\"compute_dispatch\"");
        let parsed: AtomicCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AtomicCapability::ComputeDispatch);
    }

    #[test]
    fn provider_health_map_is_dynamic() {
        let mut tower = TowerAtomic::new("testgate");
        assert!(tower.providers.is_empty());
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("mesh", PrimalHealth::Degraded);
        assert_eq!(tower.providers.len(), 2);
        assert_eq!(tower.providers["crypto"], PrimalHealth::Healthy);
        assert_eq!(tower.providers["mesh"], PrimalHealth::Degraded);
    }
}
