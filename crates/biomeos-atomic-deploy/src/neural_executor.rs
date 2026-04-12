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

use anyhow::{Context, Result};
use biomeos_graph::metrics::{GraphResult, MetricsCollector, NodeExecutionParams};
use biomeos_types::JsonRpcRequest;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::capability_domains::CapabilityRegistry;
use crate::gate_registry::GateRegistry;
use crate::neural_graph::{Graph, GraphNode};

// Re-export from executor module (single source of truth)
// This eliminates duplicate type definitions and ensures consistency
pub use crate::executor::context::{ExecutionContext, NodeStatus};
pub use crate::executor::types::{ExecutionReport, PhaseResult};

/// Graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    pub(crate) max_parallelism: usize,
    metrics: Option<MetricsCollector>,
    gate_registry: Arc<GateRegistry>,
    pub(crate) capability_registry: Arc<CapabilityRegistry>,
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
        }
    }

    /// Create graph executor with socket nucleation
    pub fn with_nucleation(
        graph: Graph,
        env: HashMap<String, String>,
        nucleation: Arc<tokio::sync::RwLock<crate::nucleation::SocketNucleation>>,
    ) -> Self {
        let gate_registry = Arc::new(GateRegistry::from_graph_env(&env));
        Self {
            graph,
            context: ExecutionContext::new(env).with_nucleation(nucleation),
            max_parallelism: 3,
            metrics: None,
            gate_registry,
            capability_registry: Arc::new(CapabilityRegistry::default()),
        }
    }

    /// Attach a `MetricsCollector` for `PathwayLearner` integration.
    ///
    /// When set, the executor records per-node and per-graph execution metrics
    /// so the `PathwayLearner` can analyze and suggest optimizations.
    #[must_use]
    pub fn with_metrics(mut self, metrics: MetricsCollector) -> Self {
        self.metrics = Some(metrics);
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
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Topological sort to get execution phases
        let phases = self.topological_sort()?;

        info!("   Execution plan: {} phases", phases.len());

        // Execute each phase
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!(
                "📍 Phase {}/{}: {} nodes",
                phase_num + 1,
                phases.len(),
                phase_nodes.len()
            );

            match self.execute_phase(phase_num, phase_nodes).await {
                Ok(phase_result) => {
                    report.add_phase_result(&phase_result);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Rollback if enabled
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

        // Record graph-level metrics for PathwayLearner
        if let Some(ref collector) = self.metrics {
            let graph_result = GraphResult {
                success: report.success,
                node_results: HashMap::new(),
                errors: vec![],
                duration_ms: report.duration_ms,
            };
            if let Err(e) =
                collector.record_execution(&self.graph.id, &graph_result, report.duration_ms, None)
            {
                warn!("Failed to record graph metrics: {e}");
            }
        }

        Ok(report)
    }

    /// Execute a single phase (parallel execution of independent nodes)
    async fn execute_phase(&mut self, phase_index: usize, nodes: &[String]) -> Result<PhaseResult> {
        self.save_checkpoint_before_phase(phase_index).await?;

        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        // Semaphore for max parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));

        // Execute nodes in parallel
        let mut handles = Vec::new();

        let execution_id = chrono::Utc::now().timestamp_millis();

        for node_id in nodes {
            let node = self
                .graph
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {node_id}"))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            let gate_reg = self.gate_registry.clone();
            let cap_reg = self.capability_registry.clone();

            let handle = tokio::spawn(async move {
                let node_start = std::time::Instant::now();
                let result = Self::execute_node(&node, &context, &gate_reg, &cap_reg).await;
                let duration_ms = node_start.elapsed().as_millis() as u64;
                drop(permit);
                (node.id.clone(), result, duration_ms)
            });

            handles.push(handle);
        }

        let graph_id = self.graph.id.clone();

        // Wait for all nodes to complete
        for handle in handles {
            let (node_id, result, duration_ms) = handle.await?;

            let success = result.is_ok();

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    let node = self.graph.nodes.iter().find(|n| n.id == node_id);
                    if node.is_some_and(|n| n.is_optional()) {
                        debug!("Optional node {} failed, skipping: {}", node_id, e);
                        phase_result.completed += 1;
                        let skip_value = serde_json::json!({"skipped": true});
                        self.context
                            .set_status(&node_id, NodeStatus::Completed(skip_value.clone()))
                            .await;
                        self.context.set_output(&node_id, skip_value).await;
                    } else {
                        phase_result.failed += 1;
                        let error_msg = e.to_string();
                        self.context
                            .set_status(&node_id, NodeStatus::Failed(error_msg.clone()))
                            .await;
                        phase_result.errors.push((node_id.clone(), error_msg));
                    }
                }
            }

            // Record per-node metrics for PathwayLearner
            if let Some(ref collector) = self.metrics {
                if let Err(e) = collector.record_node_execution(&NodeExecutionParams {
                    execution_id,
                    graph_name: &graph_id,
                    node_id: &node_id,
                    primal_id: "",
                    operation: "",
                    success,
                    duration_ms,
                    error: None,
                }) {
                    warn!("Failed to record node metrics for {node_id}: {e}");
                }
            }
        }

        phase_result.duration_ms = {
            let e = phase_start.elapsed();
            e.as_secs() * 1000 + u64::from(e.subsec_millis())
        };

        if phase_result.failed > 0 {
            error!("❌ {} nodes failed in this phase:", phase_result.failed);
            for (node_id, error_msg) in &phase_result.errors {
                error!("   • {}: {}", node_id, error_msg);
            }
            anyhow::bail!("Phase failed: {} nodes failed", phase_result.failed);
        }

        Ok(phase_result)
    }

    /// Execute a single node
    ///
    /// Delegates to shared handlers in `executor::node_handlers` for consistency
    /// and to avoid code duplication. If the node has a remote `gate`, forwards
    /// execution to the remote biomeOS Neural API via `AtomicClient`.
    async fn execute_node(
        node: &GraphNode,
        context: &ExecutionContext,
        gate_registry: &GateRegistry,
        capability_registry: &CapabilityRegistry,
    ) -> Result<serde_json::Value> {
        if let Some(ref gate) = node.gate {
            if let Some(remote_endpoint) = gate_registry.resolve(gate) {
                return Self::forward_to_remote_gate(node, remote_endpoint.clone(), gate).await;
            }
        }

        use crate::executor::node_handlers;

        let node_type_str = if let Some(ref operation) = node.operation {
            operation.name.as_str()
        } else if let Some(ref node_type) = node.node_type {
            node_type.as_str()
        } else {
            "unknown"
        };

        info!(
            "   ⚡ Executing node: {} (type: {})",
            node.id, node_type_str
        );

        context.set_status(&node.id, NodeStatus::Running).await;

        let result = match node_type_str {
            "filesystem.check_exists" => {
                node_handlers::filesystem_check_exists(node, context).await
            }
            "crypto.derive_child_seed" => node_handlers::crypto_derive_seed(node, context).await,
            "primal.launch" => node_handlers::primal_launch(node, context).await,
            "primal_start" | "start" => {
                crate::capability_handlers::primal_start_capability(node, context).await
            }
            "health_check" | "health.check" | "health.check_atomic" => {
                node_handlers::health_check(node, context).await
            }
            "health.check_all" => Self::node_health_check_all(node, context).await,
            "verification" => Self::node_verification(node, context).await,
            "lineage.verify_siblings" => node_handlers::lineage_verify(node, context).await,
            "report.deployment_success" => node_handlers::deployment_report(node, context).await,
            "log.info" => node_handlers::log_info(node, context).await,
            "log.warn" => node_handlers::log_warn(node, context).await,
            "log.error" => node_handlers::log_error(node, context).await,
            "rpc_call" => Self::node_rpc_call(node, context).await,
            "capability_call" => {
                Self::node_capability_call_with_registry(node, context, capability_registry).await
            }
            "register_capabilities" => node_handlers::register_capabilities(node, context).await,
            _ => {
                warn!("Unknown node type: {}, skipping", node_type_str);
                Ok(serde_json::json!({"skipped": true}))
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Forward a node's execution to a remote biomeOS Neural API via cross-gate routing.
    ///
    /// Wraps the node as a single-node `graph.execute` request and sends it to the
    /// remote biomeOS instance. The remote biomeOS handles the actual primal start,
    /// capability call, etc. — this gate only orchestrates.
    async fn forward_to_remote_gate(
        node: &GraphNode,
        endpoint: biomeos_core::TransportEndpoint,
        gate: &str,
    ) -> Result<serde_json::Value> {
        info!(
            "🌉 Forwarding node {} to gate '{}' @ {}",
            node.id,
            gate,
            endpoint.display_string()
        );

        let client = biomeos_core::AtomicClient::from_endpoint(endpoint);

        let operation_name = node
            .operation
            .as_ref()
            .map(|op| op.name.as_str())
            .or(node.node_type.as_deref())
            .unwrap_or("capability_call");

        let operation_params: serde_json::Value = node
            .operation
            .as_ref()
            .map(|op| serde_json::to_value(&op.params).unwrap_or_default())
            .unwrap_or_default();

        let request_params = serde_json::json!({
            "graph_id": format!("cross_gate_{}_{}", gate, node.id),
            "nodes": [{
                "id": node.id,
                "operation": {
                    "name": operation_name,
                    "params": operation_params,
                },
                "capabilities": node.capabilities,
                "depends_on": [],
            }]
        });

        let result = client
            .call("graph.execute", request_params)
            .await
            .with_context(|| {
                format!(
                    "Cross-gate execution failed for node '{}' on gate '{}'",
                    node.id, gate
                )
            })?;

        info!("   ✓ Node {} completed on gate '{}'", node.id, gate);
        Ok(result)
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
    ///
    /// Returns execution phases where each phase is a set of nodes that can
    /// run in parallel. Useful for graph validation and composition inspection.
    pub fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build adjacency list and in-degree map
        tracing::info!(
            "🔍 Building dependency graph for {} nodes...",
            self.graph.nodes.len()
        );
        for node in &self.graph.nodes {
            tracing::info!("   Node '{}' depends_on: {:?}", node.id, node.depends_on);
            in_degree.entry(node.id.clone()).or_insert(0);

            for dep in &node.depends_on {
                // FIXED: was node.dependencies, now node.depends_on
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

        // Kahn's algorithm for topological sort
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

        // Check for cycles
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

    const fn resolve_node_type(node: &GraphNode) -> &str {
        if let Some(ref operation) = node.operation {
            let s = operation.name.as_str();
            if !s.is_empty() {
                return s;
            }
        }
        if let Some(ref node_type) = node.node_type {
            return node_type.as_str();
        }
        "unknown"
    }

    /// Roll back completed nodes in reverse topological order (best-effort).
    async fn rollback(&self) -> Result<()> {
        info!("🔄 Starting graph rollback (reverse topological order)");

        let phases = match self.topological_sort() {
            Ok(p) => p,
            Err(e) => {
                warn!("Rollback: topological sort failed: {e}");
                return Ok(());
            }
        };

        let mut reverse_order: Vec<String> = phases.into_iter().flatten().collect();
        reverse_order.reverse();

        let statuses = self.context.all_statuses().await;

        for node_id in &reverse_order {
            let Some(status) = statuses.get(node_id) else {
                continue;
            };
            let NodeStatus::Completed(output) = status else {
                continue;
            };

            let Some(node) = self.graph.nodes.iter().find(|n| &n.id == node_id) else {
                warn!("Rollback: node {node_id} missing from graph, skipping");
                continue;
            };

            let node_type = Self::resolve_node_type(node);

            match node_type {
                "primal.launch" | "primal_start" | "start" => {
                    Self::rollback_primal_lifecycle(self.context.clone(), node, output).await;
                }
                "register_capabilities" => {
                    Self::rollback_register_capabilities(self.context.clone(), node, output).await;
                }
                "rpc_call" | "capability_call" => {
                    info!(
                        "Rollback: node {node_id} ({node_type}) — RPC/capability calls are not reversed"
                    );
                }
                other => {
                    debug!("Rollback: node {node_id} ({other}) — no reverse action");
                }
            }
        }

        info!("🔄 Rollback pass complete");
        Ok(())
    }

    async fn rollback_primal_lifecycle(
        context: ExecutionContext,
        node: &GraphNode,
        output: &serde_json::Value,
    ) {
        let primal = output
            .get("primal")
            .and_then(|v| v.as_str())
            .or_else(|| node.config.get("primal_name").and_then(|v| v.as_str()));

        let Some(primal) = primal else {
            warn!(
                "Rollback: cannot resolve primal name for node {} (lifecycle.stop skipped)",
                node.id
            );
            return;
        };

        let socket = output
            .get("socket")
            .and_then(|v| v.as_str())
            .map(str::to_owned);

        let socket = match socket {
            Some(s) => s,
            None => context.get_socket_path(primal).await,
        };

        if !std::path::Path::new(&socket).exists() {
            warn!(
                "Rollback: socket {} not present for node {} — skipping lifecycle.stop",
                socket, node.id
            );
            return;
        }

        let request = JsonRpcRequest::new("lifecycle.stop", serde_json::json!({}));
        info!(
            "Rollback: sending lifecycle.stop to {} for node {} (primal {})",
            socket, node.id, primal
        );

        match Self::send_jsonrpc_async(&socket, &request).await {
            Ok(resp) => {
                if let Some(err) = resp.get("error") {
                    warn!(
                        "Rollback: lifecycle.stop error for node {}: {}",
                        node.id, err
                    );
                } else {
                    info!("Rollback: lifecycle.stop completed for {}", node.id);
                }
            }
            Err(e) => warn!("Rollback: lifecycle.stop failed for {}: {}", node.id, e),
        }
    }

    async fn rollback_register_capabilities(
        context: ExecutionContext,
        node: &GraphNode,
        output: &serde_json::Value,
    ) {
        let primal = output
            .get("primal")
            .and_then(|v| v.as_str())
            .or_else(|| node.config.get("primal_name").and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        let mut caps: Vec<String> = output
            .get("registered")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        if caps.is_empty() {
            caps.clone_from(&node.capabilities);
        }

        let neural_socket = context.get_socket_path("neural-api").await;

        if !std::path::Path::new(&neural_socket).exists() {
            warn!(
                "Rollback: neural-api socket {} not present — skipping capability.unregister for {}",
                neural_socket, node.id
            );
            return;
        }

        let request = JsonRpcRequest::new(
            "capability.unregister",
            serde_json::json!({
                "primal": primal,
                "capabilities": caps,
            }),
        );

        info!(
            "Rollback: sending capability.unregister for primal {} (node {})",
            primal, node.id
        );

        match Self::send_jsonrpc_async(&neural_socket, &request).await {
            Ok(resp) => {
                if let Some(err) = resp.get("error") {
                    warn!(
                        "Rollback: capability.unregister error for {}: {}",
                        node.id, err
                    );
                } else {
                    info!("Rollback: capability.unregister completed for {}", node.id);
                }
            }
            Err(e) => warn!(
                "Rollback: capability.unregister failed for {}: {}",
                node.id, e
            ),
        }
    }
}
// Node executor implementations (verification, health_check_all, rpc_call,
// capability_call, send_jsonrpc_async) live in neural_executor_node_impls.rs.
// Tests are in neural_executor_tests.rs.
