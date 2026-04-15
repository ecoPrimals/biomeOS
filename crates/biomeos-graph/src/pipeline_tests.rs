// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::events::GraphEventBroadcaster;
use crate::graph::{CoordinationPattern, DeploymentGraph, GraphDefinition, GraphId, GraphMetadata};
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
        gate: None,
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
                            StreamItem::Error {
                                node_id: "source".to_string(),
                                message: "test error".to_string(),
                            }
                        } else {
                            StreamItem::End
                        }
                    }
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
