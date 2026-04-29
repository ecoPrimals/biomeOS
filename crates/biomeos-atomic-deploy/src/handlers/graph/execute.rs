// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Sequential graph execution and capability registration.

use super::{ExecutionStatus, GraphHandler};
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use biomeos_types::{SystemPaths, constants::files, defaults::DEFAULT_SOCKET_DIR};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

impl GraphHandler {
    /// Execute a graph.
    ///
    /// JSON-RPC method: `graph.execute`
    ///
    /// After execution, primals are registered by their CAPABILITIES, not hardcoded names.
    pub async fn execute(&self, raw_params: &Option<Value>) -> Result<Value> {
        let params = raw_params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;
        let family_id_param = params["family_id"].as_str().unwrap_or(&self.family_id);

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!(
                "Graph '{graph_id}' not found in nucleus ({}) or runtime ({})",
                self.graphs_dir.display(),
                self.runtime_graphs_dir.display()
            )
        })?;

        info!("🔍 Loading graph: {}", graph_id);
        debug!("   Graph path: {}", graph_path.display());

        let graph = Graph::from_toml_file(&graph_path)
            .with_context(|| format!("Failed to load graph from: {}", graph_path.display()))?;

        info!(
            "✅ Graph loaded: {} (version: {}, {} nodes, coordination: {})",
            graph.id,
            graph.version,
            graph.nodes.len(),
            graph.coordination.as_deref().unwrap_or("sequential"),
        );

        if graph.is_continuous() {
            info!("🔄 Graph is continuous — redirecting to start_continuous");
            return self.start_continuous(raw_params).await;
        }

        if graph.coordination.as_deref() == Some("pipeline") {
            info!("🔗 Graph is pipeline — redirecting to execute_pipeline");
            return self.execute_pipeline(raw_params).await;
        }

        self.load_translations_from_graph(&graph).await?;

        let execution_id = format!("{}-{}", graph_id, chrono::Utc::now().timestamp());
        let started_at = chrono::Utc::now().to_rfc3339();

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

        let executions = self.executions.clone();
        let execution_id_clone = execution_id.clone();
        let family_id_owned = family_id_param.to_string();
        let router = self.router.clone();

        tokio::spawn(async move {
            let mut env = HashMap::new();
            env.insert("FAMILY_ID".to_string(), family_id_owned.clone());
            env.insert(
                "UID".to_string(),
                rustix::process::getuid().as_raw().to_string(),
            );

            let socket_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_SOCKET_DIR")
                        .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.to_string())
                });
            env.insert("SOCKET_DIR".to_string(), socket_dir);
            env.insert(
                "JWT_SECRET".to_string(),
                std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                    tracing::warn!("JWT_SECRET not set — using family-derived fallback");
                    format!("biomeos-jwt-{}", family_id_owned)
                }),
            );

            let metrics_db_path = SystemPaths::new()
                .map(|p| p.data_dir().join(files::DEFAULT_NEURAL_METRICS_DB))
                .unwrap_or_else(|_| {
                    PathBuf::from(DEFAULT_SOCKET_DIR).join(files::DEFAULT_NEURAL_METRICS_DB)
                });
            let metrics = biomeos_graph::metrics::MetricsCollector::new(&metrics_db_path);

            for (k, v) in &graph.env {
                env.entry(k.clone()).or_insert_with(|| v.clone());
            }

            let capability_registry = {
                let config_path = std::path::PathBuf::from("config/capability_registry.toml");
                crate::capability_domains::CapabilityRegistry::from_toml(&config_path)
                    .unwrap_or_default()
            };

            // Clone needed: executor consumes graph, but post-execution
            // capability registration needs the node list.
            let graph_ref = graph.clone();

            let mut executor =
                GraphExecutor::new(graph, env).with_capability_registry(capability_registry);
            if let Ok(m) = metrics {
                executor = executor.with_metrics(m);
            }
            let start = std::time::Instant::now();

            match executor.execute().await {
                Ok(report) => {
                    if report.success {
                        Self::register_capabilities_from_graph(
                            &router,
                            &graph_ref,
                            &family_id_owned,
                        )
                        .await;
                    }

                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = if report.success {
                            "completed".to_string()
                        } else {
                            "failed".to_string()
                        };
                        exec_status.completed_nodes = report.completed_nodes;
                        exec_status.failed_nodes = report
                            .failed_nodes
                            .iter()
                            .map(|(id, _)| id.clone())
                            .collect();
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = report.error;
                    }
                }
                Err(e) => {
                    let mut status = executions.write().await;
                    if let Some(exec_status) = status.get_mut(&execution_id_clone) {
                        exec_status.state = "failed".to_string();
                        exec_status.duration_ms = start.elapsed().as_millis() as u64;
                        exec_status.error = Some(format!("Execution failed: {e}"));
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

    /// Register capabilities from a deployed graph by CAPABILITY, not hardcoded names.
    pub(crate) async fn register_capabilities_from_graph(
        router: &NeuralRouter,
        graph: &Graph,
        family_id: &str,
    ) {
        info!("📝 Registering capabilities from deployed graph...");

        for node in &graph.nodes {
            if node.capabilities.is_empty() {
                continue;
            }

            let primal_name = Self::resolve_primal_name(node);

            let runtime_dir = SystemPaths::new()
                .map(|p| p.runtime_dir().to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::env::var("BIOMEOS_RUNTIME_DIR")
                        .or_else(|_| std::env::var("TMPDIR"))
                        .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.to_string())
                });

            let socket_path = format!("{runtime_dir}/{primal_name}-{family_id}.sock");

            for capability in &node.capabilities {
                if let Err(e) = router
                    .register_capability_unix(
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

    /// Load capability translations from graph nodes.
    pub(super) async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        info!("📝 Loading capability translations from graph...");

        let mut registry = self.translation_registry.write().await;

        for node in &graph.nodes {
            for capability in &node.capabilities {
                let primal_name = Self::resolve_primal_name(node);
                let semantic_name = format!("{capability}.default");
                let method = format!("{capability}.invoke");

                let runtime_dir = SystemPaths::new()
                    .map(|p| p.runtime_dir().to_string_lossy().to_string())
                    .unwrap_or_else(|_| {
                        std::env::var("BIOMEOS_RUNTIME_DIR")
                            .or_else(|_| std::env::var("TMPDIR"))
                            .unwrap_or_else(|_| DEFAULT_SOCKET_DIR.to_string())
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
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use super::*;
    use crate::capability_translation::CapabilityTranslationRegistry;
    use crate::neural_graph::{Graph, GraphConfig, GraphNode};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn test_handler(graphs_dir: std::path::PathBuf) -> GraphHandler {
        GraphHandler::new(
            graphs_dir,
            "test-family",
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(NeuralRouter::new("test-family")),
            Arc::new(RwLock::new(CapabilityTranslationRegistry::new())),
        )
    }

    #[tokio::test]
    async fn execute_errors_on_missing_params() {
        let dir = tempfile::tempdir().unwrap();
        let handler = test_handler(dir.path().to_path_buf());
        let err = handler.execute(&None).await.unwrap_err();
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn execute_errors_on_missing_graph_id() {
        let dir = tempfile::tempdir().unwrap();
        let handler = test_handler(dir.path().to_path_buf());
        let err = handler.execute(&Some(json!({}))).await.unwrap_err();
        assert!(err.to_string().contains("graph_id"));
    }

    #[tokio::test]
    async fn execute_errors_when_graph_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let handler = test_handler(dir.path().to_path_buf());
        let err = handler
            .execute(&Some(json!({ "graph_id": "no_such_graph_xyz" })))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no_such_graph_xyz"));
    }

    #[tokio::test]
    async fn load_translations_from_graph_succeeds_with_empty_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let handler = test_handler(dir.path().to_path_buf());
        let graph = Graph {
            id: "g".into(),
            version: "1".into(),
            description: String::new(),
            nodes: vec![],
            config: GraphConfig::default(),
            coordination: None,
            env: HashMap::new(),
            genetics_tier: None,
        };
        handler.load_translations_from_graph(&graph).await.unwrap();
    }

    #[tokio::test]
    async fn register_capabilities_from_graph_skips_nodes_without_capabilities() {
        let router = Arc::new(NeuralRouter::new("fam"));
        let graph = Graph {
            id: "g".into(),
            version: "1".into(),
            description: String::new(),
            nodes: vec![GraphNode {
                id: "n1".into(),
                capabilities: vec![],
                ..Default::default()
            }],
            config: GraphConfig::default(),
            coordination: None,
            env: HashMap::new(),
            genetics_tier: None,
        };
        GraphHandler::register_capabilities_from_graph(&router, &graph, "fam").await;
    }
}
