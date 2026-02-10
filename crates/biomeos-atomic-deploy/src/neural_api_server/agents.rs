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

        _ => Err(anyhow::anyhow!("Unknown agent method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_agent() {
        let mut agent = PlasmodiumAgent::local("tower", "nat0", "tower");
        agent.add_route(
            "crypto",
            CapabilityRoute {
                gate_id: "tower".to_string(),
                primal: "beardog".to_string(),
                socket: "beardog-nat0.sock".to_string(),
                is_local: true,
                priority: 0,
                metadata: HashMap::new(),
            },
        );

        let route = agent.resolve("crypto").unwrap();
        assert_eq!(route.primal, "beardog");
        assert_eq!(route.gate_id, "tower");
    }

    #[test]
    fn test_meld_agents() {
        let mut tower = PlasmodiumAgent::local("tower", "nat0", "tower");
        tower.add_route(
            "crypto",
            CapabilityRoute {
                gate_id: "tower".to_string(),
                primal: "beardog".to_string(),
                socket: "beardog.sock".to_string(),
                is_local: true,
                priority: 0,
                metadata: HashMap::new(),
            },
        );
        tower.add_route(
            "compute",
            CapabilityRoute {
                gate_id: "tower".to_string(),
                primal: "toadstool".to_string(),
                socket: "toadstool.sock".to_string(),
                is_local: true,
                priority: 10, // Lower priority (higher number)
                metadata: HashMap::new(),
            },
        );

        let mut gate2 = PlasmodiumAgent::local("gate2", "nat0", "gate2");
        gate2.add_route(
            "compute",
            CapabilityRoute {
                gate_id: "gate2".to_string(),
                primal: "toadstool".to_string(),
                socket: "gate2:toadstool.sock".to_string(),
                is_local: false,
                priority: 0, // Higher priority (lower number) -- prefer gate2 for compute
                metadata: HashMap::new(),
            },
        );

        tower.meld(&gate2);

        assert_eq!(tower.state, AgentState::Melded);
        assert_eq!(tower.gates.len(), 2);

        // compute should resolve to gate2 (priority 0 < 10)
        let route = tower.resolve("compute").unwrap();
        assert_eq!(route.gate_id, "gate2");

        // crypto should still resolve to tower
        let route = tower.resolve("crypto").unwrap();
        assert_eq!(route.gate_id, "tower");
    }

    #[test]
    fn test_split_agent() {
        let mut agent = PlasmodiumAgent::local("hpc", "nat0", "tower");
        agent.gates.push("gate2".to_string());
        agent.state = AgentState::Melded;

        agent.add_route(
            "compute",
            CapabilityRoute {
                gate_id: "gate2".to_string(),
                primal: "toadstool".to_string(),
                socket: "gate2:toadstool.sock".to_string(),
                is_local: false,
                priority: 0,
                metadata: HashMap::new(),
            },
        );
        agent.add_route(
            "crypto",
            CapabilityRoute {
                gate_id: "tower".to_string(),
                primal: "beardog".to_string(),
                socket: "beardog.sock".to_string(),
                is_local: true,
                priority: 0,
                metadata: HashMap::new(),
            },
        );

        let split = agent.split("gate2").unwrap();

        // Original agent should only have tower
        assert_eq!(agent.gates, vec!["tower"]);
        assert!(agent.resolve("crypto").is_some());
        assert!(agent.resolve("compute").is_none()); // gate2's compute was split off

        // Split agent should have gate2's compute
        assert_eq!(split.gates, vec!["gate2"]);
        assert!(split.resolve("compute").is_some());
        assert_eq!(split.state, AgentState::Split);
    }

    #[tokio::test]
    async fn test_agent_registry() {
        let registry = AgentRegistry::new();

        let mut agent = PlasmodiumAgent::local("tower", "nat0", "tower");
        agent.add_route(
            "crypto",
            CapabilityRoute {
                gate_id: "tower".to_string(),
                primal: "beardog".to_string(),
                socket: "beardog.sock".to_string(),
                is_local: true,
                priority: 0,
                metadata: HashMap::new(),
            },
        );

        registry.register(agent).await;

        assert!(registry.get("tower").await.is_some());
        assert_eq!(registry.list().await.len(), 1);

        let route = registry.resolve("tower", "crypto").await.unwrap();
        assert_eq!(route.primal, "beardog");
    }
}
