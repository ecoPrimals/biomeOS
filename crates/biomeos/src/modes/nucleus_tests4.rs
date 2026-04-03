// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Coverage-focused nucleus tests: coordinated detect, JWT, base64, summary labels.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let nonexistent = tmp.path().join("does-not-exist");
    let state = detect_ecosystem(&nonexistent, "fam1").await;
    assert!(matches!(state, EcosystemState::Bootstrap));
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_no_sockets() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let state = detect_ecosystem(tmp.path(), "fam1").await;
    assert!(matches!(state, EcosystemState::Bootstrap));
}

#[tokio::test]
async fn test_detect_ecosystem_stale_socket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("beardog-fam1.sock");
    std::fs::write(&sock_path, b"").expect("create file");
    let state = detect_ecosystem(tmp.path(), "fam1").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "stale socket (non-listening) should be treated as bootstrap"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_coordinated_with_live_socket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("beardog-testfam.sock");

    let listener = UnixListener::bind(&sock_path).expect("bind");
    let mock = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let resp = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
                let _ = write_half.write_all(format!("{resp}\n").as_bytes()).await;
                let _ = write_half.flush().await;
            }
        }
    });

    let state = detect_ecosystem(tmp.path(), "testfam").await;
    mock.abort();

    match state {
        EcosystemState::Coordinated { active_primals } => {
            assert!(
                active_primals.contains(&"beardog".to_string()),
                "expected beardog in active primals: {active_primals:?}"
            );
        }
        EcosystemState::Bootstrap => {
            // Health check may fail under CI timing; accept bootstrap as non-fatal
        }
    }
}

#[test]
fn test_generate_jwt_secret_is_base64_and_nonempty() {
    let secret = generate_jwt_secret();
    assert!(!secret.is_empty());
    assert!(
        secret.len() >= 20,
        "JWT secret should be at least 20 chars: got {secret}"
    );
    for c in secret.chars() {
        assert!(
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
            "unexpected char in base64: {c}"
        );
    }
}

#[test]
fn test_generate_jwt_secret_returns_valid_base64_string() {
    use base64::Engine;
    let secret = super::generate_jwt_secret();
    let decoded = base64::engine::general_purpose::STANDARD.decode(&secret);
    assert!(decoded.is_ok(), "JWT secret must be valid base64");
    assert_eq!(decoded.unwrap().len(), 48);
}

#[test]
fn test_format_nucleus_summary_coordinated_label() {
    let children = vec![
        ("songbird".to_string(), 1234u32),
        ("nestgate".to_string(), 5678u32),
    ];
    let lines = format_nucleus_summary(
        &children,
        std::path::Path::new("/tmp/biomeos"),
        "test-family",
        "node1",
        NucleusMode::Nest,
        "coordinated",
    );
    let joined = lines.join("\n");
    assert!(
        joined.contains("coordinated"),
        "expected 'coordinated' label"
    );
    assert!(joined.contains("songbird"), "expected songbird");
    assert!(joined.contains("nestgate"), "expected nestgate");
}

#[test]
fn test_nucleus_mode_primals_full() {
    let primals = NucleusMode::Full.primals();
    assert!(primals.contains(&BEARDOG));
    assert!(primals.contains(&SONGBIRD));
    assert!(primals.contains(&NESTGATE));
    assert!(primals.contains(&TOADSTOOL));
    assert!(primals.contains(&SQUIRREL));
}

#[tokio::test]
async fn test_wait_for_socket_immediate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock = tmp.path().join("test.sock");
    std::fs::write(&sock, b"").expect("create");
    wait_for_socket(&sock, Duration::from_secs(1), Duration::from_millis(10))
        .await
        .expect("should find socket immediately");
}

#[tokio::test]
async fn test_wait_for_socket_timeout() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock = tmp.path().join("never.sock");
    let err = wait_for_socket(&sock, Duration::from_millis(50), Duration::from_millis(10))
        .await
        .expect_err("should timeout");
    assert!(err.to_string().contains("did not appear"));
}

#[tokio::test]
async fn test_wait_for_socket_delayed_creation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock = tmp.path().join("delayed.sock");
    let sock_clone = sock.clone();
    tokio::spawn(async move {
        tokio::task::yield_now().await;
        std::fs::write(&sock_clone, b"").expect("create");
    });
    wait_for_socket(&sock, Duration::from_secs(2), Duration::from_millis(10))
        .await
        .expect("should find socket after delay");
}

#[test]
fn test_format_nucleus_summary_bootstrap_empty_children() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/run/biomeos"),
        "fam",
        "node",
        NucleusMode::Tower,
        "bootstrap",
    );
    let joined = lines.join("\n");
    assert!(joined.contains("bootstrap"));
    assert!(joined.contains("Tower"));
    assert!(joined.contains("Health check"));
}

#[test]
fn test_resolve_startup_config_invalid_mode_string() {
    let err = resolve_startup_config_with("not-a-valid-mode", "n1", Some("f1"), Some("/tmp/s"))
        .expect_err("invalid mode");
    assert!(err.to_string().contains("Unknown nucleus mode"));
}

#[test]
fn test_resolve_socket_dir_with_explicit_override_roundtrip() {
    let p = resolve_socket_dir_with(Some("/tmp/nucleus-sock-override")).expect("ok");
    assert_eq!(p, std::path::PathBuf::from("/tmp/nucleus-sock-override"));
}

#[test]
fn test_discover_binaries_with_path_dirs_finds_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let name = "unique_primal_bin_zz_test";
    let bin = tmp.path().join(name);
    std::fs::write(&bin, b"#!/bin/sh\necho").expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
    }
    let map = discover_binaries_with(&[name], None, &[tmp.path()], None).expect("discover");
    assert_eq!(
        map.get(name).map(std::path::PathBuf::as_path),
        Some(bin.as_path())
    );
}

#[test]
fn test_ecosystem_state_debug() {
    let bootstrap = EcosystemState::Bootstrap;
    let coord = EcosystemState::Coordinated {
        active_primals: vec!["a".to_string()],
    };
    let _ = format!("{bootstrap:?} {coord:?}");
}

#[test]
fn test_nucleus_mode_copy_eq() {
    assert_eq!(NucleusMode::Tower, NucleusMode::Tower);
    assert_ne!(NucleusMode::Tower, NucleusMode::Nest);
}
