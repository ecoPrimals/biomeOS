// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! ComputeNode Trait - Isomorphic Interface
//!
//! The core trait that ALL compute nodes implement, regardless of:
//! - Scale (single core → planetary cluster)
//! - Topology (leaf, parent, root)
//! - Resource type (GPU, CPU, Memory, Hybrid)
//!
//! "Same interface at every level" - Nature's pattern

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// =============================================================================
// CORE TRAIT
// =============================================================================

/// ComputeNode: Isomorphic interface for compute at any scale
#[async_trait]
pub trait ComputeNode: Send + Sync {
    // =========================================================================
    // IDENTITY (same at every level)
    // =========================================================================

    /// Unique node identifier
    fn node_id(&self) -> &str;

    /// Parent node ID (None if root)
    fn parent_id(&self) -> Option<&str>;

    /// Depth in fractal tree (0 = root)
    fn depth(&self) -> usize;

    /// Node topology type
    fn topology(&self) -> NodeTopology;

    /// Is this a leaf node?
    fn is_leaf(&self) -> bool {
        self.get_child_count() == 0
    }

    /// Number of direct children
    fn get_child_count(&self) -> usize;

    // =========================================================================
    // RESOURCES (recursive aggregation)
    // =========================================================================

    /// Get resources (own + children if parent)
    async fn get_resources(&self) -> Result<ResourceInfo>;

    /// Get current capacity
    async fn get_capacity(&self) -> Result<CapacityInfo>;

    /// Get current utilization
    async fn get_utilization(&self) -> Result<UtilizationInfo>;

    // =========================================================================
    // WORKLOAD EXECUTION (same API, different implementation)
    // =========================================================================

    /// Submit workload for execution
    async fn submit_workload(&self, workload: Workload) -> Result<WorkloadId>;

    /// Cancel a workload
    async fn cancel_workload(&self, id: &WorkloadId) -> Result<()>;

    /// Get workload status
    async fn get_workload_status(&self, id: &WorkloadId) -> Result<WorkloadStatus>;

    /// List all workloads
    async fn list_workloads(&self) -> Result<Vec<WorkloadInfo>>;

    // =========================================================================
    // FRACTAL OPERATIONS (recursive)
    // =========================================================================

    /// Spawn a sub-node (fractal recursion)
    async fn spawn_sub_node(&self, config: NodeConfig) -> Result<Arc<dyn ComputeNode>>;

    /// Get direct children
    async fn get_children(&self) -> Result<Vec<Arc<dyn ComputeNode>>>;

    /// Get all descendants (recursive)
    async fn get_all_descendants(&self) -> Result<Vec<Arc<dyn ComputeNode>>>;

    /// Get total node count (self + descendants)
    async fn get_node_count(&self) -> Result<usize> {
        Ok(1 + self.get_all_descendants().await?.len())
    }

    // =========================================================================
    // HEALTH & MONITORING (recursive rollup)
    // =========================================================================

    /// Health check (own + children)
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get metrics for this node
    async fn get_metrics(&self) -> Result<NodeMetrics>;

    /// Get metrics for entire subtree
    async fn get_subtree_metrics(&self) -> Result<TreeMetrics>;
}

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// Node topology type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeTopology {
    /// Leaf node (executes workloads)
    Leaf,
    /// Binary tree (2 children)
    BinaryTree,
    /// N-ary tree (N children)
    NAryTree {
        /// Number of children per node
        branching_factor: usize,
    },
    /// Quad tree (4 children, spatial)
    QuadTree,
    /// Hybrid (mixed children types)
    Hybrid,
}

/// Workload identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkloadId(pub Uuid);

impl WorkloadId {
    /// Create a new random workload identifier
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkloadId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkloadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Workload to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    /// Unique workload identifier
    pub id: WorkloadId,
    /// Human-readable workload name
    pub name: String,
    /// Execution runtime
    pub runtime: Runtime,
    /// Workload code/payload bytes (zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub code: Bytes,
    /// Whether the workload can be split across nodes
    pub parallelizable: bool,
    /// Resource requirements for execution
    pub resource_requirements: ResourceRequirements,
    /// Scheduling priority
    pub priority: WorkloadPriority,
}

impl Workload {
    /// Create a new workload with default settings
    pub fn new(name: impl Into<String>, runtime: Runtime) -> Self {
        Self {
            id: WorkloadId::new(),
            name: name.into(),
            runtime,
            code: Bytes::new(),
            parallelizable: false,
            resource_requirements: ResourceRequirements::default(),
            priority: WorkloadPriority::Normal,
        }
    }
}

/// Runtime type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runtime {
    /// Native binary execution
    Native,
    /// WebAssembly runtime
    Wasm,
    /// Container-based execution
    Container,
    /// Python interpreter
    Python,
    /// GPU compute (CUDA/OpenCL)
    Gpu,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Required CPU cores
    pub cpu_cores: Option<usize>,
    /// Required memory in megabytes
    pub memory_mb: Option<usize>,
    /// Required GPU memory in megabytes
    pub gpu_memory_mb: Option<usize>,
    /// Required disk space in megabytes
    pub disk_mb: Option<usize>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(1),
            memory_mb: Some(256),
            gpu_memory_mb: None,
            disk_mb: None,
        }
    }
}

/// Workload priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WorkloadPriority {
    /// Low priority workload
    Low,
    /// Normal priority workload
    Normal,
    /// High priority workload
    High,
    /// Critical priority workload
    Critical,
}

/// Workload status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadStatus {
    /// Waiting in queue
    Queued,
    /// Currently executing
    Running,
    /// Finished successfully
    Completed,
    /// Execution failed
    Failed {
        /// Error description
        error: String,
    },
    /// Cancelled by user
    Cancelled,
}

/// Workload info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadInfo {
    /// Workload identifier
    pub id: WorkloadId,
    /// Workload name
    pub name: String,
    /// Current status
    pub status: WorkloadStatus,
    /// Node that owns this workload
    pub node_id: String,
    /// When the workload was submitted
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    /// When execution started (if started)
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When execution completed (if completed)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Memory in megabytes
    pub memory_mb: usize,
    /// Number of GPUs
    pub gpu_count: usize,
    /// GPU memory in megabytes
    pub gpu_memory_mb: usize,
    /// Disk space in megabytes
    pub disk_mb: usize,
}

impl ResourceInfo {
    /// Aggregate resources (for parent nodes)
    pub fn aggregate(&mut self, other: &ResourceInfo) {
        self.cpu_cores += other.cpu_cores;
        self.memory_mb += other.memory_mb;
        self.gpu_count += other.gpu_count;
        self.gpu_memory_mb += other.gpu_memory_mb;
        self.disk_mb += other.disk_mb;
    }
}

/// Capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityInfo {
    /// Maximum concurrent workloads supported
    pub max_concurrent_workloads: usize,
    /// Currently available workload slots
    pub available_slots: usize,
    /// Total resources in this node
    pub total_resources: ResourceInfo,
    /// Currently available resources
    pub available_resources: ResourceInfo,
}

/// Utilization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationInfo {
    /// CPU utilization percentage (0.0–100.0)
    pub cpu_utilization_percent: f64,
    /// Memory utilization percentage (0.0–100.0)
    pub memory_utilization_percent: f64,
    /// GPU utilization percentage (0.0–100.0)
    pub gpu_utilization_percent: f64,
    /// Number of currently active workloads
    pub active_workloads: usize,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node identifier
    pub node_id: String,
    /// Parent node identifier (None for root)
    pub parent_id: Option<String>,
    /// Depth in the fractal tree (0 = root)
    pub depth: usize,
    /// Node topology type
    pub topology: NodeTopology,
    /// Type of resources this node manages
    pub resource_type: ResourceType,
    /// How resources are allocated to children
    pub resource_allocation: ResourceAllocation,
}

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU-based compute
    Cpu,
    /// GPU-based compute
    Gpu,
    /// Memory-intensive workloads
    Memory,
    /// Mixed resource types
    Hybrid,
}

/// Resource allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAllocation {
    /// Equal split among children
    Equal,
    /// Weighted split
    Weighted {
        /// Weight per child
        weights: Vec<f64>,
    },
    /// Core affinity (pin to specific cores)
    CoreAffinity {
        /// Core ranges (start, end) per child
        core_ranges: Vec<(usize, usize)>,
    },
    /// Custom allocation
    Custom {
        /// Resource allocation per child
        allocations: Vec<ResourceInfo>,
    },
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Node is fully operational
    Healthy,
    /// Node is operational but degraded
    Degraded {
        /// Reason for degraded status
        reason: String,
    },
    /// Node is not operational
    Unhealthy {
        /// Error causing unhealthy status
        error: String,
    },
}

/// Node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node identifier
    pub node_id: String,
    /// Total workloads submitted
    pub workloads_submitted: u64,
    /// Workloads completed successfully
    pub workloads_completed: u64,
    /// Workloads that failed
    pub workloads_failed: u64,
    /// Total execution time across all workloads in milliseconds
    pub total_execution_time_ms: u64,
    /// Average workload execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Current resource utilization
    pub current_utilization: UtilizationInfo,
}

/// Tree metrics (recursive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeMetrics {
    /// Total number of nodes in the subtree
    pub total_nodes: usize,
    /// Number of currently active workloads
    pub total_workloads_active: usize,
    /// Total completed workloads
    pub total_workloads_completed: u64,
    /// Aggregate resources across the subtree
    pub aggregate_resources: ResourceInfo,
    /// Aggregate utilization across the subtree
    pub aggregate_utilization: UtilizationInfo,
}

// =============================================================================
// BUILDER PATTERN
// =============================================================================

/// WorkloadBuilder for fluent API
pub struct WorkloadBuilder {
    workload: Workload,
}

impl WorkloadBuilder {
    /// Create a new workload builder
    pub fn new(name: impl Into<String>, runtime: Runtime) -> Self {
        Self {
            workload: Workload::new(name, runtime),
        }
    }

    /// Set the workload code/payload
    pub fn code(mut self, code: impl Into<Bytes>) -> Self {
        self.workload.code = code.into();
        self
    }

    /// Set whether the workload can be parallelized
    pub fn parallelizable(mut self, parallelizable: bool) -> Self {
        self.workload.parallelizable = parallelizable;
        self
    }

    /// Set required CPU cores
    pub fn cpu_cores(mut self, cores: usize) -> Self {
        self.workload.resource_requirements.cpu_cores = Some(cores);
        self
    }

    /// Set required memory in megabytes
    pub fn memory_mb(mut self, mb: usize) -> Self {
        self.workload.resource_requirements.memory_mb = Some(mb);
        self
    }

    /// Set workload priority
    pub fn priority(mut self, priority: WorkloadPriority) -> Self {
        self.workload.priority = priority;
        self
    }

    /// Build the workload
    pub fn build(self) -> Workload {
        self.workload
    }
}

impl Workload {
    /// Create a workload builder for fluent construction
    pub fn builder(name: impl Into<String>, runtime: Runtime) -> WorkloadBuilder {
        WorkloadBuilder::new(name, runtime)
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn test_node_topology_serde_roundtrip() {
        for topo in [
            NodeTopology::Leaf,
            NodeTopology::BinaryTree,
            NodeTopology::NAryTree {
                branching_factor: 4,
            },
            NodeTopology::QuadTree,
            NodeTopology::Hybrid,
        ] {
            let json = serde_json::to_string(&topo).unwrap();
            let restored: NodeTopology = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", topo), format!("{:?}", restored));
        }
    }

    #[test]
    fn workload_id_new_and_display() {
        let id = WorkloadId::new();
        let s = id.to_string();
        assert!(!s.is_empty());
        assert!(s.len() >= 32);
    }

    #[test]
    fn test_workload_id_new_and_display() {
        let id = WorkloadId::new();
        assert!(!id.0.is_nil());
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn workload_id_default() {
        let id = WorkloadId::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_workload_id_default() {
        let id = WorkloadId::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn resource_requirements_default() {
        let req = ResourceRequirements::default();
        assert_eq!(req.cpu_cores, Some(1));
        assert_eq!(req.memory_mb, Some(256));
        assert!(req.gpu_memory_mb.is_none());
    }

    #[test]
    fn test_resource_requirements_default() {
        let r = ResourceRequirements::default();
        assert_eq!(r.cpu_cores, Some(1));
        assert_eq!(r.memory_mb, Some(256));
    }

    #[test]
    fn resource_info_aggregate() {
        let mut a = ResourceInfo {
            cpu_cores: 4,
            memory_mb: 1024,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 100,
        };
        let b = ResourceInfo {
            cpu_cores: 2,
            memory_mb: 512,
            gpu_count: 1,
            gpu_memory_mb: 4096,
            disk_mb: 50,
        };
        a.aggregate(&b);
        assert_eq!(a.cpu_cores, 6);
        assert_eq!(a.memory_mb, 1536);
        assert_eq!(a.gpu_count, 1);
        assert_eq!(a.gpu_memory_mb, 4096);
        assert_eq!(a.disk_mb, 150);
    }

    #[test]
    fn test_resource_info_aggregate() {
        let mut r1 = ResourceInfo {
            cpu_cores: 4,
            memory_mb: 1024,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 100,
        };
        let r2 = ResourceInfo {
            cpu_cores: 2,
            memory_mb: 512,
            gpu_count: 1,
            gpu_memory_mb: 4096,
            disk_mb: 50,
        };
        r1.aggregate(&r2);
        assert_eq!(r1.cpu_cores, 6);
        assert_eq!(r1.memory_mb, 1536);
        assert_eq!(r1.gpu_count, 1);
    }

    #[test]
    fn workload_priority_ordering() {
        assert!(WorkloadPriority::Critical > WorkloadPriority::High);
        assert!(WorkloadPriority::High > WorkloadPriority::Normal);
        assert!(WorkloadPriority::Normal > WorkloadPriority::Low);
    }

    #[test]
    fn test_workload_priority_ordering() {
        assert!(WorkloadPriority::Critical > WorkloadPriority::High);
        assert!(WorkloadPriority::High > WorkloadPriority::Normal);
        assert!(WorkloadPriority::Normal > WorkloadPriority::Low);
    }

    #[test]
    fn workload_status_variants() {
        let queued = WorkloadStatus::Queued;
        let running = WorkloadStatus::Running;
        let completed = WorkloadStatus::Completed;
        let failed = WorkloadStatus::Failed {
            error: "oops".into(),
        };
        let cancelled = WorkloadStatus::Cancelled;
        let _ = format!(
            "{:?} {:?} {:?} {:?} {:?}",
            queued, running, completed, failed, cancelled
        );
    }

    #[test]
    fn test_workload_status_serde_roundtrip() {
        for status in [
            WorkloadStatus::Queued,
            WorkloadStatus::Running,
            WorkloadStatus::Completed,
            WorkloadStatus::Failed {
                error: "test".to_string(),
            },
            WorkloadStatus::Cancelled,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let restored: WorkloadStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", status), format!("{:?}", restored));
        }
    }

    #[test]
    fn node_topology_variants() {
        let leaf = NodeTopology::Leaf;
        let binary = NodeTopology::BinaryTree;
        let nary = NodeTopology::NAryTree {
            branching_factor: 4,
        };
        let quad = NodeTopology::QuadTree;
        let hybrid = NodeTopology::Hybrid;
        let _ = format!("{:?} {:?} {:?} {:?} {:?}", leaf, binary, nary, quad, hybrid);
    }

    #[test]
    fn test_workload_new() {
        let w = Workload::new("test", Runtime::Wasm);
        assert_eq!(w.name, "test");
        assert_eq!(w.runtime, Runtime::Wasm);
        assert!(w.code.is_empty());
        assert!(!w.parallelizable);
    }

    #[test]
    fn runtime_variants() {
        for r in [
            Runtime::Native,
            Runtime::Wasm,
            Runtime::Container,
            Runtime::Python,
            Runtime::Gpu,
        ] {
            let _ = format!("{:?}", r);
        }
    }

    #[test]
    fn test_runtime_serde_roundtrip() {
        for r in [
            Runtime::Native,
            Runtime::Wasm,
            Runtime::Container,
            Runtime::Python,
            Runtime::Gpu,
        ] {
            let json = serde_json::to_string(&r).unwrap();
            let restored: Runtime = serde_json::from_str(&json).unwrap();
            assert_eq!(r, restored);
        }
    }

    #[test]
    fn resource_type_variants() {
        for t in [
            ResourceType::Cpu,
            ResourceType::Gpu,
            ResourceType::Memory,
            ResourceType::Hybrid,
        ] {
            let _ = format!("{:?}", t);
        }
    }

    #[test]
    fn health_status_variants() {
        let healthy = HealthStatus::Healthy;
        let degraded = HealthStatus::Degraded {
            reason: "load".into(),
        };
        let unhealthy = HealthStatus::Unhealthy {
            error: "crash".into(),
        };
        let _ = format!("{:?} {:?} {:?}", healthy, degraded, unhealthy);
    }

    #[test]
    fn test_health_status_serde_roundtrip() {
        for status in [
            HealthStatus::Healthy,
            HealthStatus::Degraded {
                reason: "load".to_string(),
            },
            HealthStatus::Unhealthy {
                error: "crash".to_string(),
            },
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let restored: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", status), format!("{:?}", restored));
        }
    }

    #[test]
    fn workload_builder_fluent() {
        let w = Workload::builder("test", Runtime::Native)
            .cpu_cores(4)
            .memory_mb(512)
            .priority(WorkloadPriority::High)
            .parallelizable(true)
            .build();
        assert_eq!(w.name, "test");
        assert_eq!(w.runtime, Runtime::Native);
        assert_eq!(w.resource_requirements.cpu_cores, Some(4));
        assert_eq!(w.resource_requirements.memory_mb, Some(512));
        assert_eq!(w.priority, WorkloadPriority::High);
        assert!(w.parallelizable);
    }

    #[test]
    fn test_workload_builder() {
        let w = Workload::builder("my-workload", Runtime::Native)
            .cpu_cores(4)
            .memory_mb(512)
            .priority(WorkloadPriority::High)
            .build();
        assert_eq!(w.name, "my-workload");
        assert_eq!(w.resource_requirements.cpu_cores, Some(4));
        assert_eq!(w.resource_requirements.memory_mb, Some(512));
        assert_eq!(w.priority, WorkloadPriority::High);
    }

    #[test]
    fn workload_new() {
        let w = Workload::new("simple", Runtime::Wasm);
        assert_eq!(w.name, "simple");
        assert_eq!(w.runtime, Runtime::Wasm);
        assert!(!w.parallelizable);
        assert_eq!(w.priority, WorkloadPriority::Normal);
    }
}
