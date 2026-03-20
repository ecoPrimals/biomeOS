// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Extra nucleus mode coverage (spawn, health errors, multi-primal ecosystem detection).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::*;
use biomeos_types::primal_names::{BEARDOG, SONGBIRD};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Notify;

#[tokio::test]
async fn test_start_primal_errors_when_binary_path_is_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock = tmp.path().join("beardog-fam.sock");
    let not_executable = tmp.path().join("fake_bin_dir");
    std::fs::create_dir_all(&not_executable).expect("mkdir");

    let err = start_primal(
        "beardog",
        &not_executable,
        &sock,
        "fam",
        "node1",
        tmp.path(),
    )
    .await
    .expect_err("spawn should fail for directory path");
    assert!(
        err.to_string().contains("Failed to spawn") || err.to_string().contains("spawn"),
        "unexpected err: {err}"
    );
}

#[tokio::test]
async fn test_start_primal_spawns_child_with_true_binary() {
    let true_bin = PathBuf::from("/bin/true");
    if !true_bin.is_file() {
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock = tmp.path().join("beardog-fam.sock");
    let mut child = start_primal("beardog", &true_bin, &sock, "fam", "node1", tmp.path())
        .await
        .expect("spawn true as binary");
    assert!(child.id().is_some());
    let _ = child.kill().await;
}

#[tokio::test]
async fn test_health_check_fails_when_rpc_errors_for_every_method() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sock_path = tmp.path().join("beardog-fam.sock");
    let ready = std::sync::Arc::new(Notify::new());
    let ready_c = std::sync::Arc::clone(&ready);
    let sock_c = sock_path.clone();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_c).expect("bind");
        ready_c.notify_one();
        // health then semantic fallback — two connections per health_check
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut r, mut w) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut r)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned();
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            });
            w.write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
        }
    });

    ready.notified().await;
    let result = super::health_check(&sock_path).await;
    server.await.expect("server task");
    assert!(
        result.is_err(),
        "expected health_check failure when all RPCs error"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_coordinated_when_two_primals_respond() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let family = "multi-eco-test";
    let sock_b = tmp.path().join(format!("{BEARDOG}-{family}.sock"));
    let sock_s = tmp.path().join(format!("{SONGBIRD}-{family}.sock"));

    let ready = std::sync::Arc::new(Notify::new());
    let ready_c = std::sync::Arc::clone(&ready);
    let b_path = sock_b.clone();
    let s_path = sock_s.clone();

    let server = tokio::spawn(async move {
        let listener_b = UnixListener::bind(&b_path).expect("bind beardog");
        let listener_s = UnixListener::bind(&s_path).expect("bind songbird");
        ready_c.notify_one();

        // `detect_ecosystem` probes CORE_PRIMALS in order. `health_check` succeeds on the first
        // successful JSON-RPC `health` call — one connection per primal, not two.
        for listener in [&listener_b, &listener_s] {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut r, mut w) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut r)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned();
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "status": "ok" }
            });
            w.write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
        }
    });

    ready.notified().await;

    let state = detect_ecosystem(tmp.path(), family).await;
    server.await.expect("server task");

    match state {
        EcosystemState::Coordinated { active_primals } => {
            assert!(
                active_primals.iter().any(|p| p == BEARDOG),
                "beardog missing: {active_primals:?}"
            );
            assert!(
                active_primals.iter().any(|p| p == SONGBIRD),
                "songbird missing: {active_primals:?}"
            );
        }
        EcosystemState::Bootstrap => {
            panic!("expected Coordinated when two sockets respond, got Bootstrap");
        }
    }
}

#[tokio::test]
async fn test_wait_for_socket_appears_after_short_delay() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("late.sock");
    let path_c = path.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(80)).await;
        std::fs::write(&path_c, b"").expect("touch");
    });
    let result = wait_for_socket(&path, Duration::from_secs(2), Duration::from_millis(20)).await;
    assert!(result.is_ok());
}

#[test]
fn test_format_nucleus_summary_includes_monitoring_line() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/tmp/s"),
        "fam",
        "n1",
        NucleusMode::Tower,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("monitoring")));
}
