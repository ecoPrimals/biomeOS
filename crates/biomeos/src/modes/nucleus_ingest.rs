// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS spore ingest/emit gateway (NC-1.1, NC-1.2).
//!
//! Orchestrates pseudoSpore ingestion through the provenance trio:
//! validate envelope → store (NestGate) → DAG session (rhizoCrypt) →
//! ledger entry (loamSpine) → attribution braid (sweetGrass) →
//! sign receipt (BearDog).
//!
//! Envelope validation is minimal here; when lithoSpore ships the
//! `pseudospore-core` crate, swap in its canonical implementation.

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, SystemPaths};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{error, info, warn};

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
        info!("  [dry run] Would retrieve spore {spore_id} and package envelope");
        return Ok(());
    }

    let request = JsonRpcRequest::new(
        "nucleus.emit_spore",
        serde_json::json!({
            "spore_id": spore_id,
            "family_id": family,
        }),
    );

    let response = send_jsonrpc(&socket_path, &request).await?;

    if let Some(result) = response.get("result") {
        info!("NUCLEUS emit succeeded");

        let emit_dir = out.join(format!("spore_{spore_id}"));
        tokio::fs::create_dir_all(&emit_dir).await?;

        let manifest_path = emit_dir.join("emit_manifest.json");
        tokio::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(result)?,
        )
        .await?;
        info!("  manifest: {}", manifest_path.display());
    } else if let Some(err) = response.get("error") {
        error!("NUCLEUS emit failed: {err}");
        anyhow::bail!("NUCLEUS emit failed: {err}");
    }

    Ok(())
}

/// Validated pseudoSpore envelope metadata.
#[derive(Debug)]
struct Envelope {
    scope_id: String,
    data_file_count: usize,
    blake3_checksums: Vec<(String, String)>,
    manifest_json: serde_json::Value,
}

impl Envelope {
    fn to_params(&self) -> serde_json::Value {
        serde_json::json!({
            "scope_id": self.scope_id,
            "data_file_count": self.data_file_count,
            "checksums": self.blake3_checksums.iter()
                .map(|(name, hash)| serde_json::json!({"file": name, "blake3": hash}))
                .collect::<Vec<_>>(),
            "manifest": self.manifest_json,
        })
    }
}

/// Validate a pseudoSpore directory envelope.
///
/// Checks:
/// 1. `liveSpore.json` exists and parses
/// 2. `scope.toml` exists (extracts scope_id)
/// 3. BLAKE3 checksums of all data files match manifest
///
/// When `pseudospore-core` ships from lithoSpore, replace this with its
/// canonical `PseudoSporeEnvelope::validate()`.
fn validate_envelope(dir: &Path) -> Result<Envelope> {
    anyhow::ensure!(dir.is_dir(), "pseudoSpore path is not a directory: {}", dir.display());

    let manifest_path = dir.join("liveSpore.json");
    anyhow::ensure!(
        manifest_path.exists(),
        "Missing liveSpore.json in {}",
        dir.display()
    );

    let manifest_raw =
        std::fs::read_to_string(&manifest_path).context("Failed to read liveSpore.json")?;
    let manifest_json: serde_json::Value =
        serde_json::from_str(&manifest_raw).context("Invalid JSON in liveSpore.json")?;

    let scope_id = if dir.join("scope.toml").exists() {
        let scope_raw =
            std::fs::read_to_string(dir.join("scope.toml")).context("Failed to read scope.toml")?;
        let scope: toml::Value =
            toml::from_str(&scope_raw).context("Invalid TOML in scope.toml")?;
        scope
            .get("scope")
            .and_then(|s| s.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        manifest_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string()
    };

    let data_dir = dir.join("data");
    let mut checksums = Vec::new();

    if data_dir.is_dir() {
        for entry in std::fs::read_dir(&data_dir).context("Failed to read data/ directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let contents = std::fs::read(&path)?;
                let hash = blake3::hash(&contents);
                let name = entry.file_name().to_string_lossy().to_string();
                checksums.push((name, hash.to_hex().to_string()));
            }
        }
    }

    if let Some(expected) = manifest_json.get("checksums").and_then(|v| v.as_object()) {
        for (name, expected_hash) in expected {
            if let Some(expected_str) = expected_hash.as_str() {
                if let Some((_, actual)) = checksums.iter().find(|(n, _)| n == name) {
                    anyhow::ensure!(
                        actual == expected_str,
                        "BLAKE3 mismatch for {name}: expected {expected_str}, got {actual}"
                    );
                } else {
                    warn!("Checksum declared for {name} but file not found in data/");
                }
            }
        }
    }

    Ok(Envelope {
        scope_id,
        data_file_count: checksums.len(),
        blake3_checksums: checksums,
        manifest_json,
    })
}

/// Write `receipts/nucleus_ingest.toml` with trio IDs from the signal result.
fn write_receipt(pseudospore_dir: &Path, result: &serde_json::Value, family: &str) -> Result<()> {
    let receipts_dir = pseudospore_dir.join("receipts");
    std::fs::create_dir_all(&receipts_dir)?;

    let receipt_path = receipts_dir.join("nucleus_ingest.toml");

    let dag_id = result
        .pointer("/receipt/dag_session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");
    let ledger_id = result
        .pointer("/receipt/ledger_entry_id")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");
    let braid_id = result
        .pointer("/receipt/braid_id")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");
    let signature = result
        .pointer("/receipt/signature")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");
    let store_id = result
        .pointer("/receipt/store_id")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let receipt_content = format!(
        r#"# NUCLEUS Ingest Receipt
# Generated by biomeOS nucleus ingest

[receipt]
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

    #[test]
    fn test_validate_envelope_missing_dir() {
        let result = validate_envelope(Path::new("/nonexistent/pseudospore"));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a directory")
        );
    }

    #[test]
    fn test_validate_envelope_missing_manifest() {
        let dir = TempDir::new().expect("temp dir");
        let result = validate_envelope(dir.path());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing liveSpore.json")
        );
    }

    #[test]
    fn test_validate_envelope_valid() {
        let dir = TempDir::new().expect("temp dir");

        let manifest = serde_json::json!({
            "id": "test-spore-001",
            "version": "1.0.0",
            "primals": ["beardog", "songbird"]
        });
        std::fs::write(
            dir.path().join("liveSpore.json"),
            serde_json::to_string_pretty(&manifest).expect("json"),
        )
        .expect("write");

        std::fs::create_dir(dir.path().join("data")).expect("mkdir");
        std::fs::write(dir.path().join("data/payload.bin"), b"hello world").expect("write");

        let envelope = validate_envelope(dir.path()).expect("validate");
        assert_eq!(envelope.scope_id, "test-spore-001");
        assert_eq!(envelope.data_file_count, 1);
        assert!(!envelope.blake3_checksums.is_empty());
    }

    #[test]
    fn test_validate_envelope_with_scope_toml() {
        let dir = TempDir::new().expect("temp dir");

        std::fs::write(
            dir.path().join("liveSpore.json"),
            r#"{"version": "2.0"}"#,
        )
        .expect("write");

        std::fs::write(
            dir.path().join("scope.toml"),
            r#"[scope]
id = "my-custom-scope"
"#,
        )
        .expect("write");

        let envelope = validate_envelope(dir.path()).expect("validate");
        assert_eq!(envelope.scope_id, "my-custom-scope");
    }

    #[test]
    fn test_validate_envelope_checksum_mismatch() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::create_dir(dir.path().join("data")).expect("mkdir");
        std::fs::write(dir.path().join("data/file.txt"), b"content").expect("write");

        let manifest = serde_json::json!({
            "id": "checksum-test",
            "checksums": {
                "file.txt": "0000000000000000000000000000000000000000000000000000000000000000"
            }
        });
        std::fs::write(
            dir.path().join("liveSpore.json"),
            serde_json::to_string(&manifest).expect("json"),
        )
        .expect("write");

        let result = validate_envelope(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BLAKE3 mismatch"));
    }

    #[test]
    fn test_envelope_to_params() {
        let envelope = Envelope {
            scope_id: "test".to_string(),
            data_file_count: 2,
            blake3_checksums: vec![
                ("a.bin".to_string(), "abc123".to_string()),
                ("b.bin".to_string(), "def456".to_string()),
            ],
            manifest_json: serde_json::json!({"version": "1.0"}),
        };
        let params = envelope.to_params();
        assert_eq!(params["scope_id"], "test");
        assert_eq!(params["data_file_count"], 2);
        assert_eq!(params["checksums"].as_array().expect("array").len(), 2);
    }

    #[test]
    fn test_write_receipt() {
        let dir = TempDir::new().expect("temp dir");
        let result = serde_json::json!({
            "receipt": {
                "store_id": "store-001",
                "dag_session_id": "dag-001",
                "ledger_entry_id": "ledger-001",
                "braid_id": "braid-001",
                "signature": "sig-abc"
            }
        });
        write_receipt(dir.path(), &result, "test-family").expect("write receipt");

        let receipt_path = dir.path().join("receipts/nucleus_ingest.toml");
        assert!(receipt_path.exists());
        let content = std::fs::read_to_string(receipt_path).expect("read");
        assert!(content.contains("store_id = \"store-001\""));
        assert!(content.contains("dag_session_id = \"dag-001\""));
        assert!(content.contains("family_id = \"test-family\""));
    }
}
