//! Graph executor - Public API
//!
//! **TRUE ecoBin v2.0 EVOLVED:** Smart refactored into domain-driven modules.
//!
//! This module provides the public API for graph execution.
//! Implementation details are in focused submodules in `executor/`.
//!
//! ## Module Organization
//!
//! - `executor/core` - Main execution logic and orchestration
//! - `executor/helpers` - Utility functions (env substitution, discovery)
//! - `executor/context` - Execution context and state management
//! - `executor/monitoring` - Metrics and progress tracking
//! - `executor/rollback` - Rollback management for failed deployments
//! - `executor/topological` - Dependency resolution via topological sort
//! - `executor/nodes/*` - Node executors organized by domain
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_graph::executor::{GraphExecutor, ExecutionContext};
//!
//! let mut executor = GraphExecutor::new(graph, env);
//! let report = executor.execute().await?;
//! ```

use anyhow::Result;
use std::collections::HashMap;

use crate::graph::{Graph, Operation};

// Re-export executor modules (implementation is in submodules)
mod executor;

pub use executor::{
    context::{ExecutionContext, NodeStatus, RollbackAction},
    core::{execute_node, GraphExecutor},
    helpers::{discover_beardog_socket, discover_primal_socket, parse_config, substitute_env},
    monitoring::{ExecutionReport, PhaseResult},
    rollback::RollbackManager,
    topological::TopologicalSorter,
};

/// Trait for executing operations on primals
///
/// This trait allows custom implementations of primal operation execution.
#[async_trait::async_trait]
pub trait PrimalOperationExecutor: Send + Sync {
    /// Execute an operation on a primal
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
    ) -> Result<serde_json::Value>;
}

/// Convenience function to execute a graph
///
/// Creates a GraphExecutor and runs the graph to completion.
///
/// # Example
///
/// ```ignore
/// let env = HashMap::from([("FAMILY_ID".to_string(), "nat0".to_string())]);
/// let report = execute_graph(graph, env).await?;
/// ```
pub async fn execute_graph(
    graph: Graph,
    env: HashMap<String, String>,
) -> Result<ExecutionReport> {
    let mut executor = GraphExecutor::new(graph, env);
    executor.execute().await
}

// All implementation is now in executor/ submodules
// This file is just the public API

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CoordinationPattern, GraphId, PrimalGraph, PrimalNode, PrimalSelector};
    use std::time::Duration;

    // Mock executor for testing
    struct MockPrimalExecutor {
        delay_ms: u64,
        should_fail: bool,
    }

    impl MockPrimalExecutor {
        fn new() -> Self {
            Self {
                delay_ms: 0,
                should_fail: false,
            }
        }

        fn with_delay(delay_ms: u64) -> Self {
            Self {
                delay_ms,
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                delay_ms: 0,
                should_fail: true,
            }
        }
    }

    #[async_trait::async_trait]
    impl PrimalOperationExecutor for MockPrimalExecutor {
        async fn execute_operation(
            &self,
            primal_id: &str,
            operation: &Operation,
        ) -> Result<serde_json::Value> {
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if self.should_fail {
                anyhow::bail!("Mock failure for testing");
            }

            Ok(serde_json::json!({
                "primal": primal_id,
                "operation": operation.name,
                "status": "success"
            }))
        }
    }

    fn create_test_graph() -> PrimalGraph {
        PrimalGraph {
            id: GraphId::new("test_graph"),
            name: "Test Graph".to_string(),
            description: "A test graph".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![
                PrimalNode {
                    id: "node1".to_string(),
                    primal: PrimalSelector::ByCapability {
                        by_capability: "storage".to_string(),
                    },
                    operation: Operation {
                        name: "store".to_string(),
                        params: serde_json::json!({"key": "test"}),
                        environment: None,
                    },
                    input: None,
                    outputs: vec![],
                },
                PrimalNode {
                    id: "node2".to_string(),
                    primal: PrimalSelector::ByCapability {
                        by_capability: "compute".to_string(),
                    },
                    operation: Operation {
                        name: "process".to_string(),
                        params: serde_json::json!({}),
                        environment: None,
                    },
                    input: Some("node1".to_string()),
                    outputs: vec![],
                },
            ],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        }
    }

    #[test]
    fn test_env_substitution() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let result = GraphExecutor::substitute_env("${FOO}/${BAZ}/test", &env);
        assert_eq!(result, "bar/qux/test");
    }

    #[test]
    fn test_env_substitution_missing_var() {
        let env = HashMap::new();
        let result = GraphExecutor::substitute_env("${MISSING}/test", &env);
        assert_eq!(result, "${MISSING}/test");
    }

    #[test]
    fn test_env_substitution_no_vars() {
        let env = HashMap::new();
        let result = GraphExecutor::substitute_env("/plain/path", &env);
        assert_eq!(result, "/plain/path");
    }

    #[test]
    fn test_node_status() {
        let pending = NodeStatus::Pending;
        assert_eq!(pending, NodeStatus::Pending);

        let completed = NodeStatus::Completed(serde_json::json!({"result": "ok"}));
        match completed {
            NodeStatus::Completed(_) => (),
            _ => panic!("Expected Completed status"),
        }

        let failed = NodeStatus::Failed("error".to_string());
        match failed {
            NodeStatus::Failed(msg) => assert_eq!(msg, "error"),
            _ => panic!("Expected Failed status"),
        }
    }

    #[tokio::test]
    async fn test_execution_context_creation() {
        let context = ExecutionContext {
            env: HashMap::new(),
            outputs: Arc::new(Mutex::new(HashMap::new())),
            status: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_dir: None,
        };

        let outputs = context.outputs.lock().await;
        assert_eq!(outputs.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_executor_success() {
        let executor = MockPrimalExecutor::new();
        let operation = Operation {
            name: "test_op".to_string(),
            params: serde_json::json!({}),
            environment: None,
        };

        let result = executor
            .execute_operation("test_primal", &operation)
            .await
            .unwrap();

        assert_eq!(result["primal"], "test_primal");
        assert_eq!(result["operation"], "test_op");
        assert_eq!(result["status"], "success");
    }

    #[tokio::test]
    async fn test_mock_executor_failure() {
        let executor = MockPrimalExecutor::with_failure();
        let operation = Operation {
            name: "test_op".to_string(),
            params: serde_json::json!({}),
            environment: None,
        };

        let result = executor.execute_operation("test_primal", &operation).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock failure"));
    }

    #[tokio::test]
    async fn test_mock_executor_with_delay() {
        let executor = MockPrimalExecutor::with_delay(10);
        let operation = Operation {
            name: "test_op".to_string(),
            params: serde_json::json!({}),
            environment: None,
        };

        let start = std::time::Instant::now();
        let _result = executor.execute_operation("test_primal", &operation).await;
        let duration = start.elapsed();

        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_graph_creation() {
        let graph = create_test_graph();
        assert_eq!(graph.id.as_str(), "test_graph");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.coordination, CoordinationPattern::Sequential);
    }

    #[test]
    fn test_primal_selector_by_capability() {
        let selector = PrimalSelector::ByCapability {
            by_capability: "storage".to_string(),
        };

        match selector {
            PrimalSelector::ByCapability { by_capability } => {
                assert_eq!(by_capability, "storage");
            }
            _ => panic!("Expected ByCapability selector"),
        }
    }

    #[test]
    fn test_primal_selector_by_id() {
        let selector = PrimalSelector::ById {
            by_id: "primal1".to_string(),
        };

        match selector {
            PrimalSelector::ById { by_id } => {
                assert_eq!(by_id, "primal1");
            }
            _ => panic!("Expected ById selector"),
        }
    }

    #[test]
    fn test_operation_creation() {
        let operation = Operation {
            name: "store_data".to_string(),
            params: serde_json::json!({"key": "value"}),
            environment: None,
        };

        assert_eq!(operation.name, "store_data");
        assert_eq!(operation.params["key"], "value");
    }
}
