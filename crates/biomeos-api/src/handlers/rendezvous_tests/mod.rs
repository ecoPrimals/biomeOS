// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

#[cfg(unix)]
pub(super) async fn spawn_neural_api_loopback_mock(
    family_id: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("neural-rendezvous-mock.sock");
    let listener = UnixListener::bind(&sock).expect("bind mock neural");
    let fam = family_id.to_string();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                continue;
            };
            let mut buf = vec![0u8; 64 * 1024];
            let n = match stream.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => continue,
            };
            let line = String::from_utf8_lossy(&buf[..n]);
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim_end()) else {
                continue;
            };
            let params = v.get("params").cloned().unwrap_or_default();
            let inner_method = params.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let result = if inner_method == "birdsong.decrypt" {
                serde_json::json!({
                    "success": true,
                    "plaintext": "ok",
                    "family_id": fam.as_str(),
                })
            } else if inner_method == "crypto.blake3_hash" {
                serde_json::json!({ "hash": "node-hash-test" })
            } else {
                serde_json::json!({})
            };
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": v.get("id").clone(),
                "result": result,
            });
            let mut out = serde_json::to_string(&body).expect("serialize");
            out.push('\n');
            let _ = stream.write_all(out.as_bytes()).await;
            drop(stream);
        }
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    (dir, sock)
}

mod handlers;
mod serialization;
mod state;
