// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Core graph execution logic
//!
//! **EVOLVED:** Smart refactored for maintainability.
//!
//! This module contains the main GraphExecutor implementation and orchestration logic.
//! Node-specific handlers are in the `node_handlers` module.

use super::{
    context::{ExecutionContext, NodeStatus},
    node_handlers,
    rollback::RollbackManager,
    topological::TopologicalSorter,
    types::{ExecutionReport, PhaseResult},
};
use crate::graph::{Graph, GraphNode};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Main graph executor
///
/// Orchestrates graph execution with:
/// - Topological sorting for dependency resolution
/// - Parallel execution within phases
/// - Checkpoint/rollback support
/// - Live monitoring
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    max_parallelism: usize,
}

impl GraphExecutor {
    /// Create new graph executor
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3, // Default from graph spec
        }
    }

    /// Execute the entire graph
    ///
    /// Returns an execution report with timing, status, and any errors.
    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);

        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Topological sort to get execution phases
        let phases = TopologicalSorter::sort(&self.graph)?;
        info!("   Execution plan: {} phases", phases.len());

        // Execute each phase sequentially
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!(
                "📍 Phase {}/{}: {} nodes",
                phase_num + 1,
                phases.len(),
                phase_nodes.len()
            );

            match self.execute_phase(phase_nodes).await {
                Ok(phase_results) => {
                    report.phase_results.push(phase_results);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Rollback if enabled
                    if self.graph.config.rollback_on_failure {
                        warn!("🔄 Rolling back deployment...");
                        let rollback_mgr = RollbackManager::new(&self.context);
                        rollback_mgr.execute_rollback().await?;
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

        Ok(report)
    }

    /// Execute a single phase (parallel execution of independent nodes)
    ///
    /// Within a phase, nodes have no dependencies on each other and can execute in parallel.
    /// Uses a semaphore to limit parallelism.
    async fn execute_phase(&mut self, nodes: &[String]) -> Result<PhaseResult> {
        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        // Semaphore for max parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));

        // Execute nodes in parallel
        let mut handles = Vec::new();

        for node_id in nodes {
            let node = self
                .graph
                .nodes
                .iter()
                .find(|n| n.id.as_str() == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            let handle = tokio::spawn(async move {
                let result = execute_node(&node, &context).await;
                drop(permit); // Release semaphore
                (node.id.as_str().to_string(), result)
            });

            handles.push(handle);
        }

        // Wait for all nodes to complete
        for handle in handles {
            let (node_id, result) = handle.await?;

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    phase_result.failed += 1;
                    let error_msg = e.to_string();
                    self.context
                        .set_status(&node_id, NodeStatus::Failed(error_msg.clone()))
                        .await;
                    phase_result.errors.push((node_id, error_msg));
                }
            }
        }

        phase_result.duration_ms = phase_start.elapsed().as_millis() as u64;

        if phase_result.failed > 0 {
            anyhow::bail!("Phase failed: {} nodes failed", phase_result.failed);
        }

        Ok(phase_result)
    }
}

/// Execute a single node (dispatcher)
///
/// This function dispatches to the appropriate node handler based on node type.
/// Node handlers are in the `node_handlers` module.
///
/// **ConditionalDag support:** Before executing, checks `should_skip` (skip_if)
/// and `condition_met` (condition). Skipped nodes return a sentinel JSON value
/// so downstream nodes can detect the skip.
pub async fn execute_node(
    node: &GraphNode,
    context: &ExecutionContext,
) -> Result<serde_json::Value> {
    debug!("   Executing node: {}", node.id);

    let env = &context.env;

    // ConditionalDag: skip_if evaluates to true
    if node.should_skip(&env) {
        info!("   ⏭️  Skipping node (skip_if): {}", node.id);
        context
            .set_status(node.id.as_str(), NodeStatus::Completed(serde_json::json!({"skipped": true, "reason": "skip_if"})))
            .await;
        return Ok(serde_json::json!({"skipped": true, "reason": "skip_if"}));
    }

    // ConditionalDag: condition evaluates to false
    if !node.condition_met(&env) {
        info!("   ⏭️  Skipping node (condition not met): {}", node.id);
        context
            .set_status(node.id.as_str(), NodeStatus::Completed(serde_json::json!({"skipped": true, "reason": "condition_not_met"})))
            .await;
        return Ok(serde_json::json!({"skipped": true, "reason": "condition_not_met"}));
    }

    // Mark as running
    context.set_status(node.id.as_str(), NodeStatus::Running).await;

    // Dispatch to node-specific handler
    let result = match node.capability.as_deref() {
        Some("filesystem.check_exists") => {
            node_handlers::node_filesystem_check_exists(node, context).await
        }
        Some("crypto.derive_child_seed") => {
            node_handlers::node_crypto_derive_seed(node, context).await
        }
        Some("primal.launch") => node_handlers::node_primal_launch(node, context).await,
        Some("health.check_atomic") => node_handlers::node_health_check(node, context).await,
        Some("lineage.verify_siblings") => {
            node_handlers::node_lineage_verify(node, context).await
        }
        Some("report.deployment_success") => {
            node_handlers::node_deployment_report(node, context).await
        }
        _ => {
            warn!("Unknown node capability: {:?}, skipping", node.capability);
            Ok(serde_json::json!({"skipped": true}))
        }
    };

    result.context(format!("Node execution failed: {}", node.id))
}
