// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Parallel node execution and max_parallelism tests.

use super::create_test_node;
use super::super::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Semaphore limits concurrent node tasks (`max_parallelism`).
#[tokio::test]
async fn test_execute_parallel_nodes_with_max_parallelism_one() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let mut nodes = Vec::new();
    for i in 0..4 {
        let mut node = create_test_node(&format!("n{i}"), vec![]);
        node.operation = Some(crate::neural_graph::Operation {
            name: "log.info".to_string(),
            target: None,
            params: HashMap::new(),
            environment: None,
        });
        node.config.insert(
            "message".to_string(),
            serde_json::Value::String(format!("msg {i}")),
        );
        nodes.push(node);
    }
    let graph = Graph {
        id: "parallel-one".to_string(),
        version: "1.0".to_string(),
        description: "Parallelism cap".to_string(),
        nodes,
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SOCKET_DIR".to_string(),
        temp.path().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    executor.max_parallelism = 1;
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.phase_results[0].completed, 4);
}
