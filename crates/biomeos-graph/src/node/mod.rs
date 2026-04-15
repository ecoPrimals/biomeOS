// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph node types with type-safe parameters.
//!
//! Each node represents an execution unit in the graph.
//! Nodes have:
//! - An ID (unique within the graph)
//! - A capability to invoke
//! - Parameters for the capability
//! - Dependencies on other nodes

mod condition;
mod node_id;
mod node_type;
mod param;

#[cfg(test)]
mod tests;

pub use node_id::NodeId;
pub use node_type::{NodeConfig, NodeType};
pub use param::{NodeParams, ParamValue};

use condition::evaluate_condition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the deployment graph.
///
/// Nodes are execution units that invoke capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier within the graph
    pub id: NodeId,

    /// Human-readable name
    pub name: String,

    /// Node type (capability, condition, etc.)
    #[serde(rename = "type", default)]
    pub node_type: NodeType,

    /// Capability to invoke (e.g., "`crypto.blake3_hash`")
    #[serde(default)]
    pub capability: Option<String>,

    /// Whether this node must succeed
    #[serde(default = "default_true")]
    pub required: bool,

    /// Execution order (lower = earlier)
    #[serde(default)]
    pub order: i32,

    /// Node IDs this node depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Condition for execution (e.g., "${VAR} == value")
    #[serde(default)]
    pub condition: Option<String>,

    /// Node configuration
    #[serde(default)]
    pub config: NodeConfig,

    /// Parameters for the capability
    #[serde(default)]
    pub params: NodeParams,

    /// Feedback edge: this node's output feeds back as input to another node on the next tick.
    /// Only meaningful in Continuous coordination graphs.
    #[serde(default)]
    pub feedback_to: Option<String>,

    /// Per-node budget in milliseconds.
    /// In Continuous graphs, if execution exceeds this, the previous output is reused.
    #[serde(default)]
    pub budget_ms: Option<f64>,

    /// Fallback behavior when execution fails or times out.
    /// "skip" = silently skip (reuse cached output or null), "error" = propagate error (default).
    /// In Continuous graphs, "skip" allows optional primals to miss ticks without killing the loop.
    #[serde(default)]
    pub fallback: Option<String>,

    /// Estimated execution cost in milliseconds.
    /// Used by Pathway Learner for cost-aware scheduling and reordering.
    #[serde(default)]
    pub cost_estimate_ms: Option<u64>,

    /// Declared operation dependencies for Pathway Learner analysis.
    /// Semantic operation names this node depends on beyond structural `depends_on`.
    #[serde(default)]
    pub operation_dependencies: Vec<String>,

    /// Target gate for cross-gate deployment.
    /// Absent or `"local"` means execute on this biomeOS instance.
    /// Any other value (e.g., `"gate2"`) is resolved via the graph's gate registry
    /// to a remote biomeOS Neural API endpoint.
    #[serde(default)]
    pub gate: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl GraphNode {
    /// Check if this node should be skipped based on condition.
    #[must_use]
    pub fn should_skip(&self, env: &HashMap<String, String>) -> bool {
        if let Some(skip_if) = &self.config.skip_if {
            // Simple condition evaluation: "${VAR} == value" or "${VAR} != value"
            evaluate_condition(skip_if, env)
        } else {
            false
        }
    }

    /// Check if this node's condition is met.
    #[must_use]
    pub fn condition_met(&self, env: &HashMap<String, String>) -> bool {
        if let Some(condition) = &self.condition {
            evaluate_condition(condition, env)
        } else {
            true
        }
    }

    /// Returns true if this node uses "skip" fallback (tolerates failures).
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.fallback.as_deref() == Some("skip")
    }
}
