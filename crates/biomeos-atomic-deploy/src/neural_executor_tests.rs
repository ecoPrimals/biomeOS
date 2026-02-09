//! Tests for GraphExecutor
//!
//! Extracted from neural_executor.rs to keep file under 1000 lines.

use super::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig, GraphNode};
use std::collections::HashMap;

#[test]
fn test_env_substitution() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    env.insert("BAZ".to_string(), "qux".to_string());

    let result = GraphExecutor::substitute_env("${FOO}/${BAZ}/test", &env);
    assert_eq!(result, "bar/qux/test");
}

#[test]
fn test_env_substitution_empty() {
    let env = HashMap::new();
    let result = GraphExecutor::substitute_env("no-vars", &env);
    assert_eq!(result, "no-vars");
}

#[test]
fn test_env_substitution_partial() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    let result = GraphExecutor::substitute_env("${FOO}/${MISSING}", &env);
    assert_eq!(result, "bar/${MISSING}");
}

#[test]
fn test_graph_executor_creation() {
    let graph = Graph {
        id: "test-graph".to_string(),
        version: "1.0.0".to_string(),
        description: "Test graph".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
    };
    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    assert_eq!(executor.max_parallelism, 3);
}

fn create_test_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        primal: None,
        output: None,
        operation: None,
        constraints: None,
        capabilities: vec![],
        capabilities_provided: None,
        parameter_mappings: None,
        node_type: None,
        dependencies: vec![],
        config: HashMap::new(),
        outputs: vec![],
    }
}

#[test]
fn test_topological_sort_simple() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec![]),
            create_test_node("node2", vec!["node1".to_string()]),
        ],
        config: GraphConfig::default(),
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0], vec!["node1"]);
    assert_eq!(phases[1], vec!["node2"]);
}

#[test]
fn test_topological_sort_parallel() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec![]),
            create_test_node("node2", vec![]),
            create_test_node("node3", vec!["node1".to_string(), "node2".to_string()]),
        ],
        config: GraphConfig::default(),
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0].len(), 2); // node1 and node2 in parallel
    assert!(phases[0].contains(&"node1".to_string()));
    assert!(phases[0].contains(&"node2".to_string()));
    assert_eq!(phases[1], vec!["node3"]);
}

#[test]
fn test_topological_sort_cycle_detection() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("node1", vec!["node2".to_string()]),
            create_test_node("node2", vec!["node1".to_string()]),
        ],
        config: GraphConfig::default(),
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let result = executor.topological_sort();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn test_topological_sort_empty_graph() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();
    assert_eq!(phases.len(), 0);
}

#[test]
fn test_topological_sort_complex_dependencies() {
    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a".to_string()]),
            create_test_node("c", vec!["a".to_string()]),
            create_test_node("d", vec!["b".to_string(), "c".to_string()]),
        ],
        config: GraphConfig::default(),
    };

    let env = HashMap::new();
    let executor = GraphExecutor::new(graph, env);
    let phases = executor.topological_sort().unwrap();

    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0], vec!["a"]);
    assert_eq!(phases[1].len(), 2); // b and c in parallel
    assert_eq!(phases[2], vec!["d"]);
}

#[tokio::test]
async fn test_execution_context_with_nucleation() {
    use crate::nucleation::SocketNucleation;
    use std::sync::Arc;

    let graph = Graph {
        id: "test".to_string(),
        version: "1.0".to_string(),
        description: "test".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
    };
    let env = HashMap::new();
    let nucleation = Arc::new(tokio::sync::RwLock::new(SocketNucleation::default()));

    let executor = GraphExecutor::with_nucleation(graph, env, nucleation);
    assert_eq!(executor.max_parallelism, 3);
}

#[test]
fn test_graph_config_default() {
    let config = GraphConfig::default();
    assert!(config.deterministic);
    assert!(config.parallel_phases);
    assert_eq!(config.max_parallelism, 3);
}
