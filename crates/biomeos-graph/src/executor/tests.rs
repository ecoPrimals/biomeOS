// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for graph executor

use super::{
    context::ExecutionContext,
    node_handlers,
    trait::PrimalOperationExecutor,
    types::{ExecutionReport, PhaseResult},
};
use crate::graph::{CoordinationPattern, GraphId, Operation, PrimalGraph, PrimalNode, PrimalSelector};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

// Mock executor for testing
pub struct MockPrimalExecutor {
    delay_ms: u64,
    should_fail: bool,
}

impl MockPrimalExecutor {
    pub fn new() -> Self {
        Self {
            delay_ms: 0,
            should_fail: false,
        }
    }

    pub fn with_delay(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            should_fail: false,
        }
    }

    pub fn with_failure() -> Self {
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

pub fn create_test_graph() -> PrimalGraph {
    PrimalGraph {
        id: GraphId::new("test_graph").unwrap(),
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
                constraints: None,
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
                constraints: None,
            },
        ],
        edges: vec![],
        coordination: CoordinationPattern::Sequential,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_substitution() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());

        let result = node_handlers::substitute_env("${FOO}/${BAZ}/test", &env);
        assert_eq!(result, "bar/qux/test");
    }

    #[test]
    fn test_env_substitution_missing_var() {
        let env = HashMap::new();
        let result = node_handlers::substitute_env("${MISSING}/test", &env);
        assert_eq!(result, "${MISSING}/test");
    }

    #[test]
    fn test_env_substitution_no_vars() {
        let env = HashMap::new();
        let result = node_handlers::substitute_env("/plain/path", &env);
        assert_eq!(result, "/plain/path");
    }

    #[tokio::test]
    async fn test_execution_context_creation() {
        let context = ExecutionContext::new(HashMap::new());

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
            .expect("Should succeed");

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

    #[tokio::test(start_paused = true)]
    async fn test_mock_executor_with_delay() {
        let executor = MockPrimalExecutor::with_delay(10);
        let operation = Operation {
            name: "test_op".to_string(),
            params: serde_json::json!({}),
            environment: None,
        };

        let handle = tokio::spawn(async move {
            executor
                .execute_operation("test_primal", &operation)
                .await
                .expect("Should succeed")
        });
        tokio::time::advance(Duration::from_millis(10)).await;
        let result = handle.await.expect("join");
        assert_eq!(result["status"], "success");
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
