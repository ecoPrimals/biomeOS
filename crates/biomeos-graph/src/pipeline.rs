// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Pipeline coordination executor for streaming graph execution.
//!
//! Enables data-pipeline niches (telemetry, pharmacology, FASTQ processing,
//! spectral analysis, ETL) by wiring graph nodes with bounded `mpsc` channels:
//!
//! ```text
//! Source ──[channel]──▶ Transform A ──[channel]──▶ Transform B ──[channel]──▶ Sink
//! ```
//!
//! Each node runs as an independent tokio task. Items flow through the
//! pipeline as soon as each node produces them, enabling true streaming:
//! node B starts processing item 1 while node A produces item 2.
//!
//! ## NDJSON Streaming
//!
//! biomeOS uses newline-delimited JSON (NDJSON) over Unix sockets.
//! A primal that produces a stream sends multiple NDJSON response lines
//! per request. The pipeline executor reads these as individual `StreamItem`s
//! and forwards them to the next channel. This means primals already have
//! the transport for streaming — they just send multiple lines.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let executor = PipelineExecutor::new(graph, broadcaster);
//! let results = executor.run(|node_id, node, item| async move {
//!     // Call the primal's capability, streaming each item through
//!     Ok(StreamItem::Data(call_primal(&node, &item).await?))
//! }).await;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::GraphError;
use crate::events::{GraphEvent, GraphEventBroadcaster};
use crate::graph::DeploymentGraph;
use crate::node::GraphNode;

/// Cheaply cloneable pipeline node identifier (`Arc<str>`), passed on every stream item.
///
/// Used as the first argument to [`PipelineExecutor::run`] so hot loops avoid cloning
/// heap-allocated [`String`] values per item.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct PipelineNodeId(Arc<str>);

impl PipelineNodeId {
    /// Wrap a node id string (typically from [`crate::node::GraphNode::id`]).
    #[must_use]
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }

    /// Reuse an existing shared id (avoids reallocating when the same `Arc` is already held).
    #[must_use]
    pub fn from_arc(id: Arc<str>) -> Self {
        Self(id)
    }

    /// Borrow the underlying id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for PipelineNodeId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PipelineNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// An item flowing through a streaming pipeline.
///
/// The streaming protocol is simple: a source produces `Data` items,
/// each node transforms them into new `Data` items, and the pipeline
/// ends when the source sends `End`. Errors are non-fatal by default —
/// they are logged and the pipeline continues with the next item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StreamItem {
    /// A data item flowing through the pipeline.
    Data(serde_json::Value),
    /// End of stream — no more items will be produced by the upstream node.
    End,
    /// A non-fatal error from a node. Logged and skipped.
    Error {
        /// The node that produced the error.
        node_id: String,
        /// Error description.
        message: String,
    },
}

impl StreamItem {
    /// Returns true if this is a data item (not End or Error).
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Returns the inner value if this is a `Data` item.
    #[must_use]
    pub fn into_data(self) -> Option<serde_json::Value> {
        match self {
            Self::Data(v) => Some(v),
            _ => None,
        }
    }
}

/// Result of a completed pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Graph identifier.
    pub graph_id: String,
    /// Total items that entered the pipeline (produced by the source node).
    pub items_in: u64,
    /// Total items that exited the pipeline (collected at the sink).
    pub items_out: u64,
    /// Items lost to errors or filtering along the way.
    pub items_dropped: u64,
    /// Per-node throughput stats.
    pub node_stats: HashMap<String, NodeThroughput>,
    /// Collected output items from the final node.
    pub outputs: Vec<serde_json::Value>,
    /// Total pipeline duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the pipeline completed without fatal errors.
    pub success: bool,
    /// Error message if the pipeline failed fatally.
    pub error: Option<String>,
}

/// Throughput statistics for a single node in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeThroughput {
    /// Items processed by this node.
    pub items_processed: u64,
    /// Items that errored in this node.
    pub items_errored: u64,
    /// Total processing time across all items (milliseconds).
    pub total_processing_ms: u64,
    /// Average processing time per item (milliseconds).
    pub avg_item_ms: f64,
}

/// Channel capacity for inter-node streaming.
///
/// Bounded channels prevent a fast producer from overwhelming a slow consumer.
/// The default of 64 balances throughput (enough items in flight) with
/// backpressure (producer blocks when consumer is behind).
const DEFAULT_CHANNEL_CAPACITY: usize = 64;

/// Executes a deployment graph as a streaming pipeline.
///
/// Nodes form a linear chain (validated at construction). Each node runs
/// as an independent tokio task connected by bounded mpsc channels.
/// Items flow through as soon as each node produces them.
pub struct PipelineExecutor {
    graph: DeploymentGraph,
    broadcaster: GraphEventBroadcaster,
    node_order: Vec<PipelineNodeId>,
    channel_capacity: usize,
}

impl PipelineExecutor {
    /// Create a new pipeline executor for the given graph.
    ///
    /// Validates that the graph forms a linear chain (each node depends on
    /// at most one predecessor, and there's exactly one source node).
    #[must_use]
    pub fn new(graph: DeploymentGraph, broadcaster: GraphEventBroadcaster) -> Self {
        let node_order = Self::compute_node_order(&graph)
            .into_iter()
            .map(PipelineNodeId::from_arc)
            .collect();
        Self {
            graph,
            broadcaster,
            node_order,
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }

    /// Set the channel capacity for inter-node streaming.
    #[must_use]
    pub const fn with_channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Run the pipeline to completion.
    ///
    /// The `node_executor` callback is invoked for each item at each node.
    /// It receives the node ID, the node definition, and the input item.
    /// It must return the transformed item (or `StreamItem::End` to stop,
    /// or `StreamItem::Error` to skip this item).
    ///
    /// The source node (first in the chain) receives a synthetic
    /// `StreamItem::Data(Value::Null)` as its initial trigger. It should
    /// produce the actual stream items by returning them one at a time.
    /// When the source is exhausted, it returns `StreamItem::End`.
    #[expect(
        clippy::too_many_lines,
        reason = "pipeline run loop coordinates many nodes and stream states"
    )]
    pub async fn run<F, Fut>(self, node_executor: F) -> Result<PipelineResult, GraphError>
    where
        F: Fn(PipelineNodeId, GraphNode, StreamItem) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = StreamItem> + Send,
    {
        let graph_id: Arc<str> = Arc::from(self.graph.id().as_str());
        let start = Instant::now();

        info!(
            "🔗 Pipeline starting: {} ({} nodes, capacity {})",
            graph_id.as_ref(),
            self.node_order.len(),
            self.channel_capacity,
        );

        let _ = self
            .broadcaster
            .broadcast(GraphEvent::SessionStarted {
                graph_id: graph_id.as_ref().to_string(),
                target_hz: 0.0,
                timestamp: Utc::now(),
            })
            .await;

        if self.node_order.is_empty() {
            return Ok(PipelineResult {
                graph_id: graph_id.as_ref().to_string(),
                items_in: 0,
                items_out: 0,
                items_dropped: 0,
                node_stats: HashMap::new(),
                outputs: vec![],
                duration_ms: start.elapsed().as_millis() as u64,
                success: true,
                error: None,
            });
        }

        // Build the channel chain: node[0]→node[1]→...→collector
        let n = self.node_order.len();
        let mut senders: Vec<mpsc::Sender<StreamItem>> = Vec::with_capacity(n);
        let mut receivers: Vec<mpsc::Receiver<StreamItem>> = Vec::with_capacity(n);

        // Create n+1 channels (source→node[0], node[0]→node[1], ..., node[n-1]→collector)
        for _ in 0..=n {
            let (tx, rx) = mpsc::channel(self.channel_capacity);
            senders.push(tx);
            receivers.push(rx);
        }

        // senders[0] = source → node[0] input
        // receivers[0] = node[0] reads from here
        // senders[1] = node[0] → node[1] output
        // receivers[1] = node[1] reads from here
        // ...
        // senders[n] = node[n-1] → collector output
        // receivers[n] = collector reads from here

        let executor = Arc::new(node_executor);
        let mut handles = Vec::with_capacity(n);
        let node_ids: Vec<PipelineNodeId> = self.node_order.clone();
        let stats_map: Arc<tokio::sync::Mutex<HashMap<PipelineNodeId, NodeThroughput>>> =
            Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        // Spawn a task for each node.
        //
        // Source node (i==0): called repeatedly with Data(Null) until it returns End.
        // This models a data producer (file reader, DB cursor, sensor stream, etc.)
        // that generates items on its own schedule.
        //
        // Transform nodes (i>0): called once per input item, producing one output item.
        // Transforms read from their input channel and write to their output channel.
        for i in 0..n {
            let node_id = node_ids[i].clone();
            let node = self
                .find_node(node_id.as_str())
                .cloned()
                .ok_or_else(|| GraphError::NodeNotFound(node_id.as_str().to_string()))?;

            let mut rx = receivers.remove(0);
            let tx = senders[i + 1].clone();
            let exec = executor.clone();
            let stats = stats_map.clone();
            let nid = node_id.clone();
            let is_source = i == 0;

            let handle = tokio::spawn(async move {
                let mut processed = 0u64;
                let mut errored = 0u64;
                let mut total_ms = 0u64;

                if is_source {
                    // Source node: call repeatedly until End.
                    // Consumes the single trigger from the channel first, then
                    // re-invokes itself with Data(Null) for each subsequent item.
                    let _ = rx.recv().await; // consume trigger
                    loop {
                        let item_start = Instant::now();
                        let result = exec(
                            nid.clone(),
                            node.clone(),
                            StreamItem::Data(serde_json::Value::Null),
                        )
                        .await;
                        total_ms += item_start.elapsed().as_millis() as u64;

                        match result {
                            StreamItem::Data(v) => {
                                processed += 1;
                                if tx.send(StreamItem::Data(v)).await.is_err() {
                                    warn!("Pipeline channel closed for source {}", nid);
                                    break;
                                }
                            }
                            StreamItem::End => {
                                let _ = tx.send(StreamItem::End).await;
                                break;
                            }
                            StreamItem::Error { .. } => {
                                errored += 1;
                                let _ = tx.send(result).await;
                            }
                        }
                    }
                } else {
                    // Transform node: process each input item from upstream
                    while let Some(item) = rx.recv().await {
                        match &item {
                            StreamItem::End => {
                                let _ = tx.send(StreamItem::End).await;
                                break;
                            }
                            StreamItem::Error { .. } => {
                                errored += 1;
                                let _ = tx.send(item).await;
                                continue;
                            }
                            StreamItem::Data(_) => {}
                        }

                        let item_start = Instant::now();
                        let result = exec(nid.clone(), node.clone(), item).await;
                        total_ms += item_start.elapsed().as_millis() as u64;

                        match result {
                            StreamItem::Data(v) => {
                                processed += 1;
                                if tx.send(StreamItem::Data(v)).await.is_err() {
                                    warn!("Pipeline channel closed for node {}", nid);
                                    break;
                                }
                            }
                            StreamItem::End => {
                                let _ = tx.send(StreamItem::End).await;
                                break;
                            }
                            StreamItem::Error { .. } => {
                                errored += 1;
                                let _ = tx.send(result).await;
                            }
                        }
                    }
                }

                let avg = if processed > 0 {
                    total_ms as f64 / processed as f64
                } else {
                    0.0
                };

                stats.lock().await.insert(
                    nid,
                    NodeThroughput {
                        items_processed: processed,
                        items_errored: errored,
                        total_processing_ms: total_ms,
                        avg_item_ms: avg,
                    },
                );
            });

            handles.push(handle);
        }

        // Send the trigger to the source node
        let source_tx = &senders[0];
        if source_tx
            .send(StreamItem::Data(serde_json::Value::Null))
            .await
            .is_err()
        {
            error!("Failed to trigger pipeline source");
        }

        drop(senders);

        // Collect results from the final channel
        let mut collector_rx = receivers.remove(receivers.len() - 1);
        let mut outputs = Vec::new();
        let mut items_out = 0u64;
        let mut items_dropped = 0u64;

        while let Some(item) = collector_rx.recv().await {
            match item {
                StreamItem::Data(v) => {
                    items_out += 1;
                    outputs.push(v);
                }
                StreamItem::End => break,
                StreamItem::Error { node_id, message } => {
                    items_dropped += 1;
                    debug!("Pipeline error from {}: {}", node_id, message);
                }
            }
        }

        // Wait for all node tasks to finish
        for handle in handles {
            if let Err(e) = handle.await {
                warn!("Pipeline node task panicked: {e}");
            }
        }

        let node_stats_raw = match Arc::try_unwrap(stats_map) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().await.clone(),
        };

        let node_stats: HashMap<String, NodeThroughput> = node_stats_raw
            .into_iter()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();

        let items_in = node_stats
            .get(node_ids[0].as_str())
            .map_or(0, |s| s.items_processed + s.items_errored);

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "🔗 Pipeline complete: {} — {items_in} in, {items_out} out, {items_dropped} dropped, {duration_ms}ms",
            graph_id.as_ref(),
        );

        Ok(PipelineResult {
            graph_id: graph_id.as_ref().to_string(),
            items_in,
            items_out,
            items_dropped,
            node_stats,
            outputs,
            duration_ms,
            success: true,
            error: None,
        })
    }

    /// Linear node order as shared ids (`Arc<str>`): one allocation per id, reused by [`PipelineNodeId`].
    fn compute_node_order(graph: &DeploymentGraph) -> Vec<Arc<str>> {
        let nodes = &graph.definition.nodes;
        if nodes.is_empty() {
            return vec![];
        }

        let id_to_node: HashMap<&str, &GraphNode> =
            nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        // Find source nodes (no depends_on)
        let sources: Vec<&str> = nodes
            .iter()
            .filter(|n| n.depends_on.is_empty())
            .map(|n| n.id.as_str())
            .collect();

        if sources.len() != 1 {
            // Not a clean linear chain — fall back to definition order
            return nodes
                .iter()
                .map(|n| Arc::<str>::from(n.id.as_str()))
                .collect();
        }

        // Build successor map: parent → child
        let mut successors: HashMap<&str, &str> = HashMap::new();
        for node in nodes {
            if node.depends_on.len() == 1 {
                successors.insert(node.depends_on[0].as_str(), node.id.as_str());
            }
        }

        // Walk the chain from source
        let mut order = Vec::with_capacity(nodes.len());
        let mut current = sources[0];
        loop {
            order.push(Arc::from(current));
            match successors.get(current) {
                Some(next) if id_to_node.contains_key(next) => current = next,
                _ => break,
            }
        }

        order
    }

    /// Compute linear execution order from the dependency graph.
    ///
    /// For a valid pipeline, this produces a topological sort where each node
    /// has at most one dependency. Falls back to node definition order if
    /// dependencies don't form a clean chain.
    #[cfg(test)]
    fn compute_linear_order(graph: &DeploymentGraph) -> Vec<String> {
        Self::compute_node_order(graph)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Find a node by ID in the graph.
    fn find_node(&self, id: &str) -> Option<&GraphNode> {
        self.graph
            .definition
            .nodes
            .iter()
            .find(|n| n.id.as_str() == id)
    }
}

#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod tests;
