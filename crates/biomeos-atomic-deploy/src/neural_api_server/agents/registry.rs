// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Agent registry — in-memory store of active Plasmodium agents.

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use super::types::{CapabilityRoute, PlasmodiumAgent};

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
            .ok_or_else(|| anyhow::anyhow!("Source agent '{source}' not found"))?;

        let mut agents = self.agents.write().await;
        let target_agent = agents
            .get_mut(target)
            .ok_or_else(|| anyhow::anyhow!("Target agent '{target}' not found"))?;

        target_agent.meld(&source_agent);
        Ok(())
    }

    /// Split a gate off from an agent
    pub async fn split(&self, agent_name: &str, gate_id: &str) -> Result<Option<PlasmodiumAgent>> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{agent_name}' not found"))?;

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
