// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use biomeos_types::JsonRpcRequest;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::info;

use super::types::{
    NucleusStatusSummary, parse_nucleus_status, parse_spore_deploy_manifest,
    resolve_lifecycle_socket,
};

/// Send a JSON-RPC request to the Neural API lifecycle endpoint.
pub(crate) async fn send_lifecycle_rpc(
    socket_path: &Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let request = JsonRpcRequest::new(method, params);
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Neural API socket not found at {}", socket_path.display()))?;

    let (reader, mut writer) = stream.into_split();
    writer
        .write_all(format!("{}\n", serde_json::to_string(&request)?).as_bytes())
        .await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    buf_reader.read_line(&mut response_line).await?;

    let response: serde_json::Value =
        serde_json::from_str(response_line.trim()).context("Invalid JSON-RPC response")?;

    if let Some(err) = response.get("error") {
        anyhow::bail!("JSON-RPC error: {err}");
    }

    Ok(response)
}

/// Start NUCLEUS via `lifecycle.start` on the Neural API.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "staged for dispatch wiring in next sprint")
)]
pub async fn run_start(
    socket: Option<PathBuf>,
    family_id: Option<String>,
    mode: String,
    node_id: String,
) -> Result<()> {
    let (socket_path, family) = resolve_lifecycle_socket(socket, family_id);

    info!("NUCLEUS start (remote lifecycle)");
    info!("  mode: {mode}");
    info!("  node: {node_id}");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let response = send_lifecycle_rpc(
        &socket_path,
        "lifecycle.start",
        serde_json::json!({
            "mode": mode,
            "node_id": node_id,
            "family_id": family,
        }),
    )
    .await?;

    if response.get("result").is_some() {
        info!("NUCLEUS start succeeded");
        Ok(())
    } else {
        anyhow::bail!("NUCLEUS start failed: unexpected response");
    }
}

/// Stop NUCLEUS via `lifecycle.shutdown_all` on the Neural API.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "staged for dispatch wiring in next sprint")
)]
pub async fn run_stop(socket: Option<PathBuf>, family_id: Option<String>) -> Result<()> {
    let (socket_path, family) = resolve_lifecycle_socket(socket, family_id);

    info!("NUCLEUS stop (remote lifecycle)");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let response = send_lifecycle_rpc(
        &socket_path,
        "lifecycle.shutdown_all",
        serde_json::json!({}),
    )
    .await?;

    if response.get("result").is_some() {
        info!("NUCLEUS stop initiated");
        Ok(())
    } else {
        anyhow::bail!("NUCLEUS stop failed: unexpected response");
    }
}

/// Query NUCLEUS status via `lifecycle.status` on the Neural API.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "staged for dispatch wiring in next sprint")
)]
pub async fn run_status(
    socket: Option<PathBuf>,
    family_id: Option<String>,
) -> Result<NucleusStatusSummary> {
    let (socket_path, family) = resolve_lifecycle_socket(socket, family_id);

    info!("NUCLEUS status");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let response =
        send_lifecycle_rpc(&socket_path, "lifecycle.status", serde_json::json!({})).await?;

    let result = response
        .get("result")
        .context("lifecycle.status response missing result")?;

    parse_nucleus_status(result)
}

/// Deploy a spore manifest via `graph.execute` on the Neural API.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "staged for dispatch wiring in next sprint")
)]
pub async fn run_deploy(
    spore_file: PathBuf,
    socket: Option<PathBuf>,
    family_id: Option<String>,
) -> Result<()> {
    if !spore_file.exists() {
        anyhow::bail!("Spore file not found: {}", spore_file.display());
    }

    let content = std::fs::read_to_string(&spore_file)
        .with_context(|| format!("Cannot read spore file: {}", spore_file.display()))?;
    let manifest = parse_spore_deploy_manifest(&content)?;

    let (socket_path, family) = resolve_lifecycle_socket(socket, family_id);

    info!("NUCLEUS deploy");
    info!("  spore: {}", spore_file.display());
    info!("  graph: {}", manifest.spore.graph_id);
    info!("  mode: {}", manifest.spore.mode);
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let response = send_lifecycle_rpc(
        &socket_path,
        "graph.execute",
        serde_json::json!({
            "graph_id": manifest.spore.graph_id,
            "params": {
                "mode": manifest.spore.mode,
                "node_id": manifest.spore.node_id,
                "family_id": family,
            },
        }),
    )
    .await?;

    if response.get("result").is_some() {
        info!("NUCLEUS deploy succeeded");
        Ok(())
    } else {
        anyhow::bail!("NUCLEUS deploy failed: unexpected response");
    }
}

/// Undeploy a single primal via `lifecycle.apoptosis` on the Neural API.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "staged for dispatch wiring in next sprint")
)]
pub async fn run_undeploy(
    primal_name: Option<String>,
    socket: Option<PathBuf>,
    family_id: Option<String>,
) -> Result<()> {
    let name = primal_name.context("Primal name required for undeploy")?;
    let (socket_path, family) = resolve_lifecycle_socket(socket, family_id);

    info!("NUCLEUS undeploy");
    info!("  primal: {name}");
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let response = send_lifecycle_rpc(
        &socket_path,
        "lifecycle.apoptosis",
        serde_json::json!({ "name": name }),
    )
    .await?;

    if response.get("result").is_some() {
        info!("NUCLEUS undeploy initiated for {name}");
        Ok(())
    } else {
        anyhow::bail!("NUCLEUS undeploy failed: unexpected response");
    }
}
