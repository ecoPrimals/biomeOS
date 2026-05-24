// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shadow deploy and graph integrity verification.

use super::GraphHandler;
use crate::neural_graph::Graph;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;

impl GraphHandler {
    /// Shadow deploy — dry-run validation for `composition.deploy`.
    ///
    /// Loads the graph, validates its structure (unique IDs, dependency existence,
    /// cycle detection, capability shape), computes the topological execution plan,
    /// and resolves each node's capability to a primal — all without spawning
    /// processes, registering capabilities, or performing any IPC.
    ///
    /// Designed for projectNUCLEUS H2 to pre-validate deployment configurations
    /// before committing to a live deploy.
    ///
    /// JSON-RPC method: `composition.deploy.shadow`
    ///
    /// # Parameters
    /// - `graph_id`: Graph identifier (required)
    ///
    /// # Returns
    /// ```json
    /// {
    ///   "valid": true,
    ///   "graph_id": "nucleus_complete",
    ///   "version": "1.0",
    ///   "node_count": 13,
    ///   "coordination": "sequential",
    ///   "phases": [["beardog"], ["songbird", "nestgate"], ["squirrel"]],
    ///   "capability_resolution": [
    ///     { "node": "beardog", "capabilities": ["security"], "resolved_provider": "beardog" }
    ///   ],
    ///   "integrity": { "content_hash": "abc...", "hash_match": true },
    ///   "validation_errors": [],
    ///   "warnings": []
    /// }
    /// ```
    pub async fn shadow_deploy(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!(
                "Graph '{graph_id}' not found in nucleus ({}) or runtime ({})",
                self.graphs_dir.display(),
                self.runtime_graphs_dir.display()
            )
        })?;

        let content = std::fs::read_to_string(&graph_path)
            .map_err(|e| anyhow::anyhow!("Failed to read graph: {e}"))?;

        let mut warnings: Vec<String> = Vec::new();
        let mut validation_errors: Vec<String> = Vec::new();

        // 1. Load as neural Graph for phase planning
        let graph = match Graph::from_toml_str(&content) {
            Ok(g) => g,
            Err(e) => {
                return Ok(json!({
                    "valid": false,
                    "graph_id": graph_id,
                    "validation_errors": [format!("Parse error: {e}")],
                    "warnings": [],
                }));
            }
        };

        // 2. Membrane composition: validate node domains (same gate as live deploy)
        if graph.composition_model == Some(biomeos_graph::CompositionModel::Membrane) {
            if let Err(e) = Self::validate_membrane_graph(&graph) {
                validation_errors.push(format!("Membrane: {e}"));
            }
        }

        // 3. Structural validation via DeploymentGraph + GraphValidator
        if let Ok(dg) = Self::load_as_deployment_graph(&content, graph_id) {
            use biomeos_graph::validation::GraphValidator as DGValidator;

            // Hard structural checks
            if let Err(e) = DGValidator::validate_unique_ids(&dg) {
                validation_errors.push(format!("Structural: {e}"));
            }
            if let Err(e) = DGValidator::validate_deps_exist(&dg) {
                validation_errors.push(format!("Structural: {e}"));
            }
            if let Err(e) = DGValidator::validate_acyclic(&dg) {
                validation_errors.push(format!("Structural: {e}"));
            }

            // Capability namespace format is a soft lint
            let mut validator = DGValidator::new();
            for ns in &[
                "content",
                "storage",
                "compute",
                "security",
                "discovery",
                "ai",
                "provenance",
                "dag",
                "attribution",
                "shader",
                "math",
            ] {
                validator.add_namespace(ns.to_string());
            }
            if let Err(e) = validator.validate_caps(&dg) {
                warnings.push(format!("Capability format: {e}"));
            }
        } else {
            warnings.push("Could not parse as DeploymentGraph for structural validation".into());
        }

        // 4. Topological sort (phase planning)
        let executor = crate::neural_executor::GraphExecutor::new(graph.clone(), HashMap::new());
        let phases = match executor.topological_sort() {
            Ok(p) => p,
            Err(e) => {
                validation_errors.push(format!("Dependency: {e}"));
                vec![]
            }
        };

        // 5. Capability resolution check
        let mut capability_resolution: Vec<Value> = Vec::new();
        for node in &graph.nodes {
            if !node.capabilities.is_empty() {
                let cap = &node.capabilities[0];
                let discovery = self.router.discover_capability(cap).await;
                let (resolved_name, resolvable) = match &discovery {
                    Ok(d) => {
                        let name = d.primals.first().map(|p| p.name.to_string());
                        (name, true)
                    }
                    Err(_) => (None, false),
                };
                capability_resolution.push(json!({
                    "node": node.id,
                    "capabilities": node.capabilities,
                    "resolved_provider": resolved_name,
                    "resolvable": resolvable,
                }));
            } else if node.primal.is_some() {
                capability_resolution.push(json!({
                    "node": node.id,
                    "primal": format!("{:?}", node.primal),
                    "capabilities": node.capabilities,
                }));
            }
        }

        for entry in &capability_resolution {
            if entry.get("resolvable") == Some(&json!(false)) {
                warnings.push(format!(
                    "Node '{}' capability '{}' has no registered provider",
                    entry["node"].as_str().unwrap_or("?"),
                    entry["capabilities"]
                        .as_array()
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .unwrap_or("?"),
                ));
            }
        }

        // 6. Integrity verification
        let integrity = {
            let (embedded_hash, embedded_sig, embedded_signer, genetics_tier) =
                if let Ok(lg) = biomeos_graph::GraphLoader::from_str(&content, Some(&graph_path)) {
                    (
                        lg.definition.metadata.content_hash.clone(),
                        lg.definition.metadata.signature.clone(),
                        lg.definition.metadata.signed_by.clone(),
                        lg.definition.metadata.genetics_tier,
                    )
                } else {
                    (None, None, None, None)
                };

            let report = biomeos_graph::verify_integrity(
                &content,
                embedded_hash.as_deref(),
                embedded_sig.as_deref(),
                embedded_signer.as_deref(),
            );

            json!({
                "content_hash": report.computed_hash,
                "hash_match": report.hash_match,
                "signature_valid": report.signature_valid,
                "genetics_tier": genetics_tier.map(|t| format!("{t:?}")),
                "acceptable": report.acceptable_for_tier(genetics_tier),
            })
        };

        let valid = validation_errors.is_empty();

        Ok(json!({
            "valid": valid,
            "graph_id": graph.id,
            "version": graph.version,
            "node_count": graph.nodes.len(),
            "coordination": graph.coordination.as_deref().unwrap_or("sequential"),
            "composition_model": graph.composition_model.as_ref().map(|m| m.to_string()),
            "phases": phases,
            "phase_count": phases.len(),
            "capability_resolution": capability_resolution,
            "integrity": integrity,
            "validation_errors": validation_errors,
            "warnings": warnings,
        }))
    }

    /// Verify graph integrity (content hash + signature check).
    ///
    /// Params: `{ "path": "/path/to/graph.toml" }` or `{ "id": "graph-id" }`
    pub async fn verify_graph(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let path = if let Some(p) = params.get("path").and_then(|v| v.as_str()) {
            PathBuf::from(p)
        } else if let Some(graph_id) = params.get("id").and_then(|v| v.as_str()) {
            self.resolve_graph_path(graph_id)
                .ok_or_else(|| anyhow::anyhow!("Graph not found: {graph_id}"))?
        } else {
            anyhow::bail!("graph.verify requires 'path' or 'id' parameter");
        };

        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read graph: {e}"))?;

        // Try typed parse for metadata
        let (embedded_hash, embedded_sig, embedded_signer, genetics_tier) =
            if let Ok(graph) = biomeos_graph::GraphLoader::from_str(&content, Some(&path)) {
                (
                    graph.definition.metadata.content_hash.clone(),
                    graph.definition.metadata.signature.clone(),
                    graph.definition.metadata.signed_by.clone(),
                    graph.definition.metadata.genetics_tier,
                )
            } else {
                (None, None, None, None)
            };

        let report = biomeos_graph::verify_integrity(
            &content,
            embedded_hash.as_deref(),
            embedded_sig.as_deref(),
            embedded_signer.as_deref(),
        );

        Ok(json!({
            "path": path.display().to_string(),
            "content_hash": report.computed_hash,
            "hash_match": report.hash_match,
            "signature_valid": report.signature_valid,
            "signer": report.signer,
            "genetics_tier": genetics_tier.map(|t| format!("{t:?}")),
            "acceptable": report.acceptable_for_tier(genetics_tier),
        }))
    }
}
