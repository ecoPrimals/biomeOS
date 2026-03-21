// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

// =============================================================================
// Fractal Compute Tests - Isomorphic Interface Validation
// =============================================================================

use biomeos_compute::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_binary_tree_structure() {
    // Build a binary tree fractal (depth 2)
    let root = FractalBuilder::new("root")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .resources(ResourceInfo {
            cpu_cores: 16,
            memory_mb: 32768,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 100_000,
        })
        .build()
        .await
        .expect("Failed to build fractal");

    // Verify structure
    assert_eq!(root.node_id(), "root");
    assert_eq!(root.depth(), 0);
    assert_eq!(root.topology(), NodeTopology::BinaryTree);
    assert_eq!(root.get_child_count(), 2);

    // Verify total node count: 1 root + 2 children + 4 grandchildren = 7
    let node_count = root
        .get_node_count()
        .await
        .expect("Failed to get node count");
    assert_eq!(node_count, 7, "Binary tree depth 2 should have 7 nodes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_resource_aggregation() {
    // Build a binary tree
    let root = FractalBuilder::new("test-root")
        .topology(NodeTopology::BinaryTree)
        .depth(1) // Just 1 level for simplicity
        .resources(ResourceInfo {
            cpu_cores: 16,
            memory_mb: 16384,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 50000,
        })
        .build()
        .await
        .expect("Failed to build fractal");

    // Get aggregated resources
    let resources = root.get_resources().await.expect("Failed to get resources");

    // Should aggregate children's resources
    // 2 children with 8 cores each = 16 total
    assert_eq!(resources.cpu_cores, 16);
    assert_eq!(resources.memory_mb, 16384);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_workload_submission() {
    // Build a simple fractal
    let root = FractalBuilder::new("workload-test")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .resources(ResourceInfo {
            cpu_cores: 8,
            memory_mb: 8192,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 10000,
        })
        .build()
        .await
        .expect("Failed to build fractal");

    // Submit a workload
    let workload = Workload::builder("test-workload", Runtime::Native)
        .code(b"print(\"Hello, fractal!\")".to_vec())
        .cpu_cores(1)
        .memory_mb(256)
        .build();

    let workload_id = root
        .submit_workload(workload)
        .await
        .expect("Failed to submit workload");

    // Verify it was submitted
    let status = root
        .get_workload_status(&workload_id)
        .await
        .expect("Failed to get workload status");

    assert_eq!(status, WorkloadStatus::Running);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_isomorphic_interface() {
    // Build a fractal
    let root = FractalBuilder::new("iso-test")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .build()
        .await
        .expect("Failed to build fractal");

    // Root node should implement ComputeNode
    let _: &dyn ComputeNode = root.as_ref();

    // Children should also implement ComputeNode
    let children = root.get_children().await.expect("Failed to get children");
    for child in children {
        let _: &dyn ComputeNode = child.as_ref();

        // Grandchildren too
        let grandchildren = child
            .get_children()
            .await
            .expect("Failed to get grandchildren");
        for grandchild in grandchildren {
            let _: &dyn ComputeNode = grandchild.as_ref();
        }
    }

    // ALL nodes respond to the same methods
    assert!(root.get_resources().await.is_ok());
    assert!(root.get_capacity().await.is_ok());
    assert!(root.get_utilization().await.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_check_recursive() {
    // Build a fractal
    let root = FractalBuilder::new("health-test")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .build()
        .await
        .expect("Failed to build fractal");

    // Health check should propagate through tree
    let health = root.health_check().await.expect("Failed to check health");
    assert_eq!(health, HealthStatus::Healthy);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_metrics_aggregation() {
    // Build a fractal
    let root = FractalBuilder::new("metrics-test")
        .topology(NodeTopology::BinaryTree)
        .depth(2)
        .build()
        .await
        .expect("Failed to build fractal");

    // Submit some workloads
    for i in 0..4 {
        let workload = Workload::builder(format!("workload-{i}"), Runtime::Native)
            .cpu_cores(1)
            .build();
        root.submit_workload(workload)
            .await
            .expect("Failed to submit workload");
    }

    // Get subtree metrics
    let metrics = root
        .get_subtree_metrics()
        .await
        .expect("Failed to get metrics");

    // Should show 7 total nodes
    assert_eq!(metrics.total_nodes, 7);

    // Should show 4 active workloads
    assert_eq!(metrics.total_workloads_active, 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_n_ary_tree() {
    // Build a quad tree (4 children)
    let root = FractalBuilder::new("quad-root")
        .topology(NodeTopology::NAryTree {
            branching_factor: 4,
        })
        .depth(2)
        .resources(ResourceInfo {
            cpu_cores: 16,
            memory_mb: 16384,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 50000,
        })
        .build()
        .await
        .expect("Failed to build fractal");

    // Should have 4 children
    assert_eq!(root.get_child_count(), 4);

    // Total nodes: 1 + 4 + 16 = 21
    let node_count = root
        .get_node_count()
        .await
        .expect("Failed to get node count");
    assert_eq!(node_count, 21, "Quad tree depth 2 should have 21 nodes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_weighted_resource_allocation() {
    // Build a tree with weighted resource allocation
    let root = FractalBuilder::new("weighted-root")
        .topology(NodeTopology::BinaryTree)
        .depth(1)
        .resource_allocation(ResourceAllocation::Weighted {
            weights: vec![0.75, 0.25], // 75% to first child, 25% to second
        })
        .resources(ResourceInfo {
            cpu_cores: 16,
            memory_mb: 16384,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 50000,
        })
        .build()
        .await
        .expect("Failed to build fractal");

    // Get children
    let children = root.get_children().await.expect("Failed to get children");
    assert_eq!(children.len(), 2);

    // First child should have ~12 cores (75% of 16)
    let child1_resources = children[0]
        .get_resources()
        .await
        .expect("Failed to get child1 resources");
    assert_eq!(child1_resources.cpu_cores, 12);

    // Second child should have ~4 cores (25% of 16)
    let child2_resources = children[1]
        .get_resources()
        .await
        .expect("Failed to get child2 resources");
    assert_eq!(child2_resources.cpu_cores, 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_leaf_node_direct() {
    // Create a leaf node directly
    let config = NodeConfig {
        node_id: "leaf-1".to_string(),
        parent_id: None,
        depth: 0,
        topology: NodeTopology::Leaf,
        resource_type: ResourceType::Cpu,
        resource_allocation: ResourceAllocation::Equal,
    };

    let resources = ResourceInfo {
        cpu_cores: 4,
        memory_mb: 4096,
        gpu_count: 0,
        gpu_memory_mb: 0,
        disk_mb: 10000,
    };

    let leaf: std::sync::Arc<dyn ComputeNode> =
        std::sync::Arc::new(LeafNode::new(config, resources.clone()));

    // Verify it's a leaf
    assert!(leaf.is_leaf());
    assert_eq!(leaf.get_child_count(), 0);

    // Submit workload
    let workload = Workload::builder("leaf-workload", Runtime::Native).build();
    let id = leaf
        .submit_workload(workload)
        .await
        .expect("Failed to submit to leaf");

    // Check status
    let status = leaf
        .get_workload_status(&id)
        .await
        .expect("Failed to get status");
    assert_eq!(status, WorkloadStatus::Running);
}
