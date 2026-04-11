// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Deploy mode - Execute deployment graph
//!
//! Uses biomeos-graph for type-safe TOML graph loading and validation.

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_graph::Graph as NeuralGraph;
use biomeos_graph::GraphLoader;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Minimal valid graph TOML for testing
#[cfg(test)]
pub(crate) const MINIMAL_VALID_GRAPH: &str = r#"
[graph]
id = "test-deploy"
name = "Test Deployment"
version = "1.0.0"

[[graph.nodes]]
id = "node-a"
name = "Node A"
capability = "test.capability"
"#;

/// Minimal valid graph in neural_graph format for testing
#[cfg(test)]
pub(crate) const MINIMAL_NEURAL_GRAPH: &str = r#"
[graph]
id = "test-neural"
version = "1.0.0"
description = "Test neural-format graph"
coordination = "Sequential"

[[nodes]]
id = "node-a"
capabilities = ["test.capability"]

[nodes.operation]
name = "start"
"#;

struct LoadedGraphInfo {
    id: String,
    node_count: usize,
    node_summaries: Vec<(String, String)>,
    /// All capabilities provided by nodes in this graph.
    provided_capabilities: Vec<String>,
    /// All capabilities consumed/required by nodes in this graph.
    consumed_capabilities: Vec<String>,
}

fn load_graph(path: &std::path::Path) -> Result<LoadedGraphInfo> {
    if let Ok(dg) = GraphLoader::from_file(path) {
        debug!("Loaded as DeploymentGraph format");
        let mut provided = Vec::new();
        let mut consumed = Vec::new();
        let node_summaries = dg
            .nodes_in_order()
            .into_iter()
            .map(|n| {
                let cap = n
                    .capability
                    .as_deref()
                    .unwrap_or("(no capability)")
                    .to_string();
                if cap != "(no capability)" {
                    provided.push(cap.clone());
                }
                consumed.extend(n.operation_dependencies.iter().cloned());
                (n.id.to_string(), cap)
            })
            .collect();
        return Ok(LoadedGraphInfo {
            id: dg.id().to_string(),
            node_count: dg.nodes().len(),
            node_summaries,
            provided_capabilities: provided,
            consumed_capabilities: consumed,
        });
    }

    let ng = NeuralGraph::from_toml_file(path)
        .with_context(|| format!("Failed to load graph: {}", path.display()))?;
    debug!("Loaded as neural_graph format");
    let mut provided = Vec::new();
    let consumed = Vec::new();
    let node_summaries = ng
        .nodes
        .iter()
        .map(|n| {
            let cap = n
                .capabilities
                .first()
                .cloned()
                .unwrap_or_else(|| "(no capability)".to_string());
            provided.extend(n.capabilities.clone());
            (n.id.clone(), cap)
        })
        .collect();
    Ok(LoadedGraphInfo {
        id: ng.id,
        node_count: ng.nodes.len(),
        node_summaries,
        provided_capabilities: provided,
        consumed_capabilities: consumed,
    })
}

/// Check that all consumed capabilities declared by nodes are satisfiable by
/// other nodes in the graph (or known ecosystem providers). Logs warnings for
/// unsatisfied dependencies rather than failing hard, since external providers
/// may satisfy them at runtime.
fn validate_consumed_capabilities(info: &LoadedGraphInfo) {
    if info.consumed_capabilities.is_empty() {
        return;
    }
    let provided_set: std::collections::HashSet<&str> = info
        .provided_capabilities
        .iter()
        .map(String::as_str)
        .collect();

    let mut unsatisfied = Vec::new();
    for consumed in &info.consumed_capabilities {
        if !provided_set.contains(consumed.as_str()) {
            unsatisfied.push(consumed.as_str());
        }
    }

    if unsatisfied.is_empty() {
        info!(
            "✅ All {} consumed capabilities satisfied within graph",
            info.consumed_capabilities.len()
        );
    } else {
        warn!(
            "⚠️  {} consumed capabilities not provided by graph nodes (may be satisfied by external primals at runtime): {}",
            unsatisfied.len(),
            unsatisfied.join(", ")
        );
    }
}

pub async fn run(graph: PathBuf, validate_only: bool, dry_run: bool) -> Result<()> {
    info!("🚀 biomeOS Deploy Mode");
    info!("Graph: {}", graph.display());

    if !graph.exists() {
        anyhow::bail!("Graph file not found: {}", graph.display());
    }

    if validate_only {
        info!("🔍 Validation mode - graph will not be executed");
    } else if dry_run {
        warn!("🧪 Dry run mode - showing what would happen");
    }

    let loaded = load_graph(&graph)?;

    info!(
        "✅ Graph loaded and validated: {} ({} nodes)",
        loaded.id, loaded.node_count
    );

    validate_consumed_capabilities(&loaded);

    if validate_only {
        info!("✅ Graph validation complete!");
        return Ok(());
    }

    if dry_run {
        info!("Would execute {} nodes:", loaded.node_count);
        for (id, cap) in &loaded.node_summaries {
            info!("  • Node '{}' (capability: {})", id, cap);
        }
        return Ok(());
    }

    // Full deployment requires Neural API
    info!("⚠️  Full deployment execution requires Neural API integration");
    info!("   Use: biomeos neural-api (in separate terminal)");
    info!("   Then: Send deployment request via JSON-RPC");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_missing_graph_file() {
        let result = run(
            PathBuf::from("/nonexistent/path/to/graph.toml"),
            false,
            false,
        )
        .await;
        let err = result.expect_err("missing file should error");
        assert!(
            err.to_string().contains("not found") || err.to_string().contains("Graph file"),
            "error should mention missing file: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_validate_only() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("graph.toml");
        std::fs::write(&graph_path, MINIMAL_VALID_GRAPH).expect("write graph");

        let result = run(graph_path, true, false).await;
        result.expect("validate_only should succeed");
    }

    #[tokio::test]
    async fn test_run_dry_run() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("graph.toml");
        std::fs::write(&graph_path, MINIMAL_VALID_GRAPH).expect("write graph");

        let result = run(graph_path, false, true).await;
        result.expect("dry_run should succeed");
    }

    #[tokio::test]
    async fn test_run_invalid_graph() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("invalid.toml");
        std::fs::write(&graph_path, "not valid toml {{{").expect("write invalid");

        let result = run(graph_path, false, false).await;
        let err = result.expect_err("invalid graph should error");
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("Failed to load"),
            "error should mention parse/load: {err}"
        );
    }

    #[tokio::test]
    async fn test_run_full_execution() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("graph.toml");
        std::fs::write(&graph_path, MINIMAL_VALID_GRAPH).expect("write graph");

        let result = run(graph_path, false, false).await;
        result.expect("full run should succeed (prints message, no actual deploy)");
    }

    #[tokio::test]
    async fn test_run_neural_graph_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("neural.toml");
        std::fs::write(&graph_path, MINIMAL_NEURAL_GRAPH).expect("write graph");

        let result = run(graph_path, true, false).await;
        result.expect("neural_graph format should load via fallback");
    }

    #[tokio::test]
    async fn test_run_neural_graph_dry_run() {
        let dir = tempfile::tempdir().expect("temp dir");
        let graph_path = dir.path().join("neural.toml");
        std::fs::write(&graph_path, MINIMAL_NEURAL_GRAPH).expect("write graph");

        let result = run(graph_path, false, true).await;
        result.expect("neural_graph dry_run should succeed");
    }
}
