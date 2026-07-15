// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{node_fs_check, node_log_info};
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig, GraphNode, Operation};
use biomeos_graph::GeneticsTier;
use std::collections::HashMap;

#[tokio::test]
async fn test_execute_single_log_info_node_succeeds() {
    let graph = Graph {
        id: "exec-log".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![node_log_info("n1", vec![], "hello")],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute");
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_two_phase_log_chain() {
    let graph = Graph {
        id: "exec-chain".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            node_log_info("a", vec![], "first"),
            node_log_info("b", vec!["a".to_string()], "second"),
        ],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute");
    assert!(report.success);
    assert_eq!(report.phase_results.len(), 2);
}

#[tokio::test]
async fn test_execute_filesystem_missing_path_fails() {
    let graph = Graph {
        id: "exec-fail".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![node_fs_check("bad", false)],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute returns report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_optional_filesystem_skip_keeps_success() {
    let graph = Graph {
        id: "exec-opt".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![node_fs_check("skipme", true)],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute");
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_unknown_operation_yields_skipped_json() {
    let graph = Graph {
        id: "exec-unknown".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![GraphNode {
            id: "u1".to_string(),
            depends_on: vec![],
            operation: Some(Operation {
                name: "not.a.real.handler".to_string(),
                target: None,
                params: HashMap::new(),
                environment: None,
            }),
            ..Default::default()
        }],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute");
    assert!(report.success);
}

#[tokio::test]
async fn execute_records_genetics_tier_preflight_in_report() {
    let graph = Graph {
        id: "tier-test".to_string(),
        version: "1.0.0".to_string(),
        description: String::new(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: Some(GeneticsTier::Nuclear),
        composition_model: None,
    };
    let mut ex = GraphExecutor::new(graph, HashMap::new());
    let report = ex.execute().await.expect("execute");
    let v = report
        .genetics_tier_validation
        .as_ref()
        .expect("genetics tier preflight should be recorded");
    assert_eq!(v.required_tier, "nuclear");
    assert!(!v.infrastructure_verified);
    assert!(report.success);
}
