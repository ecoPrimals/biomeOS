// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::get_genome_storage_dir_with;
use super::super::*;
use std::path::PathBuf;

#[test]
fn test_handle_genome_create_binary_not_found() {
    let args = CreateArgs {
        binary: PathBuf::from("/nonexistent/beardog-xyz"),
        output: PathBuf::from("/tmp/out.json"),
        arch: "x86_64".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Binary not found"));
}

#[test]
fn test_handle_genome_verify_path_not_found() {
    let args = VerifyArgs {
        path: PathBuf::from("/nonexistent/genome-xyz.json"),
    };
    let result = handle_genome_verify(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("GenomeBin not found"));
}

#[test]
fn test_handle_genome_verify_invalid_json() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("corrupt.json");
    std::fs::write(&path, b"{ not valid json").expect("write");
    let args = VerifyArgs { path };
    let result = handle_genome_verify(&args);
    assert!(result.is_err());
}

#[test]
fn test_handle_genome_compose_empty_genomes() {
    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![],
        output: PathBuf::from("/tmp/out.json"),
    };
    let result = handle_genome_compose(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("No genomes provided"));
}

#[test]
fn test_handle_genome_compose_nonexistent_genome() {
    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![PathBuf::from("/nonexistent/genome.json")],
        output: PathBuf::from("/tmp/out.json"),
    };
    let result = handle_genome_compose(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("GenomeBin not found"));
}

#[test]
fn test_handle_genome_list_nonexistent_storage() {
    let storage = PathBuf::from("/nonexistent_biomeos_test_xyz/biomeos/genomes");
    let result = handle_genome_list_at(&storage);
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_create_success_with_temp_binary() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("fake-binary");
    std::fs::write(&binary, b"#!/bin/sh\necho test").expect("write binary");
    let output = temp.path().join("genome.json");

    let args = CreateArgs {
        binary,
        output: output.clone(),
        arch: "x86_64".to_string(),
        name: Some("test-genome".to_string()),
        version: Some("1.0.0".to_string()),
        description: Some("Test genome".to_string()),
    };
    let result = handle_genome_create(args);
    assert!(result.is_ok(), "create should succeed: {:?}", result.err());
    assert!(output.exists(), "output genome should exist");
}

#[test]
fn test_handle_genome_create_uses_name_from_binary_when_not_provided() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("my-primal");
    std::fs::write(&binary, b"binary").expect("write");
    let output = temp.path().join("out.json");

    let args = CreateArgs {
        binary,
        output,
        arch: "x86_64".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_create_invalid_arch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let binary = temp.path().join("b");
    std::fs::write(&binary, b"x").expect("write");

    let args = CreateArgs {
        binary,
        output: temp.path().join("out.json"),
        arch: "invalid-arch".to_string(),
        name: None,
        version: None,
        description: None,
    };
    let result = handle_genome_create(args);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid architecture")
    );
}

#[test]
fn test_handle_genome_list_with_genomes_under_xdg() {
    let data = tempfile::tempdir().expect("tempdir");
    let store = get_genome_storage_dir_with(Some(data.path().to_str().expect("utf8 path")));
    std::fs::create_dir_all(&store).expect("mkdir genomes");

    let binary = data.path().join("list-bin");
    std::fs::write(&binary, b"x").expect("binary");
    let out = store.join("listed-genome.json");

    let args = CreateArgs {
        binary,
        output: out,
        arch: "x86_64".to_string(),
        name: Some("listed-genome".to_string()),
        version: None,
        description: None,
    };
    handle_genome_create(args).expect("create in storage dir");

    let result = handle_genome_list_at(&store);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_handle_genome_self_replicate() {
    let result = handle_genome_self_replicate();
    assert!(result.is_ok());
}

#[test]
fn test_handle_genome_list_skips_invalid_genome_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = get_genome_storage_dir_with(Some(temp.path().to_str().expect("utf8")));
    std::fs::create_dir_all(&store).expect("mkdir");
    std::fs::write(store.join("corrupt.json"), b"{ not json").expect("write");
    assert!(handle_genome_list_at(&store).is_ok());
}

#[test]
fn test_handle_genome_compose_two_genomes_success() {
    let temp = tempfile::tempdir().expect("tempdir");
    let b1 = temp.path().join("b1");
    let b2 = temp.path().join("b2");
    std::fs::write(&b1, b"bin1").unwrap();
    std::fs::write(&b2, b"bin2").unwrap();

    let g1 = temp.path().join("g1.json");
    let g2 = temp.path().join("g2.json");
    let out = temp.path().join("composed.json");

    // TOWER composition requires manifest names `beardog` and `songbird`
    handle_genome_create(CreateArgs {
        binary: b1,
        output: g1.clone(),
        arch: "x86_64".to_string(),
        name: Some("beardog".to_string()),
        version: None,
        description: None,
    })
    .unwrap();
    handle_genome_create(CreateArgs {
        binary: b2,
        output: g2.clone(),
        arch: "x86_64".to_string(),
        name: Some("songbird".to_string()),
        version: None,
        description: None,
    })
    .unwrap();

    let args = ComposeArgs {
        name: "tower".to_string(),
        nucleus_type: "TOWER".to_string(),
        genomes: vec![g1, g2],
        output: out.clone(),
    };
    assert!(handle_genome_compose(&args).is_ok());
    assert!(out.exists());
}
