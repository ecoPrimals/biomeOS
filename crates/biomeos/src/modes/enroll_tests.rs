// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::*;

#[test]
fn test_resolve_device_id_explicit() {
    let id = resolve_device_id(Some("custom-device-123"));
    assert_eq!(id, "custom-device-123");
}

#[test]
fn test_resolve_device_id_empty_string_filters() {
    let id = resolve_device_id(Some(""));
    assert!(!id.is_empty());
}

#[test]
fn test_resolve_device_id_whitespace_only_passes() {
    let id = resolve_device_id(Some("   "));
    assert_eq!(id, "   ");
}

#[test]
fn test_resolve_device_id_none_generates() {
    let id = resolve_device_id(None);
    assert!(!id.is_empty());
    assert!(id.len() >= 32);
}

#[test]
fn test_validate_enrollment_paths_family_seed_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    let family = temp.path().join("nonexistent.family.seed");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(matches!(
        result,
        Err(EnrollmentValidationError::FamilySeedNotFound)
    ));
}

#[test]
fn test_validate_enrollment_paths_already_enrolled() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    std::fs::write(&lineage, "existing").expect("write lineage");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(matches!(
        result,
        Err(EnrollmentValidationError::AlreadyEnrolled)
    ));
}

#[test]
fn test_validate_enrollment_paths_force_ok() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    std::fs::write(&lineage, "existing").expect("write lineage");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, true);
    assert!(result.is_ok());
}

#[test]
fn test_validate_enrollment_paths_fresh_enrollment() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage = temp.path().join(".lineage.seed");
    let family = temp.path().join(".family.seed");
    std::fs::write(&family, "seed").expect("write family");
    let result = validate_enrollment_paths(&lineage, &family, false);
    assert!(
        result.is_ok(),
        "fresh enrollment (no lineage) should succeed"
    );
}

#[test]
fn test_enrollment_validation_error_display() {
    let already = EnrollmentValidationError::AlreadyEnrolled;
    assert!(
        already.to_string().contains("already enrolled"),
        "AlreadyEnrolled display: {}",
        already
    );
    assert!(already.to_string().contains("force"));

    let not_found = EnrollmentValidationError::FamilySeedNotFound;
    assert!(
        not_found.to_string().contains("not found"),
        "FamilySeedNotFound display: {}",
        not_found
    );
}

#[test]
fn test_get_machine_id() {
    let _ = get_machine_id();
}

#[test]
fn test_discover_security_socket_handles_missing() {
    assert!(discover_security_socket_in(None, None).is_none());
}

#[tokio::test]
async fn test_run_fails_when_family_seed_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let args = EnrollArgs {
        family_id: "test-family".to_string(),
        node_id: "test-node".to_string(),
        device_id: Some("test-device-123".to_string()),
        family_seed: temp.path().join("nonexistent.family.seed"),
        lineage_seed: temp.path().join(".lineage.seed"),
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(result.is_err(), "run should fail when family seed missing");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Family seed not found"),
        "Expected family seed error: {err}"
    );
}

#[tokio::test]
async fn test_run_fails_when_security_socket_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed-content").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test-family".to_string(),
        node_id: "test-node".to_string(),
        device_id: Some("test-device-123".to_string()),
        family_seed,
        lineage_seed: temp.path().join(".lineage.seed"),
        security_socket: None,
        security_socket_dir: Some(temp.path().to_path_buf()),
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_err(),
        "run should fail when BearDog socket not found"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("BearDog") || err.to_string().contains("socket"),
        "Expected BearDog/socket error: {err}"
    );
}

#[tokio::test]
async fn test_run_without_device_id_uses_resolve_fallback() {
    let temp = tempfile::tempdir().expect("temp dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: None,
        family_seed,
        lineage_seed: temp.path().join(".lineage.seed"),
        security_socket: None,
        security_socket_dir: Some(temp.path().to_path_buf()),
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_err(),
        "run without device_id should fail at BearDog (or family seed) when socket missing"
    );
}

#[tokio::test]
async fn test_run_uses_device_id_when_provided() {
    let temp = tempfile::tempdir().expect("temp dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("custom-device-id-xyz".to_string()),
        family_seed,
        lineage_seed: temp.path().join(".lineage.seed"),
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_returns_ok_when_already_enrolled_no_force() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage_seed = temp.path().join(".lineage.seed");
    std::fs::write(&lineage_seed, "existing-lineage-seed").expect("write lineage");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-1".to_string()),
        family_seed,
        lineage_seed,
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_ok(),
        "already enrolled should return Ok (early exit): {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_returns_ok_when_already_enrolled_but_load_lineage_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage_seed = temp.path().join(".lineage.seed");
    std::fs::write(&lineage_seed, "x").expect("write lineage");
    let lineage_json = lineage_seed.with_extension("json");
    std::fs::write(&lineage_json, "{invalid json").expect("write invalid json");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-1".to_string()),
        family_seed,
        lineage_seed,
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_ok(),
        "already enrolled with unloadable lineage should still return Ok: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_force_attempts_enrollment_when_lineage_exists() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage_seed = temp.path().join(".lineage.seed");
    std::fs::write(&lineage_seed, "existing-lineage-seed").expect("write lineage");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-1".to_string()),
        family_seed,
        lineage_seed,
        security_socket: None,
        security_socket_dir: None,
        force: true,
    };
    let result = run(args).await;
    assert!(
        result.is_err(),
        "force re-enroll without BearDog should fail: {result:?}"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("BearDog") || err.to_string().contains("socket"),
        "Expected BearDog/socket error: {err}"
    );
}

#[tokio::test]
async fn test_run_fails_when_lineage_seed_is_directory() {
    let temp = tempfile::tempdir().expect("temp dir");
    let lineage_seed = temp.path().join(".lineage.seed");
    std::fs::create_dir_all(&lineage_seed).expect("create lineage dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "test-seed").expect("write family seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-1".to_string()),
        family_seed,
        lineage_seed,
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_ok(),
        "lineage_seed as dir: exists() is true, early exit"
    );
}

#[test]
fn test_enroll_args_construction() {
    let args = EnrollArgs {
        family_id: "fam123".to_string(),
        node_id: "tower".to_string(),
        device_id: Some("dev456".to_string()),
        family_seed: std::path::PathBuf::from(".family.seed"),
        lineage_seed: std::path::PathBuf::from(".lineage.seed"),
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    assert_eq!(args.family_id, "fam123");
    assert_eq!(args.node_id, "tower");
    assert_eq!(args.device_id, Some("dev456".to_string()));
    assert!(!args.force);
    assert_eq!(args.family_seed, std::path::PathBuf::from(".family.seed"));
    assert_eq!(args.lineage_seed, std::path::PathBuf::from(".lineage.seed"));
}

#[test]
fn test_enroll_args_with_custom_paths() {
    let custom_family = PathBuf::from("/custom/.family.seed");
    let custom_lineage = PathBuf::from("/custom/.lineage.seed");
    let args = EnrollArgs {
        family_id: "f".to_string(),
        node_id: "n".to_string(),
        device_id: None,
        family_seed: custom_family.clone(),
        lineage_seed: custom_lineage.clone(),
        security_socket: Some("/tmp/beardog.sock".to_string()),
        security_socket_dir: None,
        force: true,
    };
    assert_eq!(args.family_seed, custom_family);
    assert_eq!(args.lineage_seed, custom_lineage);
    assert!(args.force);
    assert_eq!(args.security_socket, Some("/tmp/beardog.sock".to_string()));
}

#[test]
fn test_discover_security_socket_finds_default_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let biomeos_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
    let socket_path = biomeos_dir.join("beardog.sock");
    std::fs::write(&socket_path, "").expect("create socket file");

    let result = discover_security_socket_in(Some(temp.path()), None);
    assert!(
        result.is_some(),
        "Should find socket when socket_dir/biomeos/beardog.sock exists"
    );
    assert!(result.unwrap().contains("beardog.sock"));
}

#[test]
fn test_discover_security_socket_finds_family_suffixed_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let biomeos_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).expect("create biomeos dir");
    let socket_path = biomeos_dir.join("beardog-testfamily123.sock");
    std::fs::write(&socket_path, "").expect("create socket file");

    let result = discover_security_socket_in(Some(temp.path()), Some("testfamily123"));
    assert!(
        result.is_some(),
        "Should find beardog-{{family_id}}.sock when socket_dir and family_id provided"
    );
    assert!(result.unwrap().contains("beardog-testfamily123.sock"));
}

#[tokio::test]
async fn test_run_fails_when_family_seed_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "").expect("write empty family seed");
    let lineage_seed = temp.path().join(".lineage.seed");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-xyz".to_string()),
        family_seed,
        lineage_seed,
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_err(),
        "run with empty family seed should fail at BearDog or derivation"
    );
}

#[tokio::test]
async fn test_run_fails_when_security_socket_connection_refused() {
    let temp = tempfile::tempdir().expect("temp dir");
    let family_seed = temp.path().join(".family.seed");
    std::fs::write(&family_seed, "valid-seed-content").expect("write family seed");
    let lineage_seed = temp.path().join(".lineage.seed");
    let nonexistent_socket = temp.path().join("nonexistent.sock");

    let args = EnrollArgs {
        family_id: "test".to_string(),
        node_id: "node".to_string(),
        device_id: Some("device-xyz".to_string()),
        family_seed,
        lineage_seed,
        security_socket: Some(nonexistent_socket.to_string_lossy().to_string()),
        security_socket_dir: None,
        force: false,
    };
    let result = run(args).await;
    assert!(
        result.is_err(),
        "run with nonexistent BearDog socket should fail"
    );
}
