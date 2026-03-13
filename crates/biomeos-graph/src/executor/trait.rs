// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal operation executor trait
//!
//! Defines the interface for executing operations on primals.

use crate::graph::Operation;
use anyhow::Result;

/// Trait for executing operations on primals
#[async_trait::async_trait]
pub trait PrimalOperationExecutor: Send + Sync {
    /// Execute an operation on a primal
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
    ) -> Result<serde_json::Value>;
}
