// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Plasmodium agent types and core logic.
//!
//! Defines `CapabilityRoute`, `AgentState`, and `PlasmodiumAgent` with
//! meld/split/resolve operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

/// A capability route pointing to a specific primal on a specific gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRoute {
    /// Gate that provides this capability (e.g., "tower", "gate2")
    pub gate_id: String,

    /// Primal name (e.g., "beardog", "songbird")
    pub primal: String,

    /// Socket path (local Unix socket or remote via mesh)
    pub socket: String,

    /// Whether this is a local or remote route
    pub is_local: bool,

    /// Priority (lower = preferred). Used for routing selection.
    pub priority: u32,

    /// Optional metadata (VRAM, storage backend, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

/// Agent lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is active and routing
    Active,
    /// Agent is melded from multiple gates
    Melded,
    /// Agent has split due to gate disconnection
    Split,
    /// Agent is inactive
    Inactive,
}

/// A Plasmodium agent -- a routing context for capability composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasmodiumAgent {
    /// Agent name (e.g., "hpc_coordinator", "local_tower")
    pub name: String,

    /// Family ID this agent serves
    pub family_id: String,

    /// Gate IDs participating in this agent
    pub gates: Vec<String>,

    /// Capability domain -> route mapping
    /// Key is a capability domain (e.g., "crypto", "compute", "storage")
    /// Value is a list of routes (first available is used, sorted by priority)
    pub routing_table: HashMap<String, Vec<CapabilityRoute>>,

    /// Agent state
    pub state: AgentState,
}

impl PlasmodiumAgent {
    /// Create a new agent for a single local gate
    pub fn local(name: &str, family_id: &str, gate_id: &str) -> Self {
        Self {
            name: name.to_string(),
            family_id: family_id.to_string(),
            gates: vec![gate_id.to_string()],
            routing_table: HashMap::new(),
            state: AgentState::Active,
        }
    }

    /// Add a capability route to this agent
    pub fn add_route(&mut self, domain: &str, route: CapabilityRoute) {
        self.routing_table
            .entry(domain.to_string())
            .or_default()
            .push(route);

        // Keep routes sorted by priority
        if let Some(routes) = self.routing_table.get_mut(domain) {
            routes.sort_by_key(|r| r.priority);
        }
    }

    /// Resolve a capability to the best available route
    pub fn resolve(&self, capability_domain: &str) -> Option<&CapabilityRoute> {
        self.routing_table
            .get(capability_domain)
            .and_then(|routes| routes.first())
    }

    /// Meld another agent's capabilities into this one
    pub fn meld(&mut self, other: &PlasmodiumAgent) {
        // Add the other agent's gates
        for gate in &other.gates {
            if !self.gates.contains(gate) {
                self.gates.push(gate.clone());
            }
        }

        // Merge routing tables (keep best priority for each domain)
        for (domain, routes) in &other.routing_table {
            let existing = self.routing_table.entry(domain.clone()).or_default();
            for route in routes {
                // Don't add duplicate gate+primal combinations
                let already_exists = existing
                    .iter()
                    .any(|r| r.gate_id == route.gate_id && r.primal == route.primal);
                if !already_exists {
                    existing.push(route.clone());
                }
            }
            existing.sort_by_key(|r| r.priority);
        }

        self.state = AgentState::Melded;
        info!(
            "Agent '{}' melded with '{}': now spans {} gates",
            self.name,
            other.name,
            self.gates.len()
        );
    }

    /// Split this agent by removing a gate (returns the split-off agent)
    pub fn split(&mut self, gate_id: &str) -> Option<PlasmodiumAgent> {
        if !self.gates.contains(&gate_id.to_string()) {
            return None;
        }

        // Create a new agent with just the removed gate's capabilities
        let mut split_agent = PlasmodiumAgent::local(
            &format!("{}-{}", self.name, gate_id),
            &self.family_id,
            gate_id,
        );
        split_agent.state = AgentState::Split;

        // Move routes belonging to the split gate
        for (domain, routes) in &self.routing_table {
            let split_routes: Vec<_> = routes
                .iter()
                .filter(|r| r.gate_id == gate_id)
                .cloned()
                .collect();
            for route in split_routes {
                split_agent.add_route(domain, route);
            }
        }

        // Remove the gate and its routes from this agent
        self.gates.retain(|g| g != gate_id);
        for routes in self.routing_table.values_mut() {
            routes.retain(|r| r.gate_id != gate_id);
        }

        // Update state
        if self.gates.len() <= 1 {
            self.state = AgentState::Active; // No longer melded
        }

        warn!(
            "Agent '{}' split: gate '{}' removed (now {} gates)",
            self.name,
            gate_id,
            self.gates.len()
        );

        Some(split_agent)
    }
}
