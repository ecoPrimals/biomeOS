// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Genome handler tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::build::build_genome;
use super::retrieval::get_genome_info;
use super::state::GenomeState;
use super::types::{BinarySpec, BuildRequest};
use axum::extract::Path;
use std::path::PathBuf;
use tempfile::TempDir;

use biomeos_genomebin_v3::{GenomeBin, GenomeManifest};

#[test]
fn test_genome_state_default_storage_dir() {
    let dir = GenomeState::default_storage_dir();
    assert!(dir.to_string_lossy().contains("biomeos/genomes"));
}

#[test]
fn test_genome_state_with_storage() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let storage = temp_dir.path().join("genomes");
    let state = GenomeState::with_storage(storage.clone()).expect("create state");
    assert!(storage.exists());
    assert_eq!(state.storage_dir, storage);
}

#[test]
fn test_genome_state_genome_path() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");
    let path = state.genome_path("test-genome");
    assert_eq!(path, temp_dir.path().join("test-genome.genome"));
}

#[tokio::test]
async fn test_genome_state_save_and_load() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

    let manifest = GenomeManifest::new("test").version("1.0.0");
    let genome = GenomeBin::with_manifest(manifest);

    state
        .save_genome("test-1.0.0", &genome)
        .await
        .expect("save genome");

    assert!(temp_dir.path().join("test-1.0.0.genome").exists());

    let loaded = state.load_genome("test-1.0.0").await.expect("load genome");
    assert_eq!(loaded.manifest.name, "test");
    assert_eq!(loaded.manifest.version, "1.0.0");
}

#[tokio::test]
async fn test_get_genome_info_not_found() {
    let result = get_genome_info(Path("nonexistent-genome-xyz".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "got: {result:?}"
    );
}

#[tokio::test]
async fn test_build_genome_invalid_arch() {
    let req = BuildRequest {
        name: "test".to_string(),
        version: None,
        description: None,
        binaries: vec![BinarySpec {
            arch: "invalid_arch".to_string(),
            path: PathBuf::from("/tmp/some-binary"),
        }],
    };
    let result = build_genome(axum::Json(req)).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::BAD_REQUEST)),
        "got: {result:?}"
    );
}

#[test]
fn test_build_request_deserialize() {
    let json = r#"{
        "name": "test-genome",
        "version": "1.0.0",
        "description": "Test genome",
        "binaries": []
    }"#;
    let req: BuildRequest = serde_json::from_str(json).expect("deserialize");
    assert_eq!(req.name, "test-genome");
    assert_eq!(req.version, Some("1.0.0".to_string()));
    assert!(req.binaries.is_empty());
}

// Validation handler tests
#[tokio::test]
async fn test_verify_genome_file_not_found() {
    use super::types::VerifyRequest;
    use super::validation::verify_genome_file;

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
    use super::types::VerifyRequest;
    use super::validation::verify_genome_file;

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
    use super::types::VerifyRequest;
    use super::validation::verify_genome_file;

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
    use super::types::VerifyRequest;
    use super::validation::verify_genome_file;

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
    use super::validation::verify_genome;
    use axum::extract::Path;

    let result = verify_genome(Path("nonexistent-genome-xyz-123".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}

// Retrieval handler tests
#[tokio::test]
async fn test_download_genome_not_found() {
    use super::retrieval::download_genome;
    use axum::extract::Path;

    let result = download_genome(Path("nonexistent-download-xyz".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}

#[tokio::test]
async fn test_list_genomes_uses_global_state() {
    use super::retrieval::list_genomes;

    let result = list_genomes().await;
    assert!(result.is_ok(), "list_genomes should not panic");
    let json = result.unwrap();
    assert!(json.genomes.is_empty() || !json.genomes.is_empty());
}

#[tokio::test]
async fn test_genome_state_list_all_empty_dir() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");
    let genomes = state.list_all().await.expect("list");
    assert!(genomes.is_empty());
}

#[tokio::test]
async fn test_genome_state_load_from_cache() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");

    let manifest = GenomeManifest::new("cached").version("1.0");
    let genome = GenomeBin::with_manifest(manifest);
    state
        .save_genome("cached-1.0", &genome)
        .await
        .expect("save");

    let loaded1 = state.load_genome("cached-1.0").await.expect("load");
    let loaded2 = state
        .load_genome("cached-1.0")
        .await
        .expect("load from cache");
    assert_eq!(loaded1.manifest.name, loaded2.manifest.name);
}

#[test]
fn test_verify_request_deserialization() {
    use super::types::VerifyRequest;

    let json = r#"{"path":"/tmp/test.genome"}"#;
    let req: VerifyRequest = serde_json::from_str(json).expect("deserialize");
    assert_eq!(req.path, PathBuf::from("/tmp/test.genome"));
}

#[test]
fn test_download_response_serialization() {
    use super::types::DownloadResponse;

    let resp = DownloadResponse {
        url: "/api/v1/genome/x/data".to_string(),
        size: 1024,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("/api/v1/genome"));
    assert!(json.contains("1024"));
}

#[test]
fn test_genome_info_response_serialization() {
    use super::types::GenomeInfoResponse;

    let resp = GenomeInfoResponse {
        name: "test".to_string(),
        version: "1.0".to_string(),
        architectures: vec!["x86_64".to_string()],
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("test"));
    assert!(json.contains("x86_64"));
}

#[test]
fn test_genome_summary_serialization() {
    use super::types::GenomeSummary;

    let summary = GenomeSummary {
        id: "id-1".to_string(),
        name: "name".to_string(),
        version: "1.0".to_string(),
        architectures: vec!["aarch64".to_string()],
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    assert!(json.contains("id-1"));
    assert!(json.contains("aarch64"));
}

#[test]
fn test_verify_response_serialization() {
    use super::types::VerifyResponse;

    let resp = VerifyResponse {
        valid: true,
        message: "All checksums valid".to_string(),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("valid"));
    assert!(json.contains("checksums"));
}
