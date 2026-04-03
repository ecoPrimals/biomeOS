// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2026 ecoPrimals Project

//! Pathway Learner — adaptive graph optimization via execution history.
//!
//! Learns coordination patterns from accumulated execution metrics and
//! proposes optimizations per the neuralAPI whitepaper specification:
//!
//! - **Parallelization**: Identify sequential node pairs with no data
//!   dependencies that can execute concurrently.
//! - **Prewarming**: Primals invoked frequently together should be
//!   pre-launched to reduce cold-start latency.
//! - **Batching**: Multiple operations to the same primal can be batched
//!   into a single RPC call.
//! - **Caching**: Pure nodes with identical inputs can reuse previous output.
//! - **Reordering**: Move latency-heavy nodes earlier to overlap with I/O.
//!
//! ## Architecture
//!
//! ```text
//! MetricsCollector → PathwayLearner → OptimizationSuggestion[]
//!                                            ↓
//!                                     GraphExecutor applies
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::graph::DeploymentGraph;
use crate::metrics::{MetricsCollector, NodeMetricsAggregate};

fn serialize_arc_str<S>(s: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.as_ref().serialize(serializer)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

/// Type of optimization the Pathway Learner can suggest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptimizationType {
    /// Run two sequential nodes concurrently.
    Parallelize {
        /// First node to run in parallel.
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        node_a: Arc<str>,
        /// Second node to run in parallel.
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        node_b: Arc<str>,
    },
    /// Pre-warm a primal before graph execution starts.
    Prewarm {
        /// Primal to pre-warm.
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        primal: Arc<str>,
    },
    /// Batch operations to the same primal.
    Batch {
        /// Target primal.
        #[serde(
            serialize_with = "serialize_arc_str",
            deserialize_with = "deserialize_arc_str"
        )]
        primal: Arc<str>,
        /// Nodes that can be batched.
        nodes: Vec<String>,
    },
    /// Cache a node's output (pure function, same inputs = same output).
    Cache {
        /// Node to cache.
        node_id: String,
    },
    /// Reorder a node to start earlier.
    Reorder {
        /// Node to move.
        node_id: String,
        /// Suggested new position (phase index).
        suggested_phase: usize,
    },
}

/// A single optimization suggestion with estimated impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// The optimization type and parameters.
    pub optimization: OptimizationType,
    /// Estimated speedup ratio (1.0 = no change, 1.5 = 50% faster).
    pub estimated_speedup: f64,
    /// Confidence in this suggestion (0.0 to 1.0).
    pub confidence: f64,
    /// Human-readable explanation.
    pub reason: String,
}

/// Analysis result for a single graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Graph identifier.
    pub graph_id: String,
    /// Suggested optimizations, sorted by estimated impact.
    pub suggestions: Vec<OptimizationSuggestion>,
    /// Number of executions analyzed.
    pub sample_size: u64,
}

/// Pathway Learner — learns from execution history and suggests optimizations.
pub struct PathwayLearner {
    metrics: MetricsCollector,
    min_samples: u64,
}

impl PathwayLearner {
    /// Create a new Pathway Learner backed by the given metrics collector.
    #[must_use]
    pub const fn new(metrics: MetricsCollector, min_samples: u64) -> Self {
        Self {
            metrics,
            min_samples,
        }
    }

    /// Analyze a graph and return optimization suggestions.
    ///
    /// Requires at least `min_samples` executions to have been recorded
    /// before producing suggestions.
    pub async fn analyze(&self, graph: &DeploymentGraph) -> GraphAnalysis {
        let graph_id = graph.definition.id.as_str().to_string();
        let node_metrics = self.collect_node_metrics(graph).await;

        let sample_size = node_metrics
            .values()
            .map(|m| m.total_executions)
            .max()
            .unwrap_or(0);

        if sample_size < self.min_samples {
            return GraphAnalysis {
                graph_id,
                suggestions: Vec::new(),
                sample_size,
            };
        }

        let mut suggestions = Vec::new();

        suggestions.extend(Self::find_parallelization_opportunities(
            graph,
            &node_metrics,
        ));
        suggestions.extend(Self::find_prewarm_candidates(graph, &node_metrics));
        suggestions.extend(Self::find_batch_candidates(graph, &node_metrics));
        suggestions.extend(Self::find_reorder_candidates(graph, &node_metrics));
        suggestions.extend(Self::find_cache_candidates(graph, &node_metrics));

        suggestions.sort_by(|a, b| {
            b.estimated_speedup
                .partial_cmp(&a.estimated_speedup)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        GraphAnalysis {
            graph_id,
            suggestions,
            sample_size,
        }
    }

    /// Collect per-node metrics by querying the store for each node in the graph.
    async fn collect_node_metrics(
        &self,
        graph: &DeploymentGraph,
    ) -> HashMap<String, NodeMetricsAggregate> {
        let graph_id = graph.definition.id.as_str();
        let mut out = HashMap::new();

        for node in &graph.definition.nodes {
            let nid = node.id.as_str();
            if let Ok(Some(m)) = self.metrics.get_node_metrics(graph_id, nid) {
                out.insert(nid.to_string(), m);
            }
        }

        out
    }

    /// Find pairs of sequential nodes that can be parallelized.
    fn find_parallelization_opportunities(
        graph: &DeploymentGraph,
        node_metrics: &HashMap<String, NodeMetricsAggregate>,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        let nodes = &graph.definition.nodes;

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let a = &nodes[i];
                let b = &nodes[j];

                let a_id = a.id.as_str();
                let b_id = b.id.as_str();

                let a_depends_on_b = a.depends_on.iter().any(|d| d == b_id);
                let b_depends_on_a = b.depends_on.iter().any(|d| d == a_id);

                if a_depends_on_b || b_depends_on_a {
                    continue;
                }

                let a_lat = node_metrics.get(a_id).map_or(0.0, |m| m.avg_duration_ms);
                let b_lat = node_metrics.get(b_id).map_or(0.0, |m| m.avg_duration_ms);

                if a_lat + b_lat < 1.0 {
                    continue;
                }

                let sequential = a_lat + b_lat;
                let parallel = a_lat.max(b_lat);
                let speedup = sequential / parallel;

                if speedup > 1.1 {
                    suggestions.push(OptimizationSuggestion {
                        optimization: OptimizationType::Parallelize {
                            node_a: Arc::from(a_id),
                            node_b: Arc::from(b_id),
                        },
                        estimated_speedup: speedup,
                        confidence: 0.8,
                        reason: format!(
                            "{a_id} ({a_lat:.1}ms) and {b_id} ({b_lat:.1}ms) have no dependency — \
                             parallel saves {:.1}ms",
                            sequential - parallel
                        ),
                    });
                }
            }
        }

        suggestions
    }

    /// Identify primals that appear frequently and could benefit from prewarming.
    fn find_prewarm_candidates(
        graph: &DeploymentGraph,
        node_metrics: &HashMap<String, NodeMetricsAggregate>,
    ) -> Vec<OptimizationSuggestion> {
        let mut primal_latency: HashMap<String, f64> = HashMap::new();

        for node in &graph.definition.nodes {
            if let Some(primal) = &node.config.primal {
                let latency = node_metrics
                    .get(node.id.as_str())
                    .map_or(0.0, |m| m.avg_duration_ms);

                let entry = primal_latency.entry(primal.clone()).or_default();
                *entry = entry.max(latency);
            }
        }

        primal_latency
            .iter()
            .filter(|(_, lat)| **lat > 50.0)
            .map(|(primal, lat)| OptimizationSuggestion {
                optimization: OptimizationType::Prewarm {
                    primal: Arc::from(primal.as_str()),
                },
                estimated_speedup: 1.0 + (lat / 1000.0).min(0.5),
                confidence: 0.6,
                reason: format!(
                    "{primal} has avg first-call latency of {lat:.1}ms — \
                     prewarming could eliminate cold start"
                ),
            })
            .collect()
    }

    /// Find nodes with declared cost estimates that could benefit from reordering.
    ///
    /// Expensive nodes (high `cost_estimate_ms`) with no dependents should be
    /// moved to early phases so their I/O overlaps with lighter work.
    fn find_reorder_candidates(
        graph: &DeploymentGraph,
        node_metrics: &HashMap<String, NodeMetricsAggregate>,
    ) -> Vec<OptimizationSuggestion> {
        let nodes = &graph.definition.nodes;

        let dependent_set: std::collections::HashSet<&str> = nodes
            .iter()
            .flat_map(|n| n.depends_on.iter().map(String::as_str))
            .collect();

        nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.cost_estimate_ms.is_some_and(|c| c > 100)
                    && !dependent_set.contains(node.id.as_str())
            })
            .filter_map(|(idx, node)| {
                let declared_cost = node.cost_estimate_ms?;
                let actual_avg = node_metrics
                    .get(node.id.as_str())
                    .map(|m| m.avg_duration_ms as u64);
                let cost = actual_avg.unwrap_or(declared_cost);

                if cost > 100 && idx > 0 {
                    Some(OptimizationSuggestion {
                        optimization: OptimizationType::Reorder {
                            node_id: node.id.as_str().to_string(),
                            suggested_phase: 0,
                        },
                        estimated_speedup: 1.0 + (cost as f64 / 2000.0).min(0.3),
                        confidence: if actual_avg.is_some() { 0.7 } else { 0.4 },
                        reason: format!(
                            "{} has cost {}ms (declared: {}ms) — moving earlier overlaps I/O",
                            node.id.as_str(),
                            cost,
                            declared_cost
                        ),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find pure nodes (no side effects) whose output can be cached.
    ///
    /// Nodes with high execution count and consistent success rate are good
    /// cache candidates, especially if they have no `operation_dependencies`
    /// (indicating they're referentially transparent).
    fn find_cache_candidates(
        graph: &DeploymentGraph,
        node_metrics: &HashMap<String, NodeMetricsAggregate>,
    ) -> Vec<OptimizationSuggestion> {
        graph
            .definition
            .nodes
            .iter()
            .filter(|node| node.operation_dependencies.is_empty())
            .filter_map(|node| {
                let metrics = node_metrics.get(node.id.as_str())?;
                if metrics.total_executions >= 10
                    && metrics.success_rate > 0.99
                    && metrics.avg_duration_ms > 5.0
                {
                    Some(OptimizationSuggestion {
                        optimization: OptimizationType::Cache {
                            node_id: node.id.as_str().to_string(),
                        },
                        estimated_speedup: 1.0 + (metrics.avg_duration_ms / 500.0).min(0.8),
                        confidence: metrics.success_rate * 0.9,
                        reason: format!(
                            "{} is pure (no op_deps), {:.1}ms avg, {:.0}% success over {} runs — \
                             safe to cache",
                            node.id.as_str(),
                            metrics.avg_duration_ms,
                            metrics.success_rate * 100.0,
                            metrics.total_executions
                        ),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find nodes targeting the same primal that could be batched.
    fn find_batch_candidates(
        graph: &DeploymentGraph,
        node_metrics: &HashMap<String, NodeMetricsAggregate>,
    ) -> Vec<OptimizationSuggestion> {
        let mut primal_nodes: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.definition.nodes {
            if let Some(primal) = &node.config.primal {
                primal_nodes
                    .entry(primal.clone())
                    .or_default()
                    .push(node.id.as_str().to_string());
            }
        }

        primal_nodes
            .into_iter()
            .filter(|(_, nodes)| nodes.len() >= 2)
            .filter(|(_, nodes)| {
                nodes.iter().any(|n| {
                    node_metrics
                        .get(n.as_str())
                        .is_some_and(|m| m.total_executions > 0)
                })
            })
            .map(|(primal, nodes)| {
                let total_lat: f64 = nodes
                    .iter()
                    .filter_map(|n| node_metrics.get(n.as_str()).map(|m| m.avg_duration_ms))
                    .sum();
                let batch_lat = total_lat * 0.6;
                let speedup = if batch_lat > 0.0 {
                    total_lat / batch_lat
                } else {
                    1.0
                };

                OptimizationSuggestion {
                    optimization: OptimizationType::Batch {
                        primal: Arc::from(primal.as_str()),
                        nodes: nodes.clone(),
                    },
                    estimated_speedup: speedup,
                    confidence: 0.5,
                    reason: format!(
                        "{} nodes target {primal} — batching could reduce RPC overhead \
                         ({total_lat:.1}ms → {batch_lat:.1}ms est.)",
                        nodes.len()
                    ),
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use crate::graph::{DeploymentGraph, GraphDefinition, GraphId, GraphMetadata};
    use crate::node::{GraphNode, NodeConfig, NodeId, NodeParams, NodeType};
    use std::collections::HashMap;

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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_parallelization_opportunities(&graph, &metrics);

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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_parallelization_opportunities(&graph, &metrics);
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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_batch_candidates(&graph, &metrics);
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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_prewarm_candidates(&graph, &metrics);
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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_prewarm_candidates(&graph, &metrics);
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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_reorder_candidates(&graph, &metrics);

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
        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_reorder_candidates(&graph, &metrics);

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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_cache_candidates(&graph, &metrics);

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

        let _learner = make_test_learner(0);
        let suggestions = PathwayLearner::find_cache_candidates(&graph, &metrics);

        assert!(
            suggestions.is_empty(),
            "node with operation_dependencies should not be cached"
        );
    }

    fn make_test_learner(min_samples: u64) -> PathwayLearner {
        let dir = tempfile::tempdir().unwrap();
        let metrics = MetricsCollector::new(dir.path().join("test-metrics.redb")).unwrap();
        // Leak the tempdir so it lives long enough for the test.
        std::mem::forget(dir);
        PathwayLearner::new(metrics, min_samples)
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
        let suggestions = PathwayLearner::find_parallelization_opportunities(&graph, &metrics);
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
        let suggestions = PathwayLearner::find_parallelization_opportunities(&graph, &metrics);
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
        let suggestions = PathwayLearner::find_batch_candidates(&graph, &metrics);
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
        let suggestions = PathwayLearner::find_cache_candidates(&graph, &metrics);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn reorder_skips_when_node_already_first_phase() {
        let mut expensive = make_node("root", vec![], Some("p"));
        expensive.cost_estimate_ms = Some(500);
        let graph = make_graph(vec![expensive]);
        let metrics = HashMap::from([("root".to_string(), make_node_metrics("root", 20, 200.0))]);
        let suggestions = PathwayLearner::find_reorder_candidates(&graph, &metrics);
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
        let suggestions = PathwayLearner::find_prewarm_candidates(&graph, &metrics);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].reason.contains("shared"));
    }
}
