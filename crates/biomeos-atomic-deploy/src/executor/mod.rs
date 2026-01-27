//! Executor module for graph execution and primal lifecycle management
//!
//! This module provides the core execution infrastructure for Neural API graphs,
//! including context management, primal spawning, and process coordination.
//!
//! ## Architecture
//!
//! - **context**: Execution context shared across all nodes
//! - **types**: Shared result types (ExecutionReport, PhaseResult)
//! - **node_handlers**: Individual node type execution handlers
//! - **primal_spawner**: Process spawning and socket management
//!
//! ## Deep Debt Principles
//!
//! - Capability-based discovery (no hardcoded primal names)
//! - Pure JSON-RPC communication (no HTTP in IPC)
//! - Runtime primal discovery (self-knowledge only)

pub mod context;
pub mod node_handlers;
pub mod primal_spawner;
pub mod types;

// Re-export commonly used types
pub use context::{ExecutionContext, NodeStatus};
pub use node_handlers::substitute_env;
pub use primal_spawner::{discover_primal_binary, spawn_primal_process, wait_for_socket};
pub use types::{ExecutionReport, PhaseResult, PhaseResultSummary};
