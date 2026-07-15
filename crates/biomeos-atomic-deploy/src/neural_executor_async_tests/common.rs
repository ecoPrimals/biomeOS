// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared test fixtures for GraphExecutor async integration tests.

use crate::neural_graph::GraphNode;

pub(crate) fn create_test_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}
