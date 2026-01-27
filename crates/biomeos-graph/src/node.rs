//! Graph node types with type-safe parameters.
//!
//! Each node represents an execution unit in the graph.
//! Nodes have:
//! - An ID (unique within the graph)
//! - A capability to invoke
//! - Parameters for the capability
//! - Dependencies on other nodes

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

    /// Capability to invoke (e.g., "crypto.blake3_hash")
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
}

fn default_true() -> bool {
    true
}

/// Node identifier - validated to be alphanumeric with hyphens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NodeId(String);

impl NodeId {
    /// Create a new node ID, validating format.
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty() {
            return Err("Node ID cannot be empty".into());
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(format!(
                "Node ID must be alphanumeric with hyphens/underscores: {}",
                id
            ));
        }
        Ok(Self(id))
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NodeId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NodeId> for String {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

/// Parameters for a node's capability invocation.
///
/// This is a flexible map that gets validated against
/// the capability's expected parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeParams(HashMap<String, ParamValue>);

impl NodeParams {
    /// Create new empty params.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get a parameter value.
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.0.get(key)
    }

    /// Get a string parameter.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    /// Get a bool parameter.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.0.get(key).and_then(|v| v.as_bool())
    }

    /// Get an integer parameter.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.0.get(key).and_then(|v| v.as_i64())
    }

    /// Insert a parameter.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<ParamValue>) {
        self.0.insert(key.into(), value.into());
    }

    /// Iterate over parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParamValue)> {
        self.0.iter()
    }

    /// Convert to JSON for JSON-RPC call.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or(serde_json::Value::Null)
    }
}

/// A parameter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// String value (may contain ${VAR} references)
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<ParamValue>),
    /// Nested object
    Object(HashMap<String, ParamValue>),
}

impl ParamValue {
    /// Get as string if this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool if this is a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as i64 if this is an integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as array if this is an array.
    pub fn as_array(&self) -> Option<&Vec<ParamValue>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }
}

impl From<String> for ParamValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ParamValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ParamValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<bool> for ParamValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl GraphNode {
    /// Check if this node should be skipped based on condition.
    pub fn should_skip(&self, env: &HashMap<String, String>) -> bool {
        if let Some(skip_if) = &self.config.skip_if {
            // Simple condition evaluation: "${VAR} == value" or "${VAR} != value"
            evaluate_condition(skip_if, env)
        } else {
            false
        }
    }

    /// Check if this node's condition is met.
    pub fn condition_met(&self, env: &HashMap<String, String>) -> bool {
        if let Some(condition) = &self.condition {
            evaluate_condition(condition, env)
        } else {
            true
        }
    }
}

/// Evaluate a simple condition.
fn evaluate_condition(condition: &str, env: &HashMap<String, String>) -> bool {
    // Handle == comparison
    if let Some((left, right)) = condition.split_once("==") {
        let left = resolve_var(left.trim(), env);
        let right = right.trim().to_string();
        return left == right;
    }

    // Handle != comparison
    if let Some((left, right)) = condition.split_once("!=") {
        let left = resolve_var(left.trim(), env);
        let right = right.trim().to_string();
        return left != right;
    }

    // If no operator, treat as truthy check
    let value = resolve_var(condition.trim(), env);
    !value.is_empty() && value != "false" && value != "0"
}

/// Resolve a variable reference.
fn resolve_var(s: &str, env: &HashMap<String, String>) -> String {
    if s.starts_with("${") && s.ends_with('}') {
        let var_name = &s[2..s.len() - 1];
        env.get(var_name)
            .cloned()
            .or_else(|| std::env::var(var_name).ok())
            .unwrap_or_default()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_validation() {
        assert!(NodeId::new("start-beardog").is_ok());
        assert!(NodeId::new("validate_seed").is_ok());
        assert!(NodeId::new("node123").is_ok());

        assert!(NodeId::new("").is_err());
        assert!(NodeId::new("has spaces").is_err());
    }

    #[test]
    fn test_condition_evaluation() {
        let mut env = HashMap::new();
        env.insert("MODE".to_string(), "genesis".to_string());

        assert!(evaluate_condition("${MODE} == genesis", &env));
        assert!(!evaluate_condition("${MODE} == sibling", &env));
        assert!(evaluate_condition("${MODE} != sibling", &env));
    }
}
