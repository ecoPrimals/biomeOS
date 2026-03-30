// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

mod aggregate;
mod local_gate;
mod peers;
mod remote;
mod system;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use anyhow::Result;
use tracing::{info, warn};

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
    #[must_use] 
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
                    gates.push(types::GateInfo {
                        gate_id: peer.node_id,
                        address: peer.address,
                        is_local: false,
                        primals: vec![],
                        compute: types::ComputeInfo::default(),
                        models: vec![],
                        load: 0.0,
                        reachable: false,
                        bond_type: types::BondType::Covalent,
                    });
                }
            }
        }

        // 4. Aggregate collective capabilities
        let collective = Self::aggregate_capabilities(&gates);

        Ok(types::PlasmodiumState {
            gates,
            snapshot_at: chrono::Utc::now().to_rfc3339(),
            family_id: self.family_id.clone(),
            collective,
        })
    }
}
