// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Agent routing tests (PlasmodiumAgent, registry, RPC, collective auto-meld).

#[path = "agents_tests_plasmodium.rs"]
mod plasmodium;

#[path = "agents_tests_registry.rs"]
mod registry;

#[path = "agents_tests_rpc.rs"]
mod rpc;

#[path = "agents_tests_collective.rs"]
mod collective;

mod common {
    use super::super::CapabilityRoute;
    use std::collections::HashMap;

    /// Test fixture address for a remote gate — not a real network address.
    pub(super) const TEST_REMOTE_GATE_ADDR: &str = "198.51.100.1:8080";

    pub(super) fn route(
        gate: &str,
        primal: &str,
        socket: &str,
        local: bool,
        priority: u32,
    ) -> CapabilityRoute {
        CapabilityRoute {
            gate_id: gate.to_string(),
            primal: primal.to_string(),
            socket: socket.to_string(),
            is_local: local,
            priority,
            metadata: HashMap::new(),
        }
    }
}
