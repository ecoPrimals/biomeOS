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

    /// Shorthand action name (spring deploy graph syntax).
    /// Normalized to `operation.name` during dispatch. Supports:
    /// `check_primal` → `health_check`, `start_primal` → `start`,
    /// `wire_data` → `register_capabilities`, `invoke` → `capability_call`.
    #[serde(default)]
    pub action: Option<String>,

    /// Node-level params (spring deploy graph shorthand).
    /// When `action` is used instead of `[nodes.operation]`, parameters are
    /// specified here. The executor merges these with `operation.params`.
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

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
    /// `"auto"` triggers Plasmodium workload dispatch based on `compute_requirements`.
    /// Any other value is resolved via the graph's gate registry to a remote endpoint.
    #[serde(default)]
    pub gate: Option<String>,

    /// Compute requirements for Plasmodium auto-dispatch (when `gate = "auto"`).
    /// If set, the dispatcher selects the best gate from the collective.
    #[serde(default)]
    pub compute_requirements: Option<ComputeRequirements>,
}

/// Declarative compute requirements for a graph node.
/// Used by the Plasmodium dispatcher to select the optimal gate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeRequirements {
    /// Minimum GPU VRAM in megabytes.
    #[serde(default)]
    pub min_vram_mb: u64,
    /// Minimum system RAM in gigabytes.
    #[serde(default)]
    pub min_ram_gb: u64,
    /// Minimum CPU core count.
    #[serde(default)]
    pub min_cpu_cores: usize,
    /// Required capability domain (e.g. "compute", "inference").
    #[serde(default)]
    pub capability: Option<String>,
    /// Maximum acceptable gate load (0.0-1.0).
    #[serde(default)]
    pub max_load: f64,
}

impl GraphNode {
    /// Whether this node is optional (failure won't abort the graph).
    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.fallback.as_deref() == Some("skip")
    }

    /// Resolve a parameter by key, checking (in priority order):
    /// 1. `operation.params[key]`
    /// 2. `params[key]` (spring deploy graph shorthand)
    /// 3. `config[key]` (legacy)
    ///
    /// Spring graphs alias `primal` → `primal_name` for handler compatibility.
    #[must_use]
    pub fn effective_param(&self, key: &str) -> Option<&serde_json::Value> {
        if let Some(op) = &self.operation {
            if let Some(v) = op.params.get(key) {
                return Some(v);
            }
        }
        if let Some(v) = self.params.get(key) {
            return Some(v);
        }
        if let Some(v) = self.config.get(key) {
            return Some(v);
        }
        // Spring alias: `primal` → `primal_name`
        if key == "primal_name" {
            return self.effective_param("primal");
        }
        None
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
