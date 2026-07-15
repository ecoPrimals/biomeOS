// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! PlasmodiumAgent unit tests.

use super::common::route;
use super::super::{AgentState, CapabilityRoute, PlasmodiumAgent};
use serde_json::json;
use std::collections::HashMap;

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
