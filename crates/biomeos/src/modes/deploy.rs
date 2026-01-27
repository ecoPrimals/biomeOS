//! Deploy mode - Execute deployment graph
//!
//! Uses biomeos-graph for type-safe TOML graph loading and validation.

use anyhow::{Context, Result};
use biomeos_graph::GraphLoader;
use std::path::PathBuf;
use tracing::{info, warn};

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
