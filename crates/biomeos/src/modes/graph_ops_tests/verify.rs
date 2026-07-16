// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;

use biomeos_graph::integrity::compute_content_hash;

use super::super::*;

#[tokio::test]
async fn test_verify_fails_when_path_missing() {
    let path = PathBuf::from("/nonexistent/verify/graph.toml");
    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot read graph")
    );
}

#[tokio::test]
async fn test_verify_succeeds_for_unsigned_graph() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("unsigned.toml");
    std::fs::write(&path, "[graph]\nid = \"test\"\n").expect("write graph");

    let result = verify(path).await;
    assert!(
        result.is_ok(),
        "unsigned graph should verify OK: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_verify_fails_on_hash_mismatch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("bad-hash.toml");
    std::fs::write(
        &path,
        "[graph]\nid = \"test\"\n\n[graph.metadata]\ncontent_hash = \"deadbeef\"\n",
    )
    .expect("write graph");

    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Content hash mismatch")
    );
}

#[tokio::test]
async fn test_verify_fails_on_invalid_signature() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("bad-sig.toml");
    let content = "[graph]\nid = \"test\"\n\n[graph.metadata]\n";
    let hash = compute_content_hash(content);
    let signed = format!(
        "{content}content_hash = \"{hash}\"\nsignature = \"not-valid-hex-signature\"\nsigned_by = \"also-not-valid-hex\"\n"
    );
    std::fs::write(&path, signed).expect("write graph");

    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Signature verification failed")
    );
}

#[tokio::test]
async fn test_verify_succeeds_with_matching_hash() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("good-hash.toml");
    let base = "[graph]\nid = \"test\"\n\n[graph.metadata]\n";
    let hash = compute_content_hash(base);
    std::fs::write(&path, format!("{base}content_hash = \"{hash}\"\n")).expect("write graph");

    let result = verify(path).await;
    assert!(
        result.is_ok(),
        "graph with matching hash should verify: {:?}",
        result.err()
    );
}
