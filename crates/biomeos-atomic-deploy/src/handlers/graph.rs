// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph CRUD and execution handlers.
//!
//! This module handles all graph-related JSON-RPC methods:
//! - `graph.list` - List all available graphs
//! - `graph.get` - Get a specific graph
//! - `graph.save` - Save/update a graph
//! - `graph.execute` - Execute a graph
//! - `graph.status` - Get execution status
//!
//! # Capability-Based Design
//!
//! Graph execution uses capability-based primal discovery instead of hardcoded names.
//! Instead of mapping "security" → "beardog", we discover which primal provides "security".
//!
//! # XDG Compliance (EVOLVED Jan 27, 2026)
//!
//! Socket directory is determined via SystemPaths, not hardcoded.

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use biomeos_types::SystemPaths;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Execution status tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatus {
    /// Unique execution ID
    pub execution_id: String,
    /// Current state (running, completed, failed)
    pub state: String,
    /// Current phase index (if phased execution)
    pub current_phase: Option<usize>,
    /// Total number of phases
    pub total_phases: usize,
    /// Node IDs that completed successfully
    pub completed_nodes: Vec<String>,
    /// Node IDs that failed
    pub failed_nodes: Vec<String>,
    /// Elapsed time in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Graph handler for CRUD and execution operations.
#[derive(Clone)]
pub struct GraphHandler {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Active executions
    executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,

    /// Family ID
    family_id: String,

    /// Neural Router for capability discovery
    router: Arc<NeuralRouter>,

    /// Capability Translation Registry
    translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
}

impl GraphHandler {
    /// Create a new graph handler.
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Self {
        Self {
            graphs_dir: graphs_dir.into(),
            family_id: family_id.into(),
            executions,
            router,
            translation_registry,
        }
    }

    /// List all available graphs.
    ///
    /// JSON-RPC method: `graph.list`
    pub async fn list(&self) -> Result<Value> {
        let mut graphs = Vec::new();

        let entries =
            std::fs::read_dir(&self.graphs_dir).context("Failed to read graphs directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(graph) = Graph::from_toml_file(&path) {
                    graphs.push(json!({
                        "id": graph.id,
                        "version": graph.version,
                        "description": graph.description,
                        "node_count": graph.nodes.len(),
                        "estimated_time_ms": null,
                        "tags": []
                    }));
                }
            }
        }

        Ok(json!(graphs))
    }

    /// Get a specific graph.
    ///
    /// JSON-RPC method: `graph.get`
    pub async fn get(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph_id));
        let graph = Graph::from_toml_file(&graph_path).context("Failed to load graph")?;

        Ok(serde_json::to_value(graph)?)
    }

    /// Save a graph.
    ///
    /// JSON-RPC method: `graph.save`
    pub async fn save(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph: Graph =
            serde_json::from_value(params.clone()).context("Failed to parse graph")?;

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph.id));

        let toml_str =
            toml::to_string_pretty(&graph).context("Failed to serialize graph to TOML")?;

        std::fs::write(&graph_path, toml_str).context("Failed to write graph file")?;

        info!("💾 Saved graph: {} to {}", graph.id, graph_path.display());

        Ok(json!({"graph_id": graph.id}))
    }

    /// Execute a graph.
    ///
    /// JSON-RPC method: `graph.execute`
    ///
    /// # Capability-Based Registration
    ///
    /// After execution, primals are registered by their CAPABILITIES, not hardcoded names.
    /// This enables TRUE PRIMAL discovery - consumers ask for capabilities, not primal names.
    pub async fn execute(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;
        let family_id_param = params["family_id"].as_str().unwrap_or(&self.family_id);

        let graph_path = self.graphs_dir.join(format!("{}.toml", graph_id));

        info!("🔍 Loading graph: {}", graph_id);
        debug!("   Graph path: {}", graph_path.display());

        if !graph_path.exists() {
            error!("❌ Graph file not found: {}", graph_path.display());
            anyhow::bail!("Graph file not found: {}", graph_path.display());
        }

        let graph = Graph::from_toml_file(&graph_path)
            .with_context(|| format!("Failed to load graph from: {}", graph_path.display()))?;

        info!(
            "✅ Graph loaded: {} (version: {}, {} nodes)",
            graph.id,
            graph.version,
            graph.nodes.len()
        );

        // Load capability translations from graph
        self.load_translations_from_graph(&graph).await?;

        // Generate execution ID
        let execution_id = format!("{}-{}", graph_id, chrono::Utc::now().timestamp());
        let started_at = chrono::Utc::now().to_rfc3339();

        // Create execution status
        let status = ExecutionStatus {
            execution_id: execution_id.clone(),
            state: "running".to_string(),
            current_phase: Some(0),
            total_phases: graph.nodes.len(),
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            duration_ms: 0,
            error: None,
        };

        self.executions
            .write()
            .await
            .insert(execution_id.clone(), status);

        // Clone values for async move
        let executions = self.executions.clone();
        let execution_id_clone = execution_id.clone();
        let family_id_owned = family_id_param.to_string();
        let router = self.router.clone();

        // Execute graph in background
        tokio::spawn(async move {
            let mut env = HashMap::new();
            env.insert("FAMILY_ID".to_string(), family_id_owned.clone());
            env.insert("UID".to_string(), users::get_current_uid().to_string());

            // EVOLVED (Jan 27, 2026): XDG-compliant socket directory (no hardcoding!)
            let socket_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| "/tmp".to_string())
                });
            env.insert("SOCKET_DIR".to_string(), socket_dir);
            env.insert(
                "JWT_SECRET".to_string(),
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
            );

            let mut executor = GraphExecutor::new(graph.clone(), env);
            let start = std::time::Instant::now();

            match executor.execute().await {
                Ok(report) => {
                    if report.success {
                        // Register capabilities by CAPABILITY, not hardcoded names
                        Self::register_capabilities_from_graph(&router, &graph, &family_id_owned)
                            .await;
                    }

                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = if report.success {
                            "completed".to_string()
                        } else {
                            "failed".to_string()
                        };
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = report.error;
                    }
                }
                Err(e) => {
                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = "failed".to_string();
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = Some(format!("Execution failed: {}", e));
                    }
                }
            }
        });

        Ok(json!({
            "execution_id": execution_id,
            "graph_id": graph_id,
            "started_at": started_at
        }))
    }

    /// Register capabilities from a deployed graph.
    ///
    /// This is the TRUE PRIMAL way: register by capability, discover the primal name
    /// from the node configuration instead of hardcoding.
    async fn register_capabilities_from_graph(
        router: &NeuralRouter,
        graph: &Graph,
        family_id: &str,
    ) {
        info!("📝 Registering capabilities from deployed graph...");

        for node in &graph.nodes {
            if node.capabilities.is_empty() {
                continue;
            }

            // Get primal name from node config - NO HARDCODING
            let primal_name = Self::resolve_primal_name(node);

            // Determine socket path (XDG-compliant, no hardcoding!)
            let runtime_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_RUNTIME_DIR")
                        .or_else(|_| std::env::var("TMPDIR"))
                        .unwrap_or_else(|_| "/tmp".to_string())
                });

            let socket_path = format!("{}/{}-{}.sock", runtime_dir, primal_name, family_id);

            // Register each capability
            for capability in &node.capabilities {
                if let Err(e) = router
                    .register_capability(
                        capability,
                        &primal_name,
                        PathBuf::from(&socket_path),
                        "graph_deployment",
                    )
                    .await
                {
                    warn!("Failed to register capability {}: {}", capability, e);
                } else {
                    info!("   ✅ {} → {} @ {}", capability, primal_name, socket_path);
                }
            }
        }
    }

    /// Resolve primal name from node configuration.
    ///
    /// Order of precedence:
    /// 1. Explicit `primal.by_name` in node
    /// 2. Node ID (fallback)
    ///
    /// REMOVED: Hardcoded capability → primal mapping
    fn resolve_primal_name(node: &crate::neural_graph::GraphNode) -> String {
        node.primal
            .as_ref()
            .and_then(|p| p.by_name.clone())
            .unwrap_or_else(|| node.id.clone())
    }

    /// Load translations from graph.
    ///
    /// Translations are loaded from nodes that define capability mappings.
    async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        info!("📝 Loading capability translations from graph...");

        let mut registry = self.translation_registry.write().await;

        // Extract translations from node capabilities
        for node in &graph.nodes {
            for capability in &node.capabilities {
                // Register each capability as a potential translation target
                let primal_name = Self::resolve_primal_name(node);
                let semantic_name = format!("{}.default", capability);
                let method = format!("{}.invoke", capability);

                // Determine socket path (XDG-compliant, no hardcoding!)
                let runtime_dir = SystemPaths::new()
                    .map(|p| p.runtime_dir().to_string_lossy().to_string())
                    .unwrap_or_else(|_| {
                        std::env::var("BIOMEOS_RUNTIME_DIR")
                            .or_else(|_| std::env::var("TMPDIR"))
                            .unwrap_or_else(|_| "/tmp".to_string())
                    });
                let socket_path =
                    format!("{}/{}-{}.sock", runtime_dir, primal_name, self.family_id);

                debug!(
                    "   Registering: {} → {}:{} @ {}",
                    semantic_name, primal_name, method, socket_path
                );

                registry.register_translation(
                    &semantic_name,
                    &primal_name,
                    &method,
                    &socket_path,
                    None,
                );
            }
        }

        info!("✅ Capability translations loaded");
        Ok(())
    }

    /// Get execution status.
    ///
    /// JSON-RPC method: `graph.status`
    pub async fn get_status(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let execution_id = params["execution_id"]
            .as_str()
            .context("Missing execution_id")?;

        let executions = self.executions.read().await;
        let status = executions
            .get(execution_id)
            .context("Execution not found")?;

        Ok(serde_json::to_value(status)?)
    }
}

#[cfg(test)]
/// Helper to create a minimal GraphHandler for tests.
fn make_handler(
    graphs_dir: &std::path::Path,
) -> (GraphHandler, Arc<RwLock<HashMap<String, ExecutionStatus>>>) {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    let executions = Arc::new(RwLock::new(HashMap::new()));
    let handler = GraphHandler::new(
        graphs_dir,
        "test-family",
        executions.clone(),
        router,
        registry,
    );
    (handler, executions)
}

#[cfg(test)]
/// Minimal valid graph TOML for execute tests (log.info completes quickly).
const MINIMAL_GRAPH_TOML: &str = r#"
[graph]
id = "test_minimal"
version = "1.0.0"
description = "Minimal graph for tests"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "test execution"
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── ExecutionStatus tests ─────────────────────────────────────────────

    #[test]
    fn test_execution_status_construction() {
        let status = ExecutionStatus {
            execution_id: "graph-123".to_string(),
            state: "running".to_string(),
            current_phase: Some(1),
            total_phases: 3,
            completed_nodes: vec!["node1".to_string()],
            failed_nodes: Vec::new(),
            duration_ms: 100,
            error: None,
        };
        assert_eq!(status.execution_id, "graph-123");
        assert_eq!(status.state, "running");
        assert_eq!(status.current_phase, Some(1));
        assert_eq!(status.total_phases, 3);
        assert_eq!(status.completed_nodes, vec!["node1"]);
        assert!(status.failed_nodes.is_empty());
        assert_eq!(status.duration_ms, 100);
        assert!(status.error.is_none());
    }

    #[test]
    fn test_execution_status_with_error() {
        let status = ExecutionStatus {
            execution_id: "graph-456".to_string(),
            state: "failed".to_string(),
            current_phase: Some(2),
            total_phases: 3,
            completed_nodes: vec!["node1".to_string(), "node2".to_string()],
            failed_nodes: vec!["node3".to_string()],
            duration_ms: 500,
            error: Some("Node execution failed".to_string()),
        };
        assert_eq!(status.state, "failed");
        assert_eq!(status.failed_nodes, vec!["node3"]);
        assert_eq!(status.error.as_deref(), Some("Node execution failed"));
    }

    #[test]
    fn test_execution_status_serialization_roundtrip() {
        let status = ExecutionStatus {
            execution_id: "exec-789".to_string(),
            state: "completed".to_string(),
            current_phase: Some(3),
            total_phases: 3,
            completed_nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            failed_nodes: Vec::new(),
            duration_ms: 1234,
            error: None,
        };
        let json = serde_json::to_value(&status).expect("serialize");
        let restored: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
        assert_eq!(restored.execution_id, status.execution_id);
        assert_eq!(restored.state, status.state);
        assert_eq!(restored.completed_nodes, status.completed_nodes);
        assert_eq!(restored.duration_ms, status.duration_ms);
    }

    #[test]
    fn test_execution_status_deserialize_from_json() {
        let json = json!({
            "execution_id": "test-123",
            "state": "running",
            "current_phase": 0,
            "total_phases": 2,
            "completed_nodes": [],
            "failed_nodes": [],
            "duration_ms": 0,
            "error": null
        });
        let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
        assert_eq!(status.execution_id, "test-123");
        assert_eq!(status.state, "running");
        assert_eq!(status.current_phase, Some(0));
        assert_eq!(status.total_phases, 2);
    }

    #[test]
    fn test_execution_status_deserialize_with_optional_error() {
        let json = json!({
            "execution_id": "fail-1",
            "state": "failed",
            "current_phase": null,
            "total_phases": 1,
            "completed_nodes": [],
            "failed_nodes": ["n1"],
            "duration_ms": 50,
            "error": "Something went wrong"
        });
        let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
        assert_eq!(status.error, Some("Something went wrong".to_string()));
        assert_eq!(status.current_phase, None);
    }

    // ── GraphHandler constructor ───────────────────────────────────────────

    #[tokio::test]
    async fn test_graph_handler_creation() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());
        let result = handler.list().await.expect("list");
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_graph_handler_clone() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());
        let cloned = handler.clone();
        let result = cloned.list().await.expect("list");
        assert!(result.is_array());
    }

    // ── graph.list ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_graph_handler_list_empty() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());
        let result = handler.list().await.expect("list");
        assert!(result.as_array().expect("array").is_empty());
    }

    #[tokio::test]
    async fn test_list_with_valid_graphs() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("test_graph.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write graph");
        let (handler, _) = make_handler(temp.path());

        let result = handler.list().await.expect("list");
        let arr = result.as_array().expect("array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "test_minimal");
        assert_eq!(arr[0]["version"], "1.0.0");
        assert_eq!(arr[0]["node_count"], 1);
    }

    #[tokio::test]
    async fn test_list_skips_invalid_toml() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("invalid.toml");
        std::fs::write(&path, "not valid toml {{{").expect("write");
        let (handler, _) = make_handler(temp.path());

        let result = handler.list().await.expect("list");
        let arr = result.as_array().expect("array");
        assert!(arr.is_empty(), "invalid TOML should be skipped");
    }

    #[tokio::test]
    async fn test_list_skips_non_toml_files() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("other.txt");
        std::fs::write(&path, "hello").expect("write");
        let (handler, _) = make_handler(temp.path());

        let result = handler.list().await.expect("list");
        assert!(result.as_array().expect("array").is_empty());
    }

    #[tokio::test]
    async fn test_list_nonexistent_directory() {
        let temp = tempdir().expect("tempdir");
        let bad_path = temp.path().join("nonexistent_subdir");
        let (handler, _) = make_handler(&bad_path);

        let err = handler.list().await.expect_err("should fail");
        assert!(err.to_string().contains("Failed to read graphs directory"));
    }

    // ── graph.get ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_success() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("my_graph.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "my_graph"}));
        let result = handler.get(&params).await.expect("get");
        assert_eq!(result["id"], "test_minimal");
        assert_eq!(result["version"], "1.0.0");
        assert!(result["nodes"].is_array());
    }

    #[tokio::test]
    async fn test_get_missing_params() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let err = handler.get(&None).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_get_missing_graph_id() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({}));
        let err = handler.get(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing graph_id"));
    }

    #[tokio::test]
    async fn test_get_graph_not_found() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "nonexistent_graph"}));
        let err = handler.get(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Failed to load graph"));
    }

    #[tokio::test]
    async fn test_get_graph_id_as_number_fails() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": 12345}));
        let err = handler.get(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing graph_id"));
    }

    // ── graph.save ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_save_success() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let graph_value = json!({
            "id": "saved_graph",
            "version": "1.0.0",
            "description": "Saved for test",
            "nodes": [],
            "config": {
                "deterministic": true,
                "parallel_phases": true,
                "max_parallelism": 3,
                "timeout_total_ms": 60000,
                "checkpoint_enabled": false,
                "rollback_on_failure": true
            }
        });
        let params = Some(graph_value);

        let result = handler.save(&params).await.expect("save");
        assert_eq!(result["graph_id"], "saved_graph");

        let path = temp.path().join("saved_graph.toml");
        assert!(path.exists(), "graph file should exist");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("saved_graph"));
    }

    #[tokio::test]
    async fn test_save_missing_params() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let err = handler.save(&None).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_save_invalid_graph_structure() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({
            "id": "bad",
            "version": "1.0",
            "nodes": "not an array"
        }));
        let err = handler.save(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Failed to parse graph"));
    }

    #[tokio::test]
    async fn test_save_overwrites_existing() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("overwrite.toml");
        std::fs::write(&path, "old content").expect("write");
        let (handler, _) = make_handler(temp.path());

        let graph_value = json!({
            "id": "overwrite",
            "version": "2.0.0",
            "description": "Updated",
            "nodes": [],
            "config": {
                "deterministic": true,
                "parallel_phases": true,
                "max_parallelism": 3,
                "timeout_total_ms": 60000,
                "checkpoint_enabled": false,
                "rollback_on_failure": true
            }
        });
        handler.save(&Some(graph_value)).await.expect("save");

        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("2.0.0"));
        assert!(content.contains("Updated"));
    }

    // ── graph.status ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_status_success() {
        let temp = tempdir().expect("tempdir");
        let (handler, executions) = make_handler(temp.path());

        let status = ExecutionStatus {
            execution_id: "exec-123".to_string(),
            state: "running".to_string(),
            current_phase: Some(1),
            total_phases: 3,
            completed_nodes: vec!["n1".to_string()],
            failed_nodes: Vec::new(),
            duration_ms: 100,
            error: None,
        };
        executions
            .write()
            .await
            .insert("exec-123".to_string(), status);

        let params = Some(json!({"execution_id": "exec-123"}));
        let result = handler.get_status(&params).await.expect("get_status");
        assert_eq!(result["execution_id"], "exec-123");
        assert_eq!(result["state"], "running");
        assert_eq!(result["current_phase"], 1);
        assert_eq!(result["total_phases"], 3);
        assert_eq!(result["completed_nodes"], json!(["n1"]));
    }

    #[tokio::test]
    async fn test_get_status_missing_params() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let err = handler.get_status(&None).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_get_status_missing_execution_id() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({}));
        let err = handler.get_status(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing execution_id"));
    }

    #[tokio::test]
    async fn test_get_status_execution_not_found() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"execution_id": "nonexistent-exec"}));
        let err = handler.get_status(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Execution not found"));
    }

    #[tokio::test]
    async fn test_get_status_completed_with_error_field() {
        let temp = tempdir().expect("tempdir");
        let (handler, executions) = make_handler(temp.path());

        let status = ExecutionStatus {
            execution_id: "exec-failed".to_string(),
            state: "failed".to_string(),
            current_phase: Some(2),
            total_phases: 3,
            completed_nodes: vec!["a".to_string(), "b".to_string()],
            failed_nodes: vec!["c".to_string()],
            duration_ms: 500,
            error: Some("Node c failed".to_string()),
        };
        executions
            .write()
            .await
            .insert("exec-failed".to_string(), status);

        let params = Some(json!({"execution_id": "exec-failed"}));
        let result = handler.get_status(&params).await.expect("get_status");
        assert_eq!(result["state"], "failed");
        assert_eq!(result["error"], "Node c failed");
    }

    // ── graph.execute ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_execute_missing_params() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let err = handler.execute(&None).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_execute_missing_graph_id() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({}));
        let err = handler.execute(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing graph_id"));
    }

    #[tokio::test]
    async fn test_execute_graph_not_found() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "nonexistent"}));
        let err = handler.execute(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Graph file not found"));
    }

    #[tokio::test]
    async fn test_execute_success_returns_immediate_response() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("test_minimal.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
        let (handler, executions) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "test_minimal"}));
        let result = handler.execute(&params).await.expect("execute");

        assert!(result["execution_id"]
            .as_str()
            .unwrap()
            .starts_with("test_minimal-"));
        assert_eq!(result["graph_id"], "test_minimal");
        assert!(result["started_at"].as_str().is_some());

        let exec_id = result["execution_id"].as_str().expect("execution_id");
        let execs = executions.read().await;
        assert!(execs.contains_key(exec_id));
        let status = execs.get(exec_id).expect("status");
        assert_eq!(status.state, "running");
    }

    #[tokio::test]
    async fn test_execute_with_family_id_param() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("test_minimal.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({
            "graph_id": "test_minimal",
            "family_id": "custom-family"
        }));
        let result = handler.execute(&params).await.expect("execute");
        assert_eq!(result["graph_id"], "test_minimal");
        assert!(result["execution_id"]
            .as_str()
            .unwrap()
            .starts_with("test_minimal-"));
    }

    #[tokio::test]
    async fn test_execute_uses_handler_family_id_when_param_missing() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("test_minimal.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "test_minimal"}));
        let result = handler.execute(&params).await.expect("execute");
        assert!(result["execution_id"].as_str().is_some());
    }

    // ── JSON-RPC style params ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_with_json_rpc_params_object() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("rpc_graph.toml");
        std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"graph_id": "rpc_graph"}));
        let result = handler.get(&params).await.expect("get");
        assert_eq!(result["id"], "test_minimal");
    }

    #[tokio::test]
    async fn test_save_persists_file() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let graph_value = json!({
            "id": "roundtrip_graph",
            "version": "1.0.0",
            "description": "Roundtrip test",
            "nodes": [{
                "id": "node1",
                "depends_on": [],
                "capabilities": []
            }],
            "config": {
                "deterministic": true,
                "parallel_phases": true,
                "max_parallelism": 3,
                "timeout_total_ms": 60000,
                "checkpoint_enabled": false,
                "rollback_on_failure": true
            }
        });
        handler.save(&Some(graph_value)).await.expect("save");

        let path = temp.path().join("roundtrip_graph.toml");
        assert!(path.exists(), "saved graph file should exist");
        let content = std::fs::read_to_string(&path).expect("read file");
        assert!(content.contains("roundtrip_graph"));
        assert!(content.contains("1.0.0"));
        assert!(content.contains("node1"));
    }

    // ── Edge cases ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_list_empty_graphs_dir_created() {
        let temp = tempdir().expect("tempdir");
        let empty = temp.path().join("empty");
        std::fs::create_dir(&empty).expect("create dir");
        let (handler, _) = make_handler(&empty);

        let result = handler.list().await.expect("list");
        assert!(result.as_array().expect("array").is_empty());
    }

    #[tokio::test]
    async fn test_get_status_with_non_string_execution_id() {
        let temp = tempdir().expect("tempdir");
        let (handler, _) = make_handler(temp.path());

        let params = Some(json!({"execution_id": 12345}));
        let err = handler.get_status(&params).await.expect_err("should fail");
        assert!(err.to_string().contains("Missing execution_id"));
    }
}
