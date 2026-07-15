// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AgentRegistry async tests.

use super::common::route;
use super::super::{AgentRegistry, AgentState, PlasmodiumAgent};

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
