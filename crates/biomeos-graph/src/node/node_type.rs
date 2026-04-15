// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// Invokes a capability
    #[default]
    Capability,
    /// Conditional branching
    Condition,
    /// Parallel execution group
    Parallel,
    /// Wait for external event
    Wait,
}

/// Node configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Which primal to use (e.g., "beardog")
    #[serde(default)]
    pub primal: Option<String>,

    /// Skip condition
    #[serde(default)]
    pub skip_if: Option<String>,

    /// Retry count on failure
    #[serde(default)]
    pub retry_count: Option<u32>,

    /// Timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Additional config
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}
