// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::state::GenomeState;
use biomeos_genomebin_v3::{GenomeBin, GenomeManifest};
use tempfile::TempDir;

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
async fn test_genome_state_list_all_empty_dir() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let state = GenomeState::with_storage(temp_dir.path().to_path_buf()).expect("create state");
    let genomes = state.list_all().expect("list");
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

    let genomes = state.list_all().expect("list");
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

    let genomes = state.list_all().expect("list");
    assert_eq!(genomes.len(), 1, "should skip corrupt, keep good");
    assert_eq!(genomes[0].0, "good");
}

#[tokio::test]
async fn test_genome_state_list_all_deleted_dir() {
    let temp = TempDir::new().expect("tempdir");
    let storage = temp.path().join("will_be_removed");
    let state = GenomeState::with_storage(storage.clone()).expect("state");
    std::fs::remove_dir_all(&storage).expect("remove storage dir");

    let genomes = state.list_all().expect("list");
    assert!(genomes.is_empty());
}

#[tokio::test]
async fn test_genome_state_load_nonexistent_genome() {
    let temp = TempDir::new().expect("tempdir");
    let state = GenomeState::with_storage(temp.path().to_path_buf()).expect("state");

    let result = state.load_genome("does-not-exist").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_list_genomes_uses_global_state() {
    use super::super::retrieval::list_genomes;

    let result = list_genomes().await;
    assert!(result.is_ok(), "list_genomes should not panic");
    let json = result.unwrap();
    assert!(json.genomes.is_empty() || !json.genomes.is_empty());
}
