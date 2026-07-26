// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::build::{build_genome, compose_genome, create_genome, self_replicate};
use super::super::state::genome_state;
use super::super::types::{BinarySpec, BuildRequest, ComposeRequest, CreateGenomeRequest};
use axum::Json;
use biomeos_genomebin_v3::GenomeBin;
use std::path::PathBuf;
use tempfile::TempDir;

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

#[tokio::test]
async fn test_self_replicate_handler() {
    let result = self_replicate().await;
    assert!(result.is_ok());
    let resp = result.unwrap().0;
    assert!(resp.success);
    assert_eq!(
        resp.genome_id,
        biomeos_types::primal_names::BIOMEOS_SELF_GENOME
    );
    assert!(resp.size > 0);
    assert!(resp.message.contains("Self-replicated"));
}

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
