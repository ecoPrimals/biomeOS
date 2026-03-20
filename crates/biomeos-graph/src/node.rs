// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
                "Node ID must be alphanumeric with hyphens/underscores: {id}"
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
        self.0.get(key).and_then(ParamValue::as_bool)
    }

    /// Get an integer parameter.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.0.get(key).and_then(ParamValue::as_i64)
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

    /// Returns true if this node uses "skip" fallback (tolerates failures).
    pub fn is_optional(&self) -> bool {
        self.fallback.as_deref() == Some("skip")
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

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // NodeId tests
    // =========================================================================

    #[test]
    fn test_node_id_validation() {
        assert!(NodeId::new("start-beardog").is_ok());
        assert!(NodeId::new("validate_seed").is_ok());
        assert!(NodeId::new("node123").is_ok());

        assert!(NodeId::new("").is_err());
        assert!(NodeId::new("has spaces").is_err());
        assert!(NodeId::new("has.dots").is_err());
    }

    #[test]
    fn test_node_id_as_str() {
        let id = NodeId::new("my-node").unwrap();
        assert_eq!(id.as_str(), "my-node");
    }

    #[test]
    fn test_node_id_display() {
        let id = NodeId::new("step-one").unwrap();
        assert_eq!(format!("{id}"), "step-one");
    }

    #[test]
    fn test_node_id_try_from_string() {
        let id: Result<NodeId, _> = NodeId::try_from("valid-id".to_string());
        assert!(id.is_ok());

        let id: Result<NodeId, _> = NodeId::try_from("has spaces".to_string());
        assert!(id.is_err());
    }

    #[test]
    fn test_node_id_into_string() {
        let id = NodeId::new("test-id").unwrap();
        let s: String = id.into();
        assert_eq!(s, "test-id");
    }

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::new("same").unwrap();
        let id2 = NodeId::new("same").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_node_id_serde_roundtrip() {
        let id = NodeId::new("serde-test").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    // =========================================================================
    // NodeType tests
    // =========================================================================

    #[test]
    fn test_node_type_default() {
        let nt = NodeType::default();
        assert_eq!(nt, NodeType::Capability);
    }

    #[test]
    fn test_node_type_serde() {
        let types = vec![
            (NodeType::Capability, "\"capability\""),
            (NodeType::Condition, "\"condition\""),
            (NodeType::Parallel, "\"parallel\""),
            (NodeType::Wait, "\"wait\""),
        ];
        for (nt, expected) in types {
            let json = serde_json::to_string(&nt).unwrap();
            assert_eq!(json, expected);
            let deserialized: NodeType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, nt);
        }
    }

    // =========================================================================
    // NodeParams tests
    // =========================================================================

    #[test]
    fn test_node_params_new_is_empty() {
        let params = NodeParams::new();
        assert!(params.get("anything").is_none());
    }

    #[test]
    fn test_node_params_insert_and_get() {
        let mut params = NodeParams::new();
        params.insert("key", "value");
        assert_eq!(params.get_string("key"), Some("value"));
    }

    #[test]
    fn test_node_params_get_types() {
        let mut params = NodeParams::new();
        params.insert("str_key", ParamValue::String("hello".to_string()));
        params.insert("bool_key", ParamValue::Bool(true));
        params.insert("int_key", ParamValue::Integer(42));

        assert_eq!(params.get_string("str_key"), Some("hello"));
        assert_eq!(params.get_bool("bool_key"), Some(true));
        assert_eq!(params.get_i64("int_key"), Some(42));

        // Wrong type returns None
        assert_eq!(params.get_string("bool_key"), None);
        assert_eq!(params.get_bool("str_key"), None);
        assert_eq!(params.get_i64("str_key"), None);
    }

    #[test]
    fn test_node_params_iter() {
        let mut params = NodeParams::new();
        params.insert("a", "one");
        params.insert("b", "two");

        assert_eq!(params.iter().count(), 2);
    }

    #[test]
    fn test_node_params_to_json() {
        let mut params = NodeParams::new();
        params.insert("key", "value");
        params.insert("num", ParamValue::Integer(99));

        let json = params.to_json();
        assert!(json.is_object());
        assert_eq!(json.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(
            json.get("num").and_then(serde_json::Value::as_i64),
            Some(99)
        );
    }

    #[test]
    fn test_node_params_default() {
        let params = NodeParams::default();
        assert!(params.get("anything").is_none());
    }

    // =========================================================================
    // ParamValue tests
    // =========================================================================

    #[test]
    fn test_param_value_from_string() {
        let pv: ParamValue = "hello".into();
        assert_eq!(pv.as_str(), Some("hello"));
    }

    #[test]
    fn test_param_value_from_owned_string() {
        let pv: ParamValue = String::from("owned").into();
        assert_eq!(pv.as_str(), Some("owned"));
    }

    #[test]
    fn test_param_value_from_i64() {
        let pv: ParamValue = 42i64.into();
        assert_eq!(pv.as_i64(), Some(42));
    }

    #[test]
    fn test_param_value_from_bool() {
        let pv: ParamValue = true.into();
        assert_eq!(pv.as_bool(), Some(true));
    }

    #[test]
    fn test_param_value_as_array() {
        let pv = ParamValue::Array(vec![ParamValue::Integer(1), ParamValue::Integer(2)]);
        let arr = pv.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_param_value_as_array_wrong_type() {
        let pv = ParamValue::String("not array".to_string());
        assert!(pv.as_array().is_none());
    }

    #[test]
    fn test_param_value_serde_roundtrip() {
        let values = vec![
            ParamValue::String("hello".to_string()),
            ParamValue::Integer(42),
            ParamValue::Bool(false),
            ParamValue::Array(vec![ParamValue::String("item".to_string())]),
        ];
        for val in values {
            let json = serde_json::to_string(&val).unwrap();
            let deserialized: ParamValue = serde_json::from_str(&json).unwrap();
            // Just check it roundtrips without error
            let _ = deserialized;
        }
    }

    // =========================================================================
    // NodeConfig tests
    // =========================================================================

    #[test]
    fn test_node_config_default() {
        let config = NodeConfig::default();
        assert!(config.primal.is_none());
        assert!(config.skip_if.is_none());
        assert!(config.retry_count.is_none());
        assert!(config.timeout_secs.is_none());
        assert!(config.extra.is_empty());
    }

    // =========================================================================
    // Condition evaluation tests
    // =========================================================================

    #[test]
    fn test_condition_evaluation() {
        let mut env = HashMap::new();
        env.insert("MODE".to_string(), "genesis".to_string());

        assert!(evaluate_condition("${MODE} == genesis", &env));
        assert!(!evaluate_condition("${MODE} == sibling", &env));
        assert!(evaluate_condition("${MODE} != sibling", &env));
    }

    #[test]
    fn test_condition_equality_miss() {
        let mut env = HashMap::new();
        env.insert("STATUS".to_string(), "active".to_string());
        assert!(!evaluate_condition("${STATUS} == inactive", &env));
    }

    #[test]
    fn test_condition_truthy_check() {
        let mut env = HashMap::new();
        env.insert("ENABLED".to_string(), "yes".to_string());

        // Non-empty, non-false, non-zero → true
        assert!(evaluate_condition("${ENABLED}", &env));
    }

    #[test]
    fn test_condition_falsy_check() {
        let mut env = HashMap::new();
        env.insert("DISABLED".to_string(), "false".to_string());
        assert!(!evaluate_condition("${DISABLED}", &env));

        env.insert("ZERO".to_string(), "0".to_string());
        assert!(!evaluate_condition("${ZERO}", &env));
    }

    #[test]
    fn test_condition_missing_var_is_falsy() {
        let env = HashMap::new();
        // Missing var resolves to empty string → falsy
        assert!(!evaluate_condition("${MISSING}", &env));
    }

    #[test]
    fn test_resolve_var_plain_string() {
        let env = HashMap::new();
        assert_eq!(resolve_var("plain", &env), "plain");
    }

    #[test]
    fn test_resolve_var_env_reference() {
        let mut env = HashMap::new();
        env.insert("MY_VAR".to_string(), "my_value".to_string());
        assert_eq!(resolve_var("${MY_VAR}", &env), "my_value");
    }

    #[test]
    fn test_resolve_var_missing() {
        let env = HashMap::new();
        assert_eq!(resolve_var("${NOPE}", &env), "");
    }

    // =========================================================================
    // GraphNode tests
    // =========================================================================

    fn make_test_node() -> GraphNode {
        let toml_str = r#"
            id = "test-node"
            name = "Test Node"
        "#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_graph_node_defaults() {
        let node = make_test_node();
        assert_eq!(node.id.as_str(), "test-node");
        assert_eq!(node.name, "Test Node");
        assert_eq!(node.node_type, NodeType::Capability);
        assert!(node.required);
        assert_eq!(node.order, 0);
        assert!(node.depends_on.is_empty());
        assert!(node.condition.is_none());
        assert!(node.capability.is_none());
    }

    #[test]
    fn test_graph_node_should_skip_no_condition() {
        let node = make_test_node();
        let env = HashMap::new();
        assert!(!node.should_skip(&env));
    }

    #[test]
    fn test_graph_node_should_skip_with_condition() {
        let toml_str = r#"
            id = "skip-node"
            name = "Skip Node"

            [config]
            skip_if = "${MODE} == skip"
        "#;
        let node: GraphNode = toml::from_str(toml_str).unwrap();

        let mut env = HashMap::new();
        env.insert("MODE".to_string(), "skip".to_string());
        assert!(node.should_skip(&env));

        env.insert("MODE".to_string(), "run".to_string());
        assert!(!node.should_skip(&env));
    }

    #[test]
    fn test_graph_node_condition_met_no_condition() {
        let node = make_test_node();
        let env = HashMap::new();
        assert!(node.condition_met(&env));
    }

    #[test]
    fn test_graph_node_condition_met_with_condition() {
        let toml_str = r#"
            id = "cond-node"
            name = "Cond Node"
            condition = "${DEPLOY} == true"
        "#;
        let node: GraphNode = toml::from_str(toml_str).unwrap();

        let mut env = HashMap::new();
        env.insert("DEPLOY".to_string(), "true".to_string());
        assert!(node.condition_met(&env));

        env.insert("DEPLOY".to_string(), "false".to_string());
        assert!(!node.condition_met(&env));
    }

    #[test]
    fn test_node_is_optional() {
        let node = make_test_node();
        assert!(!node.is_optional());

        let toml_str = r#"
            id = "optional-ai"
            name = "AI Narration"
            fallback = "skip"
        "#;
        let node: GraphNode = toml::from_str(toml_str).unwrap();
        assert!(node.is_optional());
    }

    #[test]
    fn test_cost_estimate_ms_deserializes() {
        let toml_str = r#"
            id = "expensive-node"
            name = "GPU Compute"
            cost_estimate_ms = 500
        "#;
        let node: GraphNode = toml::from_str(toml_str).unwrap();
        assert_eq!(node.cost_estimate_ms, Some(500));
    }

    #[test]
    fn test_cost_estimate_ms_defaults_to_none() {
        let node = make_test_node();
        assert_eq!(node.cost_estimate_ms, None);
    }

    #[test]
    fn test_operation_dependencies_deserializes() {
        let toml_str = r#"
            id = "write-node"
            name = "Storage Write"
            operation_dependencies = ["storage.write", "crypto.sign"]
        "#;
        let node: GraphNode = toml::from_str(toml_str).unwrap();
        assert_eq!(
            node.operation_dependencies,
            vec!["storage.write", "crypto.sign"]
        );
    }

    #[test]
    fn test_operation_dependencies_defaults_to_empty() {
        let node = make_test_node();
        assert!(node.operation_dependencies.is_empty());
    }
}
