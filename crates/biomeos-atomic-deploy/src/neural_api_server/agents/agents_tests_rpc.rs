// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! handle_agent_request RPC handler tests.

use super::super::{AgentRegistry, AgentState, PlasmodiumAgent, handle_agent_request};
use super::common::route;
use serde_json::json;

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
