// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::create_test_node;
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

#[test]
fn test_topological_sort_simple() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec![]),
            create_test_node("node2", vec!["node1".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0], vec!["node1"]);
    assert_eq!(phases[1], vec!["node2"]);
}

#[test]
fn test_topological_sort_parallel() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec![]),
            create_test_node("node2", vec![]),
            create_test_node("node3", vec!["node1".to_string(), "node2".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0].len(), 2); // node1 and node2 in parallel
    assert!(phases[0].contains(&"node1".to_string()));
    assert!(phases[0].contains(&"node2".to_string()));
    assert_eq!(phases[1], vec!["node3"]);
}

#[test]
fn test_topological_sort_cycle_detection() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec!["node2".to_string()]),
            create_test_node("node2", vec!["node1".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let result = executor.topological_sort();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn test_topological_sort_empty_graph() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 0);
}

#[test]
fn test_topological_sort_complex_dependencies() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec!["a".to_string()]),
            create_test_node("d", vec!["b".to_string(), "c".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0], vec!["a"]);
    assert_eq!(phases[1].len(), 2); // b and c in parallel
    assert_eq!(phases[2], vec!["d"]);
}

#[test]
fn test_topological_sort_single_node() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![create_test_node("solo", vec![])],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 1);
    assert_eq!(phases[0], vec!["solo"]);
}

#[test]
fn test_topological_sort_deep_chain() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec!["b".to_string()]),
            create_test_node("d", vec!["c".to_string()]),
            create_test_node("e", vec!["d".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 5);
    for (i, phase) in phases.iter().enumerate() {
        assert_eq!(phase.len(), 1);
        assert_eq!(phase[0], ["a", "b", "c", "d", "e"][i]);
    }
}

#[test]
fn test_topological_sort_wide_graph() {
    // All nodes independent — should all be in one phase
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec![]),
            create_test_node("c", vec![]),
            create_test_node("d", vec![]),
            create_test_node("e", vec![]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 1);
    assert_eq!(phases[0].len(), 5);
}

#[test]
fn test_topological_sort_diamond_with_tail() {
    //     a
    //    / \
    //   b   c
    //    \ /
    //     d
    //     |
    //     e
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec!["a".to_string()]),
            create_test_node("d", vec!["b".to_string(), "c".to_string()]),
            create_test_node("e", vec!["d".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 4);
    assert_eq!(phases[0], vec!["a"]);
    assert_eq!(phases[1].len(), 2); // b and c
    assert_eq!(phases[2], vec!["d"]);
    assert_eq!(phases[3], vec!["e"]);
}

#[test]
fn test_topological_sort_self_cycle() {
    // Node depends on itself — should be detected as a cycle
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![create_test_node("a", vec!["a".to_string()])],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let result = executor.topological_sort();
    assert!(result.is_err());
}

#[test]
fn test_topological_sort_three_node_cycle() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec!["c".to_string()]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec!["b".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let result = executor.topological_sort();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn test_topological_sort_unreachable_node() {
    // Node b depends on a, but c is unreachable (no path from roots)
    // Actually: a has no deps, b depends on a. If we add c with no deps, c is reachable.
    // Unreachable: node that nothing points to AND doesn't have in_degree 0?
    // In Kahn's algorithm, unreachable nodes never get in_degree 0, so they're never processed.
    // Graph: a->b, c (isolated). a and c have in_degree 0. So both get processed in phase 1.
    // Then b gets processed. All 3 in phases. So actually that works.
    // True unreachable: d depends on c, c depends on b, b depends on a, but we also have e
    // with no deps. e is in phase 1. a is in phase 1. b in phase 2, c in phase 3, d in phase 4.
    // All 5 processed. OK.
    // Cycle with unreachable: a->b->a (cycle), c (isolated). Phase 1: c only (a and b have in_degree 1).
    // Then nothing else. Sum = 1 != 3. So we detect "cycles or unreachable".
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec!["b".to_string()]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec![]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let result = executor.topological_sort();
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cycle") || err_msg.contains("unreachable"),
        "Expected cycle or unreachable error, got: {err_msg}"
    );
}

#[test]
fn test_topological_sort_depends_on_missing_node_id() {
    // `ghost` is not in the graph — Kahn's algorithm never schedules nodes blocked on it.
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "missing dep".to_string(),
        nodes: vec![
            create_test_node("a", vec!["ghost".to_string()]),
            create_test_node("b", vec![]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let executor = GraphExecutor::new(graph, HashMap::new());
    let err = executor.topological_sort().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("cycle") || msg.contains("unreachable"),
        "unexpected: {msg}"
    );
}
