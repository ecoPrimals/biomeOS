// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph executor module
//!
//! **EVOLVED:** Smart domain-based splitting for maintainability.
//!
//! This module provides graph execution capabilities with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics
//! - Node-specific handlers organized by domain

pub mod context;
pub mod core;
pub mod monitoring;
pub mod node_handlers;
pub mod rollback;
pub mod topological;
pub mod trait as executor_trait;
pub mod types;

#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use context::{ExecutionContext, NodeStatus, RollbackAction};
pub use rollback::RollbackManager;
pub use topological::TopologicalSorter;
pub use types::{ExecutionReport, PhaseResult};
