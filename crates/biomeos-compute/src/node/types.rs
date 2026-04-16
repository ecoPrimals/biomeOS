// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Compute node data types: workloads, resources, topology, metrics, and config.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// TOPOLOGY
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

// =============================================================================
// WORKLOAD
// =============================================================================

/// Workload identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkloadId(pub Uuid);

impl WorkloadId {
    /// Create a new random workload identifier
    #[must_use]
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

    /// Create a workload builder for fluent construction
    pub fn builder(name: impl Into<String>, runtime: Runtime) -> WorkloadBuilder {
        WorkloadBuilder::new(name, runtime)
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

// =============================================================================
// RESOURCES & CAPACITY
// =============================================================================

/// Resource information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    /// All fields zero — useful as an accumulator starting point.
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            cpu_cores: 0,
            memory_mb: 0,
            gpu_count: 0,
            gpu_memory_mb: 0,
            disk_mb: 0,
        }
    }

    /// Aggregate resources (for parent nodes)
    pub const fn aggregate(&mut self, other: &Self) {
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
    /// CPU utilization percentage (0.0-100.0)
    pub cpu_utilization_percent: f64,
    /// Memory utilization percentage (0.0-100.0)
    pub memory_utilization_percent: f64,
    /// GPU utilization percentage (0.0-100.0)
    pub gpu_utilization_percent: f64,
    /// Number of currently active workloads
    pub active_workloads: usize,
}

// =============================================================================
// NODE CONFIG & ALLOCATION
// =============================================================================

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

// =============================================================================
// HEALTH & METRICS
// =============================================================================

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
// BUILDER
// =============================================================================

/// `WorkloadBuilder` for fluent API
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
    pub const fn parallelizable(mut self, parallelizable: bool) -> Self {
        self.workload.parallelizable = parallelizable;
        self
    }

    /// Set required CPU cores
    pub const fn cpu_cores(mut self, cores: usize) -> Self {
        self.workload.resource_requirements.cpu_cores = Some(cores);
        self
    }

    /// Set required memory in megabytes
    pub const fn memory_mb(mut self, mb: usize) -> Self {
        self.workload.resource_requirements.memory_mb = Some(mb);
        self
    }

    /// Set workload priority
    pub const fn priority(mut self, priority: WorkloadPriority) -> Self {
        self.workload.priority = priority;
        self
    }

    /// Build the workload
    pub fn build(self) -> Workload {
        self.workload
    }
}
