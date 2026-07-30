// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::*;

#[test]
fn parse_checksums_basic() {
    let input = "abc123def456  outputs/foo.dat\n789012345678  provenance/bar.json\n";
    let entries = parse_checksums(input);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].hash, "abc123def456");
    assert_eq!(entries[0].path, "outputs/foo.dat");
    assert_eq!(entries[1].path, "provenance/bar.json");
}

#[test]
fn parse_checksums_skips_comments_and_empty() {
    let input = "# header comment\nabc123  file.dat\n\n# another comment\n";
    let entries = parse_checksums(input);
    assert_eq!(entries.len(), 1);
}

#[test]
fn format_checksums_roundtrip() {
    let entries = vec![
        ChecksumEntry {
            hash: "aaa".to_string(),
            path: "a.txt".to_string(),
        },
        ChecksumEntry {
            hash: "bbb".to_string(),
            path: "b.txt".to_string(),
        },
    ];
    let formatted = format_checksums(&entries);
    assert_eq!(formatted, "aaa  a.txt\nbbb  b.txt");
}

fn create_valid_pseudospore(dir: &Path) {
    std::fs::write(
        dir.join("scope.toml"),
        r#"[artifact]
name = "test-spore"
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
        r#"{"artifact":"test-spore","version":"1.0.0","date":"2026-05-27","modules":[{"name":"structural","status":"PASS","checks_total":3,"checks_passed":3}]}"#,
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("receipts")).unwrap();
    std::fs::write(
        dir.join("receipts/environment.toml"),
        r#"[hardware]
cpu = "x86_64"
cores = 8

[software]
os = "Linux"
rust = "1.82"
"#,
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("data/payload.bin"), b"hello world").unwrap();

    let hash = blake3::hash(b"hello world").to_hex().to_string();
    std::fs::write(
        dir.join("receipts/checksums.blake3"),
        format!("{hash}  data/payload.bin\n"),
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
fn load_valid_pseudospore() {
    let tmp = tempfile::TempDir::new().unwrap();
    create_valid_pseudospore(tmp.path());

    let manifest = load_pseudospore(tmp.path());
    assert_eq!(manifest.status, SporeStatus::Valid);
    assert!(manifest.errors.is_empty(), "errors: {:?}", manifest.errors);
    assert_eq!(manifest.scope.artifact.name, "test-spore");
    assert_eq!(manifest.checksums.len(), 1);
}

#[test]
fn verify_checksums_pass() {
    let tmp = tempfile::TempDir::new().unwrap();
    create_valid_pseudospore(tmp.path());

    let mut manifest = load_pseudospore(tmp.path());
    assert!(verify_checksums(&mut manifest));
    assert_eq!(manifest.status, SporeStatus::Verified);
}

#[test]
fn verify_checksums_fail_on_tamper() {
    let tmp = tempfile::TempDir::new().unwrap();
    create_valid_pseudospore(tmp.path());
    std::fs::write(tmp.path().join("data/payload.bin"), b"tampered").unwrap();

    let mut manifest = load_pseudospore(tmp.path());
    assert!(!verify_checksums(&mut manifest));
    assert!(manifest.errors.iter().any(|e| e.contains("mismatch")));
}

#[test]
fn check_completeness_pass() {
    let tmp = tempfile::TempDir::new().unwrap();
    create_valid_pseudospore(tmp.path());

    let mut manifest = load_pseudospore(tmp.path());
    let _ = verify_checksums(&mut manifest);
    assert!(check_completeness(&mut manifest));
    assert_eq!(manifest.status, SporeStatus::Complete);
}

#[test]
fn load_missing_scope_returns_invalid() {
    let tmp = tempfile::TempDir::new().unwrap();
    let manifest = load_pseudospore(tmp.path());
    assert_eq!(manifest.status, SporeStatus::Invalid);
    assert!(manifest.errors.iter().any(|e| e.contains("scope.toml")));
}

#[test]
fn load_wrong_type_reports_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    create_valid_pseudospore(tmp.path());
    std::fs::write(
        tmp.path().join("scope.toml"),
        "[artifact]\nname = \"x\"\nversion = \"1\"\ntype = \"liveSpore\"\n",
    )
    .unwrap();

    let manifest = load_pseudospore(tmp.path());
    assert!(
        manifest
            .errors
            .iter()
            .any(|e| e.contains("expected 'pseudoSpore'"))
    );
}

#[test]
fn compute_checksums_works() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("data")).unwrap();
    std::fs::write(tmp.path().join("data/a.bin"), b"aaa").unwrap();
    std::fs::write(tmp.path().join("data/b.bin"), b"bbb").unwrap();

    let entries = compute_checksums(tmp.path(), &["data"]);
    assert_eq!(entries.len(), 2);
    assert!(entries[0].path.contains("data/"));
}
