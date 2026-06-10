// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::io::Write;
#[test]
fn test_lineage_verification_construction() {
    let v = LineageVerification {
        valid: true,
        family_id: Some("fam-123".to_string()),
        node_id: Some("node-456".to_string()),
        details: vec!["detail1".to_string()],
        warnings: vec!["warn1".to_string()],
    };
    assert!(v.valid);
    assert_eq!(v.family_id.as_deref(), Some("fam-123"));
    assert_eq!(v.node_id.as_deref(), Some("node-456"));
    assert_eq!(v.details.len(), 1);
    assert_eq!(v.warnings.len(), 1);
}

#[tokio::test]
async fn test_run_path_not_found_returns_error() {
    let path = PathBuf::from("/nonexistent/path/that/does/not/exist/12345");
    let result = run(path, false).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Path not found") || err.to_string().contains("not found"),
        "Expected path not found error: {err}"
    );
}

#[tokio::test]
async fn test_verify_lineage_directory_basic() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().to_path_buf();

    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(v.details.contains(&"Directory exists".to_string()));
    assert!(v.warnings.contains(&"No manifest.toml found".to_string()));
    assert!(v.warnings.contains(&"No .family.seed found".to_string()));
    assert!(
        v.warnings
            .contains(&"No primals directory found".to_string())
    );
}

#[tokio::test]
async fn test_verify_lineage_directory_with_manifest() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(
        &manifest_path,
        r#"
family_id = "test-family-123"
node_id = "test-node-456"
"#,
    )
    .expect("write manifest");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert_eq!(v.family_id.as_deref(), Some("test-family-123"));
    assert_eq!(v.node_id.as_deref(), Some("test-node-456"));
    assert!(v.details.contains(&"Manifest found".to_string()));
}

#[tokio::test]
async fn test_verify_lineage_directory_with_valid_seed() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let seed_path = dir.path().join(".family.seed");
    let mut f = std::fs::File::create(&seed_path).expect("create seed");
    f.write_all(&[0u8; 64]).expect("write 64 bytes");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(
        v.details
            .contains(&"Family seed valid (64 bytes)".to_string())
    );
}

#[tokio::test]
async fn test_verify_lineage_directory_with_invalid_seed_size() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let seed_path = dir.path().join(".family.seed");
    let mut f = std::fs::File::create(&seed_path).expect("create seed");
    f.write_all(&[0u8; 32]).expect("write 32 bytes");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(!v.valid);
    assert!(v.warnings.iter().any(|w| w.contains("invalid size")));
    assert!(v.warnings.iter().any(|w| w.contains("32 bytes")));
}

#[tokio::test]
async fn test_verify_lineage_directory_with_primals() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let primals_dir = dir.path().join("primals");
    std::fs::create_dir(&primals_dir).expect("create primals dir");
    std::fs::File::create(primals_dir.join("beardog")).expect("create binary");
    std::fs::File::create(primals_dir.join("songbird")).expect("create binary");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(
        v.details
            .iter()
            .any(|d| d.contains("Primals") && d.contains("2 binaries")),
        "Expected primals directory detail, got: {:?}",
        v.details
    );
}

#[tokio::test]
async fn test_verify_lineage_single_file_64_bytes() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    let mut f = file.reopen().expect("reopen");
    f.write_all(&[0u8; 64]).expect("write 64 bytes");
    drop(f);

    let path = file.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(
        v.details
            .contains(&"Valid seed file (64 bytes)".to_string())
    );
}

#[tokio::test]
async fn test_verify_lineage_single_file_32_bytes() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    let mut f = file.reopen().expect("reopen");
    f.write_all(&[0u8; 32]).expect("write 32 bytes");
    drop(f);

    let path = file.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(
        v.details
            .contains(&"Valid hash file (32 bytes)".to_string())
    );
}

#[tokio::test]
async fn test_verify_lineage_single_file_unknown_size() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    let mut f = file.reopen().expect("reopen");
    f.write_all(&[0u8; 100]).expect("write 100 bytes");
    drop(f);

    let path = file.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(v.warnings.iter().any(|w| w.contains("Unknown file format")));
    assert!(v.warnings.iter().any(|w| w.contains("100 bytes")));
}

#[tokio::test]
async fn test_verify_lineage_manifest_partial() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "family_id = \"only-family\"\n").expect("write manifest");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert_eq!(v.family_id.as_deref(), Some("only-family"));
    assert_eq!(v.node_id, None);
}

#[tokio::test]
async fn test_verify_lineage_manifest_invalid_toml() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "invalid toml [ broken \n").expect("write manifest");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(v.details.contains(&"Manifest found".to_string()));
    assert_eq!(v.family_id, None);
    assert_eq!(v.node_id, None);
}

#[tokio::test]
async fn test_verify_lineage_manifest_empty() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "").expect("write manifest");

    let path = dir.path().to_path_buf();
    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
}

#[tokio::test]
async fn test_run_success_displays_details() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "family_id = \"f\"\nnode_id = \"n\"\n").expect("write");
    let seed_path = dir.path().join(".family.seed");
    std::fs::write(&seed_path, [0u8; 64]).expect("write seed");

    let result = run(dir.path().to_path_buf(), false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_fails_when_verification_invalid() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let seed_path = dir.path().join(".family.seed");
    std::fs::write(&seed_path, [0u8; 32]).expect("write invalid 32-byte seed");

    let result = run(dir.path().to_path_buf(), false).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("verification failed") || err.to_string().contains("failed"),
        "Expected verification failure: {err}"
    );
}

#[tokio::test]
async fn test_verify_lineage_detailed_skips_crypto_when_no_beardog() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "family_id = \"f\"\nnode_id = \"n\"\n").expect("write");
    let seed_path = dir.path().join(".family.seed");
    std::fs::write(&seed_path, [0u8; 64]).expect("write seed");

    let result = verify_lineage_with_security_provider(
        &dir.path().to_path_buf(),
        true,
        "nonexistent-beardog-xyz",
    )
    .await;

    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(
        v.warnings
            .iter()
            .any(|w| w.contains("Cryptographic") || w.contains("skipped"))
            || v.details
                .iter()
                .any(|d| d.contains("skipped") || d.contains("no seed")),
        "Expected crypto skip warning when BearDog unavailable: {:?}",
        v.warnings
    );
}

#[tokio::test]
async fn test_verify_lineage_detailed_no_seed_skips_crypto() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, "family_id = \"f\"\n").expect("write");
    std::fs::create_dir(dir.path().join("primals")).expect("create primals");

    let result = verify_lineage(&dir.path().to_path_buf(), true).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(v.warnings.contains(&"No .family.seed found".to_string()));
}

#[tokio::test]
async fn test_verify_lineage_directory_with_both_manifest_and_seed() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(
        &manifest_path,
        "family_id = \"full-fam\"\nnode_id = \"full-node\"\n",
    )
    .expect("write manifest");
    let seed_path = dir.path().join(".family.seed");
    std::fs::write(&seed_path, [0u8; 64]).expect("write seed");
    std::fs::create_dir(dir.path().join("primals")).expect("create primals");

    let result = verify_lineage(&dir.path().to_path_buf(), false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert_eq!(v.family_id.as_deref(), Some("full-fam"));
    assert_eq!(v.node_id.as_deref(), Some("full-node"));
    assert!(v.details.contains(&"Manifest found".to_string()));
    assert!(
        v.details
            .contains(&"Family seed valid (64 bytes)".to_string())
    );
}

#[tokio::test]
async fn test_verify_lineage_empty_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().to_path_buf();

    let result = verify_lineage(&path, false).await;
    let v = result.expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(v.details.contains(&"Directory exists".to_string()));
    assert!(v.warnings.iter().any(|w| w.contains("manifest")));
}

#[test]
fn test_lineage_verification_debug() {
    let v = LineageVerification {
        valid: false,
        family_id: None,
        node_id: None,
        details: vec!["d1".to_string()],
        warnings: vec!["w1".to_string()],
    };
    let debug_str = format!("{v:?}");
    assert!(debug_str.contains("LineageVerification"));
    assert!(debug_str.contains("valid"));
    assert!(debug_str.contains("d1"));
}

#[tokio::test]
async fn test_verify_lineage_path_not_found_returns_error() {
    let path = PathBuf::from("/nonexistent/verify_lineage_path_xyz_999");
    let result = verify_lineage(&path, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_lineage_primals_file_not_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let primals_path = dir.path().join("primals");
    std::fs::write(&primals_path, b"not-a-directory").expect("create primals file");

    let v = verify_lineage(&dir.path().to_path_buf(), false)
        .await
        .expect("verify_lineage should succeed");
    assert!(v.valid);
    assert!(!v.details.iter().any(|d| d.contains("Primals directory")));
}

#[tokio::test]
async fn test_verify_lineage_manifest_invalid_utf8_skips_parse() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let manifest_path = dir.path().join("manifest.toml");
    std::fs::write(&manifest_path, [0xFFu8, 0xFE, 0xFD]).expect("write invalid utf-8");

    let v = verify_lineage(&dir.path().to_path_buf(), false)
        .await
        .expect("verify_lineage should succeed");
    assert!(v.details.contains(&"Manifest found".to_string()));
    assert_eq!(v.family_id, None);
    assert_eq!(v.node_id, None);
}

#[tokio::test]
async fn test_verify_lineage_empty_primals_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join("primals")).expect("create empty primals");

    let v = verify_lineage(&dir.path().to_path_buf(), false)
        .await
        .expect("verify_lineage should succeed");
    assert!(
        v.details
            .iter()
            .any(|d| d == "Primals directory: 0 binaries"),
        "Expected zero-binary primals detail, got: {:?}",
        v.details
    );
}

#[tokio::test]
async fn test_run_success_with_warnings_empty_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let result = run(dir.path().to_path_buf(), false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_single_file_unknown_still_ok() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    let mut f = file.reopen().expect("reopen");
    f.write_all(&[0u8; 9]).expect("write 9 bytes");
    drop(f);

    let result = run(file.path().to_path_buf(), false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_passes_without_family_or_node_ids() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let seed_path = dir.path().join(".family.seed");
    std::fs::write(&seed_path, [0u8; 64]).expect("write seed");
    std::fs::create_dir(dir.path().join("primals")).expect("create primals");

    let result = run(dir.path().to_path_buf(), false).await;
    assert!(result.is_ok());
}
