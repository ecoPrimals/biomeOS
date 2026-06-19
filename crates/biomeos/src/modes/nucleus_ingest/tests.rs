// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::envelope::validate_envelope;
use super::materialize::materialize_pseudospore;
use super::receipt::{extract_receipt_field, write_emit_receipt, write_ingest_receipt};
use biomeos_pseudospore::{self as pseudospore, SporeStatus};
use std::path::{Path, PathBuf};
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
    use super::envelope::Envelope;
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
    write_ingest_receipt(dir.path(), &result, "test-family").unwrap();

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
    write_ingest_receipt(dir.path(), &result, "test-family").unwrap();

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

    let checksums = std::fs::read_to_string(emit_dir.join("receipts/checksums.blake3")).unwrap();
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

    let content = std::fs::read_to_string(dir.path().join("receipts/nucleus_emit.toml")).unwrap();
    assert!(
        content.contains("braid_id = \"pending\""),
        "missing fields should be pending"
    );
}
