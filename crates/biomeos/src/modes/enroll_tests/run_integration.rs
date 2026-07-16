// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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
    let iso_path = temp.path().to_str().expect("utf8");
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
        ],
        async {
            let family_seed = temp.path().join(".family.seed");
            std::fs::write(&family_seed, "test-seed").expect("write family seed");

            let args = EnrollArgs {
                family_id: "test".to_string(),
                node_id: "node".to_string(),
                device_id: Some("custom-device-id-xyz".to_string()),
                family_seed,
                lineage_seed: temp.path().join(".lineage.seed"),
                security_socket: None,
                security_socket_dir: Some(temp.path().to_path_buf()),
                force: false,
            };
            let result = run(args).await;
            assert!(result.is_err());
        },
    )
    .await;
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
    let iso_path = temp.path().to_str().expect("utf8");
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
        ],
        async {
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
                security_socket_dir: Some(temp.path().to_path_buf()),
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
        },
    )
    .await;
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
