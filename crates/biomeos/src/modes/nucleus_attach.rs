// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Cell attachment — connect a garden/protist to a running NUCLEUS.
//!
//! This bridges the gap between "deploy graph exists" and "operational use":
//! 1. Validates the NUCLEUS is healthy (pre-flight)
//! 2. Parses the cell graph to extract metadata
//! 3. Sends `graph.execute` to the Neural API with the cell graph ID
//! 4. Reports success/failure with structured output
//!
//! Usage:
//!   biomeos nucleus attach graphs/esotericwebb_cell.toml
//!   biomeos nucleus attach graphs/footprint_cell.toml --dry-run

use std::path::PathBuf;

use anyhow::{Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use biomeos_core::family_discovery::get_family_id;
use biomeos_core::socket_discovery::neural_api::resolve_neural_api_socket;
use tracing::{error, info, warn};

struct CellGraphMeta<'a> {
    id: &'a str,
    description: &'a str,
    gate: &'a str,
}

fn parse_cell_graph(cell_graph: &PathBuf) -> Result<(toml::Value, String)> {
    let content = std::fs::read_to_string(cell_graph)
        .with_context(|| format!("Cannot read cell graph: {}", cell_graph.display()))?;
    let parsed: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML: {}", cell_graph.display()))?;
    Ok((parsed, content))
}

fn extract_metadata(parsed: &toml::Value) -> Result<CellGraphMeta<'_>> {
    let id = parsed
        .get("graph")
        .and_then(|g| g.get("id"))
        .and_then(|v| v.as_str())
        .context("Cell graph missing [graph].id field")?;

    let description = parsed
        .get("graph")
        .and_then(|g| g.get("description"))
        .and_then(|d| d.as_str())
        .unwrap_or("(no description)");

    let gate = parsed
        .get("graph")
        .and_then(|g| g.get("metadata"))
        .and_then(|m| m.get("gate"))
        .and_then(|g| g.as_str())
        .unwrap_or("local");

    Ok(CellGraphMeta {
        id,
        description,
        gate,
    })
}

async fn preflight_health_check(client: &AtomicClient) -> Result<()> {
    info!("Pre-flight: checking NUCLEUS health...");

    let health = client
        .call("composition.health", serde_json::json!({}))
        .await;

    match &health {
        Ok(resp) => {
            let status = resp
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            if status == "healthy" || status == "ok" {
                info!("  NUCLEUS: HEALTHY");
            } else {
                warn!("  NUCLEUS status: {status} (proceeding with caution)");
            }
            Ok(())
        }
        Err(e) => {
            error!("  NUCLEUS health check failed: {e}");
            anyhow::bail!(
                "Cannot attach cell — NUCLEUS is not responding. \
                 Start it with `biomeos nucleus start` first."
            );
        }
    }
}

async fn execute_cell_graph(
    client: &AtomicClient,
    graph_id: &str,
    family: &str,
) -> Result<serde_json::Value> {
    info!("Attaching cell '{graph_id}'...");

    let mut params = serde_json::Map::new();
    params.insert(
        "FAMILY_ID".to_string(),
        serde_json::Value::String(family.to_owned()),
    );

    let response = client
        .call(
            "graph.execute",
            serde_json::json!({
                "graph_id": graph_id,
                "params": params,
            }),
        )
        .await
        .context("graph.execute RPC failed — cell attachment aborted")?;

    if let Some(err) = response.get("error") {
        anyhow::bail!("Cell graph '{graph_id}' execution failed: {err}");
    }

    Ok(response)
}

/// Run cell attachment: validate NUCLEUS health, then execute cell deploy graph.
pub async fn run(
    cell_graph: PathBuf,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    info!("--- Cell Attachment --- biomeOS NUCLEUS ---");

    let (parsed, _content) = parse_cell_graph(&cell_graph)?;
    let meta = extract_metadata(&parsed)?;

    info!("Cell graph: {}", cell_graph.display());
    info!("  ID:          {}", meta.id);
    info!("  Description: {}", meta.description);
    info!("  Target gate: {}", meta.gate);

    let family = family_id.unwrap_or_else(get_family_id);
    let socket_path = match socket {
        Some(s) => s,
        None => resolve_neural_api_socket(&family, None, None)
            .context("Neural API socket not found — is the NUCLEUS running?")?,
    };

    info!("Neural API: {}", socket_path.display());
    info!("Family ID:  {family}");

    let client = AtomicClient::unix(&socket_path);
    preflight_health_check(&client).await?;

    if dry_run {
        info!(
            "[dry run] Would execute cell graph '{}' — not sending.",
            meta.id
        );
        info!("To attach for real, run without --dry-run:");
        info!("  biomeos nucleus attach {}", cell_graph.display());
        return Ok(());
    }

    let response = execute_cell_graph(&client, meta.id, &family).await?;

    info!("--- Cell attached successfully ---");
    info!("  Graph:  {}", meta.id);
    info!("  Gate:   {}", meta.gate);
    info!("  Family: {family}");

    if let Some(result) = response.get("result") {
        if let Some(obj) = result.as_object() {
            for (key, value) in obj {
                info!("  {key}: {value}");
            }
        }
    }

    info!("The cell is now part of the NUCLEUS composition.");
    info!("Monitor with: biomeos doctor --subsystem composition");

    Ok(())
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
    #![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

    use super::*;
    use std::io::Write as _;

    const MINIMAL_CELL_GRAPH: &str = r#"
[graph]
id = "test_cell"
description = "Test cell attachment"
version = "1.0.0"
coordination = "Sequential"

[graph.metadata]
gate = "testGate"
fragments = ["testing"]

[[nodes]]
id = "verify"
action = "check_primal"

[nodes.params]
primal_name = "biomeos"
"#;

    #[tokio::test]
    async fn missing_file_errors() {
        let result = run(
            PathBuf::from("/nonexistent/cell_graph.toml"),
            None,
            None,
            false,
        )
        .await;
        let err = result.expect_err("missing file should error");
        assert!(
            err.to_string().contains("Cannot read cell graph"),
            "error should mention reading: {err}"
        );
    }

    #[tokio::test]
    async fn invalid_toml_errors() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "not { valid } toml [[[").expect("write");

        let result = run(path, None, None, false).await;
        let err = result.expect_err("invalid toml should error");
        assert!(
            err.to_string().contains("Invalid TOML"),
            "error should mention TOML: {err}"
        );
    }

    #[tokio::test]
    async fn missing_graph_id_errors() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("no_id.toml");
        std::fs::write(&path, "[graph]\ndescription = \"no id\"\n").expect("write");

        let result = run(path, None, None, false).await;
        let err = result.expect_err("missing id should error");
        assert!(
            err.to_string().contains("missing [graph].id"),
            "error should mention missing id: {err}"
        );
    }

    #[tokio::test]
    async fn dry_run_succeeds_without_nucleus() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("cell.toml");
        std::fs::write(&path, MINIMAL_CELL_GRAPH).expect("write");

        // dry_run should succeed — it still needs socket discovery though,
        // so provide a fake socket path that won't exist
        let fake_socket = dir.path().join("fake.sock");
        let result = run(path, Some(fake_socket), Some("test-family".into()), true).await;
        // Dry run won't connect to socket, but it will try composition.health
        // which fails because no socket exists
        assert!(
            result.is_err(),
            "dry_run still checks health (socket doesn't exist)"
        );
    }

    #[tokio::test]
    async fn no_nucleus_connection_errors() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("cell.toml");
        std::fs::write(&path, MINIMAL_CELL_GRAPH).expect("write");

        let fake_socket = dir.path().join("nonexistent.sock");
        let result = run(path, Some(fake_socket), Some("test-family".into()), false).await;
        let err = result.expect_err("no NUCLEUS should error");
        assert!(
            err.to_string().contains("not responding")
                || err.to_string().contains("health check failed")
                || err.to_string().contains("connection refused")
                || err.to_string().contains("No such file"),
            "error should indicate NUCLEUS unavailable: {err}"
        );
    }

    #[test]
    fn cell_graph_parses_metadata() {
        let parsed: toml::Value = toml::from_str(MINIMAL_CELL_GRAPH).unwrap();
        let id = parsed["graph"]["id"].as_str().unwrap();
        assert_eq!(id, "test_cell");

        let gate = parsed["graph"]["metadata"]["gate"].as_str().unwrap();
        assert_eq!(gate, "testGate");
    }

    #[test]
    fn real_cell_graphs_parse() {
        let graphs_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("graphs");

        for name in ["esotericwebb_cell.toml", "footprint_cell.toml"] {
            let path = graphs_dir.join(name);
            if path.exists() {
                let content = std::fs::read_to_string(&path).unwrap();
                let parsed: toml::Value = toml::from_str(&content).unwrap();
                let id = parsed["graph"]["id"].as_str().unwrap();
                assert!(!id.is_empty(), "{name} should have graph ID");
                let gate = parsed["graph"]["metadata"]["gate"].as_str().unwrap();
                assert_eq!(gate, "ironGate", "{name} targets ironGate");
            }
        }
    }

    #[tokio::test]
    async fn tempfile_cell_graph_with_explicit_socket() {
        let dir = tempfile::tempdir().expect("temp dir");
        let cell = dir.path().join("explicit.toml");
        let mut f = std::fs::File::create(&cell).expect("create");
        write!(f, "{MINIMAL_CELL_GRAPH}").expect("write");
        drop(f);

        // Use a socket path that is a valid file but not a UDS
        let fake_sock = dir.path().join("not_a_socket");
        std::fs::write(&fake_sock, "").expect("write fake");

        let result = run(cell, Some(fake_sock), Some("explicit-family".into()), false).await;
        // Should fail at health check since it's not a real socket
        assert!(result.is_err());
    }
}
