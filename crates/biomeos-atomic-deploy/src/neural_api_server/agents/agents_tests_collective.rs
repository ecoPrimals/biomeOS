// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! agents_from_collective and auto-meld tests.

use super::super::{AgentRegistry, AgentState, agents_from_collective, handle_agent_request};
use super::common::TEST_REMOTE_GATE_ADDR;

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
        mock_gate("gate2", "192.0.2.132", false, vec![("songbird", true)], 0),
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
    let mut unreachable = mock_gate("gate2", "192.0.2.132", false, vec![("toadstool", true)], 0);
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
