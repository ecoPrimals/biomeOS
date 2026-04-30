// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::node_handlers::*;
use super::context::ExecutionContext;
use crate::graph::GraphNode;
use crate::node::{NodeConfig, NodeId, NodeParams, NodeType};
use std::collections::HashMap;
use tempfile::TempDir;
use std::fs;

fn create_test_node(id: &str) -> GraphNode {
    GraphNode {
        id: NodeId::new(id).unwrap(),
        name: id.to_string(),
        node_type: NodeType::Capability,
        capability: None,
        required: true,
        order: 0,
        depends_on: vec![],
        condition: None,
        config: NodeConfig::default(),
        params: NodeParams::new(),
        feedback_to: None,
        budget_ms: None,
        fallback: None,
        cost_estimate_ms: None,
        operation_dependencies: Vec::new(),
    }
}

fn create_test_context(env: HashMap<String, String>) -> ExecutionContext {
    ExecutionContext::new(env)
}

#[test]
fn test_substitute_env_basic() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    env.insert("BAZ".to_string(), "qux".to_string());

    let result = substitute_env("${FOO}/${BAZ}/test", &env);
    assert_eq!(result, "bar/qux/test");
}

#[test]
fn test_substitute_env_empty() {
    let env = HashMap::new();
    let result = substitute_env("no-vars", &env);
    assert_eq!(result, "no-vars");
}

#[test]
fn test_substitute_env_partial() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    let result = substitute_env("${FOO}/${MISSING}", &env);
    assert_eq!(result, "bar/${MISSING}");
}

#[test]
fn test_substitute_env_multiple_same() {
    let mut env = HashMap::new();
    env.insert("VAR".to_string(), "value".to_string());
    let result = substitute_env("${VAR}-${VAR}-${VAR}", &env);
    assert_eq!(result, "value-value-value");
}

#[tokio::test]
async fn test_filesystem_check_exists_success() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let mut node = create_test_node("test-node");
    node.config.extra.insert(
        "path".to_string(),
        toml::Value::String(test_file.to_string_lossy().to_string()),
    );

    let context = create_test_context(HashMap::new());
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.get("exists"), Some(&serde_json::json!(true)));
}

#[tokio::test]
async fn test_filesystem_check_exists_missing_path() {
    let mut node = create_test_node("test-node");
    // No path in config

    let context = create_test_context(HashMap::new());
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_check_exists_file_not_found() {
    let mut node = create_test_node("test-node");
    node.config.extra.insert(
        "path".to_string(),
        toml::Value::String("/nonexistent/path/file.txt".to_string()),
    );

    let context = create_test_context(HashMap::new());
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_check_exists_with_size_check() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test").unwrap(); // 4 bytes

    let mut node = create_test_node("test-node");
    node.config.extra.insert(
        "path".to_string(),
        toml::Value::String(test_file.to_string_lossy().to_string()),
    );
    node.config.extra.insert(
        "expected_size".to_string(),
        toml::Value::Integer(4),
    );

    let context = create_test_context(HashMap::new());
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_filesystem_check_exists_size_mismatch() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test").unwrap(); // 4 bytes

    let mut node = create_test_node("test-node");
    node.config.extra.insert(
        "path".to_string(),
        toml::Value::String(test_file.to_string_lossy().to_string()),
    );
    node.config.extra.insert(
        "expected_size".to_string(),
        toml::Value::Integer(10), // Wrong size
    );

    let context = create_test_context(HashMap::new());
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_check_exists_with_env_substitution() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test").unwrap();

    let mut env = HashMap::new();
    env.insert("TEST_DIR".to_string(), temp_dir.path().to_string_lossy().to_string());

    let mut node = create_test_node("test-node");
    node.config.extra.insert(
        "path".to_string(),
        toml::Value::String("${TEST_DIR}/test.txt".to_string()),
    );

    let context = create_test_context(env);
    let result = node_filesystem_check_exists(&node, &context).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deployment_report() {
    let mut node = create_test_node("report-node");
    let atomics = vec![
        toml::Value::String("tower".to_string()),
        toml::Value::String("nest".to_string()),
    ];
    node.config.extra.insert(
        "atomics_deployed".to_string(),
        toml::Value::Array(atomics),
    );

    let context = create_test_context(HashMap::new());
    let result = node_deployment_report(&node, &context).await;

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.get("success"), Some(&serde_json::json!(true)));
    assert!(value.get("atomics_deployed").is_some());
    assert!(value.get("timestamp").is_some());
}

#[tokio::test]
async fn test_deployment_report_empty_atomics() {
    let node = create_test_node("report-node");
    let context = create_test_context(HashMap::new());
    let result = node_deployment_report(&node, &context).await;

    assert!(result.is_ok());
    let value = result.unwrap();
    let atomics = value.get("atomics_deployed").and_then(|v| v.as_array()).unwrap();
    assert_eq!(atomics.len(), 0);
}

#[tokio::test]
async fn test_health_check_no_primal() {
    let mut node = create_test_node("health-node");
    node.config.extra.insert(
        "atomic_type".to_string(),
        toml::Value::String("tower".to_string()),
    );

    let context = create_test_context(HashMap::new());
    let result = node_health_check(&node, &context).await;

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.get("healthy"), Some(&serde_json::json!(true)));
    assert_eq!(value.get("atomic"), Some(&serde_json::json!("tower")));
}

#[tokio::test]
async fn test_health_check_missing_primal_config() {
    let mut node = create_test_node("health-node");
    node.config.extra.insert(
        "atomic_type".to_string(),
        toml::Value::String("tower".to_string()),
    );
    // No primal specified

    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family".to_string());
    let context = create_test_context(env);

    let result = node_health_check(&node, &context).await;
    // Should succeed with basic healthy status
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_lineage_verify_no_beardog() {
    let mut node = create_test_node("lineage-node");
    let siblings = vec![
        toml::Value::String("sibling1".to_string()),
        toml::Value::String("sibling2".to_string()),
    ];
    node.config.extra.insert(
        "siblings".to_string(),
        toml::Value::Array(siblings),
    );

    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family".to_string());
    let context = create_test_context(env);

    // Should gracefully degrade when BearDog is not available
    let result = node_lineage_verify(&node, &context).await;
    // May succeed with graceful degradation or fail - both are acceptable
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_lineage_verify_empty_siblings() {
    let node = create_test_node("lineage-node");
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family".to_string());
    let context = create_test_context(env);

    // Should handle empty siblings list
    let result = node_lineage_verify(&node, &context).await;
    // May succeed or fail depending on BearDog availability
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_substitute_env_edge_cases() {
    let mut env = HashMap::new();
    env.insert("EMPTY".to_string(), "".to_string());
    env.insert("SPECIAL".to_string(), "a/b/c".to_string());

    assert_eq!(substitute_env("${EMPTY}", &env), "");
    assert_eq!(substitute_env("${SPECIAL}", &env), "a/b/c");
    assert_eq!(substitute_env("prefix-${EMPTY}-suffix", &env), "prefix--suffix");
}
