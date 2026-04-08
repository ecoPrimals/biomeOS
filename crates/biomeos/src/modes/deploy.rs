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
}

fn load_graph(path: &std::path::Path) -> Result<LoadedGraphInfo> {
    if let Ok(dg) = GraphLoader::from_file(path) {
        debug!("Loaded as DeploymentGraph format");
        let node_summaries = dg
            .nodes_in_order()
            .into_iter()
            .map(|n| {
                let cap = n
                    .capability
                    .as_deref()
                    .unwrap_or("(no capability)")
                    .to_string();
                (n.id.to_string(), cap)
            })
            .collect();
        return Ok(LoadedGraphInfo {
            id: dg.id().to_string(),
            node_count: dg.nodes().len(),
            node_summaries,
        });
    }

    let ng = NeuralGraph::from_toml_file(path)
        .with_context(|| format!("Failed to load graph: {}", path.display()))?;
    debug!("Loaded as neural_graph format");
    let node_summaries = ng
        .nodes
        .iter()
        .map(|n| {
            let cap = n
                .capabilities
                .first()
                .cloned()
                .unwrap_or_else(|| "(no capability)".to_string());
            (n.id.clone(), cap)
        })
        .collect();
    Ok(LoadedGraphInfo {
        id: ng.id,
        node_count: ng.nodes.len(),
        node_summaries,
    })
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
