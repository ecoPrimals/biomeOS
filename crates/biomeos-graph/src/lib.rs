// =============================================================================
// biomeos-graph - Graph-Based Orchestration
// =============================================================================
//
// Adaptive primal coordination through graph execution.
//
// Deep Debt Principles:
// - Modern idiomatic Rust (no unsafe, Result<T,E>, async/await)
// - Capability-based (discover primals, don't hardcode)
// - Self-knowledge only (no assumptions about other primals)
// - Mocks isolated to #[cfg(test)]
//
// =============================================================================

pub mod graph;
pub mod parser;
pub mod validator;
pub mod executor;
pub mod context;
pub mod error;
pub mod metrics;

// Re-export core types
pub use graph::{
    PrimalGraph,
    GraphNode,
    GraphEdge,
    GraphId,
    CoordinationPattern,
    PrimalSelector,
    Operation,
    NodeConstraints,
    RetryPolicy,
    EdgeType,
    GraphResult,
    NodeMetrics,
};

pub use parser::GraphParser;
pub use validator::GraphValidator;
pub use executor::GraphExecutor;
pub use context::ExecutionContext;
pub use error::{GraphError, Result};
pub use metrics::{MetricsCollector, GraphMetrics, NodeMetricsAggregate, ExecutionRecord};

