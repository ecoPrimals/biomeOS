// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2026 ecoPrimals Project

//! Static analysis helpers for pathway optimization suggestions.

use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::DeploymentGraph;
use crate::metrics::NodeMetricsAggregate;
use crate::pathway_learner::{OptimizationSuggestion, OptimizationType};

/// Find pairs of sequential nodes that can be parallelized.
pub(crate) fn find_parallelization_opportunities(
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
pub(crate) fn find_prewarm_candidates(
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
pub(crate) fn find_reorder_candidates(
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
pub(crate) fn find_cache_candidates(
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
pub(crate) fn find_batch_candidates(
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
