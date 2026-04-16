// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Compute node dispatch and shared data types
//!
//! Closed enum dispatch (`ComputeNodeKind`) replaces a trait object so every
//! node shares the same surface area at any scale.

mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use anyhow::Result;
use std::sync::Arc;

// =============================================================================
// COMPUTE NODE KIND (closed enum dispatch)
// =============================================================================

/// `ComputeNodeKind`: Isomorphic compute dispatch at any scale.
///
/// Replaces the former `ComputeNode` trait with closed enum dispatch — only
/// `Leaf` and `Parent` variants exist, matching nature's recursion pattern.
pub enum ComputeNodeKind {
    /// Leaf node (executes workloads directly)
    Leaf(crate::fractal::LeafNode),
    /// Parent node (distributes workloads to children)
    Parent(crate::fractal::ParentNode),
}

impl ComputeNodeKind {
    /// Unique node identifier
    pub fn node_id(&self) -> &str {
        match self {
            Self::Leaf(n) => n.node_id(),
            Self::Parent(n) => n.node_id(),
        }
    }

    /// Parent node ID (None if root)
    pub fn parent_id(&self) -> Option<&str> {
        match self {
            Self::Leaf(n) => n.parent_id(),
            Self::Parent(n) => n.parent_id(),
        }
    }

    /// Depth in fractal tree (0 = root)
    pub fn depth(&self) -> usize {
        match self {
            Self::Leaf(n) => n.depth(),
            Self::Parent(n) => n.depth(),
        }
    }

    /// Node topology
    pub fn topology(&self) -> NodeTopology {
        match self {
            Self::Leaf(n) => n.topology(),
            Self::Parent(n) => n.topology(),
        }
    }

    /// Is this a leaf node?
    pub fn is_leaf(&self) -> bool {
        self.get_child_count() == 0
    }

    /// Number of direct children
    pub fn get_child_count(&self) -> usize {
        match self {
            Self::Leaf(n) => n.get_child_count(),
            Self::Parent(n) => n.get_child_count(),
        }
    }

    /// Get resources (own + children if parent)
    pub async fn get_resources(&self) -> Result<ResourceInfo> {
        match self {
            Self::Leaf(n) => n.get_resources().await,
            Self::Parent(n) => n.get_resources().await,
        }
    }

    /// Get current capacity
    pub async fn get_capacity(&self) -> Result<CapacityInfo> {
        match self {
            Self::Leaf(n) => n.get_capacity().await,
            Self::Parent(n) => n.get_capacity().await,
        }
    }

    /// Get current utilization
    pub async fn get_utilization(&self) -> Result<UtilizationInfo> {
        match self {
            Self::Leaf(n) => n.get_utilization().await,
            Self::Parent(n) => n.get_utilization().await,
        }
    }

    /// Submit a workload for execution
    pub async fn submit_workload(&self, workload: Workload) -> Result<WorkloadId> {
        match self {
            Self::Leaf(n) => n.submit_workload(workload).await,
            Self::Parent(n) => n.submit_workload(workload).await,
        }
    }

    /// Cancel a workload
    pub async fn cancel_workload(&self, id: &WorkloadId) -> Result<()> {
        match self {
            Self::Leaf(n) => n.cancel_workload(id).await,
            Self::Parent(n) => n.cancel_workload(id).await,
        }
    }

    /// Get the status of a workload
    pub async fn get_workload_status(&self, workload_id: &WorkloadId) -> Result<WorkloadStatus> {
        match self {
            Self::Leaf(n) => n.get_workload_status(workload_id).await,
            Self::Parent(n) => n.get_workload_status(workload_id).await,
        }
    }

    /// List all workloads on this node
    pub async fn list_workloads(&self) -> Result<Vec<WorkloadInfo>> {
        match self {
            Self::Leaf(n) => n.list_workloads().await,
            Self::Parent(n) => n.list_workloads().await,
        }
    }

    /// Spawn a sub-node (fractal recursion)
    pub async fn spawn_sub_node(&self, config: NodeConfig) -> Result<Arc<ComputeNodeKind>> {
        match self {
            Self::Leaf(n) => n.spawn_sub_node(config).await,
            Self::Parent(n) => n.spawn_sub_node(config).await,
        }
    }

    /// Get direct children
    pub async fn get_children(&self) -> Result<Vec<Arc<ComputeNodeKind>>> {
        match self {
            Self::Leaf(n) => n.get_children().await,
            Self::Parent(n) => n.get_children().await,
        }
    }

    /// Get all descendants (recursive)
    pub async fn get_all_descendants(&self) -> Result<Vec<Arc<ComputeNodeKind>>> {
        match self {
            Self::Leaf(n) => n.get_all_descendants().await,
            Self::Parent(n) => n.get_all_descendants().await,
        }
    }

    /// Get total node count (self + descendants)
    pub async fn get_node_count(&self) -> Result<usize> {
        Ok(1 + self.get_all_descendants().await?.len())
    }

    /// Health check (own + children)
    pub async fn health_check(&self) -> Result<HealthStatus> {
        match self {
            Self::Leaf(n) => n.health_check().await,
            Self::Parent(n) => n.health_check().await,
        }
    }

    /// Get metrics for this node
    pub async fn get_metrics(&self) -> Result<NodeMetrics> {
        match self {
            Self::Leaf(n) => n.get_metrics().await,
            Self::Parent(n) => n.get_metrics().await,
        }
    }

    /// Get recursive subtree metrics
    pub async fn get_subtree_metrics(&self) -> Result<TreeMetrics> {
        match self {
            Self::Leaf(n) => n.get_subtree_metrics().await,
            Self::Parent(n) => n.get_subtree_metrics().await,
        }
    }
}
