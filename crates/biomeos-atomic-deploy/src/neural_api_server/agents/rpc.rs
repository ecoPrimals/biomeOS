// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC handler for agent-related requests.

use anyhow::Result;
use serde_json::{Value, json};

use super::collective::agents_from_collective;
use super::registry::AgentRegistry;
use super::types::PlasmodiumAgent;

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

        _ => Err(anyhow::anyhow!("Unknown agent method: {method}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural_api_server::agents::{
        AgentRegistry, AgentState, CapabilityRoute, PlasmodiumAgent,
    };
    use serde_json::json;
    use std::collections::HashMap;

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

    #[tokio::test]
    async fn test_agent_split_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.split", &None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn test_agent_split_missing_agent_name() {
        let registry = AgentRegistry::new();
        let params = Some(json!({ "gate_id": "gate2" }));
        let result = handle_agent_request(&registry, "agent.split", &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_split_missing_gate_id() {
        let registry = AgentRegistry::new();
        let mut agent = PlasmodiumAgent::local("hpc", "cf7e", "tower");
        agent.gates.push("gate2".to_string());
        agent.state = AgentState::Melded;
        registry.register(agent).await;

        let params = Some(json!({ "agent": "hpc" }));
        let result = handle_agent_request(&registry, "agent.split", &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_resolve_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.resolve", &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_resolve_missing_capability() {
        let registry = AgentRegistry::new();
        let mut agent = PlasmodiumAgent::local("tower", "cf7e", "tower");
        agent.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));
        registry.register(agent).await;

        let params = Some(json!({ "agent": "tower" }));
        let result = handle_agent_request(&registry, "agent.resolve", &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_route_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.route", &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_auto_meld_missing_params() {
        let registry = AgentRegistry::new();
        let result = handle_agent_request(&registry, "agent.auto_meld", &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_auto_meld_invalid_state() {
        let registry = AgentRegistry::new();
        let params = Some(json!({ "invalid": "state" }));
        let result = handle_agent_request(&registry, "agent.auto_meld", &params).await;
        assert!(result.is_err());
    }
}
