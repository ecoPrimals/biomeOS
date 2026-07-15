// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::retrieval::{download_genome, get_genome_info, list_genomes};
use super::super::state::genome_state;
use axum::Json;
use axum::extract::Path;
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeManifest};

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
async fn test_download_genome_not_found() {
    let result = download_genome(Path("nonexistent-download-xyz".to_string())).await;
    assert!(
        matches!(result, Err(axum::http::StatusCode::NOT_FOUND)),
        "expected NOT_FOUND, got: {result:?}"
    );
}
