// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_graph::GraphNode;
use super::super::{ExecutionContext};
use std::collections::HashMap;

pub(super) fn test_node_with_config(id: &str, config: HashMap<String, serde_json::Value>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        config,
        ..Default::default()
    }
}

pub(super) fn test_context() -> ExecutionContext {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family".to_string());
    ExecutionContext::new(env)
}

pub(super) fn test_context_with_env(env: HashMap<String, String>) -> ExecutionContext {
    ExecutionContext::new(env)
}

pub(super) fn test_node_with_capabilities(
    id: &str,
    config: HashMap<String, serde_json::Value>,
    capabilities: Vec<String>,
) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        config,
        capabilities,
        ..Default::default()
    }
}
