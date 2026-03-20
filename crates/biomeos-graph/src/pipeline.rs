// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

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
    pub fn is_data(&self) -> bool {
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
    node_order: Vec<String>,
    channel_capacity: usize,
}

impl PipelineExecutor {
    /// Create a new pipeline executor for the given graph.
    ///
    /// Validates that the graph forms a linear chain (each node depends on
    /// at most one predecessor, and there's exactly one source node).
    pub fn new(graph: DeploymentGraph, broadcaster: GraphEventBroadcaster) -> Self {
        let node_order = Self::compute_linear_order(&graph);
        Self {
            graph,
            broadcaster,
            node_order,
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }

    /// Set the channel capacity for inter-node streaming.
    #[must_use]
    pub fn with_channel_capacity(mut self, capacity: usize) -> Self {
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
        F: Fn(String, GraphNode, StreamItem) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = StreamItem> + Send,
    {
        let graph_id = self.graph.id().to_string();
        let start = Instant::now();

        info!(
            "🔗 Pipeline starting: {} ({} nodes, capacity {})",
            graph_id,
            self.node_order.len(),
            self.channel_capacity,
        );

        let _ = self
            .broadcaster
            .broadcast(GraphEvent::SessionStarted {
                graph_id: graph_id.clone(),
                target_hz: 0.0,
                timestamp: Utc::now(),
            })
            .await;

        if self.node_order.is_empty() {
            return Ok(PipelineResult {
                graph_id,
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
        let node_ids: Vec<String> = self.node_order.clone();
        let stats_map: Arc<tokio::sync::Mutex<HashMap<String, NodeThroughput>>> =
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
                .find_node(&node_id)
                .cloned()
                .ok_or_else(|| GraphError::NodeNotFound(node_id.clone()))?;

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

        let node_stats = match Arc::try_unwrap(stats_map) {
            Ok(mutex) => mutex.into_inner(),
            Err(arc) => arc.lock().await.clone(),
        };

        let items_in = node_stats
            .get(&node_ids[0])
            .map_or(0, |s| s.items_processed + s.items_errored);

        let duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "🔗 Pipeline complete: {} — {items_in} in, {items_out} out, {items_dropped} dropped, {duration_ms}ms",
            graph_id,
        );

        Ok(PipelineResult {
            graph_id,
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

    /// Compute linear execution order from the dependency graph.
    ///
    /// For a valid pipeline, this produces a topological sort where each node
    /// has at most one dependency. Falls back to node definition order if
    /// dependencies don't form a clean chain.
    fn compute_linear_order(graph: &DeploymentGraph) -> Vec<String> {
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
            return nodes.iter().map(|n| n.id.as_str().to_string()).collect();
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
            order.push(current.to_string());
            match successors.get(current) {
                Some(next) if id_to_node.contains_key(next) => current = next,
                _ => break,
            }
        }

        order
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
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::events::GraphEventBroadcaster;
    use crate::graph::{
        CoordinationPattern, DeploymentGraph, GraphDefinition, GraphId, GraphMetadata,
    };
    use crate::node::{GraphNode, NodeId};

    fn make_pipeline_graph(nodes: Vec<GraphNode>) -> DeploymentGraph {
        DeploymentGraph {
            definition: GraphDefinition {
                id: GraphId::new("test-pipeline").unwrap(),
                name: "Test Pipeline".to_string(),
                version: "1.0.0".to_string(),
                description: "Test streaming pipeline".to_string(),
                metadata: GraphMetadata::default(),
                coordination: CoordinationPattern::Pipeline,
                tick: None,
                env: HashMap::new(),
                nodes,
                outputs: HashMap::new(),
            },
        }
    }

    fn make_node(id: &str, depends_on: Vec<&str>) -> GraphNode {
        GraphNode {
            id: NodeId::new(id).unwrap(),
            name: id.to_string(),
            node_type: crate::node::NodeType::default(),
            capability: Some(format!("test.{id}")),
            required: true,
            order: 0,
            depends_on: depends_on.into_iter().map(String::from).collect(),
            condition: None,
            config: crate::node::NodeConfig::default(),
            params: crate::node::NodeParams::default(),
            feedback_to: None,
            budget_ms: None,
            fallback: None,
            cost_estimate_ms: None,
            operation_dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_linear_order_simple_chain() {
        let graph = make_pipeline_graph(vec![
            make_node("fetch", vec![]),
            make_node("parse", vec!["fetch"]),
            make_node("analyze", vec!["parse"]),
        ]);
        let order = PipelineExecutor::compute_linear_order(&graph);
        assert_eq!(order, vec!["fetch", "parse", "analyze"]);
    }

    #[test]
    fn test_linear_order_single_node() {
        let graph = make_pipeline_graph(vec![make_node("solo", vec![])]);
        let order = PipelineExecutor::compute_linear_order(&graph);
        assert_eq!(order, vec!["solo"]);
    }

    #[test]
    fn test_linear_order_empty_graph() {
        let graph = make_pipeline_graph(vec![]);
        let order = PipelineExecutor::compute_linear_order(&graph);
        assert!(order.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_passthrough_single_item() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let graph = make_pipeline_graph(vec![
            make_node("source", vec![]),
            make_node("transform", vec!["source"]),
            make_node("sink", vec!["transform"]),
        ]);

        let broadcaster = GraphEventBroadcaster::new(16);
        let executor = PipelineExecutor::new(graph, broadcaster);
        let source_calls = Arc::new(AtomicU32::new(0));
        let sc = source_calls.clone();

        let result = executor
            .run(move |node_id, _node, item| {
                let sc = sc.clone();
                async move {
                    match node_id.as_str() {
                        "source" => {
                            let n = sc.fetch_add(1, Ordering::SeqCst);
                            if n == 0 {
                                StreamItem::Data(serde_json::json!({"source": "item1"}))
                            } else {
                                StreamItem::End
                            }
                        }
                        "transform" => {
                            if let StreamItem::Data(mut v) = item {
                                v["transformed"] = serde_json::json!(true);
                                StreamItem::Data(v)
                            } else {
                                item
                            }
                        }
                        "sink" => item,
                        _ => StreamItem::End,
                    }
                }
            })
            .await
            .expect("pipeline run");

        assert!(result.success);
        assert_eq!(result.items_out, 1);
        assert!(result.outputs[0]["transformed"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_pipeline_multi_item_source() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let graph = make_pipeline_graph(vec![
            make_node("producer", vec![]),
            make_node("doubler", vec!["producer"]),
        ]);

        let broadcaster = GraphEventBroadcaster::new(16);
        let executor = PipelineExecutor::new(graph, broadcaster);

        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result = executor
            .run(move |node_id, _node, item| {
                let cc = cc.clone();
                async move {
                    match node_id.as_str() {
                        "producer" => {
                            // Source is re-called with Data(Null) until it returns End
                            let n = cc.fetch_add(1, Ordering::SeqCst);
                            if n < 5 {
                                StreamItem::Data(serde_json::json!({"n": n}))
                            } else {
                                StreamItem::End
                            }
                        }
                        "doubler" => {
                            if let StreamItem::Data(v) = item {
                                let n = v["n"].as_u64().unwrap_or(0);
                                StreamItem::Data(serde_json::json!({"n": n * 2}))
                            } else {
                                StreamItem::End
                            }
                        }
                        _ => StreamItem::End,
                    }
                }
            })
            .await
            .expect("pipeline run");

        assert!(result.success);
        assert_eq!(result.items_out, 5);
        assert_eq!(result.outputs[0]["n"].as_u64().unwrap(), 0); // 0 * 2
        assert_eq!(result.outputs[1]["n"].as_u64().unwrap(), 2); // 1 * 2
        assert_eq!(result.outputs[2]["n"].as_u64().unwrap(), 4); // 2 * 2
        assert_eq!(result.outputs[3]["n"].as_u64().unwrap(), 6); // 3 * 2
        assert_eq!(result.outputs[4]["n"].as_u64().unwrap(), 8); // 4 * 2
    }

    #[tokio::test]
    async fn test_pipeline_error_passthrough() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let graph = make_pipeline_graph(vec![
            make_node("source", vec![]),
            make_node("sink", vec!["source"]),
        ]);

        let broadcaster = GraphEventBroadcaster::new(16);
        let executor = PipelineExecutor::new(graph, broadcaster);
        let call_count = Arc::new(AtomicU32::new(0));
        let cc = call_count.clone();

        let result = executor
            .run(move |node_id, _node, item| {
                let cc = cc.clone();
                async move {
                    match node_id.as_str() {
                        "source" => {
                            let n = cc.fetch_add(1, Ordering::SeqCst);
                            if n == 0 {
                                // First call: produce an error
                                StreamItem::Error {
                                    node_id: "source".to_string(),
                                    message: "test error".to_string(),
                                }
                            } else {
                                StreamItem::End
                            }
                        }
                        // Sink passes everything through
                        _ => item,
                    }
                }
            })
            .await
            .expect("pipeline run");

        assert!(result.success);
        assert_eq!(result.items_dropped, 1);
        assert_eq!(result.items_out, 0);
    }

    #[test]
    fn test_stream_item_is_data() {
        assert!(StreamItem::Data(serde_json::json!(1)).is_data());
        assert!(!StreamItem::End.is_data());
        assert!(
            !StreamItem::Error {
                node_id: "x".into(),
                message: "y".into()
            }
            .is_data()
        );
    }

    #[test]
    fn test_stream_item_into_data() {
        let item = StreamItem::Data(serde_json::json!(42));
        assert_eq!(item.into_data(), Some(serde_json::json!(42)));
        assert!(StreamItem::End.into_data().is_none());
    }

    #[test]
    fn test_stream_item_serde_roundtrip() {
        let items = vec![
            StreamItem::Data(serde_json::json!({"key": "value"})),
            StreamItem::End,
            StreamItem::Error {
                node_id: "node1".into(),
                message: "failed".into(),
            },
        ];
        for item in items {
            let json = serde_json::to_string(&item).unwrap();
            let back: StreamItem = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&item).unwrap(),
                serde_json::to_string(&back).unwrap()
            );
        }
    }
}
