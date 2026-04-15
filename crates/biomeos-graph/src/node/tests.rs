// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::condition::{evaluate_condition, resolve_var};
use super::*;
use std::collections::HashMap;

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
        let _ = deserialized;
    }
}

#[test]
fn test_node_config_default() {
    let config = NodeConfig::default();
    assert!(config.primal.is_none());
    assert!(config.skip_if.is_none());
    assert!(config.retry_count.is_none());
    assert!(config.timeout_secs.is_none());
    assert!(config.extra.is_empty());
}

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
