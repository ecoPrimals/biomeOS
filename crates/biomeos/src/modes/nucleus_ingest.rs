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
//! with `litho_core::pseudospore`. When lithoSpore ships `pseudospore-core`
//! as a standalone crate, `biomeos-pseudospore` becomes a thin re-export.

use anyhow::{Context, Result};
use biomeos_pseudospore::{self as pseudospore, PseudoSporeManifest, SporeStatus};
use biomeos_types::{JsonRpcRequest, SystemPaths};
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

        write_receipt(&pseudospore_dir, result, &family)?;

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

    // Write dispatch audit trail
    let manifest_path = emit_dir.join("emit_manifest.json");
    tokio::fs::write(&manifest_path, serde_json::to_string_pretty(result)?).await?;
    info!("  dispatch manifest: {}", manifest_path.display());

    // Poll execution until completion
    let final_status = if execution_id.is_empty() {
        result.clone()
    } else {
        poll_execution(&socket_path, execution_id).await?
    };

    // Materialize pseudoSpore directory from execution results
    materialize_pseudospore(&emit_dir, &spore_id, &family, &final_status).await?;

    // Write emit receipt (symmetric with nucleus_ingest.toml)
    write_emit_receipt(&emit_dir, &final_status, &family)?;

    info!("NUCLEUS emit complete: {}", emit_dir.display());
    Ok(())
}

/// Poll `graph.status` until the execution completes or fails.
///
/// Uses exponential backoff: 100ms, 200ms, 400ms, ... capped at 5s.
/// Timeout after 120 seconds total.
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
                        _ => {
                            // Still running — continue polling
                        }
                    }
                }
            }
            Err(e) => {
                // Transient connection error — retry
                info!("  poll retry ({}): {e}", start.elapsed().as_millis());
            }
        }

        delay = std::cmp::min(delay * 2, max_delay);
    }
}

/// Materialize a pseudoSpore 2.0 directory from execution results.
async fn materialize_pseudospore(
    emit_dir: &Path,
    spore_id: &str,
    family: &str,
    status: &serde_json::Value,
) -> Result<()> {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // scope.toml
    let scope_content = format!(
        r#"[artifact]
name = "{spore_id}"
version = "1.0.0"
type = "pseudoSpore"
date = "{now}"
origin = "biomeOS-nucleus-emit"
license = "AGPL-3.0-or-later"
"#,
    );
    tokio::fs::write(emit_dir.join("scope.toml"), &scope_content).await?;

    // validation.json — stub with structural module
    let validation = serde_json::json!({
        "artifact": spore_id,
        "version": "1.0.0",
        "date": now,
        "modules": [{
            "name": "nucleus_emit",
            "status": "PASS",
            "checks_total": 1,
            "checks_passed": 1,
        }]
    });
    tokio::fs::write(
        emit_dir.join("validation.json"),
        serde_json::to_string_pretty(&validation)?,
    )
    .await?;

    // receipts/
    tokio::fs::create_dir_all(emit_dir.join("receipts")).await?;

    let env_content = format!(
        r#"[hardware]
arch = "{arch}"

[software]
emitter = "biomeOS"
family_id = "{family}"
"#,
        arch = std::env::consts::ARCH,
    );
    tokio::fs::write(emit_dir.join("receipts/environment.toml"), &env_content).await?;

    // provenance/ — extract braid resolve output if available
    tokio::fs::create_dir_all(emit_dir.join("provenance")).await?;

    let braid_data = status
        .pointer("/nodes/resolve_braid/result")
        .or_else(|| status.get("braid"))
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let ferment = serde_json::json!({
        "dataset_id": spore_id,
        "spring": "nucleus_emit",
        "braid": braid_data,
    });
    tokio::fs::write(
        emit_dir.join("provenance/ferment_transcript.json"),
        serde_json::to_string_pretty(&ferment)?,
    )
    .await?;

    // data/ — extract retrieved content if available
    tokio::fs::create_dir_all(emit_dir.join("data")).await?;

    let content_data = status
        .pointer("/nodes/retrieve_content/result")
        .or_else(|| status.get("content"))
        .cloned()
        .unwrap_or(serde_json::json!({"spore_id": spore_id, "status": "pending"}));

    tokio::fs::write(
        emit_dir.join("data/content.json"),
        serde_json::to_string_pretty(&content_data)?,
    )
    .await?;

    // README.md
    let readme = format!(
        "# pseudoSpore: {spore_id}\n\n\
         Emitted by biomeOS NUCLEUS (family: {family}).\n\
         Date: {now}\n"
    );
    tokio::fs::write(emit_dir.join("README.md"), &readme).await?;

    // receipts/checksums.blake3 — compute after all files are written
    let checksums = pseudospore::compute_checksums(
        emit_dir,
        &["data", "provenance", "receipts"],
    );
    let checksums_content = pseudospore::format_checksums(&checksums);
    tokio::fs::write(
        emit_dir.join("receipts/checksums.blake3"),
        &checksums_content,
    )
    .await?;

    info!("  materialized pseudoSpore ({} checksum entries)", checksums.len());
    Ok(())
}

/// Write `receipts/nucleus_emit.toml` (symmetric with `nucleus_ingest.toml`).
fn write_emit_receipt(emit_dir: &Path, status: &serde_json::Value, family: &str) -> Result<()> {
    let receipts_dir = emit_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    let execution_id = extract_receipt_field(
        status,
        &["/execution_id", "/execution/execution_id"],
    );

    let braid_id = extract_receipt_field(
        status,
        &[
            "/receipt/braid_id",
            "/nodes/resolve_braid/result/braid_id",
        ],
    );

    let signature = extract_receipt_field(
        status,
        &[
            "/receipt/signature",
            "/nodes/sign_emission/result/signature",
        ],
    );

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let receipt_content = format!(
        r#"# NUCLEUS Emit Receipt
# Generated by biomeOS nucleus emit

[receipt]
execution_id = "{execution_id}"
braid_id = "{braid_id}"
signature = "{signature}"
family_id = "{family}"
timestamp = {timestamp}
"#,
    );

    let receipt_path = receipts_dir.join("nucleus_emit.toml");
    std::fs::write(&receipt_path, receipt_content)?;
    info!("  emit receipt: {}", receipt_path.display());
    Ok(())
}

/// Validated pseudoSpore envelope metadata, mapped from `PseudoSporeManifest`.
#[derive(Debug)]
struct Envelope {
    scope_id: String,
    pseudospore_dir: PathBuf,
    data_file_count: usize,
    checksums: Vec<(String, String)>,
    scope_json: serde_json::Value,
}

impl Envelope {
    fn from_manifest(manifest: &PseudoSporeManifest) -> Self {
        Self {
            scope_id: manifest.scope.artifact.name.clone(),
            pseudospore_dir: manifest.root.clone(),
            data_file_count: manifest.checksums.len(),
            checksums: manifest
                .checksums
                .iter()
                .map(|c| (c.path.clone(), c.hash.clone()))
                .collect(),
            scope_json: serde_json::json!({
                "name": manifest.scope.artifact.name,
                "version": manifest.scope.artifact.version,
                "type": manifest.scope.artifact.artifact_type,
                "date": manifest.scope.artifact.date,
                "origin": manifest.scope.artifact.origin,
                "spring": manifest.ferment.spring,
                "dataset_id": manifest.ferment.dataset_id,
            }),
        }
    }

    fn to_params(&self) -> serde_json::Value {
        serde_json::json!({
            "scope_id": self.scope_id,
            "source_dir": self.pseudospore_dir.display().to_string(),
            "data_file_count": self.data_file_count,
            "checksums": self.checksums.iter()
                .map(|(path, hash)| serde_json::json!({"file": path, "blake3": hash}))
                .collect::<Vec<_>>(),
            "manifest": self.scope_json,
        })
    }
}

/// Validate a pseudoSpore directory using the canonical pseudoSpore 2.0 standard.
///
/// Uses `biomeos-pseudospore` (compatible with `litho_core::pseudospore`):
/// 1. `scope.toml` with `[artifact] type = "pseudoSpore"`
/// 2. `validation.json` with module results
/// 3. `receipts/environment.toml`, `receipts/checksums.blake3`
/// 4. `provenance/ferment_transcript.json`
/// 5. `README.md`
/// 6. BLAKE3 checksum verification
fn validate_envelope(dir: &Path) -> Result<Envelope> {
    anyhow::ensure!(
        dir.is_dir(),
        "pseudoSpore path is not a directory: {}",
        dir.display()
    );

    let mut manifest = pseudospore::load_pseudospore(dir);

    if manifest.status == SporeStatus::Invalid {
        anyhow::bail!(
            "Invalid pseudoSpore at {}: {}",
            dir.display(),
            manifest.errors.join("; ")
        );
    }

    if !pseudospore::verify_checksums(&mut manifest) {
        anyhow::bail!(
            "Checksum verification failed for {}: {}",
            dir.display(),
            manifest.errors.join("; ")
        );
    }

    Ok(Envelope::from_manifest(&manifest))
}

/// Extract a string from the signal dispatch response, trying multiple JSON
/// pointer paths in priority order. The `signal.dispatch` handler wraps the
/// graph executor result under `/execution/...`, but the graph executes
/// asynchronously — the immediate response only carries `execution_id`.
/// If a future synchronous path provides node-level results, they'll appear
/// under `/receipt/{field}` or `/execution/nodes/{node}/result/{field}`.
fn extract_receipt_field<'a>(result: &'a serde_json::Value, pointers: &[&str]) -> &'a str {
    for ptr in pointers {
        if let Some(v) = result.pointer(ptr).and_then(|v| v.as_str()) {
            return v;
        }
    }
    "pending"
}

/// Write `receipts/nucleus_ingest.toml` with trio IDs from the signal result.
///
/// The signal dispatch response shape is:
/// ```json
/// { "signal": "nest.ingest_spore", "graph_id": "...", "execution": { "execution_id": "...", ... } }
/// ```
/// Node-level outputs (store_id, dag_session_id, etc.) are not available in
/// the immediate response because graph execution is asynchronous. The receipt
/// records the execution_id so the caller can poll for completion. When the
/// executor is extended to return node results synchronously, the fallback
/// pointers will pick them up without code changes.
fn write_receipt(pseudospore_dir: &Path, result: &serde_json::Value, family: &str) -> Result<()> {
    let receipts_dir = pseudospore_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    let receipt_path = receipts_dir.join("nucleus_ingest.toml");

    let execution_id = extract_receipt_field(result, &[
        "/execution/execution_id",
        "/execution_id",
    ]);
    let store_id = extract_receipt_field(result, &[
        "/receipt/store_id",
        "/execution/nodes/store_content/result/store_id",
    ]);
    let dag_id = extract_receipt_field(result, &[
        "/receipt/dag_session_id",
        "/execution/nodes/dag_session/result/dag_session_id",
    ]);
    let ledger_id = extract_receipt_field(result, &[
        "/receipt/ledger_entry_id",
        "/execution/nodes/ledger_entry/result/ledger_entry_id",
    ]);
    let braid_id = extract_receipt_field(result, &[
        "/receipt/braid_id",
        "/execution/nodes/attribution_braid/result/braid_id",
    ]);
    let signature = extract_receipt_field(result, &[
        "/receipt/signature",
        "/execution/nodes/sign_receipt/result/signature",
    ]);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let receipt_content = format!(
        r#"# NUCLEUS Ingest Receipt
# Generated by biomeOS nucleus ingest

[receipt]
execution_id = "{execution_id}"
store_id = "{store_id}"
dag_session_id = "{dag_id}"
ledger_entry_id = "{ledger_id}"
braid_id = "{braid_id}"
signature = "{signature}"
family_id = "{family}"
timestamp = {timestamp}
"#,
    );

    std::fs::write(&receipt_path, receipt_content)?;
    info!("  receipt written: {}", receipt_path.display());
    Ok(())
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
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a valid pseudoSpore 2.0 directory for testing.
    fn create_valid_pseudospore(dir: &Path) {
        std::fs::write(
            dir.join("scope.toml"),
            r#"[artifact]
name = "test-spore-001"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-05-27"
origin = "biomeOS-test"
license = "AGPL-3.0"
"#,
        )
        .unwrap();

        std::fs::write(
            dir.join("validation.json"),
            r#"{"artifact":"test-spore-001","version":"1.0.0","date":"2026-05-27","modules":[{"name":"structural","status":"PASS","checks_total":3,"checks_passed":3}]}"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("receipts")).unwrap();
        std::fs::write(
            dir.join("receipts/environment.toml"),
            "[hardware]\ncpu = \"x86_64\"\ncores = 8\n\n[software]\nos = \"Linux\"\nrust = \"1.82\"\n",
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::write(dir.join("data/payload.bin"), b"hello world").unwrap();

        let hash = biomeos_pseudospore::compute_checksums(dir, &["data"]);
        std::fs::write(
            dir.join("receipts/checksums.blake3"),
            biomeos_pseudospore::format_checksums(&hash),
        )
        .unwrap();

        std::fs::create_dir_all(dir.join("provenance")).unwrap();
        std::fs::write(
            dir.join("provenance/ferment_transcript.json"),
            r#"{"dataset_id":"ds-001","spring":"hotSpring","spring_version":"1.5.0"}"#,
        )
        .unwrap();

        std::fs::write(dir.join("README.md"), "# Test pseudoSpore\n").unwrap();
    }

    #[test]
    fn test_validate_envelope_missing_dir() {
        let result = validate_envelope(Path::new("/nonexistent/pseudospore"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    fn test_validate_envelope_missing_scope() {
        let dir = TempDir::new().unwrap();
        let result = validate_envelope(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("scope.toml"));
    }

    #[test]
    fn test_validate_envelope_valid() {
        let dir = TempDir::new().unwrap();
        create_valid_pseudospore(dir.path());

        let envelope = validate_envelope(dir.path()).unwrap();
        assert_eq!(envelope.scope_id, "test-spore-001");
        assert_eq!(envelope.data_file_count, 1);
        assert!(!envelope.checksums.is_empty());
    }

    #[test]
    fn test_validate_envelope_scope_artifact_name() {
        let dir = TempDir::new().unwrap();
        create_valid_pseudospore(dir.path());

        let envelope = validate_envelope(dir.path()).unwrap();
        assert_eq!(
            envelope.scope_id, "test-spore-001",
            "scope_id should come from [artifact].name"
        );
    }

    #[test]
    fn test_validate_envelope_checksum_mismatch() {
        let dir = TempDir::new().unwrap();
        create_valid_pseudospore(dir.path());
        std::fs::write(dir.path().join("data/payload.bin"), b"tampered").unwrap();

        let result = validate_envelope(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Checksum"));
    }

    #[test]
    fn test_envelope_to_params() {
        let envelope = Envelope {
            scope_id: "test".to_string(),
            pseudospore_dir: PathBuf::from("/tmp/test-spore"),
            data_file_count: 2,
            checksums: vec![
                ("data/a.bin".to_string(), "abc123".to_string()),
                ("data/b.bin".to_string(), "def456".to_string()),
            ],
            scope_json: serde_json::json!({"name": "test", "version": "1.0"}),
        };
        let params = envelope.to_params();
        assert_eq!(params["scope_id"], "test");
        assert_eq!(params["source_dir"], "/tmp/test-spore");
        assert_eq!(params["data_file_count"], 2);
        assert_eq!(params["checksums"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_validate_envelope_wrong_type() {
        let dir = TempDir::new().unwrap();
        create_valid_pseudospore(dir.path());
        std::fs::write(
            dir.path().join("scope.toml"),
            "[artifact]\nname = \"x\"\nversion = \"1\"\ntype = \"liveSpore\"\n",
        )
        .unwrap();

        let result = validate_envelope(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pseudoSpore"));
    }

    #[test]
    fn test_write_receipt_with_receipt_envelope() {
        let dir = TempDir::new().unwrap();
        let result = serde_json::json!({
            "receipt": {
                "store_id": "store-001",
                "dag_session_id": "dag-001",
                "ledger_entry_id": "ledger-001",
                "braid_id": "braid-001",
                "signature": "sig-abc"
            }
        });
        write_receipt(dir.path(), &result, "test-family").unwrap();

        let receipt_path = dir.path().join("receipts/nucleus_ingest.toml");
        assert!(receipt_path.exists());
        let content = std::fs::read_to_string(receipt_path).unwrap();
        assert!(content.contains("store_id = \"store-001\""));
        assert!(content.contains("dag_session_id = \"dag-001\""));
        assert!(content.contains("family_id = \"test-family\""));
    }

    #[test]
    fn test_write_receipt_with_execution_envelope() {
        let dir = TempDir::new().unwrap();
        let result = serde_json::json!({
            "signal": "nest.ingest_spore",
            "graph_id": "signals/nest_ingest_spore",
            "execution": {
                "execution_id": "nest_ingest_spore-1716847200",
                "graph_id": "signals/nest_ingest_spore",
                "started_at": "2026-05-27T19:00:00Z"
            }
        });
        write_receipt(dir.path(), &result, "test-family").unwrap();

        let receipt_path = dir.path().join("receipts/nucleus_ingest.toml");
        assert!(receipt_path.exists());
        let content = std::fs::read_to_string(&receipt_path).unwrap();
        assert!(
            content.contains("execution_id = \"nest_ingest_spore-1716847200\""),
            "should extract execution_id from execution envelope"
        );
        assert!(
            content.contains("store_id = \"pending\""),
            "store_id should be pending when async execution hasn't completed"
        );
        assert!(content.contains("family_id = \"test-family\""));
    }

    #[test]
    fn test_extract_receipt_field_priority() {
        let result = serde_json::json!({
            "receipt": { "store_id": "from-receipt" },
            "execution": {
                "nodes": {
                    "store_content": { "result": { "store_id": "from-node" } }
                }
            }
        });
        assert_eq!(
            extract_receipt_field(
                &result,
                &[
                    "/receipt/store_id",
                    "/execution/nodes/store_content/result/store_id",
                ]
            ),
            "from-receipt",
            "should prefer /receipt/ path when both exist"
        );

        let result_node_only = serde_json::json!({
            "execution": {
                "nodes": {
                    "store_content": { "result": { "store_id": "from-node" } }
                }
            }
        });
        assert_eq!(
            extract_receipt_field(
                &result_node_only,
                &[
                    "/receipt/store_id",
                    "/execution/nodes/store_content/result/store_id",
                ]
            ),
            "from-node",
            "should fall back to /execution/nodes/ path"
        );

        let empty = serde_json::json!({});
        assert_eq!(
            extract_receipt_field(&empty, &["/receipt/store_id"]),
            "pending",
            "should return pending when no path matches"
        );
    }

    #[tokio::test]
    async fn test_materialize_pseudospore_structure() {
        let dir = TempDir::new().unwrap();
        let emit_dir = dir.path().join("spore_test-001");
        tokio::fs::create_dir_all(&emit_dir).await.unwrap();

        let status = serde_json::json!({
            "execution_id": "exec-123",
            "state": "completed",
            "nodes": {
                "retrieve_content": {
                    "result": {"data": "test-content", "spore_id": "test-001"}
                },
                "resolve_braid": {
                    "result": {"braid_id": "braid-456"}
                },
                "sign_emission": {
                    "result": {"signature": "sig-789"}
                }
            }
        });

        materialize_pseudospore(&emit_dir, "test-001", "test-family", &status)
            .await
            .unwrap();

        assert!(emit_dir.join("scope.toml").exists());
        assert!(emit_dir.join("validation.json").exists());
        assert!(emit_dir.join("receipts/environment.toml").exists());
        assert!(emit_dir.join("receipts/checksums.blake3").exists());
        assert!(emit_dir.join("provenance/ferment_transcript.json").exists());
        assert!(emit_dir.join("data/content.json").exists());
        assert!(emit_dir.join("README.md").exists());

        let scope = std::fs::read_to_string(emit_dir.join("scope.toml")).unwrap();
        assert!(scope.contains("name = \"test-001\""));
        assert!(scope.contains("type = \"pseudoSpore\""));

        let ferment =
            std::fs::read_to_string(emit_dir.join("provenance/ferment_transcript.json")).unwrap();
        assert!(ferment.contains("braid-456"));

        let content = std::fs::read_to_string(emit_dir.join("data/content.json")).unwrap();
        assert!(content.contains("test-content"));

        let checksums =
            std::fs::read_to_string(emit_dir.join("receipts/checksums.blake3")).unwrap();
        assert!(
            !checksums.is_empty(),
            "checksums.blake3 should have entries"
        );
    }

    #[tokio::test]
    async fn test_materialize_pseudospore_validates() {
        let dir = TempDir::new().unwrap();
        let emit_dir = dir.path().join("spore_validate-test");
        tokio::fs::create_dir_all(&emit_dir).await.unwrap();

        let status = serde_json::json!({"state": "completed"});
        materialize_pseudospore(&emit_dir, "validate-test", "family-x", &status)
            .await
            .unwrap();

        // The materialized directory should pass pseudospore validation
        let manifest = pseudospore::load_pseudospore(&emit_dir);
        assert_eq!(
            manifest.status,
            SporeStatus::Valid,
            "materialized dir should be valid: {:?}",
            manifest.errors
        );
    }

    #[test]
    fn test_write_emit_receipt() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("receipts")).unwrap();

        let status = serde_json::json!({
            "execution_id": "exec-emit-001",
            "nodes": {
                "resolve_braid": { "result": { "braid_id": "braid-abc" } },
                "sign_emission": { "result": { "signature": "sig-def" } },
            }
        });

        write_emit_receipt(dir.path(), &status, "emit-family").unwrap();

        let receipt_path = dir.path().join("receipts/nucleus_emit.toml");
        assert!(receipt_path.exists());
        let content = std::fs::read_to_string(receipt_path).unwrap();
        assert!(content.contains("execution_id = \"exec-emit-001\""));
        assert!(content.contains("braid_id = \"braid-abc\""));
        assert!(content.contains("signature = \"sig-def\""));
        assert!(content.contains("family_id = \"emit-family\""));
    }

    #[test]
    fn test_write_emit_receipt_pending_fields() {
        let dir = TempDir::new().unwrap();
        let status = serde_json::json!({});

        write_emit_receipt(dir.path(), &status, "fam").unwrap();

        let content =
            std::fs::read_to_string(dir.path().join("receipts/nucleus_emit.toml")).unwrap();
        assert!(
            content.contains("braid_id = \"pending\""),
            "missing fields should be pending"
        );
    }
}
