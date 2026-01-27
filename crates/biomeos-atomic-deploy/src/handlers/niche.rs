//! Niche template deployment handlers.
//!
//! This module handles niche-related JSON-RPC methods:
//! - `niche.list` - List available niche templates
//! - `niche.deploy` - Deploy a niche from template
//!
//! # Architecture
//!
//! Niches are pre-configured deployments that bundle multiple primals
//! and graphs for specific use cases (NUCLEUS, UI Atomic, etc.).

use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Niche handler for template deployment.
#[derive(Clone)]
pub struct NicheHandler {
    /// Path to graphs directory
    graphs_dir: PathBuf,

    /// Family ID
    family_id: String,

    /// Neural Router for capability registration
    router: Arc<NeuralRouter>,

    /// Active executions
    executions: Arc<RwLock<HashMap<String, super::graph::ExecutionStatus>>>,
}

impl NicheHandler {
    /// Create a new niche handler.
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        router: Arc<NeuralRouter>,
        executions: Arc<RwLock<HashMap<String, super::graph::ExecutionStatus>>>,
    ) -> Self {
        Self {
            graphs_dir: graphs_dir.into(),
            family_id: family_id.into(),
            router,
            executions,
        }
    }

    /// List available niche templates.
    ///
    /// JSON-RPC method: `niche.list`
    pub async fn list(&self) -> Result<Value> {
        // Built-in templates
        let templates = vec![
            json!({
                "id": "nucleus",
                "name": "NUCLEUS",
                "description": "Complete biomeOS infrastructure (Tower + Node + Nest)",
                "category": "infrastructure",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 8192,
                    "gpu_count": null,
                    "storage_gb": 50
                },
                "graph_id": "nucleus-simple",
                "parameters": []
            }),
            json!({
                "id": "tower-atomic",
                "name": "Tower Atomic",
                "description": "Security + Discovery atomic pair (BearDog + Songbird)",
                "category": "infrastructure",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "tower_atomic_bootstrap",
                "parameters": []
            }),
            json!({
                "id": "ui-atomic",
                "name": "UI Atomic",
                "description": "User interface and AI layer (Squirrel + petalTongue)",
                "category": "user-interface",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 4096,
                    "gpu_count": 1,
                    "storage_gb": 10
                },
                "graph_id": "ui-atomic",
                "parameters": []
            }),
            json!({
                "id": "livespore",
                "name": "LiveSpore",
                "description": "Portable deployment on removable media",
                "category": "deployment",
                "required_resources": {
                    "cpu_cores": 1,
                    "memory_mb": 512,
                    "gpu_count": null,
                    "storage_gb": 1
                },
                "graph_id": "livespore-create",
                "parameters": [
                    {"name": "SPORE_TARGET", "type": "path", "required": true},
                    {"name": "LINEAGE_MODE", "type": "enum", "values": ["genesis", "sibling"]}
                ]
            }),
        ];

        Ok(json!(templates))
    }

    /// Deploy a niche from template.
    ///
    /// JSON-RPC method: `niche.deploy`
    pub async fn deploy(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let template_id = params["template_id"]
            .as_str()
            .context("Missing template_id")?;

        info!("🚀 Deploying niche: {}", template_id);

        // Map template to graph
        let graph_id = match template_id {
            "nucleus" => "nucleus-simple",
            "tower-atomic" => "tower_atomic_bootstrap",
            "ui-atomic" => "ui-atomic",
            "livespore" => "livespore-create",
            _ => anyhow::bail!("Unknown template: {}", template_id),
        };

        // Load the graph
        let graph_path = self.graphs_dir.join(format!("{}.toml", graph_id));
        if !graph_path.exists() {
            anyhow::bail!("Graph not found: {}", graph_id);
        }

        let graph = Graph::from_toml_file(&graph_path)?;

        // Generate execution ID
        let execution_id = format!("niche-{}-{}", template_id, chrono::Utc::now().timestamp());

        // Store execution status
        let status = super::graph::ExecutionStatus {
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

        // Execute via graph.execute (reuse existing logic)
        let graph_params = json!({
            "graph_id": graph_id,
            "family_id": self.family_id
        });

        // Create graph handler for execution
        let graph_handler = super::graph::GraphHandler::new(
            self.graphs_dir.clone(),
            self.family_id.clone(),
            self.executions.clone(),
            self.router.clone(),
            Arc::new(RwLock::new(
                crate::capability_translation::CapabilityTranslationRegistry::new(),
            )),
        );

        graph_handler.execute(&Some(graph_params)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_niche_list() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));

        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.list().await.unwrap();
        let templates = result.as_array().unwrap();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t["id"] == "nucleus"));
    }
}
