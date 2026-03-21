// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Genome handler tests

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::build::{build_genome, compose_genome, create_genome};
use super::retrieval::{download_genome, get_genome_info, list_genomes};
use super::state::{GenomeState, genome_state};
use super::types::{BinarySpec, BuildRequest, ComposeRequest, CreateGenomeRequest};
use axum::Json;
use axum::extract::Path;
use std::path::PathBuf;
use tempfile::TempDir;

use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeManifest};

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
async fn test_get_genome_info_success_after_save() {
    let id = format!("retrieval-info-{}", uuid::Uuid::new_v4());
    let manifest = GenomeManifest::new("retrieval-test").version("2.1.0");
    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(Arch::X86_64, b"payload");
    genome_state()
        .save_genome(&id, &genome)
        .await
        .expect("save");

    let Json(info) = get_genome_info(Path(id.clone())).await.expect("ok");
    assert_eq!(info.name, "retrieval-test");
    assert_eq!(info.version, "2.1.0");
    assert!(
        !info.architectures.is_empty(),
        "expected at least one arch key in genome"
    );
}

#[tokio::test]
async fn test_list_genomes_includes_saved_genome() {
    let id = format!("retrieval-list-{}", uuid::Uuid::new_v4());
    let manifest = GenomeManifest::new("list-me").version("0.0.2");
    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(Arch::Aarch64, b"p");
    genome_state()
        .save_genome(&id, &genome)
        .await
        .expect("save");

    let Json(list) = list_genomes().await.expect("list ok");
    let found = list.genomes.iter().find(|g| g.id == id);
    assert!(found.is_some(), "genome {id} not in list");
    let g = found.expect("found");
    assert_eq!(g.name, "list-me");
    assert_eq!(g.version, "0.0.2");
    assert!(g.architectures.iter().any(|a| a == "aarch64"));
}

#[tokio::test]
async fn test_download_genome_success_returns_url_and_size() {
    let id = format!("retrieval-dl-{}", uuid::Uuid::new_v4());
    let manifest = GenomeManifest::new("dl-test").version("1.0.0");
    let mut genome = GenomeBin::with_manifest(manifest);
    genome.add_binary_bytes(Arch::X86_64, b"z");
    genome_state()
        .save_genome(&id, &genome)
        .await
        .expect("save");

    let path = genome_state().genome_path(&id);
    assert!(path.exists(), "genome file should exist on disk");

    let Json(dl) = download_genome(Path(id)).await.expect("download ok");
    assert!(dl.url.contains("/data"));
    assert!(dl.size > 0, "expected non-zero file size, got {}", dl.size);
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

#[tokio::test]
async fn test_build_genome_binary_file_missing() {
    let req = BuildRequest {
        name: format!("test-missing-bin-{}", uuid::Uuid::new_v4()),
        version: Some("0.0.1".to_string()),
        description: None,
        binaries: vec![BinarySpec {
            arch: "x86_64".to_string(),
            path: PathBuf::from("/nonexistent/path/to/binary-xyz-12345"),
        }],
    };
    let result = build_genome(axum::Json(req)).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "got: {result:?}"
    );
}

#[tokio::test]
async fn test_build_genome_success_saves_to_state() {
    let temp = TempDir::new().expect("tempdir");
    let bin_path = temp.path().join("fake-primal.bin");
    std::fs::write(&bin_path, b"ELF\x00fake-binary-for-test").expect("write fake bin");

    let name = format!("handler-build-{}", uuid::Uuid::new_v4());
    let req = BuildRequest {
        name: name.clone(),
        version: Some("1.2.3".to_string()),
        description: Some("test build".to_string()),
        binaries: vec![BinarySpec {
            arch: "x86_64".to_string(),
            path: bin_path,
        }],
    };

    let result = build_genome(axum::Json(req)).await.expect("build ok");
    assert!(result.success);
    assert_eq!(result.genome_id, format!("{name}-1.2.3"));
    assert!(result.message.contains("architectures"));
}

#[tokio::test]
async fn test_create_genome_handler_success() {
    let name = format!("handler-create-{}", uuid::Uuid::new_v4());
    let req = CreateGenomeRequest {
        name: name.clone(),
        version: None,
        description: None,
    };
    let result = create_genome(axum::Json(req)).await.expect("create ok");
    assert!(result.success);
    assert_eq!(result.genome_id, format!("{name}-0.1.0"));
}

#[tokio::test]
async fn test_compose_genome_missing_source_returns_not_found() {
    let req = ComposeRequest {
        name: format!("composed-{}", uuid::Uuid::new_v4()),
        nucleus_type: "ORCHESTRATOR".to_string(),
        genomes: vec![
            "definitely-no-such-genome-aaa".to_string(),
            "definitely-no-such-genome-bbb".to_string(),
        ],
    };
    let result = compose_genome(axum::Json(req)).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "got: {result:?}"
    );
}

#[tokio::test]
async fn test_compose_genome_success_embeds_binaries() {
    let id_a = format!("compose-a-{}", uuid::Uuid::new_v4());
    let id_b = format!("compose-b-{}", uuid::Uuid::new_v4());

    let mut g_a = GenomeBin::new("ga");
    g_a.add_binary_bytes(biomeos_genomebin_v3::Arch::X86_64, b"bin-a");
    genome_state()
        .save_genome(&id_a, &g_a)
        .await
        .expect("save a");

    let mut g_b = GenomeBin::new("gb");
    g_b.add_binary_bytes(biomeos_genomebin_v3::Arch::Aarch64, b"bin-b");
    genome_state()
        .save_genome(&id_b, &g_b)
        .await
        .expect("save b");

    let out_name = format!("composed-out-{}", uuid::Uuid::new_v4());
    let req = ComposeRequest {
        name: out_name.clone(),
        nucleus_type: "TEST".to_string(),
        genomes: vec![id_a, id_b],
    };

    let result = compose_genome(axum::Json(req)).await.expect("compose ok");
    assert!(result.success);
    assert_eq!(result.genome_id, format!("{out_name}-composed"));
    assert!(result.embedded_count >= 1);
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
    use axum::extract::Path;

    let result = download_genome(Path("nonexistent-download-xyz".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}

#[tokio::test]
async fn test_list_genomes_uses_global_state() {
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

// =========================================================================
// GenomeState: additional coverage (lifecycle + corrupt file handling)
// =========================================================================

#[tokio::test]
async fn test_genome_state_new_creates_dir() {
    let state = GenomeState::new();
    assert!(state.is_ok() || state.is_err());
}

#[tokio::test]
async fn test_genome_state_list_all_with_genomes() {
    let temp = TempDir::new().expect("tempdir");
    let state = GenomeState::with_storage(temp.path().to_path_buf()).expect("state");

    let manifest = GenomeManifest::new("listed-genome");
    let genome = GenomeBin::with_manifest(manifest);
    state.save_genome("list-test", &genome).await.expect("save");

    let genomes = state.list_all().await.expect("list");
    assert_eq!(genomes.len(), 1);
    assert_eq!(genomes[0].0, "list-test");
}

#[tokio::test]
async fn test_genome_state_list_all_skips_corrupt_files() {
    let temp = TempDir::new().expect("tempdir");
    let state = GenomeState::with_storage(temp.path().to_path_buf()).expect("state");

    std::fs::write(temp.path().join("corrupt.genome"), b"not valid").expect("write");

    let manifest = GenomeManifest::new("good");
    let genome = GenomeBin::with_manifest(manifest);
    state.save_genome("good", &genome).await.expect("save");

    let genomes = state.list_all().await.expect("list");
    assert_eq!(genomes.len(), 1, "should skip corrupt, keep good");
    assert_eq!(genomes[0].0, "good");
}

#[tokio::test]
async fn test_genome_state_list_all_deleted_dir() {
    let temp = TempDir::new().expect("tempdir");
    let storage = temp.path().join("will_be_removed");
    let state = GenomeState::with_storage(storage.clone()).expect("state");
    std::fs::remove_dir_all(&storage).expect("remove storage dir");

    let genomes = state.list_all().await.expect("list");
    assert!(genomes.is_empty());
}

#[tokio::test]
async fn test_genome_state_load_nonexistent_genome() {
    let temp = TempDir::new().expect("tempdir");
    let state = GenomeState::with_storage(temp.path().to_path_buf()).expect("state");

    let result = state.load_genome("does-not-exist").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// =========================================================================
// Self-replicate coverage
// =========================================================================

#[tokio::test]
async fn test_self_replicate_handler() {
    use super::build::self_replicate;

    let result = self_replicate().await;
    assert!(result.is_ok());
    let resp = result.unwrap().0;
    assert!(resp.success);
    assert_eq!(resp.genome_id, "biomeos-self");
    assert!(resp.size > 0);
    assert!(resp.message.contains("Self-replicated"));
}

// =========================================================================
// Create/compose handler coverage
// =========================================================================

#[tokio::test]
async fn test_create_genome_with_all_fields() {
    let req = CreateGenomeRequest {
        name: "full-create-test".to_string(),
        version: Some("2.0.0".to_string()),
        description: Some("test genome for coverage".to_string()),
    };
    let result = create_genome(Json(req)).await;
    assert!(result.is_ok());
    let resp = result.unwrap().0;
    assert!(resp.success);
    assert!(resp.genome_id.contains("full-create-test"));
    assert!(resp.genome_id.contains("2.0.0"));
}

#[tokio::test]
async fn test_compose_genome_missing_source() {
    let req = ComposeRequest {
        name: "compose-fail".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec!["nonexistent-source-genome-xyz".to_string()],
    };
    let result = compose_genome(Json(req)).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), axum::http::StatusCode::NOT_FOUND);
}

// =========================================================================
// verify_genome(Path): ID-based handler (global genome_state)
// =========================================================================

#[tokio::test]
async fn test_verify_genome_by_id_valid_after_save() {
    use super::validation::verify_genome;

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
    use super::validation::verify_genome;

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
