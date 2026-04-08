// SPDX-License-Identifier: AGPL-3.0-or-later
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
use crate::pathway_analysis;

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

        suggestions.extend(pathway_analysis::find_parallelization_opportunities(
            graph,
            &node_metrics,
        ));
        suggestions.extend(pathway_analysis::find_prewarm_candidates(
            graph,
            &node_metrics,
        ));
        suggestions.extend(pathway_analysis::find_batch_candidates(
            graph,
            &node_metrics,
        ));
        suggestions.extend(pathway_analysis::find_reorder_candidates(
            graph,
            &node_metrics,
        ));
        suggestions.extend(pathway_analysis::find_cache_candidates(
            graph,
            &node_metrics,
        ));

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
}
