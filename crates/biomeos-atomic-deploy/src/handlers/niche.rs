// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
                "graph_id": "nucleus_simple",
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
                "graph_id": "ui_atomic",
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
                "graph_id": "livespore_create",
                "parameters": [
                    {"name": "SPORE_TARGET", "type": "path", "required": true},
                    {"name": "LINEAGE_MODE", "type": "enum", "values": ["genesis", "sibling"]}
                ]
            }),
            json!({
                "id": "gaming",
                "name": "Game Engine",
                "description": "Interactive game engine niche (ludoSpring + petalTongue + Tower)",
                "category": "gaming",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 4096,
                    "gpu_count": null,
                    "storage_gb": 2
                },
                "graph_id": "gaming_niche_deploy",
                "parameters": [
                    {"name": "RENDER_MODE", "type": "enum", "values": ["gui", "tui", "web", "headless"]},
                    {"name": "GPU_ACCELERATION", "type": "boolean"}
                ]
            }),
            json!({
                "id": "ludospring",
                "name": "Game Science",
                "description": "ludoSpring game science primal atop Node Atomic",
                "category": "science",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 1
                },
                "graph_id": "ludospring_deploy",
                "parameters": []
            }),
            json!({
                "id": "petaltongue",
                "name": "Visualization",
                "description": "petalTongue universal visualization primal",
                "category": "visualization",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 1
                },
                "graph_id": "petaltongue_deploy",
                "parameters": [
                    {"name": "RENDER_MODE", "type": "enum", "values": ["gui", "tui", "web", "headless"]}
                ]
            }),
            json!({
                "id": "game-engine-tick",
                "name": "Game Engine Tick Loop",
                "description": "60 Hz continuous game loop (input → logic → physics → scene → render)",
                "category": "continuous",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 4096,
                    "gpu_count": 1,
                    "storage_gb": 0
                },
                "graph_id": "game_engine_tick",
                "parameters": [
                    {"name": "TARGET_HZ", "type": "float", "default": 60.0},
                    {"name": "VSYNC", "type": "boolean", "default": true}
                ]
            }),
            json!({
                "id": "surgical-vr",
                "name": "Surgical VR Training",
                "description": "Immersive surgical simulation (healthSpring + petalTongue + ludoSpring)",
                "category": "medical",
                "required_resources": {
                    "cpu_cores": 8,
                    "memory_mb": 16384,
                    "gpu_count": 1,
                    "storage_gb": 20
                },
                "graph_id": "surgical_vr_deploy",
                "parameters": [
                    {"name": "PROCEDURE", "type": "string", "required": true},
                    {"name": "TRACKING_BACKEND", "type": "enum", "values": ["openxr", "steamvr", "custom"]},
                    {"name": "HAPTIC_ENABLED", "type": "boolean", "default": true}
                ]
            }),
            json!({
                "id": "ecology-pipeline",
                "name": "Cross-Spring Ecology",
                "description": "Multi-spring ecology pipeline (airSpring ET₀ → wetSpring diversity → neuralSpring spectral)",
                "category": "science",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 4096,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "cross_spring_ecology",
                "parameters": []
            }),
            json!({
                "id": "hotspring",
                "name": "Physics Simulation",
                "description": "hotSpring computational physics primal (MD, lattice QCD, transport)",
                "category": "science",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 8192,
                    "gpu_count": 1,
                    "storage_gb": 10
                },
                "graph_id": "hotspring_deploy",
                "parameters": []
            }),
            json!({
                "id": "groundspring",
                "name": "Measurement Science",
                "description": "groundSpring measurement and sensing primal (stats, FAO-56, seismic, ESN)",
                "category": "science",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "groundspring_deploy",
                "parameters": []
            }),
            json!({
                "id": "healthspring",
                "name": "Medical Science",
                "description": "healthSpring medical primal (PK/PD, biosignal, microbiome, NLME)",
                "category": "medical",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 4096,
                    "gpu_count": null,
                    "storage_gb": 10
                },
                "graph_id": "healthspring_deploy",
                "parameters": []
            }),
            json!({
                "id": "rootpulse",
                "name": "RootPulse",
                "description": "Emergent version control: rhizoCrypt (DAG) + LoamSpine (linear) + sweetGrass (attribution)",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 4096,
                    "gpu_count": null,
                    "storage_gb": 10
                },
                "graph_id": "rootpulse_commit",
                "parameters": [
                    { "name": "SESSION_ID", "required": true, "description": "rhizoCrypt session to commit" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for signing" }
                ]
            }),
            json!({
                "id": "provenance-pipeline",
                "name": "Provenance Pipeline",
                "description": "Universal provenance: any Spring experiment → permanent history + attribution",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "provenance_pipeline",
                "parameters": [
                    { "name": "SESSION_ID", "required": true, "description": "rhizoCrypt session to dehydrate" },
                    { "name": "EXPERIMENT_ID", "required": true, "description": "Experiment identifier" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for signing" }
                ]
            }),
            json!({
                "id": "rootpulse-branch",
                "name": "RootPulse Branch",
                "description": "Fork history at a commit point into a new spine",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "rootpulse_branch",
                "parameters": [
                    { "name": "PARENT_COMMIT_ID", "required": true, "description": "Commit to branch from" },
                    { "name": "BRANCH_NAME", "required": true, "description": "Name for the new branch" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for attribution" }
                ]
            }),
            json!({
                "id": "rootpulse-merge",
                "name": "RootPulse Merge",
                "description": "Merge a branch spine into a target spine",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "rootpulse_merge",
                "parameters": [
                    { "name": "SOURCE_SPINE_ID", "required": true, "description": "Branch spine to merge from" },
                    { "name": "TARGET_SPINE_ID", "required": true, "description": "Target spine to merge into" },
                    { "name": "SOURCE_SESSION_ID", "required": true, "description": "rhizoCrypt session for source" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for attribution" }
                ]
            }),
            json!({
                "id": "rootpulse-diff",
                "name": "RootPulse Diff",
                "description": "Compare two commits and produce a structured diff",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 1
                },
                "graph_id": "rootpulse_diff",
                "parameters": [
                    { "name": "COMMIT_A", "required": true, "description": "First commit to compare" },
                    { "name": "COMMIT_B", "required": true, "description": "Second commit to compare" },
                    { "name": "SESSION_A", "required": true, "description": "rhizoCrypt session for commit A" },
                    { "name": "SESSION_B", "required": true, "description": "rhizoCrypt session for commit B" }
                ]
            }),
            json!({
                "id": "rootpulse-federate",
                "name": "RootPulse Federate",
                "description": "Synchronize provenance across peer nodes via Songbird discovery",
                "category": "provenance",
                "required_resources": {
                    "cpu_cores": 2,
                    "memory_mb": 2048,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "rootpulse_federate",
                "parameters": [
                    { "name": "SPINE_ID", "required": true, "description": "Spine to synchronize" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for attribution" }
                ]
            }),
            json!({
                "id": "soil-microbiome",
                "name": "Cross-Spring Soil Microbiome",
                "description": "airSpring soil moisture → wetSpring microbial diversity → provenance",
                "category": "science",
                "required_resources": {
                    "cpu_cores": 4,
                    "memory_mb": 4096,
                    "gpu_count": null,
                    "storage_gb": 5
                },
                "graph_id": "cross_spring_soil_microbiome",
                "parameters": [
                    { "name": "EXPERIMENT_ID", "required": true, "description": "Experiment identifier" },
                    { "name": "AGENT_DID", "required": false, "description": "Agent DID for provenance" }
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
            "nucleus" => "nucleus_simple",
            "tower-atomic" => "tower_atomic_bootstrap",
            "ui-atomic" => "ui_atomic",
            "livespore" => "livespore_create",
            "gaming" => "gaming_niche_deploy",
            "ludospring" => "ludospring_deploy",
            "petaltongue" => "petaltongue_deploy",
            "game-engine-tick" => "game_engine_tick",
            "surgical-vr" => "surgical_vr_deploy",
            "ecology-pipeline" => "cross_spring_ecology",
            "hotspring" => "hotspring_deploy",
            "groundspring" => "groundspring_deploy",
            "healthspring" => "healthspring_deploy",
            "rootpulse" => "rootpulse_commit",
            "provenance-pipeline" => "provenance_pipeline",
            "rootpulse-branch" => "rootpulse_branch",
            "rootpulse-merge" => "rootpulse_merge",
            "rootpulse-diff" => "rootpulse_diff",
            "rootpulse-federate" => "rootpulse_federate",
            "soil-microbiome" => "cross_spring_soil_microbiome",
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
            "rootpulse",
            "provenance-pipeline",
            "rootpulse-branch",
            "rootpulse-merge",
            "rootpulse-diff",
            "rootpulse-federate",
            "soil-microbiome",
        ];
        for id in expected_ids {
            assert!(
                templates.iter().any(|t| t["id"] == id),
                "Template {} should be in list",
                id
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
            "Error should mention graph not found: {}",
            err_msg
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
}
