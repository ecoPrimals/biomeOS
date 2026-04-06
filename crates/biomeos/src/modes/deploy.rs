// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Deploy mode - Execute deployment graph
//!
//! Uses biomeos-graph for type-safe TOML graph loading and validation.

use anyhow::{Context, Result};
use biomeos_graph::GraphLoader;
use std::path::PathBuf;
use tracing::{info, warn};

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

    // Load graph with type-safe parsing (includes validation)
    let deployment_graph = GraphLoader::from_file(&graph)
        .with_context(|| format!("Failed to load graph: {}", graph.display()))?;

    info!(
        "✅ Graph loaded and validated: {} ({} nodes)",
        deployment_graph.id(),
        deployment_graph.nodes().len()
    );

    if validate_only {
        info!("✅ Graph validation complete!");
        return Ok(());
    }

    if dry_run {
        info!("Would execute {} nodes:", deployment_graph.nodes().len());
        for node in deployment_graph.nodes_in_order() {
            let cap = node.capability.as_deref().unwrap_or("(no capability)");
            info!("  • Node '{}' (capability: {})", node.id, cap);
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
}
