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

pub mod fractal;
pub mod node;

// Re-export core types
pub use node::{
    CapacityInfo, ComputeNode, HealthStatus, NodeConfig, NodeMetrics, NodeTopology,
    ResourceAllocation, ResourceInfo, ResourceType, Runtime, TreeMetrics, UtilizationInfo,
    Workload, WorkloadId, WorkloadInfo, WorkloadStatus,
};

pub use fractal::{FractalBuilder, LeafNode, ParentNode};
