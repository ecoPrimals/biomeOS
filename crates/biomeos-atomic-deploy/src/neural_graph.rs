// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph data structures for Neural API
//!
//! Split into submodules:
//! - [`types`]: Core structs (`Graph`, `GraphNode`, `GraphConfig`, etc.)
//! - [`parsing`]: TOML loading and deserialization
//! - [`convert`]: `DeploymentGraph` → `GraphNode` conversion

mod convert;
mod parsing;
mod types;

pub use types::{
    Constraints, Graph, GraphConfig, GraphNode, NodeOutput, Operation, PrimalSelector, RetryConfig,
};
