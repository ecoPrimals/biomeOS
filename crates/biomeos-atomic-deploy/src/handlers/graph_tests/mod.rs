// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for graph handlers (graph.list, graph.get, graph.save, graph.execute, graph.status).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::type_complexity)]

mod continuous;
mod coverage_more;
mod crud;
mod execute;
mod execution_status;
mod graph_branches;
mod optimization;
mod pipeline;
mod pure_logic;

pub use super::graph::{ExecutionStatus, GraphHandler};
use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper to create a minimal GraphHandler for tests.
pub fn make_handler(
    graphs_dir: &std::path::Path,
) -> (GraphHandler, Arc<RwLock<HashMap<String, ExecutionStatus>>>) {
    let (handler, executions, _) = make_handler_with_registry(graphs_dir);
    (handler, executions)
}

/// Same as [`make_handler`], but also returns the capability translation registry for setup.
pub fn make_handler_with_registry(
    graphs_dir: &std::path::Path,
) -> (
    GraphHandler,
    Arc<RwLock<HashMap<String, ExecutionStatus>>>,
    Arc<RwLock<CapabilityTranslationRegistry>>,
) {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    let executions = Arc::new(RwLock::new(HashMap::new()));
    let handler = GraphHandler::new(
        graphs_dir,
        "test-family",
        executions.clone(),
        router,
        registry.clone(),
    );
    (handler, executions, registry)
}

/// Minimal valid graph TOML for execute tests (log.info completes quickly).
pub const MINIMAL_GRAPH_TOML: &str = r#"
[graph]
id = "test_minimal"
version = "1.0.0"
description = "Minimal graph for tests"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "test execution"
"#;
