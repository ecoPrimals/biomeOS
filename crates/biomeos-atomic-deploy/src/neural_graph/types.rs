// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Core data structures for Neural API graphs.

use biomeos_graph::GeneticsTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Neural API graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Unique graph identifier
    pub id: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Execution nodes in dependency order
    pub nodes: Vec<GraphNode>,
    /// Execution configuration (parallelism, timeouts, etc.)
    pub config: GraphConfig,
    /// Coordination pattern (sequential, parallel, continuous, etc.)
    /// Populated from `[graph]`.coordination when loading `DeploymentGraph` format.
    #[serde(default)]
    pub coordination: Option<String>,
    /// Environment variables defined in `[graph.env]`.
    /// Used for gate endpoint definitions and variable substitution.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Declared in `[graph.metadata]` — required genetics tier for this deployment graph.
    #[serde(default)]
    pub genetics_tier: Option<GeneticsTier>,
    /// Deployment topology model (nucleated vs membrane).
    #[serde(default)]
    pub composition_model: Option<biomeos_graph::CompositionModel>,
}

impl Graph {
    /// Returns true if this graph uses continuous coordination (e.g., 60 Hz game loop).
    #[must_use]
    pub fn is_continuous(&self) -> bool {
        self.coordination
            .as_deref()
            .is_some_and(|c| c.eq_ignore_ascii_case("continuous"))
    }
}

/// Graph node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,
    /// How to select the primal (by capability or name)
    #[serde(default)]
    pub primal: Option<PrimalSelector>,
    /// Output key for downstream nodes
    #[serde(default)]
    pub output: Option<String>,
    /// Operation to invoke on the primal
    #[serde(default)]
    pub operation: Option<Operation>,
    /// Timeout and retry constraints
    #[serde(default)]
    pub constraints: Option<Constraints>,
    /// Node IDs this node depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Capabilities this primal provides (for capability registry)
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Capability translation mappings (semantic → actual method)
    #[serde(default)]
    pub capabilities_provided: Option<HashMap<String, String>>,

    /// Parameter name mappings (semantic → actual parameter names)
    #[serde(default)]
    pub parameter_mappings: Option<HashMap<String, HashMap<String, String>>>,

    /// Legacy: node type (prefer primal + operation)
    #[serde(default)]
    pub node_type: Option<String>,
    /// Legacy: dependency list (alias for `depends_on`)
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Node-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    /// Output definitions for this node
    #[serde(default)]
    pub outputs: Vec<NodeOutput>,

    /// Fallback behavior when execution fails.
    /// "skip" = silently skip (optional node), "error" = propagate error (default).
    #[serde(default)]
    pub fallback: Option<String>,

    /// Estimated execution cost in milliseconds (for Pathway Learner optimization).
    #[serde(default)]
    pub cost_estimate_ms: Option<u64>,

    /// Declared operation dependencies for Pathway Learner cost analysis.
    #[serde(default)]
    pub operation_dependencies: Vec<String>,

    /// Target gate for cross-gate deployment.
    /// Absent or `"local"` means execute on this biomeOS instance.
    /// Any other value is resolved via the graph's gate registry to a remote endpoint.
    #[serde(default)]
    pub gate: Option<String>,
}

impl GraphNode {
    /// Whether this node is optional (failure won't abort the graph).
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.fallback.as_deref() == Some("skip")
    }
}

/// Primal selector (capability-based discovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSelector {
    /// Select primal by capability (e.g., "crypto.encrypt")
    #[serde(default)]
    pub by_capability: Option<String>,
    /// Select primal by name (e.g., "beardog")
    #[serde(default)]
    pub by_name: Option<String>,
}

/// Operation definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Operation {
    /// Operation/capability name (e.g., "crypto.encrypt")
    pub name: String,
    /// Target primal for RPC operations (e.g., "songbird" for `rpc_call` nodes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Parameters for the operation
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Environment variables to pass to the primal (NEW - Jan 21, 2026)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Node constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Per-node timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Retry configuration on failure
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts before failure
    pub max_attempts: u32,
    /// Delay between retries in milliseconds
    pub backoff_ms: u64,
}

/// Node output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    /// Output name for downstream binding
    pub name: String,
    /// Output type (e.g., "string", "json")
    #[serde(rename = "type")]
    pub output_type: String,
}

/// Graph execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Run in deterministic mode (reproducible execution order)
    pub deterministic: bool,
    /// Allow parallel execution of independent phases
    pub parallel_phases: bool,
    /// Maximum concurrent node executions
    pub max_parallelism: usize,
    /// Total graph timeout in milliseconds
    pub timeout_total_ms: u64,
    /// Enable checkpointing for resume
    pub checkpoint_enabled: bool,
    /// Rollback on any node failure
    pub rollback_on_failure: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            deterministic: true,
            parallel_phases: true,
            max_parallelism: 3,
            timeout_total_ms: 60000,
            checkpoint_enabled: false,
            rollback_on_failure: true,
        }
    }
}
