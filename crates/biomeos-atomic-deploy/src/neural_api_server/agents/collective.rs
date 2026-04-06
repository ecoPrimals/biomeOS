// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Auto-meld from Plasmodium collective state.
//!
//! Builds agents automatically from a Plasmodium collective snapshot,
//! bridging discovery (gates and primals) with agent routing.

use std::collections::HashMap;

use super::types::{CapabilityRoute, PlasmodiumAgent};

/// Build agents automatically from a Plasmodium collective snapshot.
///
/// This is the bridge between Plasmodium discovery (which finds gates and their
/// primals) and the Agent routing system (which routes capability calls to the
/// best primal). Call this after `Plasmodium::query_collective()`.
///
/// ## What It Creates
///
/// 1. **Per-gate agents**: One agent per reachable gate, with routes for each
///    healthy primal's capability domains.
/// 2. **Melded collective agent**: A single `"collective"` agent that merges
///    all gate agents, preferring local routes and higher-VRAM compute targets.
///
/// ## Priority Heuristic
///
/// - Local gates get priority 0 (best)
/// - Remote gates get base priority 10
/// - Compute routes are further adjusted by VRAM (more VRAM = lower priority number = preferred)
#[must_use]
pub fn agents_from_collective(
    state: &biomeos_core::plasmodium::PlasmodiumState,
) -> Vec<PlasmodiumAgent> {
    let mut agents = Vec::new();

    // 1. Create per-gate agents
    for gate in &state.gates {
        if !gate.reachable {
            continue;
        }

        let mut agent = PlasmodiumAgent::local(&gate.gate_id, &state.family_id, &gate.gate_id);

        let base_priority = if gate.is_local { 0u32 } else { 10 };

        for primal in &gate.primals {
            if !primal.healthy {
                continue;
            }

            let domains = biomeos_types::capability_taxonomy::capabilities_for_primal(&primal.name);

            for domain in &domains {
                let priority = if domain == "compute" {
                    // Compute routes: VRAM is the primary factor (locality is secondary).
                    // This ensures that gate2 with 24GB beats tower with 12GB even though
                    // tower is local. The relay/mesh overhead is negligible compared to
                    // the benefit of more GPU memory for model inference.
                    let max_vram = gate
                        .compute
                        .gpus
                        .iter()
                        .map(|g| g.vram_mb)
                        .max()
                        .unwrap_or(0);
                    // More VRAM → lower priority number → preferred
                    // 24GB → 0, 12GB → 5, 0GB → 20, + small locality bonus
                    let vram_score = if max_vram >= 20_000 {
                        0u32
                    } else if max_vram >= 10_000 {
                        5
                    } else if max_vram > 0 {
                        10
                    } else {
                        20
                    };
                    let locality_bonus = if gate.is_local { 0 } else { 1 };
                    vram_score + locality_bonus
                } else {
                    // Non-compute routes: locality is the primary factor
                    base_priority
                };

                let mut metadata = HashMap::new();
                if domain == "compute" && !gate.compute.gpus.is_empty() {
                    metadata.insert(
                        "gpus".to_string(),
                        serde_json::to_value(&gate.compute.gpus).unwrap_or_default(),
                    );
                    metadata.insert("ram_gb".to_string(), serde_json::json!(gate.compute.ram_gb));
                }
                if domain == "storage" {
                    metadata.insert("gate".to_string(), serde_json::json!(gate.gate_id));
                }

                let socket_path = if gate.is_local {
                    format!("{}-{}.sock", primal.name, state.family_id)
                } else {
                    format!("{}:{}-{}.sock", gate.address, primal.name, state.family_id)
                };

                agent.add_route(
                    domain,
                    CapabilityRoute {
                        gate_id: gate.gate_id.clone(),
                        primal: primal.name.clone(),
                        socket: socket_path,
                        is_local: gate.is_local,
                        priority,
                        metadata,
                    },
                );
            }
        }

        agents.push(agent);
    }

    // 2. Create the melded collective agent
    if agents.len() > 1 {
        let mut collective = agents[0].clone();
        collective.name = "collective".to_string();

        for agent in agents.iter().skip(1) {
            collective.meld(agent);
        }

        agents.push(collective);
    } else if agents.len() == 1 {
        // Single gate — make a collective alias
        let mut collective = agents[0].clone();
        collective.name = "collective".to_string();
        agents.push(collective);
    }

    agents
}
