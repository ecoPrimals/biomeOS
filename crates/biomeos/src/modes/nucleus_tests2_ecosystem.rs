// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::time::Duration;

#[tokio::test]
async fn test_detect_ecosystem_coordinated_when_socket_responds() {
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::Notify;

    let tmp = tempfile::tempdir().expect("tempdir");
    let family = "coord-test-family";
    let sock_name = format!("beardog-{family}.sock");
    let sock_path = tmp.path().join(&sock_name);

    let ready = Arc::new(Notify::new());
    let ready_c = Arc::clone(&ready);
    let sock_path_c = sock_path.clone();
    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_path_c).expect("bind");
        ready_c.notify_one();
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut r, mut w) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut r)
            .read_line(&mut line)
            .await
            .expect("read");
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse rpc");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {"status": "ok"}
        });
        w.write_all(format!("{resp}\n").as_bytes())
            .await
            .expect("write");
    });

    ready.notified().await;
    let state = detect_ecosystem(tmp.path(), family).await;
    server.await.expect("server");

    match state {
        EcosystemState::Coordinated { active_primals } => {
            assert!(
                active_primals.iter().any(|p| p == "beardog"),
                "expected beardog active, got {active_primals:?}"
            );
        }
        EcosystemState::Bootstrap => {
            panic!("expected Coordinated when health RPC succeeds, got Bootstrap");
        }
    }
}

#[tokio::test]
async fn test_wait_for_socket_immediate_with_zero_poll() {
    let tmp = tempfile::tempdir().expect("temp");
    let p = tmp.path().join("s.sock");
    std::fs::write(&p, b"").expect("touch");
    let r = wait_for_socket(&p, Duration::from_secs(1), Duration::ZERO).await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn test_health_check_semantic_fallback_when_plain_health_fails() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::Notify;

    let tmp = tempfile::tempdir().expect("tempdir");
    let family = "hfam-semantic";
    let sock_name = format!("beardog-{family}.sock");
    let sock_path = tmp.path().join(&sock_name);

    let ready = std::sync::Arc::new(Notify::new());
    let ready_c = std::sync::Arc::clone(&ready);
    let sock_path_c = sock_path.clone();
    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_path_c).expect("bind");
        ready_c.notify_one();
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut r, mut w) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut r)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse rpc");
            let method = req["method"].as_str().expect("method string");
            let id = req.get("id").cloned();
            let resp = if method == "health" {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                })
            } else if method == "health.status" {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "status": "ok" }
                })
            } else {
                panic!("unexpected method {method}");
            };
            w.write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
        }
    });

    ready.notified().await;
    let result = health_check(&sock_path).await;
    server.await.expect("server task");
    assert!(
        result.is_ok(),
        "semantic health fallback should succeed: {:?}",
        result.err()
    );
}
