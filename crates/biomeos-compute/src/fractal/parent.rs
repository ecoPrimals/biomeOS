// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Parent node — distributes workloads across children via `ComputeNodeKind` dispatch.

use crate::node::{
    CapacityInfo, ComputeNodeKind, HealthStatus, NodeConfig, NodeMetrics, NodeTopology,
    ResourceInfo, TreeMetrics, UtilizationInfo, Workload, WorkloadId, WorkloadInfo, WorkloadStatus,
};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Parent node — distributes workloads to children.
///
/// The `_resources` field represents this node's allocated resources but is
/// currently unused in favor of aggregating child resources dynamically.
/// Reserved for future resource reservation/limits.
pub struct ParentNode {
    config: NodeConfig,
    _resources: ResourceInfo,
    children: Vec<Arc<ComputeNodeKind>>,
}

#[allow(missing_docs)]
impl ParentNode {
    /// Create a new parent node with the given configuration, resources, and children.
    #[must_use]
    pub fn new(
        config: NodeConfig,
        resources: ResourceInfo,
        children: Vec<Arc<ComputeNodeKind>>,
    ) -> Self {
        Self {
            config,
            _resources: resources,
            children,
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
        self.config.topology
    }

    pub fn get_child_count(&self) -> usize {
        self.children.len()
    }

    pub fn get_resources(&self) -> BoxFuture<'_, Result<ResourceInfo>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut total = ResourceInfo {
                cpu_cores: 0,
                memory_mb: 0,
                gpu_count: 0,
                gpu_memory_mb: 0,
                disk_mb: 0,
            };
            for child in &children {
                total.aggregate(&child.get_resources().await?);
            }
            Ok(total)
        })
    }

    pub fn get_capacity(&self) -> BoxFuture<'_, Result<CapacityInfo>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut total_slots = 0;
            let mut available_slots = 0;
            let mut total_resources = ResourceInfo::zeroed();
            let mut available_resources = ResourceInfo::zeroed();

            for child in &children {
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
        })
    }

    pub fn get_utilization(&self) -> BoxFuture<'_, Result<UtilizationInfo>> {
        let children = self.children.clone();
        Box::pin(async move {
            let (mut total_active, mut avg_cpu, mut avg_memory, mut avg_gpu) = (0, 0.0, 0.0, 0.0);

            for child in &children {
                let util = child.get_utilization().await?;
                total_active += util.active_workloads;
                avg_cpu += util.cpu_utilization_percent;
                avg_memory += util.memory_utilization_percent;
                avg_gpu += util.gpu_utilization_percent;
            }

            let count = children.len().max(1) as f64;
            Ok(UtilizationInfo {
                cpu_utilization_percent: avg_cpu / count,
                memory_utilization_percent: avg_memory / count,
                gpu_utilization_percent: avg_gpu / count,
                active_workloads: total_active,
            })
        })
    }

    pub fn submit_workload(&self, workload: Workload) -> BoxFuture<'_, Result<WorkloadId>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut best: Option<&Arc<ComputeNodeKind>> = None;
            let mut min_load = usize::MAX;

            for child in &children {
                let util = child.get_utilization().await?;
                if util.active_workloads < min_load {
                    min_load = util.active_workloads;
                    best = Some(child);
                }
            }

            match best {
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
        })
    }

    pub fn cancel_workload(&self, id: &WorkloadId) -> BoxFuture<'_, Result<()>> {
        let children = self.children.clone();
        let id = id.clone();
        Box::pin(async move {
            for child in &children {
                let _ = child.cancel_workload(&id).await;
            }
            Ok(())
        })
    }

    pub fn get_workload_status(&self, id: &WorkloadId) -> BoxFuture<'_, Result<WorkloadStatus>> {
        let children = self.children.clone();
        let id = id.clone();
        Box::pin(async move {
            for child in &children {
                if let Ok(status) = child.get_workload_status(&id).await {
                    return Ok(status);
                }
            }
            anyhow::bail!("Workload not found in any child")
        })
    }

    pub fn list_workloads(&self) -> BoxFuture<'_, Result<Vec<WorkloadInfo>>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut all_workloads = Vec::new();
            for child in &children {
                all_workloads.extend(child.list_workloads().await?);
            }
            Ok(all_workloads)
        })
    }

    pub async fn spawn_sub_node(&self, _config: NodeConfig) -> Result<Arc<ComputeNodeKind>> {
        anyhow::bail!(
            "ParentNode children are immutable after construction — \
             use ParentNodeBuilder to define topology at creation time"
        )
    }

    pub async fn get_children(&self) -> Result<Vec<Arc<ComputeNodeKind>>> {
        Ok(self.children.clone())
    }

    pub fn get_all_descendants(&self) -> BoxFuture<'_, Result<Vec<Arc<ComputeNodeKind>>>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut descendants = Vec::new();
            for child in &children {
                descendants.push(child.clone());
                descendants.extend(child.get_all_descendants().await?);
            }
            Ok(descendants)
        })
    }

    pub fn health_check(&self) -> BoxFuture<'_, Result<HealthStatus>> {
        let children = self.children.clone();
        Box::pin(async move {
            for child in &children {
                let status = child.health_check().await?;
                if !matches!(status, HealthStatus::Healthy) {
                    return Ok(HealthStatus::Degraded {
                        reason: format!("Child {} unhealthy", child.node_id()),
                    });
                }
            }
            Ok(HealthStatus::Healthy)
        })
    }

    pub fn get_metrics(&self) -> BoxFuture<'_, Result<NodeMetrics>> {
        let children = self.children.clone();
        let node_id = self.config.node_id.clone();
        Box::pin(async move {
            let (mut submitted, mut completed, mut failed, mut exec_time) =
                (0u64, 0u64, 0u64, 0u64);

            for child in &children {
                let m = child.get_metrics().await?;
                submitted += m.workloads_submitted;
                completed += m.workloads_completed;
                failed += m.workloads_failed;
                exec_time += m.total_execution_time_ms;
            }

            let avg_time = if completed > 0 {
                exec_time as f64 / completed as f64
            } else {
                0.0
            };

            let (mut total_active, mut avg_cpu, mut avg_memory, mut avg_gpu) = (0, 0.0, 0.0, 0.0);
            for child in &children {
                let util = child.get_utilization().await?;
                total_active += util.active_workloads;
                avg_cpu += util.cpu_utilization_percent;
                avg_memory += util.memory_utilization_percent;
                avg_gpu += util.gpu_utilization_percent;
            }
            let count = children.len().max(1) as f64;

            Ok(NodeMetrics {
                node_id,
                workloads_submitted: submitted,
                workloads_completed: completed,
                workloads_failed: failed,
                total_execution_time_ms: exec_time,
                average_execution_time_ms: avg_time,
                current_utilization: UtilizationInfo {
                    cpu_utilization_percent: avg_cpu / count,
                    memory_utilization_percent: avg_memory / count,
                    gpu_utilization_percent: avg_gpu / count,
                    active_workloads: total_active,
                },
            })
        })
    }

    pub fn get_subtree_metrics(&self) -> BoxFuture<'_, Result<TreeMetrics>> {
        let children = self.children.clone();
        Box::pin(async move {
            let mut total_nodes = 1_usize;
            let mut total_active = 0;
            let mut total_completed = 0;
            let mut aggregate_resources = ResourceInfo::zeroed();

            for child in &children {
                let cm = child.get_subtree_metrics().await?;
                total_nodes += cm.total_nodes;
                total_active += cm.total_workloads_active;
                total_completed += cm.total_workloads_completed;
                aggregate_resources.aggregate(&cm.aggregate_resources);
            }

            let (mut avg_cpu, mut avg_memory, mut avg_gpu, mut util_active) = (0.0, 0.0, 0.0, 0);
            for child in &children {
                let util = child.get_utilization().await?;
                util_active += util.active_workloads;
                avg_cpu += util.cpu_utilization_percent;
                avg_memory += util.memory_utilization_percent;
                avg_gpu += util.gpu_utilization_percent;
            }
            let count = children.len().max(1) as f64;

            Ok(TreeMetrics {
                total_nodes,
                total_workloads_active: total_active,
                total_workloads_completed: total_completed,
                aggregate_resources,
                aggregate_utilization: UtilizationInfo {
                    cpu_utilization_percent: avg_cpu / count,
                    memory_utilization_percent: avg_memory / count,
                    gpu_utilization_percent: avg_gpu / count,
                    active_workloads: util_active,
                },
            })
        })
    }
}
