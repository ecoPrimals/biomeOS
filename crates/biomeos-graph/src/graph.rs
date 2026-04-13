// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Deployment graph types with compile-time validation.
//!
//! These types ensure that when a graph is loaded from TOML,
//! it is structurally valid before any runtime execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::node::GraphNode;

/// A validated deployment graph.
///
/// This struct represents a graph that has been:
/// 1. Parsed from TOML
/// 2. Validated for structural correctness
/// 3. Checked for dependency cycles
///
/// If you have a `DeploymentGraph`, it is guaranteed to be valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGraph {
    /// Graph definition section
    #[serde(rename = "graph")]
    pub definition: GraphDefinition,
}

/// Coordination pattern for graph execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CoordinationPattern {
    /// Nodes execute one at a time in dependency order
    #[default]
    Sequential,
    /// Independent nodes execute concurrently
    Parallel,
    /// DAG with conditional branching
    ConditionalDag,
    /// Streaming pipeline between nodes
    Pipeline,
    /// Fixed-timestep loop — nodes execute every tick
    Continuous,
}

/// Tick configuration for continuous coordination graphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickConfig {
    /// Target tick rate in Hz (e.g. 60 for a game loop)
    pub target_hz: f64,
    /// Maximum accumulator slack before frame-skipping (ms)
    #[serde(default = "default_max_accumulator_ms")]
    pub max_accumulator_ms: f64,
    /// Per-node budget warning threshold (ms); log if exceeded
    #[serde(default = "default_budget_warning_ms")]
    pub budget_warning_ms: f64,
}

const fn default_max_accumulator_ms() -> f64 {
    100.0
}

const fn default_budget_warning_ms() -> f64 {
    4.0
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            target_hz: 60.0,
            max_accumulator_ms: default_max_accumulator_ms(),
            budget_warning_ms: default_budget_warning_ms(),
        }
    }
}

/// Core graph definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDefinition {
    /// Unique identifier for the graph
    pub id: GraphId,

    /// Human-readable name (defaults to graph id if absent)
    #[serde(default)]
    pub name: String,

    /// Semantic version (defaults to "0.0.0" if absent)
    #[serde(default)]
    pub version: String,

    /// Description of what this graph does
    #[serde(default)]
    pub description: String,

    /// Graph metadata
    #[serde(default)]
    pub metadata: GraphMetadata,

    /// Coordination pattern (defaults to Sequential)
    #[serde(default)]
    pub coordination: CoordinationPattern,

    /// Tick configuration (only used when coordination = Continuous)
    #[serde(default)]
    pub tick: Option<TickConfig>,

    /// Environment variable definitions
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Nodes in the graph (execution units)
    #[serde(default)]
    pub nodes: Vec<GraphNode>,

    /// Output definitions
    #[serde(default)]
    pub outputs: HashMap<String, String>,
}

/// Graph identifier - validated to be lowercase alphanumeric with hyphens and underscores.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GraphId(String);

impl GraphId {
    /// Create a new graph ID, validating format.
    ///
    /// Accepts lowercase ASCII, digits, hyphens, and underscores — aligned with
    /// `NodeId` rules and ecosystem graph conventions (e.g. `tower_atomic_bootstrap`).
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty() {
            return Err("Graph ID cannot be empty".into());
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(format!(
                "Graph ID must be lowercase alphanumeric with hyphens/underscores: {id}"
            ));
        }
        Ok(Self(id))
    }

    /// Get the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GraphId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GraphId> for String {
    fn from(id: GraphId) -> Self {
        id.0
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about the graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Family ID this graph belongs to
    #[serde(default)]
    pub family_id: Option<String>,

    /// Author of the graph
    #[serde(default)]
    pub author: Option<String>,

    /// Creation date
    #[serde(default)]
    pub created: Option<String>,

    /// Category (deployment, validation, etc.)
    #[serde(default)]
    pub category: Option<GraphCategory>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

/// Graph category for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GraphCategory {
    /// Deployment graphs
    Deployment,
    /// Validation graphs
    Validation,
    /// Testing graphs
    Testing,
    /// Utility graphs
    #[default]
    Utility,
    /// Lifecycle graphs
    Lifecycle,
}

// ---------------------------------------------------------------------------
// Flat TOML graph DSL (`GraphParser`) — distinct from nested `DeploymentGraph`
// ---------------------------------------------------------------------------

/// Edge kind in the flat graph DSL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Dependency ordering
    Dependency,
    /// Control-flow edge
    ControlFlow,
    /// Data-flow edge
    DataFlow,
}

/// Directed edge between node IDs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    /// Source node id
    pub from: String,
    /// Target node id
    pub to: String,
    /// Edge classification
    pub edge_type: EdgeType,
}

/// Retry policy for node operations in the flat DSL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum attempts
    pub max_attempts: u32,
    /// Backoff between attempts (ms)
    pub backoff_ms: u64,
}

/// Optional execution constraints (used by tooling / future node wiring).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeConstraints {
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Retry policy if any
    pub retry_policy: Option<RetryPolicy>,
    /// Capabilities required on the host
    pub required_capabilities: Option<Vec<String>>,
}

/// Operation invoked on a primal in the flat DSL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[expect(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value contains f64, cannot impl Eq"
)]
pub struct Operation {
    /// RPC / method name
    pub name: String,
    /// JSON parameters
    pub params: serde_json::Value,
    /// Optional environment for the operation
    pub environment: Option<HashMap<String, String>>,
}

/// Capability-based or id-based primal selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalSelector {
    /// Select by registered primal id
    ById {
        /// Primal instance id
        by_id: String,
    },
    /// Select by single capability
    ByCapability {
        /// Capability string
        by_capability: String,
    },
    /// Select by multiple capabilities (AND semantics at runtime)
    ByCapabilities {
        /// Capability strings
        by_capabilities: Vec<String>,
    },
}

/// One node in the flat graph DSL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimalNode {
    /// Node id
    pub id: String,
    /// How to pick the primal
    pub primal: PrimalSelector,
    /// Operation to run
    pub operation: Operation,
    /// Runtime input (filled during execution)
    pub input: Option<serde_json::Value>,
    /// Named outputs
    pub outputs: Vec<String>,
    /// Optional execution constraints (timeout, retry, required capabilities)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<NodeConstraints>,
}

/// Parsed flat graph (see `crate::parser::GraphParser`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimalGraph {
    /// Validated graph id
    pub id: GraphId,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Semantic version string
    pub version: String,
    /// Nodes
    pub nodes: Vec<PrimalNode>,
    /// Edges
    pub edges: Vec<GraphEdge>,
    /// Coordination pattern
    pub coordination: CoordinationPattern,
}

impl DeploymentGraph {
    /// Get the graph ID.
    #[must_use]
    pub const fn id(&self) -> &GraphId {
        &self.definition.id
    }

    /// Get the graph name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Get all nodes in the graph.
    #[must_use]
    pub fn nodes(&self) -> &[GraphNode] {
        &self.definition.nodes
    }

    /// Get nodes in topological order (respecting dependencies).
    #[must_use]
    pub fn nodes_in_order(&self) -> Vec<&GraphNode> {
        // Simple topological sort using Kahn's algorithm
        let mut result = Vec::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut node_map: HashMap<&str, &GraphNode> = HashMap::new();

        // Initialize
        for node in &self.definition.nodes {
            in_degree.insert(node.id.as_str(), node.depends_on.len());
            node_map.insert(node.id.as_str(), node);
        }

        // Find nodes with no dependencies
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| *id)
            .collect();

        while let Some(node_id) = queue.pop() {
            if let Some(node) = node_map.get(node_id) {
                result.push(*node);

                // Decrease in-degree of dependent nodes
                for other in &self.definition.nodes {
                    if other.depends_on.contains(&node_id.to_string()) {
                        if let Some(deg) = in_degree.get_mut(other.id.as_str()) {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push(other.id.as_str());
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Get environment variables with defaults resolved.
    #[must_use]
    pub const fn env(&self) -> &HashMap<String, String> {
        &self.definition.env
    }

    /// Resolve an environment variable reference.
    ///
    /// Handles formats like:
    /// - `${VAR}` - Direct reference
    /// - `${VAR:-default}` - With default value
    ///
    /// Note: This resolves against system env first, then graph defaults.
    /// Graph env values like `"${VAR:-default}"` are treated as default specs,
    /// not literal values.
    #[must_use]
    pub fn resolve_env(&self, value: &str) -> String {
        let mut result = value.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;

        // Find all ${...} patterns
        while let Some(start) = result.find("${") {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                // Prevent infinite loops from self-referential patterns
                break;
            }

            if let Some(end) = result[start..].find('}') {
                let var_spec = &result[start + 2..start + end];

                // Handle ${VAR:-default} syntax
                let (var_name, inline_default) = if let Some(pos) = var_spec.find(":-") {
                    (&var_spec[..pos], Some(&var_spec[pos + 2..]))
                } else {
                    (var_spec, None)
                };

                // Check system env first
                let resolved = std::env::var(var_name)
                    .ok()
                    .or_else(|| {
                        // Then check graph env - but extract default if it's a ${VAR:-default} pattern
                        self.definition.env.get(var_name).and_then(|v| {
                            if v.starts_with("${") && v.contains(":-") {
                                // Extract default from "${VAR:-default}" pattern
                                v.find(":-")
                                    .and_then(|pos| v[pos + 2..].strip_suffix('}'))
                                    .map(String::from)
                            } else if !v.contains("${") {
                                // Literal value
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                    })
                    .or_else(|| inline_default.map(String::from))
                    .unwrap_or_default();

                result = format!(
                    "{}{}{}",
                    &result[..start],
                    resolved,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }

        result
    }
}
