// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Leaf node — executes workloads directly without delegation.

use crate::node::{
    CapacityInfo, ComputeNodeKind, HealthStatus, NodeConfig, NodeMetrics, NodeTopology,
    ResourceInfo, TreeMetrics, UtilizationInfo, Workload, WorkloadId, WorkloadInfo, WorkloadStatus,
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::debug;

/// Leaf node — executes workloads directly.
pub struct LeafNode {
    config: NodeConfig,
    resources: ResourceInfo,
    workloads: tokio::sync::RwLock<Vec<WorkloadInfo>>,
}

#[expect(
    missing_docs,
    reason = "fractal compute methods are internal and self-documenting"
)]
impl LeafNode {
    /// Create a new leaf node with the given configuration and resources.
    #[must_use]
    pub fn new(config: NodeConfig, resources: ResourceInfo) -> Self {
        Self {
            config,
            resources,
            workloads: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    pub fn parent_id(&self) -> Option<&str> {
        self.config.parent_id.as_deref()
    }

    pub fn depth(&self) -> usize {
        self.config.depth
    }

    pub fn topology(&self) -> NodeTopology {
        NodeTopology::Leaf
    }

    pub fn get_child_count(&self) -> usize {
        0
    }

    pub async fn get_resources(&self) -> Result<ResourceInfo> {
        Ok(self.resources)
    }

    pub async fn get_capacity(&self) -> Result<CapacityInfo> {
        let workloads = self.workloads.read().await;
        Ok(CapacityInfo {
            max_concurrent_workloads: 4,
            available_slots: 4 - workloads.len(),
            total_resources: self.resources,
            available_resources: self.resources,
        })
    }

    pub async fn get_utilization(&self) -> Result<UtilizationInfo> {
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

    pub async fn submit_workload(&self, workload: Workload) -> Result<WorkloadId> {
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

    pub async fn cancel_workload(&self, id: &WorkloadId) -> Result<()> {
        let mut workloads = self.workloads.write().await;
        if let Some(workload) = workloads.iter_mut().find(|w| &w.id == id) {
            workload.status = WorkloadStatus::Cancelled;
        }
        Ok(())
    }

    pub async fn get_workload_status(&self, id: &WorkloadId) -> Result<WorkloadStatus> {
        let workloads = self.workloads.read().await;
        workloads
            .iter()
            .find(|w| &w.id == id)
            .map(|w| w.status.clone())
            .context("Workload not found")
    }

    pub async fn list_workloads(&self) -> Result<Vec<WorkloadInfo>> {
        let workloads = self.workloads.read().await;
        Ok(workloads.clone())
    }

    pub async fn spawn_sub_node(&self, _config: NodeConfig) -> Result<Arc<ComputeNodeKind>> {
        anyhow::bail!("Leaf nodes cannot spawn sub-nodes")
    }

    pub async fn get_children(&self) -> Result<Vec<Arc<ComputeNodeKind>>> {
        Ok(Vec::new())
    }

    pub async fn get_all_descendants(&self) -> Result<Vec<Arc<ComputeNodeKind>>> {
        Ok(Vec::new())
    }

    pub async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    pub async fn get_metrics(&self) -> Result<NodeMetrics> {
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

    pub async fn get_subtree_metrics(&self) -> Result<TreeMetrics> {
        Ok(TreeMetrics {
            total_nodes: 1,
            total_workloads_active: self.get_utilization().await?.active_workloads,
            total_workloads_completed: self.get_metrics().await?.workloads_completed,
            aggregate_resources: self.resources,
            aggregate_utilization: self.get_utilization().await?,
        })
    }
}
