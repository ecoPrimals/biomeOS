// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::create_test_node;
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_graph_executor_creation() {
    let graph = Graph {
        id: "test-graph".to_string(),
        version: "1.0.0".to_string(),
        description: "Test graph".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    assert_eq!(executor.max_parallelism, 3);
}

#[tokio::test]
async fn test_execution_context_with_nucleation() {
    use crate::nucleation::SocketNucleation;

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
    let nucleation = Arc::new(tokio::sync::RwLock::new(SocketNucleation::default()));

    let executor = GraphExecutor::with_nucleation(graph, env, nucleation, false);
    assert_eq!(executor.max_parallelism, 3);
}

#[test]
fn test_graph_config_default() {
    let config = GraphConfig::default();
    assert!(config.deterministic);
    assert!(config.parallel_phases);
    assert_eq!(config.max_parallelism, 3);
}

#[test]
fn test_graph_config_custom() {
    let config = GraphConfig {
        deterministic: false,
        parallel_phases: false,
        max_parallelism: 10,
        rollback_on_failure: true,
        ..Default::default()
    };
    assert!(!config.deterministic);
    assert!(!config.parallel_phases);
    assert_eq!(config.max_parallelism, 10);
    assert!(config.rollback_on_failure);
}

#[test]
fn test_executor_with_custom_env() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "custom-family".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp/test".to_string());

    let graph = Graph {
        id: "env-test".to_string(),
        version: "1.0".to_string(),
        description: "test with env".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };

    let executor = GraphExecutor::new(graph, env);
    assert_eq!(executor.max_parallelism, 3);
}
