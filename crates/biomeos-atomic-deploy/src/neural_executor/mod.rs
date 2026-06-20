// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph executor for deterministic deployment orchestration
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics
//!
//! ## Architecture
//!
//! Uses shared types from `crate::executor`:
//! - `ExecutionContext`: Shared state across nodes
//! - `NodeStatus`: Node execution status
//! - `ExecutionReport`: Final execution report
//! - `PhaseResult`: Result from a single phase
//!
//! ## Deep Debt Principles
//!
//! - Capability-based discovery (no hardcoded primal names)
//! - Pure JSON-RPC communication (no HTTP in IPC)
//! - Runtime primal discovery (self-knowledge only)

mod dispatch;
mod phase;

use anyhow::Result;
use biomeos_graph::metrics::{GraphResult, MetricsCollector};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::capability_domains::CapabilityRegistry;
use crate::gate_registry::GateRegistry;
use crate::neural_graph::{Graph, GraphNode};

// Re-export from executor module (single source of truth)
pub use crate::executor::context::{ExecutionContext, NodeStatus};
pub use crate::executor::types::{ExecutionReport, GeneticsTierValidationReport, PhaseResult};

/// Graph executor
pub struct GraphExecutor {
    pub(crate) graph: Graph,
    pub(crate) context: ExecutionContext,
    pub(crate) max_parallelism: usize,
    metrics: Option<MetricsCollector>,
    gate_registry: Arc<GateRegistry>,
    pub(crate) capability_registry: Arc<CapabilityRegistry>,
    /// When set, metrics are recorded under this namespace prefix
    /// (e.g. "signal:tower.publish") so PathwayLearner can group
    /// signal executions separately from regular graph runs.
    signal_namespace: Option<String>,
}

impl GraphExecutor {
    /// Create new graph executor
    #[must_use]
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        let gate_registry = Arc::new(GateRegistry::from_graph_env(&env));
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3,
            metrics: None,
            gate_registry,
            capability_registry: Arc::new(CapabilityRegistry::default()),
            signal_namespace: None,
        }
    }

    /// Create graph executor with socket nucleation
    pub fn with_nucleation(
        graph: Graph,
        env: HashMap<String, String>,
        nucleation: Arc<tokio::sync::RwLock<crate::nucleation::SocketNucleation>>,
        tcp_only: bool,
    ) -> Self {
        let gate_registry = Arc::new(GateRegistry::from_graph_env(&env));
        let mut context = ExecutionContext::new(env).with_nucleation(nucleation);
        if tcp_only {
            context = context.with_tcp_only();
        }
        Self {
            graph,
            context,
            max_parallelism: 3,
            metrics: None,
            gate_registry,
            capability_registry: Arc::new(CapabilityRegistry::default()),
            signal_namespace: None,
        }
    }

    /// Attach a `MetricsCollector` for `PathwayLearner` integration.
    #[must_use]
    pub fn with_metrics(mut self, metrics: MetricsCollector) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Tag this execution with a signal namespace for PathwayLearner grouping.
    #[must_use]
    pub fn with_signal_namespace(mut self, namespace: String) -> Self {
        self.signal_namespace = Some(namespace);
        self
    }

    /// Set a TOML-loaded capability registry (overrides compiled-in fallback).
    #[must_use]
    pub(crate) fn with_capability_registry(mut self, registry: CapabilityRegistry) -> Self {
        self.capability_registry = Arc::new(registry);
        self
    }

    /// Set a custom gate registry (overrides env-derived registry).
    #[must_use]
    pub fn with_gate_registry(mut self, registry: GateRegistry) -> Self {
        self.gate_registry = Arc::new(registry);
        self
    }

    /// Execute the entire graph
    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);

        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(&self.graph.id);

        if let Some(tier) = self.graph.genetics_tier {
            let validation = GeneticsTierValidationReport::pending_bear_dog_probe(tier);
            warn!(
                graph_id = %self.graph.id,
                required_tier = %validation.required_tier,
                "Genetics tier preflight: family infrastructure not verified against declared tier (security provider genetics.tier_available pending)"
            );
            report.genetics_tier_validation = Some(validation);
        }

        let phases = self.topological_sort()?;

        // One deep clone per node into Arc; phase workers share the Arc (cheap ref-count),
        // avoiding per-spawn `GraphNode` clones from linear `Vec` scans.
        let node_map: HashMap<String, Arc<GraphNode>> = self
            .graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), Arc::new(n.clone())))
            .collect();

        info!("   Execution plan: {} phases", phases.len());

        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!(
                "📍 Phase {}/{}: {} nodes",
                phase_num + 1,
                phases.len(),
                phase_nodes.len()
            );

            match self.execute_phase(phase_num, phase_nodes, &node_map).await {
                Ok(phase_result) => {
                    let failed: HashSet<&str> = phase_result
                        .errors
                        .iter()
                        .map(|(id, _)| id.as_str())
                        .collect();
                    report.completed_nodes.extend(
                        phase_nodes
                            .iter()
                            .filter(|id| !failed.contains(id.as_str()))
                            .cloned(),
                    );
                    report.add_phase_result(&phase_result);
                    report.failed_nodes.extend(phase_result.errors);

                    let phase_failed = phase_result.failed > 0;
                    if phase_failed {
                        report.success = false;
                        report.error = Some(format!(
                            "Phase {} failed: {} nodes failed",
                            phase_num + 1,
                            phase_result.failed
                        ));

                        if self.graph.config.rollback_on_failure {
                            warn!("🔄 Rolling back deployment...");
                            self.rollback().await?;
                        }

                        break;
                    }
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    if self.graph.config.rollback_on_failure {
                        warn!("🔄 Rolling back deployment...");
                        self.rollback().await?;
                    }

                    break;
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;

        if report.success {
            info!("✅ Graph execution complete: {} ms", report.duration_ms);
        } else {
            error!("❌ Graph execution failed: {} ms", report.duration_ms);
        }

        if let Some(ref collector) = self.metrics {
            let metrics_graph_name: Cow<'_, str> = match &self.signal_namespace {
                Some(ns) => Cow::Owned(format!("{ns}:{}", self.graph.id)),
                None => Cow::Borrowed(self.graph.id.as_str()),
            };
            let graph_result = GraphResult {
                success: report.success,
                node_results: HashMap::new(),
                errors: report
                    .failed_nodes
                    .iter()
                    .map(|(id, err)| format!("{id}: {err}"))
                    .collect(),
                duration_ms: report.duration_ms,
            };
            if let Err(e) = collector.record_execution(
                &metrics_graph_name,
                &graph_result,
                report.duration_ms,
                None,
            ) {
                warn!("Failed to record graph metrics: {e}");
            }
        }

        Ok(report)
    }

    /// Split capability string into (domain, operation) for capability.call semantics.
    /// e.g. "`ecology.et0_fao56`" -> ("ecology", "`et0_fao56`"), "single" -> ("single", "execute")
    pub(crate) fn split_capability(capability: &str) -> (String, String) {
        if let Some(dot_pos) = capability.find('.') {
            (
                capability[..dot_pos].to_string(),
                capability[dot_pos + 1..].to_string(),
            )
        } else {
            (capability.to_string(), "execute".to_string())
        }
    }

    /// Substitute environment variables in a string (used by tests; production uses executor::substitute_env)
    #[cfg(test)]
    pub(crate) fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
        let mut result = s.to_string();

        for (key, value) in env {
            let placeholder = format!("${{{key}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Perform topological sort to determine execution phases.
    pub fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

        tracing::info!(
            "🔍 Building dependency graph for {} nodes...",
            self.graph.nodes.len()
        );
        for node in &self.graph.nodes {
            tracing::info!("   Node '{}' depends_on: {:?}", node.id, node.depends_on);
            in_degree.entry(node.id.clone()).or_insert(0);

            for dep in &node.depends_on {
                graph_map
                    .entry(dep.clone())
                    .or_default()
                    .push(node.id.clone());
                *in_degree.entry(node.id.clone()).or_insert(0) += 1;
            }
        }

        tracing::info!("🔍 In-degree calculation:");
        for (id, degree) in &in_degree {
            tracing::info!("   {} → in_degree={}", id, degree);
        }

        let mut phases = Vec::new();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while !queue.is_empty() {
            let mut current_phase = Vec::new();
            let phase_size = queue.len();

            for _ in 0..phase_size {
                if let Some(node_id) = queue.pop_front() {
                    if let Some(dependents) = graph_map.get(&node_id) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(dependent.clone());
                                }
                            }
                        }
                    }
                    current_phase.push(node_id);
                }
            }

            if !current_phase.is_empty() {
                phases.push(current_phase);
            }
        }

        if phases.iter().map(|p| p.len()).sum::<usize>() != self.graph.nodes.len() {
            anyhow::bail!("Graph contains cycles or unreachable nodes");
        }

        Ok(phases)
    }

    /// Persist current `outputs` and `status` to `checkpoint_dir` before a phase runs.
    async fn save_checkpoint_before_phase(&self, phase_index: usize) -> Result<()> {
        if self.context.checkpoint_dir.is_none() {
            return Ok(());
        }
        self.context.save_checkpoint().await?;
        info!(
            "💾 Checkpoint saved before phase {} (execution_state.json)",
            phase_index + 1
        );
        Ok(())
    }

    /// Reload node `outputs` and `status` from `execution_state.json` under `checkpoint_dir`.
    pub async fn restore_from_checkpoint(&mut self) -> Result<()> {
        self.context.load_checkpoint().await
    }
}
// Node executor implementations (verification, health_check_all, rpc_call,
// capability_call, send_jsonrpc_async) live in neural_executor_node_impls.rs.
// Rollback (resolve_node_type, rollback, rollback_primal_lifecycle,
// rollback_register_capabilities) lives in neural_executor_rollback.rs.
// Tests are in neural_executor_tests.rs.
