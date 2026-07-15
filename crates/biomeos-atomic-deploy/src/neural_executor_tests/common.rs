// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_graph::{GraphNode, Operation};
use std::collections::HashMap;

pub(crate) fn create_test_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}

pub(crate) fn node_log_info(id: &str, depends_on: Vec<String>, message: &str) -> GraphNode {
    let mut config = HashMap::new();
    config.insert(
        "message".to_string(),
        serde_json::Value::String(message.to_string()),
    );
    GraphNode {
        id: id.to_string(),
        depends_on,
        operation: Some(Operation {
            name: "log.info".to_string(),
            target: None,
            params: HashMap::new(),
            environment: None,
        }),
        config,
        ..Default::default()
    }
}

pub(crate) fn node_fs_check(id: &str, optional: bool) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on: vec![],
        operation: Some(Operation {
            name: "filesystem.check_exists".to_string(),
            target: None,
            params: HashMap::new(),
            environment: None,
        }),
        config: HashMap::new(),
        fallback: optional.then(|| "skip".to_string()),
        ..Default::default()
    }
}
