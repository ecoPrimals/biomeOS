// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// =============================================================================
// Fractal Builder - Recursive Node Construction
// =============================================================================
//
// Builds isomorphic fractal compute structures:
// - Binary trees
// - N-ary trees
// - Quad trees
// - Hybrid fractals
//
// "Build once, scales infinitely" - Nature's recursion
//
// =============================================================================

use crate::node::{
    CapacityInfo, ComputeNode, HealthStatus, NodeConfig, NodeMetrics, NodeTopology,
    ResourceAllocation, ResourceInfo, ResourceType, TreeMetrics, UtilizationInfo, Workload,
    WorkloadId, WorkloadInfo, WorkloadStatus,
};
use anyhow::{Context, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info};

/// Type alias for boxed async computations
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// =============================================================================
// FRACTAL BUILDER
// =============================================================================

/// Builder for fractal compute structures
pub struct FractalBuilder {
    root_id: String,
    topology: NodeTopology,
    depth: usize,
    resource_type: ResourceType,
    resource_allocation: ResourceAllocation,
    base_resources: ResourceInfo,
}

impl FractalBuilder {
    /// Create a new fractal builder
    pub fn new(root_id: impl Into<String>) -> Self {
        Self {
            root_id: root_id.into(),
            topology: NodeTopology::BinaryTree,
            depth: 2,
            resource_type: ResourceType::Cpu,
            resource_allocation: ResourceAllocation::Equal,
            base_resources: ResourceInfo {
                cpu_cores: 8,
                memory_mb: 16384,
                gpu_count: 0,
                gpu_memory_mb: 0,
                disk_mb: 100_000,
            },
        }
    }

    /// Set topology
    pub fn topology(mut self, topology: NodeTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Set depth
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Set resource type
    pub fn resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = resource_type;
        self
    }

    /// Set resource allocation strategy
    pub fn resource_allocation(mut self, allocation: ResourceAllocation) -> Self {
        self.resource_allocation = allocation;
        self
    }

    /// Set base resources
    pub fn resources(mut self, resources: ResourceInfo) -> Self {
        self.base_resources = resources;
        self
    }

    /// Build the fractal structure
    pub async fn build(self) -> Result<Arc<dyn ComputeNode>> {
        info!(
            "Building fractal compute structure: root={}, topology={:?}, depth={}",
            self.root_id, self.topology, self.depth
        );

        let root_config = NodeConfig {
            node_id: self.root_id.clone(),
            parent_id: None,
            depth: 0,
            topology: self.topology,
            resource_type: self.resource_type,
            resource_allocation: self.resource_allocation.clone(),
        };

        let root = self
            .build_node_recursive(root_config, self.base_resources, 0)
            .await?;

        info!(
            "Fractal structure built successfully: {} nodes",
            root.get_node_count().await?
        );

        Ok(root)
    }

    /// Recursive node construction
    fn build_node_recursive(
        &self,
        config: NodeConfig,
        resources: ResourceInfo,
        current_depth: usize,
    ) -> BoxFuture<'_, Result<Arc<dyn ComputeNode>>> {
        Box::pin(async move {
            debug!(
                "Building node: {} at depth {}",
                config.node_id, current_depth
            );

            // If we've reached max depth, create a leaf node
            if current_depth >= self.depth {
                debug!("Creating leaf node: {}", config.node_id);
                return Ok(Arc::new(LeafNode::new(config, resources)) as Arc<dyn ComputeNode>);
            }

            // Otherwise, create a parent node with children
            let branching_factor = self.get_branching_factor();
            let mut children = Vec::new();

            // Split resources among children
            let child_resources = self.split_resources(&resources, branching_factor);

            // Recursively build children
            for (i, _) in child_resources.iter().enumerate().take(branching_factor) {
                let child_id = format!("{}-{}", config.node_id, i);
                let child_config = NodeConfig {
                    node_id: child_id,
                    parent_id: Some(config.node_id.clone()),
                    depth: current_depth + 1,
                    topology: config.topology,
                    resource_type: config.resource_type,
                    resource_allocation: config.resource_allocation.clone(),
                };

                let child = self
                    .build_node_recursive(child_config, child_resources[i], current_depth + 1)
                    .await?;

                children.push(child);
            }

            debug!(
                "Creating parent node: {} with {} children",
                config.node_id,
                children.len()
            );
            Ok(Arc::new(ParentNode::new(config, resources, children)) as Arc<dyn ComputeNode>)
        })
    }

    /// Get branching factor for topology
    fn get_branching_factor(&self) -> usize {
        match self.topology {
            NodeTopology::Leaf => 0,
            NodeTopology::BinaryTree | NodeTopology::Hybrid => 2, // Default for hybrid
            NodeTopology::NAryTree { branching_factor } => branching_factor,
            NodeTopology::QuadTree => 4,
        }
    }

    /// Split resources among children
    fn split_resources(&self, resources: &ResourceInfo, num_children: usize) -> Vec<ResourceInfo> {
        match &self.resource_allocation {
            ResourceAllocation::Equal => {
                // Equal split
                let mut splits = Vec::new();
                for _ in 0..num_children {
                    splits.push(ResourceInfo {
                        cpu_cores: resources.cpu_cores / num_children,
                        memory_mb: resources.memory_mb / num_children,
                        gpu_count: resources.gpu_count / num_children,
                        gpu_memory_mb: resources.gpu_memory_mb / num_children,
                        disk_mb: resources.disk_mb / num_children,
                    });
                }
                splits
            }
            ResourceAllocation::Weighted { weights } => {
                let total_weight: f64 = weights.iter().sum();
                let scale = |count: usize, ratio: f64| -> usize {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        clippy::cast_precision_loss,
                        reason = "resource counts are small positive integers; \
                                  floor-truncation is intentional for allocation splits"
                    )]
                    {
                        (count as f64 * ratio) as usize
                    }
                };
                let mut splits = Vec::new();
                for weight in weights {
                    let ratio = weight / total_weight;
                    splits.push(ResourceInfo {
                        cpu_cores: scale(resources.cpu_cores, ratio),
                        memory_mb: scale(resources.memory_mb, ratio),
                        gpu_count: scale(resources.gpu_count, ratio),
                        gpu_memory_mb: scale(resources.gpu_memory_mb, ratio),
                        disk_mb: scale(resources.disk_mb, ratio),
                    });
                }
                splits
            }
            _ => {
                // Default to equal for other strategies
                self.split_resources(resources, num_children)
            }
        }
    }
}

// =============================================================================
// LEAF NODE IMPLEMENTATION
// =============================================================================

/// Leaf node - executes workloads directly
pub struct LeafNode {
    config: NodeConfig,
    resources: ResourceInfo,
    workloads: tokio::sync::RwLock<Vec<WorkloadInfo>>,
}

impl LeafNode {
    /// Create a new leaf node with the given configuration and resources
    pub fn new(config: NodeConfig, resources: ResourceInfo) -> Self {
        Self {
            config,
            resources,
            workloads: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl ComputeNode for LeafNode {
    fn node_id(&self) -> &str {
        &self.config.node_id
    }

    fn parent_id(&self) -> Option<&str> {
        self.config.parent_id.as_deref()
    }

    fn depth(&self) -> usize {
        self.config.depth
    }

    fn topology(&self) -> NodeTopology {
        NodeTopology::Leaf
    }

    fn get_child_count(&self) -> usize {
        0
    }

    async fn get_resources(&self) -> Result<ResourceInfo> {
        Ok(self.resources)
    }

    async fn get_capacity(&self) -> Result<CapacityInfo> {
        let workloads = self.workloads.read().await;
        Ok(CapacityInfo {
            max_concurrent_workloads: 4,
            available_slots: 4 - workloads.len(),
            total_resources: self.resources,
            available_resources: self.resources,
        })
    }

    async fn get_utilization(&self) -> Result<UtilizationInfo> {
        let workloads = self.workloads.read().await;
        let active_count = workloads
            .iter()
            .filter(|w| matches!(w.status, WorkloadStatus::Running))
            .count();

        Ok(UtilizationInfo {
            cpu_utilization_percent: (active_count as f64 / 4.0) * 100.0,
            memory_utilization_percent: (active_count as f64 / 4.0) * 100.0,
            gpu_utilization_percent: 0.0,
            active_workloads: active_count,
        })
    }

    async fn submit_workload(&self, workload: Workload) -> Result<WorkloadId> {
        let mut workloads = self.workloads.write().await;

        let info = WorkloadInfo {
            id: workload.id.clone(),
            name: workload.name.clone(),
            status: WorkloadStatus::Running,
            node_id: self.config.node_id.clone(),
            submitted_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
        };

        workloads.push(info);

        debug!(
            "Workload {} submitted to leaf node {}",
            workload.id, self.config.node_id
        );
        Ok(workload.id)
    }

    async fn cancel_workload(&self, id: &WorkloadId) -> Result<()> {
        let mut workloads = self.workloads.write().await;
        if let Some(workload) = workloads.iter_mut().find(|w| &w.id == id) {
            workload.status = WorkloadStatus::Cancelled;
        }
        Ok(())
    }

    async fn get_workload_status(&self, id: &WorkloadId) -> Result<WorkloadStatus> {
        let workloads = self.workloads.read().await;
        workloads
            .iter()
            .find(|w| &w.id == id)
            .map(|w| w.status.clone())
            .context("Workload not found")
    }

    async fn list_workloads(&self) -> Result<Vec<WorkloadInfo>> {
        let workloads = self.workloads.read().await;
        Ok(workloads.clone())
    }

    async fn spawn_sub_node(&self, _config: NodeConfig) -> Result<Arc<dyn ComputeNode>> {
        anyhow::bail!("Leaf nodes cannot spawn sub-nodes")
    }

    async fn get_children(&self) -> Result<Vec<Arc<dyn ComputeNode>>> {
        Ok(Vec::new())
    }

    async fn get_all_descendants(&self) -> Result<Vec<Arc<dyn ComputeNode>>> {
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    async fn get_metrics(&self) -> Result<NodeMetrics> {
        let workloads = self.workloads.read().await;
        let completed = workloads
            .iter()
            .filter(|w| matches!(w.status, WorkloadStatus::Completed))
            .count();

        Ok(NodeMetrics {
            node_id: self.config.node_id.clone(),
            workloads_submitted: workloads.len() as u64,
            workloads_completed: completed as u64,
            workloads_failed: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            current_utilization: self.get_utilization().await?,
        })
    }

    async fn get_subtree_metrics(&self) -> Result<TreeMetrics> {
        Ok(TreeMetrics {
            total_nodes: 1,
            total_workloads_active: self.get_utilization().await?.active_workloads,
            total_workloads_completed: self.get_metrics().await?.workloads_completed,
            aggregate_resources: self.resources,
            aggregate_utilization: self.get_utilization().await?,
        })
    }
}

// =============================================================================
// PARENT NODE IMPLEMENTATION
// =============================================================================

/// Parent node - distributes workloads to children
///
/// Note: The `resources` field represents this node's allocated resources
/// but is currently unused in favor of aggregating child resources dynamically.
/// This is intentional - we may use it in future for resource reservation/limits.
pub struct ParentNode {
    config: NodeConfig,
    /// Planned: wire up for resource reservation/limits in Phase 3.
    _resources: ResourceInfo,
    children: Vec<Arc<dyn ComputeNode>>,
}

impl ParentNode {
    /// Create a new parent node with the given configuration, resources, and children
    pub fn new(
        config: NodeConfig,
        resources: ResourceInfo,
        children: Vec<Arc<dyn ComputeNode>>,
    ) -> Self {
        Self {
            config,
            _resources: resources,
            children,
        }
    }
}

#[async_trait::async_trait]
impl ComputeNode for ParentNode {
    fn node_id(&self) -> &str {
        &self.config.node_id
    }

    fn parent_id(&self) -> Option<&str> {
        self.config.parent_id.as_deref()
    }

    fn depth(&self) -> usize {
        self.config.depth
    }

    fn topology(&self) -> NodeTopology {
        self.config.topology
    }

    fn get_child_count(&self) -> usize {
        self.children.len()
    }

    async fn get_resources(&self) -> Result<ResourceInfo> {
        // Aggregate children's resources
        let mut total = ResourceInfo {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 0,
        };

        for child in &self.children {
            let child_resources = child.get_resources().await?;
            total.aggregate(&child_resources);
        }

        Ok(total)
    }

    async fn get_capacity(&self) -> Result<CapacityInfo> {
        // Aggregate children's capacity
        let mut total_slots = 0;
        let mut available_slots = 0;
        let mut total_resources = ResourceInfo {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 0,
        };
        let mut available_resources = ResourceInfo {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 0,
        };

        for child in &self.children {
            let capacity = child.get_capacity().await?;
            total_slots += capacity.max_concurrent_workloads;
            available_slots += capacity.available_slots;
            total_resources.aggregate(&capacity.total_resources);
            available_resources.aggregate(&capacity.available_resources);
        }

        Ok(CapacityInfo {
            max_concurrent_workloads: total_slots,
            available_slots,
            total_resources,
            available_resources,
        })
    }

    async fn get_utilization(&self) -> Result<UtilizationInfo> {
        // Aggregate children's utilization
        let mut total_active = 0;
        let mut avg_cpu = 0.0;
        let mut avg_memory = 0.0;
        let mut avg_gpu = 0.0;

        for child in &self.children {
            let util = child.get_utilization().await?;
            total_active += util.active_workloads;
            avg_cpu += util.cpu_utilization_percent;
            avg_memory += util.memory_utilization_percent;
            avg_gpu += util.gpu_utilization_percent;
        }

        let count = self.children.len() as f64;
        Ok(UtilizationInfo {
            cpu_utilization_percent: avg_cpu / count,
            memory_utilization_percent: avg_memory / count,
            gpu_utilization_percent: avg_gpu / count,
            active_workloads: total_active,
        })
    }

    async fn submit_workload(&self, workload: Workload) -> Result<WorkloadId> {
        // Find least loaded child and submit there
        let mut best_child = None;
        let mut min_load = usize::MAX;

        for child in &self.children {
            let util = child.get_utilization().await?;
            if util.active_workloads < min_load {
                min_load = util.active_workloads;
                best_child = Some(child);
            }
        }

        match best_child {
            Some(child) => {
                debug!(
                    "Routing workload {} to child {}",
                    workload.id,
                    child.node_id()
                );
                child.submit_workload(workload).await
            }
            None => anyhow::bail!("No children available"),
        }
    }

    async fn cancel_workload(&self, id: &WorkloadId) -> Result<()> {
        // Try to cancel in all children
        for child in &self.children {
            let _ = child.cancel_workload(id).await;
        }
        Ok(())
    }

    async fn get_workload_status(&self, id: &WorkloadId) -> Result<WorkloadStatus> {
        // Search all children
        for child in &self.children {
            if let Ok(status) = child.get_workload_status(id).await {
                return Ok(status);
            }
        }
        anyhow::bail!("Workload not found in any child")
    }

    async fn list_workloads(&self) -> Result<Vec<WorkloadInfo>> {
        let mut all_workloads = Vec::new();
        for child in &self.children {
            all_workloads.extend(child.list_workloads().await?);
        }
        Ok(all_workloads)
    }

    async fn spawn_sub_node(&self, _config: NodeConfig) -> Result<Arc<dyn ComputeNode>> {
        anyhow::bail!(
            "ParentNode children are immutable after construction — \
             use ParentNodeBuilder to define topology at creation time"
        )
    }

    async fn get_children(&self) -> Result<Vec<Arc<dyn ComputeNode>>> {
        Ok(self.children.clone())
    }

    async fn get_all_descendants(&self) -> Result<Vec<Arc<dyn ComputeNode>>> {
        let mut descendants = Vec::new();
        for child in &self.children {
            descendants.push(child.clone());
            descendants.extend(child.get_all_descendants().await?);
        }
        Ok(descendants)
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // Check all children
        for child in &self.children {
            let status = child.health_check().await?;
            if !matches!(status, HealthStatus::Healthy) {
                return Ok(HealthStatus::Degraded {
                    reason: format!("Child {} unhealthy", child.node_id()),
                });
            }
        }
        Ok(HealthStatus::Healthy)
    }

    async fn get_metrics(&self) -> Result<NodeMetrics> {
        // Aggregate children's metrics
        let mut total_submitted = 0;
        let mut total_completed = 0;
        let mut total_failed = 0;
        let mut total_time = 0;

        for child in &self.children {
            let metrics = child.get_metrics().await?;
            total_submitted += metrics.workloads_submitted;
            total_completed += metrics.workloads_completed;
            total_failed += metrics.workloads_failed;
            total_time += metrics.total_execution_time_ms;
        }

        let avg_time = if total_completed > 0 {
            total_time as f64 / total_completed as f64
        } else {
            0.0
        };

        Ok(NodeMetrics {
            node_id: self.config.node_id.clone(),
            workloads_submitted: total_submitted,
            workloads_completed: total_completed,
            workloads_failed: total_failed,
            total_execution_time_ms: total_time,
            average_execution_time_ms: avg_time,
            current_utilization: self.get_utilization().await?,
        })
    }

    async fn get_subtree_metrics(&self) -> Result<TreeMetrics> {
        let mut total_nodes = 1; // Self
        let mut total_active = 0;
        let mut total_completed = 0;
        let mut aggregate_resources = ResourceInfo {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 0,
        };

        for child in &self.children {
            let child_metrics = child.get_subtree_metrics().await?;
            total_nodes += child_metrics.total_nodes;
            total_active += child_metrics.total_workloads_active;
            total_completed += child_metrics.total_workloads_completed;
            aggregate_resources.aggregate(&child_metrics.aggregate_resources);
        }

        Ok(TreeMetrics {
            total_nodes,
            total_workloads_active: total_active,
            total_workloads_completed: total_completed,
            aggregate_resources,
            aggregate_utilization: self.get_utilization().await?,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::node::Runtime;

    #[tokio::test]
    async fn test_fractal_builder_binary_tree() {
        let root = FractalBuilder::new("root")
            .topology(NodeTopology::BinaryTree)
            .depth(2)
            .build()
            .await
            .unwrap();
        let count = root.get_node_count().await.unwrap();
        assert!(count >= 3); // root + 2 children at least, or more at depth 2
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
        assert!(count >= 5); // root + 4 children
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
}
