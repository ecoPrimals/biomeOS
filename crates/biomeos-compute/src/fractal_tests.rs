// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

use super::*;
use crate::ComputeNodeKind;
use crate::node::Runtime;
use std::sync::Arc;

#[tokio::test]
async fn test_fractal_builder_binary_tree() {
    let root = FractalBuilder::new("root")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .build()
        .await
        .unwrap();
    let count = root.get_node_count().await.unwrap();
    assert!(count >= 3);
}

#[tokio::test]
async fn test_fractal_builder_quad_tree() {
    let root = FractalBuilder::new("quad")
        .topology(NodeTopology::QuadTree)
        .depth(1)
        .build()
        .await
        .unwrap();
    let count = root.get_node_count().await.unwrap();
    assert!(count >= 5);
}

#[tokio::test]
async fn test_fractal_builder_hybrid_matches_binary_branching() {
    let root = FractalBuilder::new("hybrid-root")
        .topology(NodeTopology::Hybrid)
        .depth(2)
        .resources(ResourceInfo {
            cpu_cores: 8,
            memory_mb: 8192,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 10_000,
        })
        .build()
        .await
        .unwrap();
    assert_eq!(root.get_child_count(), 2);
    let count = root.get_node_count().await.unwrap();
    assert_eq!(count, 7);
}

#[tokio::test]
async fn test_fractal_builder_leaf_node() {
    let root = FractalBuilder::new("leaf")
        .topology(NodeTopology::Leaf)
        .depth(0)
        .build()
        .await
        .unwrap();
    assert!(root.is_leaf());
    assert_eq!(root.get_child_count(), 0);
}

#[tokio::test]
async fn test_fractal_leaf_node_workload() {
    let root = FractalBuilder::new("leaf")
        .topology(NodeTopology::Leaf)
        .depth(0)
        .build()
        .await
        .unwrap();
    let workload = Workload::new("test", Runtime::Native);
    let id = root.submit_workload(workload).await.unwrap();
    let status = root.get_workload_status(&id).await.unwrap();
    assert!(matches!(status, WorkloadStatus::Running));
}

#[tokio::test]
async fn test_fractal_health_check() {
    let root = FractalBuilder::new("health")
        .depth(1)
        .build()
        .await
        .unwrap();
    let status = root.health_check().await.unwrap();
    assert!(matches!(status, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_fractal_n_ary_branching_factor_three() {
    let root = FractalBuilder::new("n3")
        .topology(NodeTopology::NAryTree {
            branching_factor: 3,
        })
        .depth(1)
        .resources(ResourceInfo {
            cpu_cores: 9,
            memory_mb: 9000,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 50_000,
        })
        .build()
        .await
        .unwrap();
    assert_eq!(root.get_child_count(), 3);
    let count = root.get_node_count().await.unwrap();
    assert_eq!(count, 4);
}

#[tokio::test]
async fn test_leaf_spawn_sub_node_errors() {
    let config = NodeConfig {
        node_id: "solo".to_string(),
        parent_id: None,
        depth: 0,
        topology: NodeTopology::Leaf,
        resource_type: ResourceType::Cpu,
        resource_allocation: ResourceAllocation::Equal,
    };
    let resources = ResourceInfo {
        cpu_cores: 2,
        memory_mb: 1024,
        gpu_count: 0,
        gpu_memory_mb: 0,
        disk_mb: 1000,
    };
    let leaf: Arc<ComputeNodeKind> =
        Arc::new(ComputeNodeKind::Leaf(LeafNode::new(config, resources)));
    let err = leaf
        .spawn_sub_node(NodeConfig {
            node_id: "child".to_string(),
            parent_id: Some("solo".to_string()),
            depth: 1,
            topology: NodeTopology::Leaf,
            resource_type: ResourceType::Cpu,
            resource_allocation: ResourceAllocation::Equal,
        })
        .await;
    assert!(err.is_err());
}
