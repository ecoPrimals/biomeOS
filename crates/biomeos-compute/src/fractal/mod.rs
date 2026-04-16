// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Fractal Builder — recursive node construction for isomorphic compute trees.
//!
//! Builds binary, n-ary, quad, and hybrid fractal topologies using the same
//! resource-splitting strategy at every depth level.

mod leaf;
mod parent;

pub use leaf::LeafNode;
pub use parent::ParentNode;

use crate::node::{
    ComputeNodeKind, NodeConfig, NodeTopology, ResourceAllocation, ResourceInfo, ResourceType,
};
#[cfg(test)]
pub(crate) use crate::node::{HealthStatus, Workload, WorkloadStatus};
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

/// Builder for fractal compute structures.
pub struct FractalBuilder {
    root_id: String,
    topology: NodeTopology,
    depth: usize,
    resource_type: ResourceType,
    resource_allocation: ResourceAllocation,
    base_resources: ResourceInfo,
}

impl FractalBuilder {
    /// Create a new fractal builder.
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

    /// Set topology.
    #[must_use]
    pub const fn topology(mut self, topology: NodeTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Set depth.
    #[must_use]
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Set resource type.
    #[must_use]
    pub const fn resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = resource_type;
        self
    }

    /// Set resource allocation strategy.
    #[must_use]
    pub fn resource_allocation(mut self, allocation: ResourceAllocation) -> Self {
        self.resource_allocation = allocation;
        self
    }

    /// Set base resources.
    #[must_use]
    pub const fn resources(mut self, resources: ResourceInfo) -> Self {
        self.base_resources = resources;
        self
    }

    /// Build the fractal structure.
    pub async fn build(self) -> Result<Arc<ComputeNodeKind>> {
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

    /// Recursive node construction — returns a boxed future for finite async state machine.
    fn build_node_recursive(
        &self,
        config: NodeConfig,
        resources: ResourceInfo,
        current_depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Arc<ComputeNodeKind>>> + Send + '_>,
    > {
        Box::pin(async move {
            debug!(
                "Building node: {} at depth {}",
                config.node_id, current_depth
            );

            if current_depth >= self.depth {
                debug!("Creating leaf node: {}", config.node_id);
                return Ok(Arc::new(ComputeNodeKind::Leaf(LeafNode::new(
                    config, resources,
                ))));
            }

            let branching_factor = self.get_branching_factor();
            let mut children = Vec::new();
            let child_resources = self.split_resources(&resources, branching_factor);

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
            Ok(Arc::new(ComputeNodeKind::Parent(ParentNode::new(
                config, resources, children,
            ))))
        })
    }

    const fn get_branching_factor(&self) -> usize {
        match self.topology {
            NodeTopology::Leaf => 0,
            NodeTopology::BinaryTree | NodeTopology::Hybrid => 2,
            NodeTopology::NAryTree { branching_factor } => branching_factor,
            NodeTopology::QuadTree => 4,
        }
    }

    fn split_resources(&self, resources: &ResourceInfo, num_children: usize) -> Vec<ResourceInfo> {
        match &self.resource_allocation {
            ResourceAllocation::Equal => {
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
            _ => self.split_resources(resources, num_children),
        }
    }
}

#[cfg(test)]
#[path = "../fractal_tests.rs"]
mod tests;
