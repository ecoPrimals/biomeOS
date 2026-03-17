// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph data structures for Neural API

use anyhow::Context;
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
    /// Populated from `[graph]`.coordination when loading DeploymentGraph format.
    #[serde(default)]
    pub coordination: Option<String>,
}

impl Graph {
    /// Load graph from TOML file
    pub fn from_toml_file(path: &std::path::Path) -> anyhow::Result<Self> {
        tracing::debug!("📖 Reading graph file: {}", path.display());
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        tracing::debug!("   File size: {} bytes", contents.len());

        Self::from_toml_str(&contents)
            .with_context(|| format!("Failed to parse TOML from: {}", path.display()))
    }

    /// Load graph from TOML string
    pub fn from_toml_str(toml: &str) -> anyhow::Result<Self> {
        tracing::debug!("🔍 Parsing TOML structure...");

        // Parse TOML
        let value: toml::Value = toml::from_str(toml).context("Failed to parse TOML syntax")?;

        tracing::debug!("✅ TOML syntax valid");

        // Extract graph metadata
        tracing::debug!("🔍 Looking for [graph] section...");
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow::anyhow!("Missing [graph] section in TOML"))?;

        tracing::debug!("✅ Found [graph] section");

        let id = graph_table
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = graph_table
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let description = graph_table
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract nodes
        // Accept both [[nodes]] (neural_graph) and [[graph.nodes]] (DeploymentGraph) formats.
        // The DeploymentGraph format nests nodes under the [graph] section and uses
        // a different field schema (capability, budget_ms, feedback_to, config, params).
        // We convert DeploymentGraph nodes to the neural_graph schema on the fly.
        tracing::debug!("🔍 Looking for [[nodes]] or [[graph.nodes]] array...");

        let (nodes_array, from_deployment_graph) = if let Some(arr) =
            value.get("nodes").and_then(|v| v.as_array())
        {
            (arr.clone(), false)
        } else if let Some(arr) = graph_table.get("nodes").and_then(|v| v.as_array()) {
            tracing::debug!("   Found [[graph.nodes]] format — converting to neural_graph schema");
            (arr.clone(), true)
        } else {
            tracing::error!("❌ Missing [[nodes]] and [[graph.nodes]] arrays in TOML");
            anyhow::bail!(
                "Missing [[nodes]] or [[graph.nodes]] array. Found keys: {:?}",
                value.as_table().map(|t| t.keys().collect::<Vec<_>>())
            );
        };

        tracing::debug!(
            "✅ Found {} nodes (deployment_graph={})",
            nodes_array.len(),
            from_deployment_graph
        );

        let mut nodes = Vec::new();
        for (idx, node_value) in nodes_array.iter().enumerate() {
            tracing::debug!("   Parsing node {}...", idx);
            if from_deployment_graph {
                let node = Self::convert_deployment_node(node_value)
                    .with_context(|| format!("Failed to convert deployment node {idx}"))?;
                tracing::debug!("   ✅ Node {}: id={}", idx, node.id);
                nodes.push(node);
            } else {
                let node: GraphNode = toml::from_str(&toml::to_string(node_value)?)
                    .with_context(|| format!("Failed to parse node {idx} structure"))?;
                tracing::debug!("   ✅ Node {}: id={}", idx, node.id);
                nodes.push(node);
            }
        }

        tracing::info!("✅ Parsed {} nodes successfully", nodes.len());

        // Extract execution config
        let config = if let Some(exec_table) = value.get("execution").and_then(|v| v.as_table()) {
            GraphConfig {
                deterministic: exec_table
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "deterministic")
                    .unwrap_or(true),
                parallel_phases: exec_table
                    .get("parallel_phases")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                max_parallelism: exec_table
                    .get("max_parallelism")
                    .and_then(|v| v.as_integer())
                    .and_then(|v| usize::try_from(v).ok())
                    .unwrap_or(3),
                timeout_total_ms: exec_table
                    .get("timeout_total_ms")
                    .and_then(|v| v.as_integer())
                    .and_then(|v| u64::try_from(v).ok())
                    .unwrap_or(60000),
                checkpoint_enabled: exec_table
                    .get("checkpoint_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                rollback_on_failure: exec_table
                    .get("rollback_on_failure")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            }
        } else {
            GraphConfig::default()
        };

        let coordination = graph_table
            .get("coordination")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Self {
            id,
            version,
            description,
            nodes,
            config,
            coordination,
        })
    }

    /// Returns true if this graph uses continuous coordination (e.g., 60 Hz game loop).
    #[must_use]
    pub fn is_continuous(&self) -> bool {
        self.coordination
            .as_deref()
            .is_some_and(|c| c.eq_ignore_ascii_case("continuous"))
    }

    /// Convert a `[[graph.nodes]]` (DeploymentGraph) node into the neural_graph `GraphNode` schema.
    ///
    /// DeploymentGraph nodes have: id, name, capability, depends_on, feedback_to,
    /// budget_ms, config.primal, params.*
    ///
    /// Neural graph nodes have: id, operation.name, operation.params, constraints.timeout_ms,
    /// depends_on, capabilities, config.*
    fn convert_deployment_node(node_value: &toml::Value) -> anyhow::Result<GraphNode> {
        let table = node_value.as_table().context("Node must be a TOML table")?;

        let id = table
            .get("id")
            .and_then(|v| v.as_str())
            .context("Node missing 'id'")?
            .to_string();

        let capability = table
            .get("capability")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let depends_on: Vec<String> = table
            .get("depends_on")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let budget_ms: Option<u64> = table.get("budget_ms").and_then(|v| {
            let f = v.as_float().or_else(|| {
                v.as_integer().map(|i| {
                    // budget_ms values are small (1-16 ms), no precision loss
                    i as f64
                })
            })?;
            Some(f as u64)
        });

        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let feedback_to = table
            .get("feedback_to")
            .and_then(|v| v.as_str())
            .map(String::from);

        let fallback = table
            .get("fallback")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Extract params from [graph.nodes.params]
        let params: HashMap<String, serde_json::Value> = table
            .get("params")
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| toml_value_to_json(v).map(|jv| (k.clone(), jv)))
                    .collect()
            })
            .unwrap_or_default();

        // Extract primal hint from [graph.nodes.config]
        let primal_name = table
            .get("config")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("primal"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let operation = if capability.is_empty() {
            None
        } else {
            Some(Operation {
                name: "capability_call".to_string(),
                params: {
                    let mut p = HashMap::new();
                    p.insert(
                        "capability".to_string(),
                        serde_json::Value::String(capability.clone()),
                    );
                    for (k, v) in &params {
                        p.insert(k.clone(), v.clone());
                    }
                    p
                },
                environment: None,
            })
        };

        let constraints = budget_ms.map(|ms| Constraints {
            timeout_ms: Some(ms),
            retry: None,
        });

        let capabilities = if capability.is_empty() {
            vec![]
        } else {
            vec![capability]
        };

        // Store feedback_to and primal hint in node config for downstream use
        let mut config = HashMap::new();
        if let Some(ft) = feedback_to {
            config.insert("feedback_to".to_string(), serde_json::Value::String(ft));
        }
        if let Some(pn) = primal_name {
            config.insert("primal".to_string(), serde_json::Value::String(pn));
        }
        if !name.is_empty() {
            config.insert("name".to_string(), serde_json::Value::String(name));
        }

        let cost_estimate_ms = table
            .get("cost_estimate_ms")
            .and_then(|v| v.as_integer())
            .and_then(|v| u64::try_from(v).ok());

        let operation_dependencies: Vec<String> = table
            .get("operation_dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(GraphNode {
            id,
            primal: None,
            output: None,
            operation,
            constraints,
            depends_on,
            capabilities,
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config,
            outputs: vec![],
            fallback,
            cost_estimate_ms,
            operation_dependencies,
        })
    }
}

/// Convert a TOML value to a serde_json Value.
fn toml_value_to_json(v: &toml::Value) -> Option<serde_json::Value> {
    match v {
        toml::Value::String(s) => Some(serde_json::Value::String(s.clone())),
        toml::Value::Integer(i) => Some(serde_json::json!(i)),
        toml::Value::Float(f) => Some(serde_json::json!(f)),
        toml::Value::Boolean(b) => Some(serde_json::Value::Bool(*b)),
        toml::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().filter_map(toml_value_to_json).collect();
            Some(serde_json::Value::Array(items))
        }
        toml::Value::Table(t) => {
            let map: serde_json::Map<String, serde_json::Value> = t
                .iter()
                .filter_map(|(k, v)| toml_value_to_json(v).map(|jv| (k.clone(), jv)))
                .collect();
            Some(serde_json::Value::Object(map))
        }
        toml::Value::Datetime(dt) => Some(serde_json::Value::String(dt.to_string())),
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
    /// Legacy: dependency list (alias for depends_on)
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
}


impl GraphNode {
    /// Whether this node is optional (failure won't abort the graph).
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation/capability name (e.g., "crypto.encrypt")
    pub name: String,
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

// Tests are in neural_graph_tests.rs to keep this file under 1000 lines
