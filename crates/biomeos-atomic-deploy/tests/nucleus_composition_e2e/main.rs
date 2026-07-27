// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

//! Multi-primal NUCLEUS composition end-to-end tests
//!
//! Validates that biomeOS can correctly parse, order, and execute graphs with
//! 5+ primals — the threshold required for full NUCLEUS composition.
//! This unblocks chimera compositions and garden deployments.
//!
//! Three levels:
//! 1. Parse the canonical `nucleus_complete.toml` and verify all 13 nodes load
//! 2. Verify topological sort produces correct phased execution order
//! 3. Execute a synthetic NUCLEUS-shaped graph end-to-end, proving the executor
//!    handles multi-phase parallel composition correctly

mod cross_gate;
mod execution;
mod parsing;
mod topology;

use std::path::PathBuf;

/// All expected node IDs in nucleus_complete.toml
pub(crate) const NUCLEUS_NODE_IDS: &[&str] = &[
    "tower_beardog",
    "tower_songbird",
    "init_sovereign_onion",
    "init_beacon_mesh",
    "tower_validate",
    "node_toadstool",
    "node_validate",
    "nest_nestgate",
    "register_barracuda",
    "register_coralreef",
    "germinate_squirrel",
    "nucleus_validate",
    "announce_relay",
];

pub(crate) fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("graphs")
}
