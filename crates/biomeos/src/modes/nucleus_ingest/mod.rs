// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS spore ingest/emit gateway (NC-1.1, NC-1.2, NC-1.4).
//!
//! Orchestrates pseudoSpore ingestion through the provenance trio:
//! validate envelope → store (NestGate) → DAG session (rhizoCrypt) →
//! ledger entry (loamSpine) → attribution braid (sweetGrass) →
//! sign receipt (BearDog).
//!
//! Envelope validation uses `biomeos-pseudospore` (NC-1.4 resolved),
//! which implements the canonical pseudoSpore 2.0 standard compatible
//! with `litho_core::pseudospore`.

mod envelope;
mod materialize;
mod receipt;

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, SystemPaths};
use envelope::validate_envelope;
use materialize::materialize_pseudospore;
use receipt::{write_emit_receipt, write_ingest_receipt};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{error, info};

/// Run `biomeos nucleus ingest <pseudospore-dir>`.
pub async fn run_ingest(
    pseudospore_dir: PathBuf,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket
        .unwrap_or_else(|| SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}")));

    info!("NUCLEUS ingest");
    info!("  source: {}", pseudospore_dir.display());
    info!("  family: {family}");
    info!("  socket: {}", socket_path.display());

    let envelope = validate_envelope(&pseudospore_dir)?;
    info!(
        "  envelope: {} ({} data files)",
        envelope.scope_id, envelope.data_file_count
    );

    if dry_run {
        info!("  [dry run] Would execute nest_ingest_spore signal graph");
        info!(
            "    params = {}",
            serde_json::to_string_pretty(&envelope.to_params())?
        );
        return Ok(());
    }

    let request = JsonRpcRequest::new(
        "signal.dispatch",
        serde_json::json!({
            "signal": "nest.ingest_spore",
            "params": envelope.to_params(),
        }),
    );

    let response = send_jsonrpc(&socket_path, &request).await?;

    if let Some(result) = response.get("result") {
        info!("NUCLEUS ingest succeeded");

        write_ingest_receipt(&pseudospore_dir, result, &family)?;

        if let Some(receipt_ids) = result.get("receipt") {
            info!(
                "  receipt: {}",
                serde_json::to_string_pretty(receipt_ids)?
            );
        }
    } else if let Some(err) = response.get("error") {
        error!("NUCLEUS ingest failed: {err}");
        anyhow::bail!("NUCLEUS ingest failed: {err}");
    }

    Ok(())
}

/// Run `biomeos nucleus emit <spore-id>`.
///
/// Full materialization pipeline:
/// 1. Dispatch `nest.emit_spore` signal graph
/// 2. Poll `graph.status` until execution completes
/// 3. Extract node results (content, braid, signature)
/// 4. Materialize pseudoSpore 2.0 directory
/// 5. Write `emit_manifest.json` as dispatch audit trail
pub async fn run_emit(
    spore_id: String,
    output_dir: Option<PathBuf>,
    socket: Option<PathBuf>,
    family_id: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let family = family_id.unwrap_or_else(biomeos_core::family_discovery::get_family_id);
    let socket_path = socket
        .unwrap_or_else(|| SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}")));

    let out = output_dir.unwrap_or_else(|| PathBuf::from("."));

    info!("NUCLEUS emit");
    info!("  spore_id: {spore_id}");
    info!("  output: {}", out.display());
    info!("  family: {family}");

    if dry_run {
        info!("  [dry run] Would execute nest_emit_spore signal graph for {spore_id}");
        return Ok(());
    }

    let request = JsonRpcRequest::new(
        "signal.dispatch",
        serde_json::json!({
            "signal": "nest.emit_spore",
            "params": {
                "spore_id": spore_id,
                "family_id": family,
            },
        }),
    );

    let response = send_jsonrpc(&socket_path, &request).await?;

    let result = response
        .get("result")
        .context("No result in signal dispatch response")?;

    let execution_id = result
        .pointer("/execution/execution_id")
        .or_else(|| result.get("execution_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    info!("NUCLEUS emit dispatched (execution_id: {execution_id})");

    let emit_dir = out.join(format!("spore_{spore_id}"));
    tokio::fs::create_dir_all(&emit_dir).await?;

    let manifest_path = emit_dir.join("emit_manifest.json");
    tokio::fs::write(&manifest_path, serde_json::to_string_pretty(result)?).await?;
    info!("  dispatch manifest: {}", manifest_path.display());

    let final_status = if execution_id.is_empty() {
        result.clone()
    } else {
        poll_execution(&socket_path, execution_id).await?
    };

    materialize_pseudospore(&emit_dir, &spore_id, &family, &final_status).await?;
    write_emit_receipt(&emit_dir, &final_status, &family)?;

    info!("NUCLEUS emit complete: {}", emit_dir.display());
    Ok(())
}

/// Poll `graph.status` until the execution completes or fails.
///
/// Exponential backoff: 100ms, 200ms, 400ms, ... capped at 5s. Timeout 120s.
async fn poll_execution(socket_path: &Path, execution_id: &str) -> Result<serde_json::Value> {
    let mut delay = std::time::Duration::from_millis(100);
    let max_delay = std::time::Duration::from_secs(5);
    let timeout = std::time::Duration::from_secs(120);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!(
                "Execution {execution_id} timed out after {}s",
                timeout.as_secs()
            );
        }

        tokio::time::sleep(delay).await;

        let request = JsonRpcRequest::new(
            "graph.status",
            serde_json::json!({ "execution_id": execution_id }),
        );

        match send_jsonrpc(socket_path, &request).await {
            Ok(response) => {
                if let Some(status) = response.get("result") {
                    let state = status
                        .get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    match state {
                        "completed" => {
                            info!("  execution completed in {}ms", start.elapsed().as_millis());
                            return Ok(status.clone());
                        }
                        "failed" => {
                            let err_msg = status
                                .get("error")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown error");
                            anyhow::bail!("Execution {execution_id} failed: {err_msg}");
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                info!("  poll retry ({}): {e}", start.elapsed().as_millis());
            }
        }

        delay = std::cmp::min(delay * 2, max_delay);
    }
}

/// Send a JSON-RPC request over a Unix socket and parse the response.
async fn send_jsonrpc(socket_path: &Path, request: &JsonRpcRequest) -> Result<serde_json::Value> {
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Failed to connect to Neural API at {}", socket_path.display()))?;

    let (reader, mut writer) = stream.into_split();
    let request_bytes = serde_json::to_vec(request)?;
    writer.write_all(&request_bytes).await?;
    writer.write_all(b"\n").await?;
    writer.shutdown().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    buf_reader.read_line(&mut response_line).await?;

    let response: serde_json::Value =
        serde_json::from_str(response_line.trim()).context("Invalid JSON-RPC response")?;
    Ok(response)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
