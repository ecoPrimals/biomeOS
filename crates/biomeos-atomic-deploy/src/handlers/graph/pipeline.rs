// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Pipeline graph execution — streaming coordination via `PipelineExecutor`.

use super::GraphHandler;
use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use biomeos_graph::events::GraphEventBroadcaster;
use biomeos_graph::graph::{CoordinationPattern, DeploymentGraph};
use biomeos_graph::pipeline::{PipelineExecutor, StreamItem};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{debug, info};

impl GraphHandler {
    /// Execute a pipeline graph — streaming coordination.
    ///
    /// JSON-RPC method: `graph.execute_pipeline`
    ///
    /// Loads the graph as a `DeploymentGraph`, validates that it uses
    /// `Pipeline` coordination, then runs it via `PipelineExecutor`.
    /// The source node produces items, each transform node processes them,
    /// and the result collects all outputs.
    ///
    /// Returns the full `PipelineResult` with per-node throughput stats.
    pub async fn execute_pipeline(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!(
                "Graph '{graph_id}' not found in nucleus ({}) or runtime ({})",
                self.graphs_dir.display(),
                self.runtime_graphs_dir.display()
            )
        })?;

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        if deployment_graph.definition.coordination != CoordinationPattern::Pipeline {
            anyhow::bail!(
                "Graph '{}' has coordination '{:?}', not Pipeline",
                graph_id,
                deployment_graph.definition.coordination
            );
        }

        let channel_capacity = params["channel_capacity"].as_u64().unwrap_or(64) as usize;

        info!(
            "🔗 Executing pipeline graph: {} ({} nodes, capacity {})",
            graph_id,
            deployment_graph.definition.nodes.len(),
            channel_capacity,
        );

        let broadcaster = GraphEventBroadcaster::new(16);
        let executor = PipelineExecutor::new(deployment_graph, broadcaster)
            .with_channel_capacity(channel_capacity);

        let router = self.router.clone();
        let translation_registry = self.translation_registry.clone();

        let result = executor
            .run(move |node_id, node, item| {
                let router = router.clone();
                let translation_registry = translation_registry.clone();
                async move {
                    Self::execute_pipeline_node(
                        &router,
                        &translation_registry,
                        node_id,
                        &node,
                        item,
                    )
                    .await
                }
            })
            .await
            .context("Pipeline execution failed")?;

        Ok(serde_json::to_value(result)?)
    }

    /// Route a single pipeline node through capability translation or discovery.
    async fn execute_pipeline_node(
        router: &NeuralRouter,
        translation_registry: &RwLock<CapabilityTranslationRegistry>,
        node_id: String,
        node: &biomeos_graph::node::GraphNode,
        item: StreamItem,
    ) -> StreamItem {
        let capability = match &node.capability {
            Some(c) => c.clone(),
            None => {
                return StreamItem::Error {
                    node_id: node_id.clone(),
                    message: format!("Node '{node_id}' has no capability"),
                };
            }
        };

        let input = match item {
            StreamItem::Data(v) => v,
            other => return other,
        };

        let call_params = json!({
            "capability": capability,
            "node_id": node_id,
            "input": input,
        });

        let registry = translation_registry.read().await;
        if let Some(translation) = registry.get_translation(&capability) {
            let ep =
                biomeos_core::TransportEndpoint::parse(&translation.socket).unwrap_or_else(|| {
                    biomeos_core::TransportEndpoint::UnixSocket {
                        path: PathBuf::from(&translation.socket),
                    }
                });
            let method = translation.actual_method.clone();
            drop(registry);
            match router.forward_request(&ep, &method, &call_params).await {
                Ok(result) => StreamItem::Data(result),
                Err(e) => {
                    debug!("Pipeline node '{}' capability call failed: {}", node_id, e);
                    StreamItem::Error {
                        node_id,
                        message: format!("{e}"),
                    }
                }
            }
        } else {
            drop(registry);
            let domain = capability
                .find('.')
                .map_or(capability.as_str(), |pos| &capability[..pos]);
            match router.discover_capability(domain).await {
                Ok(discovered) => {
                    match router
                        .forward_request(&discovered.primary_endpoint, &capability, &call_params)
                        .await
                    {
                        Ok(result) => StreamItem::Data(result),
                        Err(e) => StreamItem::Error {
                            node_id,
                            message: format!("{e}"),
                        },
                    }
                }
                Err(e) => {
                    debug!("Pipeline node '{}' discovery failed: {}", node_id, e);
                    StreamItem::Error {
                        node_id,
                        message: format!("Capability not found: {capability}: {e}"),
                    }
                }
            }
        }
    }
}
