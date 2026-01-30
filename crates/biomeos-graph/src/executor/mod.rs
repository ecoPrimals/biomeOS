//! Graph Executor Module - Smart Refactored
//!
//! **EVOLVED:** Responsibility-based module organization for maintainability.
//!
//! This module provides deterministic graph execution with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics
//!
//! ## Module Organization
//!
//! Each module has a single, clear responsibility:
//!
//! - **`context`** - Execution context and state management
//!   - NodeStatus, RollbackAction types
//!   - ExecutionContext for shared state
//!   - Output and status tracking
//!
//! - **`topological`** - Dependency resolution
//!   - TopologicalSorter for phase planning
//!   - Cycle detection
//!   - Parallel execution grouping
//!
//! - **`monitoring`** - Reporting and metrics
//!   - ExecutionReport for overall results
//!   - PhaseResult for phase-level metrics
//!   - Success tracking
//!
//! - **`rollback`** - Rollback management
//!   - RollbackManager for failed deployments
//!   - Graceful process termination
//!   - File/directory cleanup
//!   - Custom JSON-RPC rollback
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_graph::executor::{GraphExecutor, ExecutionContext};
//!
//! let context = ExecutionContext::new(env_vars);
//! let mut executor = GraphExecutor::new(graph, context);
//! let report = executor.execute().await?;
//! ```

pub mod context;
pub mod monitoring;
pub mod rollback;
pub mod topological;

// Re-export main types for convenience
pub use context::{ExecutionContext, NodeStatus, RollbackAction};
pub use monitoring::{ExecutionReport, PhaseResult};
pub use rollback::RollbackManager;
pub use topological::TopologicalSorter;
