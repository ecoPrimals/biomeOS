// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Node type dispatch and remote gate forwarding for `GraphExecutor`.

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use super::GraphExecutor;
use crate::capability_domains::CapabilityRegistry;
use crate::executor::context::ExecutionContext;
use crate::gate_registry::GateRegistry;
use crate::neural_graph::GraphNode;

impl GraphExecutor {
    /// Execute a single node
    ///
    /// Delegates to shared handlers in `executor::node_handlers` for consistency
    /// and to avoid code duplication. If the node has a remote `gate`, forwards
    /// execution to the remote biomeOS Neural API via `AtomicClient`.
    pub(super) async fn execute_node(
        node: &GraphNode,
        context: &ExecutionContext,
        gate_registry: &GateRegistry,
        capability_registry: &CapabilityRegistry,
    ) -> Result<serde_json::Value> {
        if let Some(ref gate) = node.gate {
            if gate == "local" {
                // Explicit local execution — fall through to local handlers
            } else if let Some(remote_endpoint) = gate_registry.resolve(gate) {
                return Self::forward_to_remote_gate(node, remote_endpoint, gate).await;
            } else {
                anyhow::bail!(
                    "Node '{}' targets gate '{}' but it is not registered. \
                     Register it via [graph.env] (e.g. {} = \"tcp://host:port\") \
                     or use gate = \"local\" for explicit local execution. \
                     Known gates: {:?}",
                    node.id,
                    gate,
                    gate,
                    gate_registry.gate_names()
                );
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

        context
            .set_status(&node.id, super::NodeStatus::Running)
            .await;

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
            "register_capabilities" | "register_only" => {
                node_handlers::register_capabilities(node, context).await
            }
            _ => {
                let has_capability = node
                    .primal
                    .as_ref()
                    .is_some_and(|p| p.by_capability.is_some())
                    || node.capabilities.first().is_some_and(|c| !c.is_empty());

                if has_capability {
                    debug!(
                        "Node '{}' has capability — dispatching as capability_call (was: {})",
                        node.id, node_type_str
                    );
                    Self::node_capability_call_with_registry(node, context, capability_registry)
                        .await
                } else {
                    warn!("Unknown node type: {}, skipping", node_type_str);
                    Ok(serde_json::json!({"skipped": true}))
                }
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Forward a node's execution to a remote biomeOS Neural API via cross-gate routing.
    async fn forward_to_remote_gate(
        node: &GraphNode,
        endpoint: &biomeos_core::TransportEndpoint,
        gate: &str,
    ) -> Result<serde_json::Value> {
        info!(
            "🌉 Forwarding node {} to gate '{}' @ {}",
            node.id,
            gate,
            endpoint.display_string()
        );

        let client = biomeos_core::AtomicClient::from_endpoint(endpoint.clone());

        let operation_name = node
            .operation
            .as_ref()
            .map(|op| op.name.as_str())
            .or(node.node_type.as_deref())
            .unwrap_or("capability_call");

        let operation_params: serde_json::Value = node
            .operation
            .as_ref()
            .and_then(|op| match serde_json::to_value(&op.params) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!("Failed to serialize cross-gate node params: {e}");
                    None
                }
            })
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
}
