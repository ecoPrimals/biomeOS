// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Parallel phase execution for `GraphExecutor`.

use anyhow::Result;
use biomeos_graph::metrics::NodeExecutionParams;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, warn};

use super::{GraphExecutor, NodeStatus, PhaseResult};
use crate::neural_graph::GraphNode;

impl GraphExecutor {
    /// Execute a single phase (parallel execution of independent nodes)
    pub(super) async fn execute_phase(
        &mut self,
        phase_index: usize,
        nodes: &[String],
        node_map: &HashMap<String, Arc<GraphNode>>,
    ) -> Result<PhaseResult> {
        self.save_checkpoint_before_phase(phase_index).await?;

        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));
        let mut handles = Vec::new();
        let execution_id = chrono::Utc::now().timestamp_millis();

        for node_id in nodes {
            let node = Arc::clone(
                node_map
                    .get(node_id)
                    .ok_or_else(|| anyhow::anyhow!("Node not found: {node_id}"))?,
            );

            let node_id = node_id.clone();
            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            let gate_reg = Arc::clone(&self.gate_registry);
            let cap_reg = Arc::clone(&self.capability_registry);

            let handle = tokio::spawn(async move {
                let node_start = std::time::Instant::now();
                let result = Self::execute_node(&node, &context, &gate_reg, &cap_reg).await;
                let duration_ms = node_start.elapsed().as_millis() as u64;
                drop(permit);
                (node_id, result, duration_ms)
            });

            handles.push(handle);
        }

        let metrics_graph_name: Option<Cow<'_, str>> =
            self.metrics.as_ref().map(|_| match &self.signal_namespace {
                Some(ns) => Cow::Owned(format!("{ns}:{}", self.graph.id)),
                None => Cow::Borrowed(self.graph.id.as_str()),
            });

        for handle in handles {
            let (node_id, result, duration_ms) = handle.await?;

            let success = result.is_ok();
            let mut pending_error_msg: Option<String> = None;

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    let optional = node_map.get(&node_id).is_some_and(|n| n.is_optional());
                    if optional {
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
                        pending_error_msg = Some(error_msg);
                    }
                }
            }

            if let (Some(collector), Some(metrics_graph_name)) =
                (&self.metrics, &metrics_graph_name)
            {
                let node_ref = node_map.get(&node_id);
                let primal_id_str = node_ref
                    .and_then(|n| {
                        n.primal
                            .as_ref()
                            .and_then(|p| p.by_name.as_deref().or(p.by_capability.as_deref()))
                    })
                    .unwrap_or("");
                let operation_str = node_ref
                    .and_then(|n| n.operation.as_ref().map(|o| o.name.as_str()))
                    .unwrap_or("");
                if let Err(e) = collector.record_node_execution(&NodeExecutionParams {
                    execution_id,
                    graph_name: metrics_graph_name,
                    node_id: &node_id,
                    primal_id: primal_id_str,
                    operation: operation_str,
                    success,
                    duration_ms,
                    error: None,
                }) {
                    warn!("Failed to record node metrics for {node_id}: {e}");
                }
            }

            if let Some(msg) = pending_error_msg {
                phase_result.errors.push((node_id, msg));
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
        }

        Ok(phase_result)
    }
}
