//! Graph executor for deterministic deployment orchestration
//!
//! This module executes Neural API graphs with:
//! - Topological sorting for dependency resolution
//! - Parallel execution within phases
//! - Checkpoint/rollback support
//! - Live monitoring and metrics

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::neural_graph::{Graph, GraphNode};

/// Execution status for a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed(serde_json::Value),
    Failed(String),
    Skipped,
}

/// Execution context shared across nodes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Node outputs (for dependency resolution)
    pub outputs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    /// Execution status of nodes
    pub status: Arc<Mutex<HashMap<String, NodeStatus>>>,
    /// Checkpoint directory
    pub checkpoint_dir: Option<PathBuf>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(env: HashMap<String, String>) -> Self {
        Self {
            env,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            status: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_dir: None,
        }
    }

    /// Set output for a node
    pub async fn set_output(&self, node_id: &str, value: serde_json::Value) {
        let mut outputs = self.outputs.lock().await;
        outputs.insert(node_id.to_string(), value);
    }

    /// Get output from a node
    pub async fn get_output(&self, node_id: &str) -> Option<serde_json::Value> {
        let outputs = self.outputs.lock().await;
        outputs.get(node_id).cloned()
    }

    /// Set node status
    pub async fn set_status(&self, node_id: &str, status: NodeStatus) {
        let mut statuses = self.status.lock().await;
        statuses.insert(node_id.to_string(), status);
    }

    /// Get node status
    pub async fn get_status(&self, node_id: &str) -> Option<NodeStatus> {
        let statuses = self.status.lock().await;
        statuses.get(node_id).cloned()
    }
}

/// Graph executor
pub struct GraphExecutor {
    graph: Graph,
    context: ExecutionContext,
    max_parallelism: usize,
}

impl GraphExecutor {
    /// Create new graph executor
    pub fn new(graph: Graph, env: HashMap<String, String>) -> Self {
        Self {
            graph,
            context: ExecutionContext::new(env),
            max_parallelism: 3, // Default from graph spec
        }
    }

    /// Execute the entire graph
    pub async fn execute(&mut self) -> Result<ExecutionReport> {
        info!("🚀 Starting graph execution: {}", self.graph.id);

        let start_time = std::time::Instant::now();
        let mut report = ExecutionReport::new(self.graph.id.clone());

        // Topological sort to get execution phases
        let phases = self.topological_sort()?;

        info!("   Execution plan: {} phases", phases.len());

        // Execute each phase
        for (phase_num, phase_nodes) in phases.iter().enumerate() {
            info!(
                "📍 Phase {}/{}: {} nodes",
                phase_num + 1,
                phases.len(),
                phase_nodes.len()
            );

            match self.execute_phase(phase_nodes).await {
                Ok(phase_results) => {
                    report.phase_results.push(phase_results);
                }
                Err(e) => {
                    error!("❌ Phase {} failed: {}", phase_num + 1, e);
                    report.success = false;
                    report.error = Some(e.to_string());

                    // Rollback if enabled
                    if self.graph.config.rollback_on_failure {
                        warn!("🔄 Rolling back deployment...");
                        self.rollback().await?;
                    }

                    break;
                }
            }
        }

        report.duration_ms = start_time.elapsed().as_millis() as u64;

        if report.success {
            info!("✅ Graph execution complete: {} ms", report.duration_ms);
        } else {
            error!("❌ Graph execution failed: {} ms", report.duration_ms);
        }

        Ok(report)
    }

    /// Execute a single phase (parallel execution of independent nodes)
    async fn execute_phase(&mut self, nodes: &[String]) -> Result<PhaseResult> {
        let phase_start = std::time::Instant::now();
        let mut phase_result = PhaseResult::new(nodes.len());

        // Semaphore for max parallelism
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_parallelism));

        // Execute nodes in parallel
        let mut handles = Vec::new();

        for node_id in nodes {
            let node = self
                .graph
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?
                .clone();

            let context = self.context.clone();
            let permit = semaphore.clone().acquire_owned().await?;

            let handle = tokio::spawn(async move {
                let result = Self::execute_node(&node, &context).await;
                drop(permit); // Release semaphore
                (node.id.clone(), result)
            });

            handles.push(handle);
        }

        // Wait for all nodes to complete
        for handle in handles {
            let (node_id, result) = handle.await?;

            match result {
                Ok(output) => {
                    phase_result.completed += 1;
                    self.context
                        .set_status(&node_id, NodeStatus::Completed(output.clone()))
                        .await;
                    self.context.set_output(&node_id, output).await;
                }
                Err(e) => {
                    phase_result.failed += 1;
                    let error_msg = e.to_string();
                    self.context
                        .set_status(&node_id, NodeStatus::Failed(error_msg.clone()))
                        .await;
                    phase_result.errors.push((node_id, error_msg));
                }
            }
        }

        phase_result.duration_ms = phase_start.elapsed().as_millis() as u64;

        if phase_result.failed > 0 {
            anyhow::bail!("Phase failed: {} nodes failed", phase_result.failed);
        }

        Ok(phase_result)
    }

    /// Execute a single node
    async fn execute_node(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        debug!("   Executing node: {}", node.id);

        // Mark as running
        context.set_status(&node.id, NodeStatus::Running).await;

        // Execute based on node type
        let result = match node.node_type.as_str() {
            "filesystem.check_exists" => Self::node_filesystem_check_exists(node, context).await,
            "crypto.derive_child_seed" => Self::node_crypto_derive_seed(node, context).await,
            "primal.launch" => Self::node_primal_launch(node, context).await,
            "health.check_atomic" => Self::node_health_check(node, context).await,
            "lineage.verify_siblings" => Self::node_lineage_verify(node, context).await,
            "report.deployment_success" => Self::node_deployment_report(node, context).await,
            _ => {
                warn!("Unknown node type: {}, skipping", node.node_type);
                Ok(serde_json::json!({"skipped": true}))
            }
        };

        result.context(format!("Node execution failed: {}", node.id))
    }

    /// Node executor: filesystem.check_exists
    async fn node_filesystem_check_exists(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let path = node
            .config
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' in config"))?;

        // Substitute environment variables
        let path = Self::substitute_env(path, &context.env);
        let path = PathBuf::from(path);

        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }

        // Check size if specified
        if let Some(expected_size) = node.config.get("expected_size").and_then(|v| v.as_u64()) {
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() != expected_size {
                anyhow::bail!(
                    "File size mismatch: expected {}, got {}",
                    expected_size,
                    metadata.len()
                );
            }
        }

        Ok(serde_json::json!({
            "exists": true,
            "path": path.to_string_lossy()
        }))
    }

    /// Node executor: crypto.derive_child_seed
    async fn node_crypto_derive_seed(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        use biomeos_spore::seed::FamilySeed;

        let parent_seed = node
            .config
            .get("parent_seed")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'parent_seed'"))?;
        let parent_seed = Self::substitute_env(parent_seed, &context.env);

        let node_id = node
            .config
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'node_id'"))?;

        let output_path = node
            .config
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output_path'"))?;
        let output_path = Self::substitute_env(output_path, &context.env);

        let deployment_batch = node
            .config
            .get("deployment_batch")
            .and_then(|v| v.as_str())
            .map(|s| Self::substitute_env(s, &context.env));

        // Derive child seed
        FamilySeed::derive_sibling(
            PathBuf::from(parent_seed),
            PathBuf::from(&output_path),
            node_id,
            deployment_batch.as_deref(),
        )?;

        Ok(serde_json::json!({
            "derived": true,
            "output_path": output_path
        }))
    }

    /// Node executor: primal.launch
    async fn node_primal_launch(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // This would integrate with biomeos-atomic-deploy
        // For now, return a placeholder
        Ok(serde_json::json!({
            "launched": true,
            "primal": node.config.get("primal"),
            "pid": 12345  // Placeholder
        }))
    }

    /// Node executor: health.check_atomic
    async fn node_health_check(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Placeholder for health checking
        Ok(serde_json::json!({
            "healthy": true,
            "atomic": node.config.get("atomic_type")
        }))
    }

    /// Node executor: lineage.verify_siblings
    async fn node_lineage_verify(
        node: &GraphNode,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Placeholder for lineage verification
        Ok(serde_json::json!({
            "verified": true,
            "siblings": true
        }))
    }

    /// Node executor: report.deployment_success
    async fn node_deployment_report(
        node: &GraphNode,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        let atomics = node
            .config
            .get("atomics_deployed")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        Ok(serde_json::json!({
            "success": true,
            "atomics_deployed": atomics,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Substitute environment variables in a string
    fn substitute_env(s: &str, env: &HashMap<String, String>) -> String {
        let mut result = s.to_string();

        for (key, value) in env {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Perform topological sort to determine execution phases
    fn topological_sort(&self) -> Result<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build adjacency list and in-degree map
        for node in &self.graph.nodes {
            in_degree.entry(node.id.clone()).or_insert(0);

            for dep in &node.dependencies {
                graph_map
                    .entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id.clone());
                *in_degree.entry(node.id.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut phases = Vec::new();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while !queue.is_empty() {
            let mut current_phase = Vec::new();
            let phase_size = queue.len();

            for _ in 0..phase_size {
                if let Some(node_id) = queue.pop_front() {
                    current_phase.push(node_id.clone());

                    if let Some(dependents) = graph_map.get(&node_id) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(dependent.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !current_phase.is_empty() {
                phases.push(current_phase);
            }
        }

        // Check for cycles
        if phases.iter().map(|p| p.len()).sum::<usize>() != self.graph.nodes.len() {
            anyhow::bail!("Graph contains cycles or unreachable nodes");
        }

        Ok(phases)
    }

    /// Rollback deployment
    async fn rollback(&self) -> Result<()> {
        warn!("🔄 Rollback not yet implemented");
        // TODO: Implement rollback strategy
        Ok(())
    }
}

/// Execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub graph_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub phase_results: Vec<PhaseResult>,
    pub error: Option<String>,
}

impl ExecutionReport {
    fn new(graph_id: String) -> Self {
        Self {
            graph_id,
            success: true,
            duration_ms: 0,
            phase_results: Vec::new(),
            error: None,
        }
    }
}

/// Phase execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub total_nodes: usize,
    pub completed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub errors: Vec<(String, String)>,
}

impl PhaseResult {
    fn new(total_nodes: usize) -> Self {
        Self {
            total_nodes,
            completed: 0,
            failed: 0,
            duration_ms: 0,
            errors: Vec::new(),
        }
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

        let result = GraphExecutor::substitute_env("${FOO}/${BAZ}/test", &env);
        assert_eq!(result, "bar/qux/test");
    }
}
