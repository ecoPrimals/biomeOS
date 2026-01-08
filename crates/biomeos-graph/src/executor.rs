// =============================================================================
// Graph Executor - Sequential Execution (Phase 1.1)
// =============================================================================
//
// Modern idiomatic Rust executor:
// - Async/await (no blocking)
// - Clear error propagation
// - Metrics collection
// - No unsafe code
//
// Future phases will add:
// - Parallel execution (Phase 1.2)
// - DAG execution (Phase 1.3)
// - Pipeline execution (Phase 1.4)
//
// =============================================================================

use crate::context::ExecutionContext;
use crate::error::{GraphError, Result};
use crate::graph::*;
use async_trait::async_trait;
use chrono::Utc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Trait for executing operations on primals
///
/// This is the interface to biomeOS's primal registry.
/// We don't hardcode primal knowledge here!
#[async_trait]
pub trait PrimalOperationExecutor: Send + Sync {
    /// Execute an operation on a primal
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value>;
    
    /// Discover primals by capability
    async fn discover_primals(&self) -> Result<Vec<(String, Vec<String>)>>;
}

/// Graph executor
pub struct GraphExecutor<E: PrimalOperationExecutor> {
    operation_executor: E,
}

impl<E: PrimalOperationExecutor> GraphExecutor<E> {
    /// Create a new graph executor
    pub fn new(operation_executor: E) -> Self {
        Self { operation_executor }
    }
    
    /// Execute a graph
    pub async fn execute(&self, graph: PrimalGraph) -> Result<GraphResult> {
        info!(graph_name = %graph.name, "Starting graph execution");
        
        // Create execution context
        let context = ExecutionContext::new();
        
        // Discover primals (capability-based!)
        self.discover_and_register_primals(&context).await?;
        
        // Execute based on coordination pattern
        let metrics = match graph.coordination {
            CoordinationPattern::Sequential => {
                self.execute_sequential(&graph, &context).await?
            },
            CoordinationPattern::Parallel => {
                warn!("Parallel execution not yet implemented, falling back to sequential");
                self.execute_sequential(&graph, &context).await?
            },
            CoordinationPattern::ConditionalDAG => {
                warn!("DAG execution not yet implemented, falling back to sequential");
                self.execute_sequential(&graph, &context).await?
            },
            CoordinationPattern::Pipeline => {
                warn!("Pipeline execution not yet implemented, falling back to sequential");
                self.execute_sequential(&graph, &context).await?
            },
        };
        
        // Check if all succeeded
        let success = metrics.iter().all(|m| m.success);
        
        info!(
            graph_name = %graph.name,
            success = success,
            nodes_executed = metrics.len(),
            "Graph execution complete"
        );
        
        Ok(GraphResult {
            success,
            outputs: context.get_all_outputs(),
            metrics,
        })
    }
    
    /// Discover and register primals in context
    async fn discover_and_register_primals(&self, context: &ExecutionContext) -> Result<()> {
        debug!("Discovering primals");
        
        let primals = self.operation_executor.discover_primals().await?;
        
        for (primal_id, capabilities) in primals {
            debug!(
                primal_id = %primal_id,
                capabilities = ?capabilities,
                "Registered primal"
            );
            context.register_primal(primal_id, capabilities);
        }
        
        Ok(())
    }
    
    /// Execute graph sequentially (one node after another)
    async fn execute_sequential(
        &self,
        graph: &PrimalGraph,
        context: &ExecutionContext,
    ) -> Result<Vec<NodeMetrics>> {
        let mut metrics = Vec::new();
        
        for node in &graph.nodes {
            let node_metrics = self.execute_node(node, context).await?;
            metrics.push(node_metrics);
        }
        
        Ok(metrics)
    }
    
    /// Execute a single node
    async fn execute_node(
        &self,
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<NodeMetrics> {
        let started_at = Utc::now();
        let start_instant = Instant::now();
        
        info!(node_id = %node.id, operation = %node.operation.name, "Executing node");
        
        // Resolve primal (capability-based discovery!)
        let primal_id = self.resolve_primal(&node.primal, context)?;
        
        debug!(
            node_id = %node.id,
            primal_id = %primal_id,
            "Resolved primal for node"
        );
        
        // Execute operation
        let result = self.execute_with_retry(
            &primal_id,
            &node.operation,
            context,
            &node.constraints,
        ).await;
        
        let duration_ms = start_instant.elapsed().as_millis() as u64;
        let completed_at = Utc::now();
        
        match result {
            Ok(output) => {
                info!(
                    node_id = %node.id,
                    duration_ms = duration_ms,
                    "Node execution succeeded"
                );
                
                // Store output if specified
                if let Some(output_var) = &node.output {
                    context.set_output(output_var.clone(), output.clone());
                }
                
                Ok(NodeMetrics {
                    node_id: node.id.clone(),
                    primal_id,
                    operation: node.operation.name.clone(),
                    duration_ms,
                    success: true,
                    error: None,
                    started_at,
                    completed_at,
                })
            },
            Err(e) => {
                error!(
                    node_id = %node.id,
                    duration_ms = duration_ms,
                    error = %e,
                    "Node execution failed"
                );
                
                Ok(NodeMetrics {
                    node_id: node.id.clone(),
                    primal_id,
                    operation: node.operation.name.clone(),
                    duration_ms,
                    success: false,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                })
            }
        }
    }
    
    /// Resolve primal ID from selector (CAPABILITY-BASED!)
    fn resolve_primal(
        &self,
        selector: &PrimalSelector,
        context: &ExecutionContext,
    ) -> Result<String> {
        match selector {
            PrimalSelector::ById { by_id } => {
                Ok(by_id.clone())
            },
            PrimalSelector::ByCapability { by_capability } => {
                context.find_primal_by_capability(by_capability)
                    .ok_or_else(|| GraphError::CapabilityNotFound(by_capability.clone()))
            },
            PrimalSelector::ByCapabilities { by_capabilities } => {
                context.find_primal_by_capabilities(by_capabilities)
                    .ok_or_else(|| GraphError::CapabilityNotFound(
                        format!("No primal with all capabilities: {:?}", by_capabilities)
                    ))
            },
        }
    }
    
    /// Execute with retry policy
    async fn execute_with_retry(
        &self,
        primal_id: &str,
        operation: &Operation,
        context: &ExecutionContext,
        constraints: &Option<NodeConstraints>,
    ) -> Result<serde_json::Value> {
        let retry_policy = constraints.as_ref()
            .and_then(|c| c.retry.as_ref());
        
        if let Some(retry) = retry_policy {
            let mut attempts = 0;
            let mut last_error = None;
            
            while attempts < retry.max_attempts {
                attempts += 1;
                
                match self.operation_executor.execute_operation(primal_id, operation, context).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        warn!(
                            primal_id = %primal_id,
                            operation = %operation.name,
                            attempt = attempts,
                            max_attempts = retry.max_attempts,
                            error = %e,
                            "Operation failed, retrying"
                        );
                        last_error = Some(e);
                        
                        if attempts < retry.max_attempts {
                            tokio::time::sleep(tokio::time::Duration::from_millis(retry.backoff_ms)).await;
                        }
                    }
                }
            }
            
            Err(last_error.unwrap_or_else(|| {
                GraphError::ExecutionError("All retry attempts failed".to_string())
            }))
        } else {
            // No retry, execute once
            self.operation_executor.execute_operation(primal_id, operation, context).await
        }
    }
}

// =============================================================================
// Mock Executor (ONLY FOR TESTING!)
// =============================================================================

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    
    /// Mock primal operation executor for testing
    pub struct MockPrimalOperationExecutor {
        primals: Vec<(String, Vec<String>)>,
        operation_results: HashMap<String, serde_json::Value>,
    }
    
    impl MockPrimalOperationExecutor {
        pub fn new() -> Self {
            Self {
                primals: vec![],
                operation_results: HashMap::new(),
            }
        }
        
        pub fn with_primal(mut self, id: &str, capabilities: Vec<&str>) -> Self {
            self.primals.push((
                id.to_string(),
                capabilities.iter().map(|s| s.to_string()).collect(),
            ));
            self
        }
        
        pub fn with_operation_result(
            mut self,
            operation: &str,
            result: serde_json::Value,
        ) -> Self {
            self.operation_results.insert(operation.to_string(), result);
            self
        }
    }
    
    #[async_trait]
    impl PrimalOperationExecutor for MockPrimalOperationExecutor {
        async fn execute_operation(
            &self,
            _primal_id: &str,
            operation: &Operation,
            _context: &ExecutionContext,
        ) -> Result<serde_json::Value> {
            Ok(self.operation_results
                .get(&operation.name)
                .cloned()
                .unwrap_or(serde_json::Value::Null))
        }
        
        async fn discover_primals(&self) -> Result<Vec<(String, Vec<String>)>> {
            Ok(self.primals.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::MockPrimalOperationExecutor;
    
    #[tokio::test]
    async fn test_execute_sequential_graph() {
        let executor = GraphExecutor::new(
            MockPrimalOperationExecutor::new()
                .with_primal("test-primal", vec!["test"])
                .with_operation_result("start", serde_json::Value::Bool(true))
        );
        
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![
                GraphNode {
                    id: "node1".to_string(),
                    primal: PrimalSelector::ById { by_id: "test-primal".to_string() },
                    operation: Operation {
                        name: "start".to_string(),
                        params: serde_json::Value::Null,
                    },
                    input: None,
                    output: None,
                    constraints: None,
                    parallel_group: None,
                },
            ],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };
        
        let result = executor.execute(graph).await.unwrap();
        assert!(result.success);
        assert_eq!(result.metrics.len(), 1);
    }
    
    #[tokio::test]
    async fn test_capability_based_discovery() {
        let executor = GraphExecutor::new(
            MockPrimalOperationExecutor::new()
                .with_primal("songbird-1", vec!["discovery", "tunneling"])
                .with_operation_result("discover", serde_json::json!({"found": true}))
        );
        
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![
                GraphNode {
                    id: "discover-node".to_string(),
                    primal: PrimalSelector::ByCapability { by_capability: "discovery".to_string() },
                    operation: Operation {
                        name: "discover".to_string(),
                        params: serde_json::Value::Null,
                    },
                    input: None,
                    output: Some("discovered".to_string()),
                    constraints: None,
                    parallel_group: None,
                },
            ],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };
        
        let result = executor.execute(graph).await.unwrap();
        assert!(result.success);
        assert!(result.outputs.contains_key("discovered"));
    }
}

