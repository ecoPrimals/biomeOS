// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
#[test]
fn test_dist_manifest_serialization() {
    let manifest = DistManifest {
        version: "1.0.0".to_string(),
        generated: "2026-02-12".to_string(),
        primals: HashMap::new(),
        atomics: HashMap::new(),
    };
    let json = serde_json::to_string(&manifest).expect("serialize");
    assert!(json.contains("1.0.0"));
}

#[test]
fn test_dist_manifest_with_primals_and_atomics() {
    let mut primals = HashMap::new();
    primals.insert(
        "beardog".to_string(),
        PrimalInfo {
            name: "BearDog".to_string(),
            description: "Security".to_string(),
            latest: "0.9.0".to_string(),
            versions: vec!["0.9.0".to_string()],
            architectures: vec!["x86_64-linux-musl".to_string()],
            capabilities: vec!["crypto".to_string()],
            ecobin_grade: "A++".to_string(),
        },
    );
    let mut atomics = HashMap::new();
    atomics.insert(
        "full-stack".to_string(),
        AtomicInfo {
            name: "Full Stack".to_string(),
            description: "Complete bundle".to_string(),
            primals: vec!["beardog".to_string()],
            latest: "1.0.0".to_string(),
            versions: vec!["1.0.0".to_string()],
        },
    );
    let manifest = DistManifest {
        version: "2.0.0".to_string(),
        generated: "2026-03-11".to_string(),
        primals,
        atomics,
    };
    let json = serde_json::to_string(&manifest).expect("serialize");
    assert!(json.contains("BearDog"));
    assert!(json.contains("full-stack"));
    let deserialized: DistManifest = serde_json::from_str(&json).expect("round-trip deserialize");
    assert_eq!(deserialized.version, "2.0.0");
    assert_eq!(deserialized.primals.len(), 1);
    assert_eq!(deserialized.atomics.len(), 1);
}

#[test]
fn test_primal_info_serialization() {
    let info = PrimalInfo {
        name: "BearDog".to_string(),
        description: "Security primal".to_string(),
        latest: "0.9.0".to_string(),
        versions: vec!["0.9.0".to_string()],
        architectures: vec!["x86_64-linux-musl".to_string()],
        capabilities: vec!["crypto".to_string()],
        ecobin_grade: "A++".to_string(),
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("BearDog"));
    assert!(json.contains("A++"));
}

#[test]
fn test_atomic_info_serialization() {
    let info = AtomicInfo {
        name: "Full Stack".to_string(),
        description: "All primals".to_string(),
        primals: vec!["beardog".to_string(), "songbird".to_string()],
        latest: "1.0.0".to_string(),
        versions: vec!["0.9.0".to_string(), "1.0.0".to_string()],
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("Full Stack"));
    assert!(json.contains("beardog"));
    let deserialized: AtomicInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.primals.len(), 2);
}

#[test]
fn test_checksum_response_serialization() {
    let resp = ChecksumResponse {
        primal: "beardog".to_string(),
        version: "0.9.0".to_string(),
        arch: "x86_64-linux-musl".to_string(),
        sha256: "abc123".to_string(),
        size: 1_234_567,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("abc123"));
    assert!(json.contains("1234567"));
}

#[test]
fn test_dist_error_serialization() {
    let err = DistError {
        error: "Genome distribution not configured".to_string(),
        code: "GENOMEBIN_NOT_FOUND".to_string(),
    };
    let json = serde_json::to_string(&err).expect("serialize");
    assert!(json.contains("GENOMEBIN_NOT_FOUND"));
    assert!(json.contains("Genome distribution not configured"));
}

#[tokio::test]
async fn test_get_manifest_success() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let manifest_content = r#"
[manifest]
version = "2.0.0"
generated = "2026-03-11"

[primals.beardog]
name = "BearDog"
description = "Security primal"
latest = "0.9.0"
versions = ["0.9.0", "0.8.0"]
architectures = ["x86_64-linux-musl"]
capabilities = ["crypto"]
ecobin_grade = "A++"

[atomics.full]
name = "Full Stack"
primals = ["beardog"]
latest = "1.0.0"
versions = ["1.0.0"]
"#;
    std::fs::write(temp.path().join("manifest.toml"), manifest_content).expect("write manifest");
    let result = get_manifest_from(temp.path().to_path_buf()).await;
    let json = result.expect("get_manifest should succeed");
    assert_eq!(json.version, "2.0.0");
    assert_eq!(json.generated, "2026-03-11");
    assert!(json.primals.contains_key("beardog"));
    assert_eq!(json.primals["beardog"].name, "BearDog");
    assert_eq!(json.primals["beardog"].latest, "0.9.0");
    assert!(json.atomics.contains_key("full"));
}

#[tokio::test]
async fn test_get_manifest_manifest_file_missing() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let result = get_manifest_from(temp.path().to_path_buf()).await;
    let Err((status, body)) = result else {
        panic!("expected Err when manifest.toml missing");
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "MANIFEST_READ_ERROR");
}

#[tokio::test]
async fn test_get_manifest_parse_error() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(temp.path().join("manifest.toml"), "invalid toml {{{")
        .expect("write bad manifest");
    let result = get_manifest_from(temp.path().to_path_buf()).await;
    let Err((status, body)) = result else {
        panic!("expected Err for invalid TOML");
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "MANIFEST_PARSE_ERROR");
}

#[tokio::test]
async fn test_get_manifest_partial_primal_fields() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let manifest_content = r#"
[manifest]
version = "1.0"

[primals.minimal]
latest = "0.1.0"
"#;
    std::fs::write(temp.path().join("manifest.toml"), manifest_content).expect("write manifest");
    let result = get_manifest_from(temp.path().to_path_buf()).await;
    let json = result.expect("should succeed with partial fields");
    assert!(json.primals.contains_key("minimal"));
    let p = &json.primals["minimal"];
    assert_eq!(p.name, "minimal");
    assert_eq!(p.description, "");
    assert_eq!(p.latest, "0.1.0");
    assert!(p.versions.is_empty());
    assert!(p.architectures.is_empty());
    assert_eq!(p.ecobin_grade, "unknown");
}

#[tokio::test]
async fn test_get_latest_success() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let manifest_content = r#"
[manifest]
version = "1.0"
generated = "2026"

[primals.beardog]
name = "BearDog"
latest = "0.9.0"
versions = ["0.9.0"]
architectures = ["x86_64-linux-musl"]
"#;
    std::fs::write(temp.path().join("manifest.toml"), manifest_content).expect("write manifest");
    let result = get_latest_from(temp.path().to_path_buf(), "beardog".to_string()).await;
    let json = result.expect("get_latest should succeed");
    assert_eq!(json["primal"], "beardog");
    assert_eq!(json["latest"], "0.9.0");
}

#[tokio::test]
async fn test_get_latest_primal_not_found() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("manifest.toml"),
        "[manifest]\nversion = \"1.0\"",
    )
    .expect("write manifest");
    let result = get_latest_from(temp.path().to_path_buf(), "nonexistent-primal".to_string()).await;
    let Err((status, body)) = result else {
        panic!("expected Err for unknown primal");
    };
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.code, "PRIMAL_NOT_FOUND");
}

#[tokio::test]
async fn test_get_checksum_success() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("manifest.toml"),
        "[manifest]\nversion = \"1.0\"",
    )
    .expect("write manifest");
    let checksums_content = r#"
[beardog]
[beardog."0.9.0"]
[beardog."0.9.0"."x86_64-linux-musl"]
sha256 = "deadbeef123"
size = 999
"#;
    std::fs::write(temp.path().join("checksums.toml"), checksums_content).expect("write checksums");
    let result = get_checksum_from(
        temp.path().to_path_buf(),
        "beardog".to_string(),
        "0.9.0".to_string(),
        "x86_64-linux-musl".to_string(),
    )
    .await;
    let json = result.expect("get_checksum should succeed");
    assert_eq!(json.primal, "beardog");
    assert_eq!(json.version, "0.9.0");
    assert_eq!(json.arch, "x86_64-linux-musl");
    assert_eq!(json.sha256, "deadbeef123");
    assert_eq!(json.size, 999);
}

#[tokio::test]
async fn test_get_checksum_not_found() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("manifest.toml"),
        "[manifest]\nversion = \"1.0\"",
    )
    .expect("write manifest");
    std::fs::write(
        temp.path().join("checksums.toml"),
        "[other]\nversion = \"1.0\"",
    )
    .expect("write checksums");
    let result = get_checksum_from(
        temp.path().to_path_buf(),
        "beardog".to_string(),
        "0.9.0".to_string(),
        "x86_64-linux-musl".to_string(),
    )
    .await;
    let Err((status, body)) = result else {
        panic!("expected Err for missing checksum");
    };
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.code, "CHECKSUM_NOT_FOUND");
}

#[tokio::test]
async fn test_get_checksum_checksums_file_missing() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("manifest.toml"),
        "[manifest]\nversion = \"1.0\"",
    )
    .expect("write manifest");
    let result = get_checksum_from(
        temp.path().to_path_buf(),
        "beardog".to_string(),
        "0.9.0".to_string(),
        "x86_64".to_string(),
    )
    .await;
    let Err((status, body)) = result else {
        panic!("expected Err when checksums.toml missing");
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "CHECKSUMS_READ_ERROR");
}

#[tokio::test]
async fn test_get_manifest_genome_bin_not_configured() {
    let result = match discovery::get_genome_bin_path_with(None, &[]) {
        Some(p) => get_manifest_from(p).await.map(Json),
        None => Err(genome_bin_not_found_err()),
    };
    let Err((status, body)) = result else {
        panic!("expected Err when genome bin is not discoverable");
    };
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body.code, "GENOMEBIN_NOT_FOUND");
}

#[tokio::test]
async fn test_get_latest_genome_bin_not_configured() {
    let result = match discovery::get_genome_bin_path_with(None, &[]) {
        Some(p) => get_latest_from(p, "any-primal".to_string()).await,
        None => Err(genome_bin_not_found_err()),
    };
    let Err((status, body)) = result else {
        panic!("expected Err when genome bin is not discoverable");
    };
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body.code, "GENOMEBIN_NOT_FOUND");
}

#[tokio::test]
async fn test_get_checksum_genome_bin_not_configured() {
    let result = match discovery::get_genome_bin_path_with(None, &[]) {
        Some(p) => get_checksum_from(p, "p".to_string(), "v".to_string(), "arch".to_string()).await,
        None => Err(genome_bin_not_found_err()),
    };
    let Err((status, body)) = result else {
        panic!("expected Err when genome bin is not discoverable");
    };
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body.code, "GENOMEBIN_NOT_FOUND");
}

#[tokio::test]
async fn test_get_checksum_from_invalid_toml_checksums() {
    let temp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        temp.path().join("manifest.toml"),
        "[manifest]\nversion = \"1.0\"",
    )
    .expect("write manifest");
    std::fs::write(temp.path().join("checksums.toml"), "invalid toml {{{")
        .expect("write bad checksums");
    let result = get_checksum_from(
        temp.path().to_path_buf(),
        "beardog".to_string(),
        "0.9.0".to_string(),
        "x86_64".to_string(),
    )
    .await;
    let Err((status, body)) = result else {
        panic!("expected Err for invalid checksums TOML");
    };
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "CHECKSUMS_PARSE_ERROR");
}
