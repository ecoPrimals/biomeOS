// =============================================================================
// biomeos-compute - Isomorphic & Fractal Compute Architecture
// =============================================================================
//
// A fractal, isomorphic compute system where the same interface works at
// every scale - from a single CPU core to planetary-scale clusters.
//
// "Same structure at every level" - Nature's pattern
//
// =============================================================================

pub mod node;
pub mod fractal;

// Re-export core types
pub use node::{
    ComputeNode,
    NodeTopology,
    Workload,
    WorkloadId,
    WorkloadStatus,
    WorkloadInfo,
    Runtime,
    ResourceInfo,
    CapacityInfo,
    UtilizationInfo,
    NodeConfig,
    ResourceType,
    ResourceAllocation,
    HealthStatus,
    NodeMetrics,
    TreeMetrics,
};

pub use fractal::{
    FractalBuilder,
    LeafNode,
    ParentNode,
};

