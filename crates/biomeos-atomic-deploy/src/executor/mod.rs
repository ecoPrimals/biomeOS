//! Executor module for graph execution and primal lifecycle management
//!
//! This module provides the core execution infrastructure for Neural API graphs,
//! including context management, primal spawning, and process coordination.

pub mod context;
pub mod primal_spawner;

// Re-export commonly used types
pub use context::{ExecutionContext, NodeStatus};
pub use primal_spawner::{discover_primal_binary, spawn_primal_process, wait_for_socket};

