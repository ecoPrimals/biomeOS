// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for graph handlers (graph.list, graph.get, graph.save, graph.execute, graph.status).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod continuous;
mod crud;
mod execute;
mod execution_status;
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
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    let executions = Arc::new(RwLock::new(HashMap::new()));
    let handler = GraphHandler::new(
        graphs_dir,
        "test-family",
        executions.clone(),
        router,
        registry,
    );
    (handler, executions)
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
