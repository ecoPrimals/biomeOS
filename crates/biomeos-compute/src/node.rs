// =============================================================================
// ComputeNode Trait - Isomorphic Interface
// =============================================================================
//
// The core trait that ALL compute nodes implement, regardless of:
// - Scale (single core → planetary cluster)
// - Topology (leaf, parent, root)
// - Resource type (GPU, CPU, Memory, Hybrid)
//
// "Same interface at every level" - Nature's pattern
//
// =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;
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
    NAryTree { branching_factor: usize },
    /// Quad tree (4 children, spatial)
    QuadTree,
    /// Hybrid (mixed children types)
    Hybrid,
}

/// Workload identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkloadId(pub Uuid);

impl WorkloadId {
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
    pub id: WorkloadId,
    pub name: String,
    pub runtime: Runtime,
    pub code: Vec<u8>,
    pub parallelizable: bool,
    pub resource_requirements: ResourceRequirements,
    pub priority: WorkloadPriority,
}

impl Workload {
    pub fn new(name: impl Into<String>, runtime: Runtime) -> Self {
        Self {
            id: WorkloadId::new(),
            name: name.into(),
            runtime,
            code: Vec::new(),
            parallelizable: false,
            resource_requirements: ResourceRequirements::default(),
            priority: WorkloadPriority::Normal,
        }
    }
}

/// Runtime type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runtime {
    Native,
    Wasm,
    Container,
    Python,
    Gpu,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<usize>,
    pub memory_mb: Option<usize>,
    pub gpu_memory_mb: Option<usize>,
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
    Low,
    Normal,
    High,
    Critical,
}

/// Workload status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadStatus {
    Queued,
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

/// Workload info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadInfo {
    pub id: WorkloadId,
    pub name: String,
    pub status: WorkloadStatus,
    pub node_id: String,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub cpu_cores: usize,
    pub memory_mb: usize,
    pub gpu_count: usize,
    pub gpu_memory_mb: usize,
    pub disk_mb: usize,
}

impl ResourceInfo {
    /// Aggregate resources (for parent nodes)
    pub fn aggregate(&mut self, other: ResourceInfo) {
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
    pub max_concurrent_workloads: usize,
    pub available_slots: usize,
    pub total_resources: ResourceInfo,
    pub available_resources: ResourceInfo,
}

/// Utilization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationInfo {
    pub cpu_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub gpu_utilization_percent: f64,
    pub active_workloads: usize,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub depth: usize,
    pub topology: NodeTopology,
    pub resource_type: ResourceType,
    pub resource_allocation: ResourceAllocation,
}

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Cpu,
    Gpu,
    Memory,
    Hybrid,
}

/// Resource allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAllocation {
    /// Equal split among children
    Equal,
    /// Weighted split
    Weighted { weights: Vec<f64> },
    /// Core affinity (pin to specific cores)
    CoreAffinity { core_ranges: Vec<(usize, usize)> },
    /// Custom allocation
    Custom { allocations: Vec<ResourceInfo> },
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { error: String },
}

/// Node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub workloads_submitted: u64,
    pub workloads_completed: u64,
    pub workloads_failed: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub current_utilization: UtilizationInfo,
}

/// Tree metrics (recursive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeMetrics {
    pub total_nodes: usize,
    pub total_workloads_active: usize,
    pub total_workloads_completed: u64,
    pub aggregate_resources: ResourceInfo,
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
    pub fn new(name: impl Into<String>, runtime: Runtime) -> Self {
        Self {
            workload: Workload::new(name, runtime),
        }
    }
    
    pub fn code(mut self, code: Vec<u8>) -> Self {
        self.workload.code = code;
        self
    }
    
    pub fn parallelizable(mut self, parallelizable: bool) -> Self {
        self.workload.parallelizable = parallelizable;
        self
    }
    
    pub fn cpu_cores(mut self, cores: usize) -> Self {
        self.workload.resource_requirements.cpu_cores = Some(cores);
        self
    }
    
    pub fn memory_mb(mut self, mb: usize) -> Self {
        self.workload.resource_requirements.memory_mb = Some(mb);
        self
    }
    
    pub fn priority(mut self, priority: WorkloadPriority) -> Self {
        self.workload.priority = priority;
        self
    }
    
    pub fn build(self) -> Workload {
        self.workload
    }
}

impl Workload {
    pub fn builder(name: impl Into<String>, runtime: Runtime) -> WorkloadBuilder {
        WorkloadBuilder::new(name, runtime)
    }
}

