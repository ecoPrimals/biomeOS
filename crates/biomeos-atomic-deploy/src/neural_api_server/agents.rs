//! Plasmodium Agent Routing
//!
//! Agents are lightweight routing contexts that compose capabilities from
//! multiple gates into a single view. They are NOT processes -- they are
//! routing tables maintained by the Neural API.
//!
//! ## Architecture
//!
//! ```text
//! Agent = { name, family_id, routing_table }
//!
//! routing_table maps capability domains to gate:socket targets:
//!   "crypto.*"  -> local:beardog.sock
//!   "compute.*" -> remote:gate2/toadstool.sock (via Songbird mesh)
//!   "storage.*" -> remote:gate2/nestgate.sock (via Songbird mesh)
//! ```
//!
//! ## Meld/Split/Mix
//!
//! - **Meld**: Combine capabilities from multiple gates into one agent
//! - **Split**: Decompose a melded agent when a gate goes offline
//! - **Mix**: Compose a custom agent from selective capabilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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

// ─── Agent Registry ────────────────────────────────────────────────────

/// Registry of all active Plasmodium agents
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, PlasmodiumAgent>>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create or update an agent
    pub async fn register(&self, agent: PlasmodiumAgent) {
        let name = agent.name.clone();
        info!(
            "Registering agent '{}' (family: {}, gates: {:?})",
            name, agent.family_id, agent.gates
        );
        self.agents.write().await.insert(name, agent);
    }

    /// Get an agent by name
    pub async fn get(&self, name: &str) -> Option<PlasmodiumAgent> {
        self.agents.read().await.get(name).cloned()
    }

    /// List all agents
    pub async fn list(&self) -> Vec<PlasmodiumAgent> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Remove an agent
    pub async fn remove(&self, name: &str) -> Option<PlasmodiumAgent> {
        self.agents.write().await.remove(name)
    }

    /// Meld two agents together
    pub async fn meld(&self, target: &str, source: &str) -> Result<()> {
        let source_agent = self
            .agents
            .read()
            .await
            .get(source)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Source agent '{}' not found", source))?;

        let mut agents = self.agents.write().await;
        let target_agent = agents
            .get_mut(target)
            .ok_or_else(|| anyhow::anyhow!("Target agent '{}' not found", target))?;

        target_agent.meld(&source_agent);
        Ok(())
    }

    /// Split a gate off from an agent
    pub async fn split(&self, agent_name: &str, gate_id: &str) -> Result<Option<PlasmodiumAgent>> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        let split_agent = agent.split(gate_id);

        // Register the split-off agent
        if let Some(ref split) = split_agent {
            agents.insert(split.name.clone(), split.clone());
        }

        Ok(split_agent)
    }

    /// Resolve a capability through a named agent
    pub async fn resolve(
        &self,
        agent_name: &str,
        capability_domain: &str,
    ) -> Option<CapabilityRoute> {
        self.agents
            .read()
            .await
            .get(agent_name)
            .and_then(|agent| agent.resolve(capability_domain).cloned())
    }

    /// Serialize all agents as JSON
    pub async fn to_json(&self) -> Value {
        let agents = self.agents.read().await;
        let agent_list: Vec<_> = agents.values().collect();
        json!({
            "agents": agent_list,
            "count": agent_list.len()
        })
    }
}

// ─── Auto-Meld from Collective State ──────────────────────────────────

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

// ─── JSON-RPC Handler Methods ──────────────────────────────────────────

/// Handle agent-related JSON-RPC requests
pub async fn handle_agent_request(
    registry: &AgentRegistry,
    method: &str,
    params: &Option<Value>,
) -> Result<Value> {
    match method {
        "agent.create" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let agent: PlasmodiumAgent = serde_json::from_value(params.clone())?;
            let name = agent.name.clone();
            registry.register(agent).await;
            Ok(json!({ "created": name }))
        }

        "agent.list" => Ok(registry.to_json().await),

        "agent.get" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let name = params["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
            match registry.get(name).await {
                Some(agent) => Ok(json!(agent)),
                None => Ok(json!({ "error": format!("Agent '{}' not found", name) })),
            }
        }

        "agent.remove" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let name = params["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
            let removed = registry.remove(name).await.is_some();
            Ok(json!({ "removed": removed }))
        }

        "agent.meld" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let target = params["target"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing target agent"))?;
            let source = params["source"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing source agent"))?;
            registry.meld(target, source).await?;
            Ok(json!({ "melded": true, "target": target }))
        }

        "agent.split" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let agent_name = params["agent"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
            let gate_id = params["gate_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing gate_id"))?;
            let split = registry.split(agent_name, gate_id).await?;
            Ok(json!({
                "split": split.is_some(),
                "new_agent": split.map(|a| a.name)
            }))
        }

        "agent.resolve" => {
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let agent_name = params["agent"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
            let capability = params["capability"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing capability"))?;
            match registry.resolve(agent_name, capability).await {
                Some(route) => Ok(json!(route)),
                None => Ok(json!({ "error": "No route found" })),
            }
        }

        "agent.auto_meld" => {
            // Auto-meld from a serialized PlasmodiumState snapshot.
            // Caller provides the collective state (from `biomeos plasmodium status`)
            // and this handler creates per-gate agents + a melded "collective" agent.
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let state: biomeos_core::plasmodium::PlasmodiumState =
                serde_json::from_value(params.clone())?;

            let agents = agents_from_collective(&state);
            let count = agents.len();

            for agent in agents {
                registry.register(agent).await;
            }

            Ok(json!({
                "auto_melded": true,
                "agents_created": count,
                "family_id": state.family_id,
            }))
        }

        "agent.route" => {
            // Convenience: resolve capability through an agent AND return the
            // connection info needed to dispatch the call. The caller (or the
            // Neural API capability handler) uses this to forward the actual
            // JSON-RPC call to the target primal socket.
            let params = params
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;
            let agent_name = params["agent"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
            let capability = params["capability"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing capability domain"))?;
            let operation = params["operation"].as_str().unwrap_or("call");

            match registry.resolve(agent_name, capability).await {
                Some(route) => {
                    // Return the route info + dispatch instructions
                    Ok(json!({
                        "routed": true,
                        "agent": agent_name,
                        "capability": capability,
                        "operation": operation,
                        "route": {
                            "gate_id": route.gate_id,
                            "primal": route.primal,
                            "socket": route.socket,
                            "is_local": route.is_local,
                        },
                        "dispatch": {
                            // For local routes: call the primal socket directly
                            // For remote routes: proxy via Songbird mesh relay
                            "method": format!("{}.{}", capability, operation),
                            "transport": if route.is_local { "unix_socket" } else { "mesh_relay" },
                            "target": route.socket,
                        }
                    }))
                }
                None => Ok(json!({
                    "routed": false,
                    "agent": agent_name,
                    "capability": capability,
                    "error": format!(
                        "No route for capability '{}' in agent '{}'",
                        capability, agent_name
                    )
                })),
            }
        }

        _ => Err(anyhow::anyhow!("Unknown agent method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper: create a test route ────────────────────────────────────

    fn route(
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

    // ── PlasmodiumAgent unit tests ─────────────────────────────────────

    #[test]
    fn test_local_agent() {
        let mut agent = PlasmodiumAgent::local("tower", "test_cf7e8729", "tower");
        agent.add_route(
            "crypto",
            route("tower", "beardog", "beardog-test_cf7e8729.sock", true, 0),
        );

        let resolved = agent.resolve("crypto").unwrap();
        assert_eq!(resolved.primal, "beardog");
        assert_eq!(resolved.gate_id, "tower");
    }

    #[test]
    fn test_local_agent_state() {
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        assert_eq!(agent.state, AgentState::Active);
        assert_eq!(agent.name, "tower");
        assert_eq!(agent.family_id, "cf7e");
        assert_eq!(agent.gates, vec!["tower"]);
        assert!(agent.routing_table.is_empty());
    }

    #[test]
    fn test_resolve_unknown_domain() {
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        assert!(agent.resolve("nonexistent").is_none());
    }

    #[test]
    fn test_priority_ordering() {
        let mut agent = PlasmodiumAgent::local("multi", "cf7e", "tower");
        // Add low-priority route first
        agent.add_route("compute", route("tower", "toadstool", "t.sock", true, 10));
        // Add high-priority route second
        agent.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));

        // Should resolve to gate2 (priority 0 < 10)
        let resolved = agent.resolve("compute").unwrap();
        assert_eq!(resolved.gate_id, "gate2");
        assert_eq!(resolved.priority, 0);
    }

    #[test]
    fn test_meld_agents() {
        let mut tower = PlasmodiumAgent::local("tower", "test_cf7e8729", "tower");
        tower.add_route("crypto", route("tower", "beardog", "beardog.sock", true, 0));
        tower.add_route(
            "compute",
            route("tower", "toadstool", "toadstool.sock", true, 10),
        );

        let mut gate2 = PlasmodiumAgent::local("gate2", "test_cf7e8729", "gate2");
        gate2.add_route(
            "compute",
            route("gate2", "toadstool", "gate2:toadstool.sock", false, 0),
        );

        tower.meld(&gate2);

        assert_eq!(tower.state, AgentState::Melded);
        assert_eq!(tower.gates.len(), 2);

        // compute should resolve to gate2 (priority 0 < 10)
        let resolved = tower.resolve("compute").unwrap();
        assert_eq!(resolved.gate_id, "gate2");

        // crypto should still resolve to tower
        let resolved = tower.resolve("crypto").unwrap();
        assert_eq!(resolved.gate_id, "tower");
    }

    #[test]
    fn test_meld_no_duplicate_gates() {
        let mut a = PlasmodiumAgent::local("a", "cf7e", "tower");
        let b = PlasmodiumAgent::local("b", "cf7e", "tower"); // same gate

        a.meld(&b);
        assert_eq!(a.gates.len(), 1); // No duplicate
    }

    #[test]
    fn test_meld_no_duplicate_routes() {
        let mut a = PlasmodiumAgent::local("a", "cf7e", "tower");
        a.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));

        let mut b = PlasmodiumAgent::local("b", "cf7e", "tower");
        b.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0)); // same

        a.meld(&b);
        let routes = a.routing_table.get("crypto").unwrap();
        assert_eq!(routes.len(), 1); // No duplicate route
    }

    #[test]
    fn test_split_agent() {
        let mut agent = PlasmodiumAgent::local("hpc", "test_cf7e8729", "tower");
        agent.gates.push("gate2".to_string());
        agent.state = AgentState::Melded;

        agent.add_route(
            "compute",
            route("gate2", "toadstool", "gate2:toadstool.sock", false, 0),
        );
        agent.add_route("crypto", route("tower", "beardog", "beardog.sock", true, 0));

        let split = agent.split("gate2").unwrap();

        // Original agent should only have tower
        assert_eq!(agent.gates, vec!["tower"]);
        assert!(agent.resolve("crypto").is_some());
        assert!(agent.resolve("compute").is_none()); // gate2's compute was split off
        assert_eq!(agent.state, AgentState::Active); // No longer melded (single gate)

        // Split agent should have gate2's compute
        assert_eq!(split.gates, vec!["gate2"]);
        assert!(split.resolve("compute").is_some());
        assert_eq!(split.state, AgentState::Split);
        assert_eq!(split.name, "hpc-gate2");
    }

    #[test]
    fn test_split_unknown_gate() {
        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        assert!(agent.split("nonexistent").is_none());
    }

    #[test]
    fn test_agent_serialization_roundtrip() {
        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        agent.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 5));

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: PlasmodiumAgent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "tower");
        assert_eq!(deserialized.family_id, "cf7e");
        assert_eq!(deserialized.gates.len(), 1);
        assert!(deserialized.resolve("crypto").is_some());
        assert!(deserialized.resolve("compute").is_some());
    }

    #[test]
    fn test_agent_state_serialization() {
        for state in &[
            AgentState::Active,
            AgentState::Melded,
            AgentState::Split,
            AgentState::Inactive,
        ] {
            let json = serde_json::to_string(state).unwrap();
            let deserialized: AgentState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, deserialized);
        }
    }

    #[test]
    fn test_capability_route_metadata() {
        let mut meta = HashMap::new();
        meta.insert("vram_gb".to_string(), json!(24));
        meta.insert("backend".to_string(), json!("zfs"));

        let route_with_meta = CapabilityRoute {
            gate_id: "gate2".to_string(),
            primal: "toadstool".to_string(),
            socket: "g2.sock".to_string(),
            is_local: false,
            priority: 0,
            metadata: meta,
        };

        let json = serde_json::to_string(&route_with_meta).unwrap();
        assert!(json.contains("vram_gb"));
        assert!(json.contains("24"));
        assert!(json.contains("zfs"));

        let deserialized: CapabilityRoute = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.metadata["vram_gb"], json!(24));
    }

    // ── AgentRegistry async tests ──────────────────────────────────────

    #[tokio::test]
    async fn test_agent_registry() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("tower", "test_cf7e8729", "tower");
        agent.add_route("crypto", route("tower", "beardog", "beardog.sock", true, 0));

        registry.register(agent).await;

        assert!(registry.get("tower").await.is_some());
        assert_eq!(registry.list().await.len(), 1);

        let resolved = registry.resolve("tower", "crypto").await.unwrap();
        assert_eq!(resolved.primal, "beardog");
    }

    #[tokio::test]
    async fn test_registry_remove() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        assert!(registry.get("tower").await.is_some());
        let removed = registry.remove("tower").await;
        assert!(removed.is_some());
        assert!(registry.get("tower").await.is_none());
        assert_eq!(registry.list().await.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_remove_nonexistent() {
        let registry = AgentRegistry::new();
        assert!(registry.remove("nope").await.is_none());
    }

    #[tokio::test]
    async fn test_registry_meld() {
        let registry = AgentRegistry::new();

        let mut tower = PlasmodiumAgent::local("tower", "cf7e", "tower");
        tower.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));

        let mut gate2 = PlasmodiumAgent::local("gate2", "cf7e", "gate2");
        gate2.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));

        registry.register(tower).await;
        registry.register(gate2).await;

        registry.meld("tower", "gate2").await.unwrap();

        let melded = registry.get("tower").await.unwrap();
        assert_eq!(melded.state, AgentState::Melded);
        assert_eq!(melded.gates.len(), 2);
        assert!(melded.resolve("crypto").is_some());
        assert!(melded.resolve("compute").is_some());
    }

    #[tokio::test]
    async fn test_registry_meld_missing_source() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        let err = registry.meld("tower", "nonexistent").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_registry_split() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("hpc", "cf7e", "tower");
        agent.gates.push("gate2".to_string());
        agent.state = AgentState::Melded;
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        agent.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));

        registry.register(agent).await;

        let split = registry.split("hpc", "gate2").await.unwrap();
        assert!(split.is_some());

        let split_agent = split.unwrap();
        assert_eq!(split_agent.name, "hpc-gate2");

        // Both agents should be in the registry
        assert!(registry.get("hpc").await.is_some());
        assert!(registry.get("hpc-gate2").await.is_some());
    }

    #[tokio::test]
    async fn test_registry_resolve_missing() {
        let registry = AgentRegistry::new();
        assert!(registry.resolve("nonexistent", "crypto").await.is_none());
    }

    #[tokio::test]
    async fn test_registry_to_json() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        let json = registry.to_json().await;
        assert_eq!(json["count"], 1);
        assert!(json["agents"].is_array());
    }

    // ── handle_agent_request RPC handler tests ─────────────────────────

    #[tokio::test]
    async fn test_rpc_agent_create() {
        let registry = AgentRegistry::new();
        let params = Some(json!({
            "name": "tower",
            "family_id": "cf7e",
            "gates": ["tower"],
            "routing_table": {},
            "state": "Active"
        }));

        let result = handle_agent_request(&registry, "agent.create", &params)
            .await
            .unwrap();
        assert_eq!(result["created"], "tower");
        assert!(registry.get("tower").await.is_some());
    }

    #[tokio::test]
    async fn test_rpc_agent_create_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.create", &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_agent_list_empty() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.list", &None)
            .await
            .unwrap();
        assert_eq!(result["count"], 0);
    }

    #[tokio::test]
    async fn test_rpc_agent_list_with_agents() {
        let registry = AgentRegistry::new();
        registry
            .register(PlasmodiumAgent::local("a", "cf7e", "tower"))
            .await;
        registry
            .register(PlasmodiumAgent::local("b", "cf7e", "gate2"))
            .await;

        let result = handle_agent_request(&registry, "agent.list", &None)
            .await
            .unwrap();
        assert_eq!(result["count"], 2);
    }

    #[tokio::test]
    async fn test_rpc_agent_get() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        let params = Some(json!({ "name": "tower" }));
        let result = handle_agent_request(&registry, "agent.get", &params)
            .await
            .unwrap();
        assert_eq!(result["name"], "tower");
        assert_eq!(result["family_id"], "cf7e");
    }

    #[tokio::test]
    async fn test_rpc_agent_get_not_found() {
        let registry = AgentRegistry::new();
        let params = Some(json!({ "name": "nonexistent" }));
        let result = handle_agent_request(&registry, "agent.get", &params)
            .await
            .unwrap();
        assert!(result["error"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_rpc_agent_get_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.get", &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_agent_remove() {
        let registry = AgentRegistry::new();
        registry
            .register(PlasmodiumAgent::local("tower", "cf7e", "tower"))
            .await;

        let params = Some(json!({ "name": "tower" }));
        let result = handle_agent_request(&registry, "agent.remove", &params)
            .await
            .unwrap();
        assert_eq!(result["removed"], true);
        assert!(registry.get("tower").await.is_none());
    }

    #[tokio::test]
    async fn test_rpc_agent_remove_nonexistent() {
        let registry = AgentRegistry::new();
        let params = Some(json!({ "name": "nonexistent" }));
        let result = handle_agent_request(&registry, "agent.remove", &params)
            .await
            .unwrap();
        assert_eq!(result["removed"], false);
    }

    #[tokio::test]
    async fn test_rpc_agent_meld() {
        let registry = AgentRegistry::new();

        let mut tower = PlasmodiumAgent::local("tower", "cf7e", "tower");
        tower.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        registry.register(tower).await;

        let mut gate2 = PlasmodiumAgent::local("gate2", "cf7e", "gate2");
        gate2.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));
        registry.register(gate2).await;

        let params = Some(json!({ "target": "tower", "source": "gate2" }));
        let result = handle_agent_request(&registry, "agent.meld", &params)
            .await
            .unwrap();
        assert_eq!(result["melded"], true);

        let melded = registry.get("tower").await.unwrap();
        assert_eq!(melded.state, AgentState::Melded);
    }

    #[tokio::test]
    async fn test_rpc_agent_meld_missing_target() {
        let registry = AgentRegistry::new();
        let params = Some(json!({ "target": "nonexistent", "source": "also_nope" }));
        let result = handle_agent_request(&registry, "agent.meld", &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_agent_split() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("hpc", "cf7e", "tower");
        agent.gates.push("gate2".to_string());
        agent.state = AgentState::Melded;
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        agent.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));
        registry.register(agent).await;

        let params = Some(json!({ "agent": "hpc", "gate_id": "gate2" }));
        let result = handle_agent_request(&registry, "agent.split", &params)
            .await
            .unwrap();
        assert_eq!(result["split"], true);
        assert_eq!(result["new_agent"], "hpc-gate2");
    }

    #[tokio::test]
    async fn test_rpc_agent_resolve() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        registry.register(agent).await;

        let params = Some(json!({ "agent": "tower", "capability": "crypto" }));
        let result = handle_agent_request(&registry, "agent.resolve", &params)
            .await
            .unwrap();
        assert_eq!(result["primal"], "beardog");
        assert_eq!(result["gate_id"], "tower");
    }

    #[tokio::test]
    async fn test_rpc_agent_resolve_not_found() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        let params = Some(json!({ "agent": "tower", "capability": "nonexistent" }));
        let result = handle_agent_request(&registry, "agent.resolve", &params)
            .await
            .unwrap();
        assert!(result["error"].as_str().unwrap().contains("No route"));
    }

    #[tokio::test]
    async fn test_rpc_agent_route_local() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        registry.register(agent).await;

        let params = Some(json!({
            "agent": "tower",
            "capability": "crypto",
            "operation": "sign"
        }));
        let result = handle_agent_request(&registry, "agent.route", &params)
            .await
            .unwrap();
        assert_eq!(result["routed"], true);
        assert_eq!(result["route"]["primal"], "beardog");
        assert_eq!(result["route"]["is_local"], true);
        assert_eq!(result["dispatch"]["method"], "crypto.sign");
        assert_eq!(result["dispatch"]["transport"], "unix_socket");
        assert_eq!(result["dispatch"]["target"], "bd.sock");
    }

    #[tokio::test]
    async fn test_rpc_agent_route_remote() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("hpc", "cf7e", "tower");
        agent.gates.push("gate2".to_string());
        agent.add_route(
            "compute",
            route("gate2", "toadstool", "gate2:toadstool.sock", false, 0),
        );
        registry.register(agent).await;

        let params = Some(json!({
            "agent": "hpc",
            "capability": "compute",
            "operation": "submit"
        }));
        let result = handle_agent_request(&registry, "agent.route", &params)
            .await
            .unwrap();
        assert_eq!(result["routed"], true);
        assert_eq!(result["route"]["gate_id"], "gate2");
        assert_eq!(result["route"]["is_local"], false);
        assert_eq!(result["dispatch"]["method"], "compute.submit");
        assert_eq!(result["dispatch"]["transport"], "mesh_relay");
    }

    #[tokio::test]
    async fn test_rpc_agent_route_default_operation() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        registry.register(agent).await;

        // No explicit operation — should default to "call"
        let params = Some(json!({ "agent": "tower", "capability": "crypto" }));
        let result = handle_agent_request(&registry, "agent.route", &params)
            .await
            .unwrap();
        assert_eq!(result["dispatch"]["method"], "crypto.call");
    }

    #[tokio::test]
    async fn test_rpc_agent_route_no_route() {
        let registry = AgentRegistry::new();
        let agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        registry.register(agent).await;

        let params = Some(json!({ "agent": "tower", "capability": "nonexistent" }));
        let result = handle_agent_request(&registry, "agent.route", &params)
            .await
            .unwrap();
        assert_eq!(result["routed"], false);
        assert!(result["error"].as_str().unwrap().contains("No route"));
    }

    #[tokio::test]
    async fn test_rpc_unknown_method() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.unknown", &None).await;
        assert!(result.is_err());
    }

    // ── agents_from_collective tests ───────────────────────────────────

    fn mock_state(
        gates: Vec<biomeos_core::plasmodium::GateInfo>,
    ) -> biomeos_core::plasmodium::PlasmodiumState {
        biomeos_core::plasmodium::PlasmodiumState {
            gates,
            snapshot_at: "2026-02-11T20:00:00Z".to_string(),
            family_id: "test_cf7e".to_string(),
            collective: Default::default(),
        }
    }

    fn mock_gate(
        id: &str,
        addr: &str,
        local: bool,
        primals: Vec<(&str, bool)>,
        vram_mb: u64,
    ) -> biomeos_core::plasmodium::GateInfo {
        use biomeos_core::plasmodium::*;
        GateInfo {
            gate_id: id.to_string(),
            address: addr.to_string(),
            is_local: local,
            primals: primals
                .into_iter()
                .map(|(name, healthy)| PrimalStatus {
                    name: name.to_string(),
                    healthy,
                    version: None,
                })
                .collect(),
            compute: ComputeInfo {
                gpus: if vram_mb > 0 {
                    vec![GpuInfo {
                        name: "GPU".to_string(),
                        vram_mb,
                        gate_id: id.to_string(),
                    }]
                } else {
                    vec![]
                },
                ram_gb: 32,
                cpu_cores: 16,
            },
            models: vec![],
            load: 0.1,
            reachable: true,
            bond_type: BondType::Covalent,
        }
    }

    #[test]
    fn test_auto_meld_single_gate() {
        let state = mock_state(vec![mock_gate(
            "tower",
            "local",
            true,
            vec![("beardog", true), ("songbird", true)],
            12288,
        )]);

        let agents = agents_from_collective(&state);

        // 1 per-gate agent + 1 "collective" alias
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "tower");
        assert_eq!(agents[1].name, "collective");

        // tower agent should have crypto (beardog) and network/discovery (songbird)
        assert!(agents[0].resolve("crypto").is_some());
        assert!(agents[0].resolve("network").is_some());
    }

    #[test]
    fn test_auto_meld_two_gates() {
        let state = mock_state(vec![
            mock_gate(
                "tower",
                "local",
                true,
                vec![("beardog", true), ("songbird", true), ("toadstool", true)],
                12288,
            ),
            mock_gate(
                "gate2",
                "192.168.1.132:8080",
                false,
                vec![("toadstool", true), ("nestgate", true)],
                24576,
            ),
        ]);

        let agents = agents_from_collective(&state);

        // 2 per-gate agents + 1 "collective" melded
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name, "tower");
        assert_eq!(agents[1].name, "gate2");
        assert_eq!(agents[2].name, "collective");

        let collective = &agents[2];
        assert_eq!(collective.state, AgentState::Melded);
        assert_eq!(collective.gates.len(), 2);

        // Collective should route crypto to tower (only source)
        let crypto = collective.resolve("crypto").unwrap();
        assert_eq!(crypto.gate_id, "tower");
        assert!(crypto.is_local);

        // Collective should prefer gate2 for compute (24GB > 12GB VRAM)
        let compute = collective.resolve("compute").unwrap();
        assert_eq!(compute.gate_id, "gate2");
        assert!(!compute.is_local);
    }

    #[test]
    fn test_auto_meld_local_priority() {
        let state = mock_state(vec![
            mock_gate("tower", "local", true, vec![("songbird", true)], 0),
            mock_gate("gate2", "192.168.1.132", false, vec![("songbird", true)], 0),
        ]);

        let agents = agents_from_collective(&state);
        let collective = &agents[2];

        // Both have songbird → network. Local (tower) should win with priority 0 vs 10
        let network = collective.resolve("network").unwrap();
        assert_eq!(network.gate_id, "tower");
        assert!(network.is_local);
    }

    #[test]
    fn test_auto_meld_skips_unreachable() {
        let mut state = mock_state(vec![mock_gate(
            "tower",
            "local",
            true,
            vec![("beardog", true)],
            0,
        )]);
        // Add an unreachable gate
        let mut unreachable = mock_gate(
            "gate2",
            "192.168.1.132",
            false,
            vec![("toadstool", true)],
            0,
        );
        unreachable.reachable = false;
        state.gates.push(unreachable);

        let agents = agents_from_collective(&state);

        // Only tower + collective (unreachable gate2 skipped)
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "tower");
    }

    #[test]
    fn test_auto_meld_skips_unhealthy_primals() {
        let state = mock_state(vec![mock_gate(
            "tower",
            "local",
            true,
            vec![("beardog", false), ("songbird", true)],
            0,
        )]);

        let agents = agents_from_collective(&state);
        let tower = &agents[0];

        // beardog is unhealthy → no crypto route
        assert!(tower.resolve("crypto").is_none());
        // songbird is healthy → has network route
        assert!(tower.resolve("network").is_some());
    }

    #[test]
    fn test_auto_meld_empty_collective() {
        let state = mock_state(vec![]);
        let agents = agents_from_collective(&state);
        assert!(agents.is_empty());
    }

    #[test]
    fn test_auto_meld_socket_paths() {
        let state = mock_state(vec![
            mock_gate("tower", "local", true, vec![("beardog", true)], 0),
            mock_gate(
                "gate2",
                "192.168.1.132:8080",
                false,
                vec![("toadstool", true)],
                0,
            ),
        ]);

        let agents = agents_from_collective(&state);

        // Local socket: primal-family.sock
        let tower_route = agents[0].resolve("crypto").unwrap();
        assert_eq!(tower_route.socket, "beardog-test_cf7e.sock");

        // Remote socket: addr:primal-family.sock
        let gate2_route = agents[1].resolve("compute").unwrap();
        assert_eq!(
            gate2_route.socket,
            "192.168.1.132:8080:toadstool-test_cf7e.sock"
        );
    }

    #[test]
    fn test_auto_meld_compute_metadata() {
        let state = mock_state(vec![mock_gate(
            "tower",
            "local",
            true,
            vec![("toadstool", true)],
            12288,
        )]);

        let agents = agents_from_collective(&state);
        let compute = agents[0].resolve("compute").unwrap();

        // Metadata should include GPU info
        assert!(compute.metadata.contains_key("gpus"));
        assert!(compute.metadata.contains_key("ram_gb"));
    }

    #[tokio::test]
    async fn test_rpc_agent_auto_meld() {
        let registry = AgentRegistry::new();

        let state = mock_state(vec![
            mock_gate(
                "tower",
                "local",
                true,
                vec![("beardog", true), ("songbird", true)],
                12288,
            ),
            mock_gate(
                "gate2",
                "192.168.1.132:8080",
                false,
                vec![("toadstool", true)],
                24576,
            ),
        ]);

        let params = Some(serde_json::to_value(&state).unwrap());
        let result = handle_agent_request(&registry, "agent.auto_meld", &params)
            .await
            .unwrap();

        assert_eq!(result["auto_melded"], true);
        assert_eq!(result["agents_created"], 3); // tower + gate2 + collective

        // Verify agents were registered
        assert!(registry.get("tower").await.is_some());
        assert!(registry.get("gate2").await.is_some());
        assert!(registry.get("collective").await.is_some());

        // Verify collective routes
        let collective = registry.get("collective").await.unwrap();
        assert!(collective.resolve("crypto").is_some());
        assert!(collective.resolve("compute").is_some());

        // compute should prefer gate2 (24GB VRAM)
        let compute = collective.resolve("compute").unwrap();
        assert_eq!(compute.gate_id, "gate2");
    }
}
