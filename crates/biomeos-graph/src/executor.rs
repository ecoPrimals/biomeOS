// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Graph executor for deterministic deployment orchestration
//!
//! **EVOLVED:** Now uses focused modules for maintainability.
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution (via `TopologicalSorter`)
//! - Parallel execution within phases
//! - Checkpoint/rollback support (via `RollbackManager`)
//! - Live monitoring and metrics (via `ExecutionReport`)

// Use refactored modules
mod executor;
pub use executor::{
    ExecutionContext, ExecutionReport, NodeStatus, PhaseResult, RollbackAction, RollbackManager,
    TopologicalSorter,
};

// Re-export core executor types
pub use executor::core::{execute_node, GraphExecutor};
pub use executor::executor_trait::PrimalOperationExecutor;
