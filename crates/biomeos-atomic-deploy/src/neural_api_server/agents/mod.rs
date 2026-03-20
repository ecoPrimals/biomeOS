// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

mod collective;
mod registry;
mod rpc;
mod types;

pub use collective::agents_from_collective;
pub use registry::AgentRegistry;
pub use rpc::handle_agent_request;
pub use types::{AgentState, CapabilityRoute, PlasmodiumAgent};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// Test fixture address for a remote gate — not a real network address.
    const TEST_REMOTE_GATE_ADDR: &str = "198.51.100.1:8080";

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
        agent.add_route("compute", route("tower", "toadstool", "t.sock", true, 10));
        agent.add_route("compute", route("gate2", "toadstool", "g2.sock", false, 0));

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

        let resolved = tower.resolve("compute").unwrap();
        assert_eq!(resolved.gate_id, "gate2");

        let resolved = tower.resolve("crypto").unwrap();
        assert_eq!(resolved.gate_id, "tower");
    }

    #[test]
    fn test_meld_no_duplicate_gates() {
        let mut a = PlasmodiumAgent::local("a", "cf7e", "tower");
        let b = PlasmodiumAgent::local("b", "cf7e", "tower");
        a.meld(&b);
        assert_eq!(a.gates.len(), 1);
    }

    #[test]
    fn test_meld_no_duplicate_routes() {
        let mut a = PlasmodiumAgent::local("a", "cf7e", "tower");
        a.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));

        let mut b = PlasmodiumAgent::local("b", "cf7e", "tower");
        b.add_route("crypto", route("tower", "beardog", "bd.sock", true, 0));

        a.meld(&b);
        let routes = a.routing_table.get("crypto").unwrap();
        assert_eq!(routes.len(), 1);
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

        assert_eq!(agent.gates, vec!["tower"]);
        assert!(agent.resolve("crypto").is_some());
        assert!(agent.resolve("compute").is_none());
        assert_eq!(agent.state, AgentState::Active);

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
            collective: biomeos_core::plasmodium::CollectiveCapabilities::default(),
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

        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "tower");
        assert_eq!(agents[1].name, "collective");

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
                TEST_REMOTE_GATE_ADDR,
                false,
                vec![("toadstool", true), ("nestgate", true)],
                24576,
            ),
        ]);

        let agents = agents_from_collective(&state);

        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].name, "tower");
        assert_eq!(agents[1].name, "gate2");
        assert_eq!(agents[2].name, "collective");

        let collective = &agents[2];
        assert_eq!(collective.state, AgentState::Melded);
        assert_eq!(collective.gates.len(), 2);

        let crypto = collective.resolve("crypto").unwrap();
        assert_eq!(crypto.gate_id, "tower");
        assert!(crypto.is_local);

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

        assert!(tower.resolve("crypto").is_none());
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
                TEST_REMOTE_GATE_ADDR,
                false,
                vec![("toadstool", true)],
                0,
            ),
        ]);

        let agents = agents_from_collective(&state);

        let tower_route = agents[0].resolve("crypto").unwrap();
        assert_eq!(tower_route.socket, "beardog-test_cf7e.sock");

        let gate2_route = agents[1].resolve("compute").unwrap();
        let expected_socket = format!("{TEST_REMOTE_GATE_ADDR}:toadstool-test_cf7e.sock");
        assert_eq!(gate2_route.socket, expected_socket);
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
                TEST_REMOTE_GATE_ADDR,
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
        assert_eq!(result["agents_created"], 3);

        assert!(registry.get("tower").await.is_some());
        assert!(registry.get("gate2").await.is_some());
        assert!(registry.get("collective").await.is_some());

        let collective = registry.get("collective").await.unwrap();
        assert!(collective.resolve("crypto").is_some());
        assert!(collective.resolve("compute").is_some());

        let compute = collective.resolve("compute").unwrap();
        assert_eq!(compute.gate_id, "gate2");
    }
}
