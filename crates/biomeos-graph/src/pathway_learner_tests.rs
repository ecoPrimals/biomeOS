// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2026 ecoPrimals Project

//! Tests for [`crate::pathway_learner::PathwayLearner`] and pathway analysis.

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::{DeploymentGraph, GraphDefinition, GraphId, GraphMetadata};
use crate::metrics::{MetricsCollector, NodeMetricsAggregate};
use crate::node::{GraphNode, NodeConfig, NodeId, NodeParams, NodeType};
use crate::pathway_analysis;
use crate::pathway_learner::{
    GraphAnalysis, OptimizationSuggestion, OptimizationType, PathwayLearner,
};

fn test_graph_id(id: &str) -> GraphId {
    GraphId::new(id).unwrap()
}

fn test_node_id(id: &str) -> NodeId {
    NodeId::new(id).unwrap()
}

fn make_graph(nodes: Vec<GraphNode>) -> DeploymentGraph {
    DeploymentGraph {
        definition: GraphDefinition {
            id: test_graph_id("test-graph"),
            name: "Test Graph".to_string(),
            version: "1.0.0".to_string(),
            description: "test".to_string(),
            coordination: crate::graph::CoordinationPattern::Sequential,
            tick: None,
            metadata: GraphMetadata::default(),
            env: HashMap::default(),
            nodes,
            outputs: HashMap::default(),
        },
    }
}

fn make_node(id: &str, depends_on: Vec<&str>, primal: Option<&str>) -> GraphNode {
    GraphNode {
        id: test_node_id(id),
        name: id.to_string(),
        node_type: NodeType::default(),
        capability: Some(format!("test.{id}")),
        required: true,
        order: 0,
        depends_on: depends_on.into_iter().map(String::from).collect(),
        condition: None,
        config: NodeConfig {
            primal: primal.map(String::from),
            skip_if: None,
            retry_count: None,
            timeout_secs: None,
            extra: HashMap::default(),
        },
        params: NodeParams::default(),
        feedback_to: None,
        budget_ms: None,
        fallback: None,
        cost_estimate_ms: None,
        operation_dependencies: Vec::new(),
        gate: None,
    }
}

fn make_node_metrics(node_id: &str, executions: u64, avg_ms: f64) -> NodeMetricsAggregate {
    NodeMetricsAggregate {
        node_id: node_id.to_string(),
        total_executions: executions,
        successful_executions: executions,
        avg_duration_ms: avg_ms,
        success_rate: 1.0,
    }
}

fn make_test_learner(min_samples: u64) -> (PathwayLearner, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let metrics = MetricsCollector::new(dir.path().join("test-metrics.redb")).unwrap();
    (PathwayLearner::new(metrics, min_samples), dir)
}

#[test]
fn parallelization_detects_independent_nodes() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("p1")),
        make_node("b", vec![], Some("p2")),
        make_node("c", vec!["a", "b"], Some("p1")),
    ]);

    let metrics = HashMap::from([
        ("a".to_string(), make_node_metrics("a", 100, 50.0)),
        ("b".to_string(), make_node_metrics("b", 100, 30.0)),
    ]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_parallelization_opportunities(&graph, &metrics);

    assert!(!suggestions.is_empty(), "should find a parallelization");
    let s = &suggestions[0];
    match &s.optimization {
        OptimizationType::Parallelize { node_a, node_b } => {
            assert_eq!(node_a.as_ref(), "a");
            assert_eq!(node_b.as_ref(), "b");
        }
        other => panic!("expected Parallelize, got {other:?}"),
    }
    assert!(s.estimated_speedup > 1.0);
}

#[test]
fn parallelization_skips_dependent_nodes() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("p1")),
        make_node("b", vec!["a"], Some("p2")),
    ]);

    let metrics = HashMap::from([
        ("a".to_string(), make_node_metrics("a", 100, 50.0)),
        ("b".to_string(), make_node_metrics("b", 100, 30.0)),
    ]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_parallelization_opportunities(&graph, &metrics);
    assert!(
        suggestions.is_empty(),
        "dependent nodes should not be parallelized"
    );
}

#[test]
fn batch_candidates_group_by_primal() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("rhizocrypt")),
        make_node("b", vec![], Some("rhizocrypt")),
        make_node("c", vec![], Some("loamspine")),
    ]);

    let metrics = HashMap::from([
        ("a".to_string(), make_node_metrics("a", 50, 10.0)),
        ("b".to_string(), make_node_metrics("b", 50, 10.0)),
        ("c".to_string(), make_node_metrics("c", 50, 10.0)),
    ]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_batch_candidates(&graph, &metrics);
    assert_eq!(suggestions.len(), 1, "only rhizocrypt has 2+ nodes");
    match &suggestions[0].optimization {
        OptimizationType::Batch { primal, nodes } => {
            assert_eq!(primal.as_ref(), "rhizocrypt");
            assert_eq!(nodes.len(), 2);
        }
        other => panic!("expected Batch, got {other:?}"),
    }
}

#[test]
fn prewarm_detects_high_latency_primals() {
    let graph = make_graph(vec![make_node("a", vec![], Some("beardog"))]);

    let metrics = HashMap::from([("a".to_string(), make_node_metrics("a", 100, 200.0))]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_prewarm_candidates(&graph, &metrics);
    assert_eq!(suggestions.len(), 1);
    match &suggestions[0].optimization {
        OptimizationType::Prewarm { primal } => assert_eq!(primal.as_ref(), "beardog"),
        other => panic!("expected Prewarm, got {other:?}"),
    }
}

#[test]
fn prewarm_ignores_low_latency_primals() {
    let graph = make_graph(vec![make_node("a", vec![], Some("speedy"))]);

    let metrics = HashMap::from([("a".to_string(), make_node_metrics("a", 100, 5.0))]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_prewarm_candidates(&graph, &metrics);
    assert!(suggestions.is_empty(), "5ms is below the 50ms threshold");
}

#[test]
fn optimization_suggestion_round_trips_json() {
    let suggestion = OptimizationSuggestion {
        optimization: OptimizationType::Parallelize {
            node_a: Arc::from("a"),
            node_b: Arc::from("b"),
        },
        estimated_speedup: 1.6,
        confidence: 0.8,
        reason: "test".to_string(),
    };
    let json = serde_json::to_string(&suggestion).unwrap();
    let parsed: OptimizationSuggestion = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.optimization, suggestion.optimization);
}

#[tokio::test]
async fn analyze_requires_minimum_samples() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("p1")),
        make_node("b", vec![], Some("p2")),
    ]);

    let dir = tempfile::tempdir().unwrap();
    let metrics = MetricsCollector::new(dir.path().join("test.redb")).unwrap();
    let learner = PathwayLearner::new(metrics, 1000);

    let analysis = learner.analyze(&graph).await;
    assert!(
        analysis.suggestions.is_empty(),
        "no metrics recorded = under min_samples"
    );
    assert_eq!(analysis.sample_size, 0);
}

#[test]
fn graph_analysis_round_trips_json() {
    let analysis = GraphAnalysis {
        graph_id: "test-graph".to_string(),
        suggestions: vec![],
        sample_size: 42,
    };
    let json = serde_json::to_string(&analysis).unwrap();
    let parsed: GraphAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.graph_id, "test-graph");
    assert_eq!(parsed.sample_size, 42);
}

#[test]
fn reorder_detects_expensive_non_dependent_nodes() {
    let mut expensive_node = make_node("expensive", vec![], Some("toadstool"));
    expensive_node.cost_estimate_ms = Some(500);

    let graph = make_graph(vec![
        make_node("a", vec![], Some("p1")),
        expensive_node,
        make_node("c", vec!["a"], Some("p2")),
    ]);

    let metrics = HashMap::from([
        ("a".to_string(), make_node_metrics("a", 100, 10.0)),
        (
            "expensive".to_string(),
            make_node_metrics("expensive", 100, 450.0),
        ),
        ("c".to_string(), make_node_metrics("c", 100, 20.0)),
    ]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_reorder_candidates(&graph, &metrics);

    assert!(
        !suggestions.is_empty(),
        "should suggest reordering expensive node"
    );
    match &suggestions[0].optimization {
        OptimizationType::Reorder {
            node_id,
            suggested_phase,
        } => {
            assert_eq!(node_id, "expensive");
            assert_eq!(*suggested_phase, 0);
        }
        other => panic!("expected Reorder, got {other:?}"),
    }
}

#[test]
fn reorder_ignores_cheap_nodes() {
    let mut cheap_node = make_node("cheap", vec![], Some("p1"));
    cheap_node.cost_estimate_ms = Some(10);

    let graph = make_graph(vec![make_node("a", vec![], Some("p1")), cheap_node]);

    let metrics = HashMap::new();
    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_reorder_candidates(&graph, &metrics);

    assert!(suggestions.is_empty(), "10ms is below 100ms threshold");
}

#[test]
fn cache_detects_pure_high_success_nodes() {
    let graph = make_graph(vec![make_node("pure-hash", vec![], Some("rhizocrypt"))]);

    let metrics = HashMap::from([(
        "pure-hash".to_string(),
        NodeMetricsAggregate {
            node_id: "pure-hash".to_string(),
            total_executions: 50,
            successful_executions: 50,
            avg_duration_ms: 30.0,
            success_rate: 1.0,
        },
    )]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_cache_candidates(&graph, &metrics);

    assert_eq!(suggestions.len(), 1);
    match &suggestions[0].optimization {
        OptimizationType::Cache { node_id } => assert_eq!(node_id, "pure-hash"),
        other => panic!("expected Cache, got {other:?}"),
    }
}

#[test]
fn cache_ignores_nodes_with_operation_dependencies() {
    let mut impure_node = make_node("side-effect", vec![], Some("p1"));
    impure_node.operation_dependencies = vec!["storage.write".to_string()];

    let graph = make_graph(vec![impure_node]);

    let metrics = HashMap::from([(
        "side-effect".to_string(),
        make_node_metrics("side-effect", 50, 30.0),
    )]);

    let (_learner, _dir) = make_test_learner(0);
    let suggestions = pathway_analysis::find_cache_candidates(&graph, &metrics);

    assert!(
        suggestions.is_empty(),
        "node with operation_dependencies should not be cached"
    );
}

#[test]
fn parallelization_skips_low_combined_latency() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("p1")),
        make_node("b", vec![], Some("p2")),
    ]);
    let metrics = HashMap::from([
        ("a".to_string(), make_node_metrics("a", 10, 0.2)),
        ("b".to_string(), make_node_metrics("b", 10, 0.3)),
    ]);
    let suggestions = pathway_analysis::find_parallelization_opportunities(&graph, &metrics);
    assert!(
        suggestions.is_empty(),
        "combined latency < 1ms should not suggest parallelization"
    );
}

#[test]
fn parallelization_speedup_cap_when_equal_latency() {
    let graph = make_graph(vec![
        make_node("x", vec![], Some("p1")),
        make_node("y", vec![], Some("p2")),
    ]);
    let metrics = HashMap::from([
        ("x".to_string(), make_node_metrics("x", 50, 60.0)),
        ("y".to_string(), make_node_metrics("y", 50, 60.0)),
    ]);
    let suggestions = pathway_analysis::find_parallelization_opportunities(&graph, &metrics);
    assert!(!suggestions.is_empty());
    let s = &suggestions[0];
    assert!(s.estimated_speedup > 1.1);
}

#[test]
fn batch_speedup_one_when_total_latency_zero() {
    let graph = make_graph(vec![
        make_node("a", vec![], Some("same")),
        make_node("b", vec![], Some("same")),
    ]);
    let metrics = HashMap::new();
    let suggestions = pathway_analysis::find_batch_candidates(&graph, &metrics);
    assert!(suggestions.is_empty() || suggestions[0].estimated_speedup >= 1.0);
}

#[test]
fn cache_rejects_low_execution_count() {
    let graph = make_graph(vec![make_node("pure", vec![], Some("p"))]);
    let metrics = HashMap::from([(
        "pure".to_string(),
        NodeMetricsAggregate {
            node_id: "pure".to_string(),
            total_executions: 5,
            successful_executions: 5,
            avg_duration_ms: 50.0,
            success_rate: 1.0,
        },
    )]);
    let suggestions = pathway_analysis::find_cache_candidates(&graph, &metrics);
    assert!(suggestions.is_empty());
}

#[test]
fn reorder_skips_when_node_already_first_phase() {
    let mut expensive = make_node("root", vec![], Some("p"));
    expensive.cost_estimate_ms = Some(500);
    let graph = make_graph(vec![expensive]);
    let metrics = HashMap::from([("root".to_string(), make_node_metrics("root", 20, 200.0))]);
    let suggestions = pathway_analysis::find_reorder_candidates(&graph, &metrics);
    assert!(
        suggestions.is_empty(),
        "idx 0 should not reorder (no earlier phase)"
    );
}

#[test]
fn prewarm_picks_max_latency_per_primal() {
    let graph = make_graph(vec![
        make_node("n1", vec![], Some("shared")),
        make_node("n2", vec![], Some("shared")),
    ]);
    let metrics = HashMap::from([
        ("n1".to_string(), make_node_metrics("n1", 10, 60.0)),
        ("n2".to_string(), make_node_metrics("n2", 10, 120.0)),
    ]);
    let suggestions = pathway_analysis::find_prewarm_candidates(&graph, &metrics);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].reason.contains("shared"));
}
