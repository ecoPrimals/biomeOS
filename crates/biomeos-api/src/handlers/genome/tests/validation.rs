// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::state::genome_state;
use super::super::types::VerifyRequest;
use super::super::validation::{verify_genome, verify_genome_file};
use axum::Json;
use axum::extract::Path;
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeManifest};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_verify_genome_file_not_found() {
    let req = VerifyRequest {
        path: PathBuf::from("/nonexistent/genome/path/12345.genome"),
    };
    let result = verify_genome_file(axum::Json(req)).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}

#[tokio::test]
async fn test_verify_genome_file_valid() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let genome_path = temp_dir.path().join("valid.genome");

    let mut genome = GenomeBin::new("valid-test");
    genome.add_binary_bytes(biomeos_genomebin_v3::Arch::X86_64, b"test binary");
    genome.save(&genome_path).expect("save genome");

    let req = VerifyRequest {
        path: genome_path.clone(),
    };
    let result = verify_genome_file(axum::Json(req)).await;
    let resp = result.expect("verify should succeed");
    assert!(resp.valid);
    assert_eq!(resp.message, "All checksums valid");
}

#[tokio::test]
async fn test_verify_genome_file_invalid_checksum() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let genome_path = temp_dir.path().join("invalid.genome");

    let mut genome = GenomeBin::new("invalid-test");
    genome.add_binary_bytes(biomeos_genomebin_v3::Arch::X86_64, b"test binary");
    let mut compressed = genome
        .binaries
        .get(&biomeos_genomebin_v3::Arch::X86_64)
        .unwrap()
        .clone();
    compressed.checksum[0] ^= 0xff;
    genome
        .binaries
        .insert(biomeos_genomebin_v3::Arch::X86_64, compressed);
    genome.save(&genome_path).expect("save genome");

    let req = VerifyRequest {
        path: genome_path.clone(),
    };
    let result = verify_genome_file(axum::Json(req)).await;
    let resp = result.expect("verify should return Ok with valid: false");
    assert!(!resp.valid);
    assert_eq!(resp.message, "Checksum verification failed");
}

#[tokio::test]
async fn test_verify_genome_file_load_error() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let genome_path = temp_dir.path().join("invalid-json.genome");
    std::fs::write(&genome_path, "not valid json").expect("write");

    let req = VerifyRequest {
        path: genome_path.clone(),
    };
    let result = verify_genome_file(axum::Json(req)).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::BAD_REQUEST)),
        "expected BAD_REQUEST for invalid JSON, got: {result:?}"
    );
}

#[tokio::test]
async fn test_verify_genome_not_found() {
    let result = verify_genome(Path("nonexistent-genome-xyz-123".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}

#[tokio::test]
async fn test_verify_genome_by_id_valid_after_save() {
    let id = format!("verify-by-id-{}", uuid::Uuid::new_v4());
    let manifest = GenomeManifest::new("verify-id-test").version("1.0.0");
    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(Arch::X86_64, b"payload");
    genome_state()
        .save_genome(&id, &genome)
        .await
        .expect("save");

    let Json(resp) = verify_genome(Path(id)).await.expect("verify ok");
    assert!(resp.valid);
    assert_eq!(resp.message, "All checksums valid");
}

#[tokio::test]
async fn test_verify_genome_by_id_invalid_checksum_after_tamper_save() {
    let id = format!("verify-tamper-{}", uuid::Uuid::new_v4());
    let manifest = GenomeManifest::new("tamper-id-test").version("1.0.0");
    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(Arch::X86_64, b"payload");
    genome_state()
        .save_genome(&id, &genome)
        .await
        .expect("save");

    let mut tampered = genome;
    let mut compressed = tampered
        .binaries
        .get(&Arch::X86_64)
        .expect("x86_64 binary")
        .clone();
    compressed.checksum[0] ^= 0xff;
    tampered.binaries.insert(Arch::X86_64, compressed);
    genome_state()
        .save_genome(&id, &tampered)
        .await
        .expect("save tampered");

    let Json(resp) = verify_genome(Path(id)).await.expect("verify returns body");
    assert!(!resp.valid);
    assert_eq!(resp.message, "Checksum verification failed");
}

#[test]
fn test_verify_request_deserialization() {
    let json = r#"{"path":"/tmp/test.genome"}"#;
    let req: VerifyRequest = serde_json::from_str(json).expect("deserialize");
    assert_eq!(req.path, PathBuf::from("/tmp/test.genome"));
}
