// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Rollback support for graph execution.
//!
//! Reverses completed nodes in reverse topological order (best-effort):
//! - `primal.launch` / `start` nodes → `lifecycle.stop`
//! - `register_capabilities` nodes → `capability.unregister`
//! - `rpc_call` / `capability_call` nodes → not reversed (logged)

use anyhow::Result;
use biomeos_types::JsonRpcRequest;
use tracing::{debug, info, warn};

use super::neural_executor::{ExecutionContext, GraphExecutor, NodeStatus};
use crate::neural_graph::GraphNode;

impl GraphExecutor {
    pub(crate) const fn resolve_node_type(node: &GraphNode) -> &str {
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
    pub(crate) async fn rollback(&self) -> Result<()> {
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
