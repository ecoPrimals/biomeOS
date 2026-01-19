//! Deploy mode - Execute deployment graph
//!
//! TODO: Integration with biomeos-atomic-deploy pending proper exports

use anyhow::{Context, Result};
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

    // Load graph
    let graph_content = std::fs::read_to_string(&graph)
        .with_context(|| format!("Failed to read graph: {}", graph.display()))?;

    // Parse as TOML
    let parsed: toml::Value = toml::from_str(&graph_content)
        .with_context(|| format!("Failed to parse graph: {}", graph.display()))?;

    // Count nodes
    let node_count = parsed
        .get("nodes")
        .and_then(|n| n.as_table())
        .map(|t| t.len())
        .unwrap_or(0);

    info!("✅ Graph loaded: {} nodes", node_count);

    if validate_only {
        info!("✅ Graph validation complete!");
        return Ok(());
    }

    if dry_run {
        info!("Would execute {} nodes", node_count);
        if let Some(nodes) = parsed.get("nodes").and_then(|n| n.as_table()) {
            for (name, _node) in nodes {
                info!("  • Node '{}'", name);
            }
        }
        return Ok(());
    }

    // TODO: Actual deployment execution requires biomeos-atomic-deploy refactoring
    info!("⚠️  Full deployment execution requires Neural API integration");
    info!("   Use: biomeos neural-api (in separate terminal)");
    info!("   Then: Send deployment request via JSON-RPC");

    Ok(())
}
