// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_health_status_healthy() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "healthy");
    assert_eq!(status.primals_healthy, 2);
    assert_eq!(status.primals_total, 2);
}

#[test]
fn test_health_status_degraded() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "degraded");
    assert_eq!(status.primals_healthy, 1);
}

#[test]
fn test_extract_node_id_three_parts() {
    let result = extract_node_id_from_primal("beardog-fam-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_two_parts() {
    let result = extract_node_id_from_primal("beardog-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_one_part() {
    let result = extract_node_id_from_primal("standalone");
    assert_eq!(result, Some("standalone".to_string()));
}

#[test]
fn test_standalone_topology() {
    let (nodes, edges) = get_standalone_topology();
    assert!(!nodes.is_empty());
    assert!(!edges.is_empty());

    // Check nodes have proper capabilities
    for node in &nodes {
        assert!(!node.capabilities.is_empty());
        assert!(!node.id.is_empty());
    }
}

#[test]
fn test_extract_node_id_four_parts() {
    let result = extract_node_id_from_primal("beardog-fam-node-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_empty_string() {
    let result = extract_node_id_from_primal("");
    assert_eq!(result, Some(String::new()));
}

#[test]
fn test_health_status_all_unhealthy() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "unhealthy");
    assert_eq!(status.primals_healthy, 0);
    assert_eq!(status.primals_total, 2);
}

#[test]
fn test_health_status_empty() {
    let nodes: Vec<TopologyNode> = vec![];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "healthy");
    assert_eq!(status.primals_healthy, 0);
    assert_eq!(status.primals_total, 0);
}
