// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! TOML loading and parsing for Neural API graphs.

use anyhow::Context;
use biomeos_graph::GeneticsTier;
use std::collections::HashMap;

use super::{Graph, GraphConfig, GraphNode};

impl Graph {
    /// Load graph from TOML file
    pub fn from_toml_file(path: &std::path::Path) -> anyhow::Result<Self> {
        tracing::debug!("📖 Reading graph file: {}", path.display());
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        tracing::debug!("   File size: {} bytes", contents.len());

        let mut graph = Self::from_toml_str(&contents)
            .with_context(|| format!("Failed to parse TOML from: {}", path.display()))?;

        // Derive ID from filename if parser defaulted to "unknown"
        if graph.id == "unknown" {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let derived = stem.to_lowercase().replace(' ', "-");
                if !derived.is_empty() {
                    graph.id = derived;
                    tracing::debug!(derived_id = %graph.id, "Derived neural graph ID from filename");
                }
            }
        }

        Ok(graph)
    }

    /// Load graph from TOML string
    pub fn from_toml_str(toml: &str) -> anyhow::Result<Self> {
        tracing::debug!("🔍 Parsing TOML structure...");

        // Parse TOML
        let value: toml::Value = toml::from_str(toml).context("Failed to parse TOML syntax")?;

        tracing::debug!("✅ TOML syntax valid");

        // Extract graph metadata
        tracing::debug!("🔍 Looking for [graph] section...");
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow::anyhow!("Missing [graph] section in TOML"))?;

        tracing::debug!("✅ Found [graph] section");

        let id = graph_table
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = graph_table
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let description = graph_table
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract nodes
        // Accept both [[nodes]] (neural_graph) and [[graph.nodes]] (DeploymentGraph) formats.
        // The DeploymentGraph format nests nodes under the [graph] section and uses
        // a different field schema (capability, budget_ms, feedback_to, config, params).
        // We convert DeploymentGraph nodes to the neural_graph schema on the fly.
        tracing::debug!("🔍 Looking for [[nodes]] or [[graph.nodes]] array...");

        let (nodes_array, from_deployment_graph) = if let Some(arr) =
            value.get("nodes").and_then(|v| v.as_array())
        {
            (arr.clone(), false)
        } else if let Some(arr) = graph_table.get("nodes").and_then(|v| v.as_array()) {
            tracing::debug!("   Found [[graph.nodes]] format — converting to neural_graph schema");
            (arr.clone(), true)
        } else {
            tracing::error!("❌ Missing [[nodes]] and [[graph.nodes]] arrays in TOML");
            anyhow::bail!(
                "Missing [[nodes]] or [[graph.nodes]] array. Found keys: {:?}",
                value.as_table().map(|t| t.keys().collect::<Vec<_>>())
            );
        };

        tracing::debug!(
            "✅ Found {} nodes (deployment_graph={})",
            nodes_array.len(),
            from_deployment_graph
        );

        let mut nodes = Vec::new();
        for (idx, node_value) in nodes_array.iter().enumerate() {
            tracing::debug!("   Parsing node {}...", idx);
            if from_deployment_graph {
                let node = Self::convert_deployment_node(node_value)
                    .with_context(|| format!("Failed to convert deployment node {idx}"))?;
                tracing::debug!("   ✅ Node {}: id={}", idx, node.id);
                nodes.push(node);
            } else {
                let node: GraphNode = toml::from_str(&toml::to_string(node_value)?)
                    .with_context(|| format!("Failed to parse node {idx} structure"))?;
                tracing::debug!("   ✅ Node {}: id={}", idx, node.id);
                nodes.push(node);
            }
        }

        tracing::info!("✅ Parsed {} nodes successfully", nodes.len());

        // Integrity check: warn if unsigned
        let integrity = biomeos_graph::integrity::verify_integrity(
            toml, None, // neural_graph doesn't have typed metadata fields for hash
            None, None,
        );
        tracing::debug!(
            graph_id = %id,
            content_hash = %integrity.computed_hash,
            "Graph integrity: content hash computed"
        );

        // Extract execution config
        let config = if let Some(exec_table) = value.get("execution").and_then(|v| v.as_table()) {
            GraphConfig {
                deterministic: exec_table
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "deterministic")
                    .unwrap_or(true),
                parallel_phases: exec_table
                    .get("parallel_phases")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                max_parallelism: exec_table
                    .get("max_parallelism")
                    .and_then(|v| v.as_integer())
                    .and_then(|v| usize::try_from(v).ok())
                    .unwrap_or(3),
                timeout_total_ms: exec_table
                    .get("timeout_total_ms")
                    .and_then(|v| v.as_integer())
                    .and_then(|v| u64::try_from(v).ok())
                    .unwrap_or(60000),
                checkpoint_enabled: exec_table
                    .get("checkpoint_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                rollback_on_failure: exec_table
                    .get("rollback_on_failure")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            }
        } else {
            GraphConfig::default()
        };

        let coordination = graph_table
            .get("coordination")
            .and_then(|v| v.as_str())
            .map(String::from);

        let env: HashMap<String, String> = graph_table
            .get("env")
            .or_else(|| graph_table.get("environment"))
            .and_then(|v| v.as_table())
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let metadata_table = graph_table.get("metadata");

        let genetics_tier = metadata_table
            .and_then(|m| m.get("genetics_tier"))
            .map(|v| {
                let s = v
                    .as_str()
                    .context("graph.metadata.genetics_tier must be a string")?;
                s.parse::<GeneticsTier>()
                    .context("Invalid graph.metadata.genetics_tier")
            })
            .transpose()?;

        let composition_model = metadata_table
            .and_then(|m| m.get("composition_model"))
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "nucleated" => Ok(biomeos_graph::CompositionModel::Nucleated),
                "membrane" => Ok(biomeos_graph::CompositionModel::Membrane),
                other => Err(anyhow::anyhow!(
                    "Unknown composition_model '{other}': expected 'nucleated' or 'membrane'"
                )),
            })
            .transpose()?;

        Ok(Self {
            id,
            version,
            description,
            nodes,
            config,
            coordination,
            env,
            genetics_tier,
            composition_model,
        })
    }
}
