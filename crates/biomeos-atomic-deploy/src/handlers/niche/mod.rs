// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
//! The template catalog lives in [`catalog`] as a single source of truth —
//! both `list` and `deploy` derive from the same data.

pub mod catalog;

use crate::neural_graph::Graph;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use serde_json::{Value, json};
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
        Ok(catalog::templates_json())
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

        let graph_id = catalog::resolve_graph_id(template_id)
            .with_context(|| format!("Unknown template: {template_id}"))?;

        let graph_path = self.graphs_dir.join(format!("{graph_id}.toml"));
        if !graph_path.exists() {
            anyhow::bail!("Graph not found: {graph_id}");
        }

        let graph = Graph::from_toml_file(&graph_path)?;

        let execution_id = format!("niche-{}-{}", template_id, chrono::Utc::now().timestamp());

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

        let graph_params = json!({
            "graph_id": graph_id,
            "family_id": self.family_id
        });

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
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use biomeos_types::primal_names;
    use tempfile::TempDir;

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

    #[tokio::test]
    async fn test_niche_list_all_templates() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.list().await.unwrap();
        let templates = result.as_array().unwrap();

        let expected_ids = [
            "nucleus",
            "tower-atomic",
            "ui-atomic",
            "livespore",
            "gaming",
            "ludospring",
            "petaltongue",
            "game-engine-tick",
            "surgical-vr",
            "ecology-pipeline",
            "hotspring",
            "groundspring",
            "healthspring",
            primal_names::ROOTPULSE,
            "provenance-pipeline",
            "rootpulse-branch",
            "rootpulse-merge",
            "rootpulse-diff",
            "rootpulse-federate",
            "soil-microbiome",
            "airspring",
            "wetspring",
            "neuralspring",
        ];
        for id in expected_ids {
            assert!(
                templates.iter().any(|t| t["id"] == id),
                "Template {id} should be in list"
            );
        }
    }

    #[tokio::test]
    async fn test_niche_list_template_structure() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.list().await.unwrap();
        let templates = result.as_array().unwrap();

        let nucleus = templates
            .iter()
            .find(|t| t["id"] == "nucleus")
            .expect("nucleus template");
        assert_eq!(nucleus["name"], "NUCLEUS");
        assert!(nucleus["description"].as_str().unwrap().contains("biomeOS"));
        assert!(nucleus["required_resources"].get("cpu_cores").is_some());
        assert_eq!(nucleus["graph_id"], "nucleus_simple");
    }

    #[tokio::test]
    async fn test_niche_list_livespore_parameters() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.list().await.unwrap();
        let templates = result.as_array().unwrap();
        let livespore = templates
            .iter()
            .find(|t| t["id"] == "livespore")
            .expect("livespore template");

        let params = livespore["parameters"].as_array().unwrap();
        assert!(!params.is_empty());
        assert!(params.iter().any(|p| p["name"] == "SPORE_TARGET"));
    }

    #[tokio::test]
    async fn test_niche_deploy_missing_params() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.deploy(&None).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("missing"),
            "Error should mention missing params"
        );
    }

    #[tokio::test]
    async fn test_niche_deploy_missing_template_id() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let params = Some(json!({}));
        let result = handler.deploy(&params).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("template_id"),
            "Error should mention template_id"
        );
    }

    #[tokio::test]
    async fn test_niche_deploy_unknown_template() {
        let temp_dir = TempDir::new().expect("temp dir");
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new(temp_dir.path(), "test-family", router, executions);

        let params = Some(json!({ "template_id": "unknown-template-xyz" }));
        let result = handler.deploy(&params).await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("Unknown template"),
            "Error should mention unknown template"
        );
    }

    #[tokio::test]
    async fn test_niche_deploy_graph_not_found() {
        let temp_dir = TempDir::new().expect("temp dir");
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new(temp_dir.path(), "test-family", router, executions);

        let params = Some(json!({ "template_id": "nucleus" }));
        let result = handler.deploy(&params).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.to_lowercase().contains("not found") || err_msg.contains("nucleus_simple"),
            "Error should mention graph not found: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_niche_handler_new() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new(
            PathBuf::from("/graphs"),
            "my-family",
            router.clone(),
            executions.clone(),
        );

        let result = handler.list().await.unwrap();
        assert!(!result.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_niche_list_serialization_roundtrip() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let handler = NicheHandler::new("/tmp", "test-family", router, executions);

        let result = handler.list().await.unwrap();
        let json_str = serde_json::to_string(&result).expect("serialize");
        let parsed: Value = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(result, parsed);
    }

    #[tokio::test]
    async fn test_catalog_resolve_all_templates() {
        for template in catalog::BUILTIN_TEMPLATES {
            assert!(
                catalog::resolve_graph_id(template.id).is_some(),
                "Template {} should resolve to a graph_id",
                template.id
            );
        }
    }

    #[tokio::test]
    async fn test_catalog_json_matches_count() {
        let json = catalog::templates_json();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), catalog::BUILTIN_TEMPLATES.len());
    }
}
