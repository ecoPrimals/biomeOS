// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::probe_unix_socket_capabilities_list;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

// ── Socket probe tests ──

#[tokio::test]
async fn probe_nonexistent_socket_returns_empty() {
    let path = std::env::temp_dir().join("biomeos_cap_probe_absent_sock.sock");
    let _ = std::fs::remove_file(&path);
    assert!(!path.exists());
    let caps = probe_unix_socket_capabilities_list(&path).await;
    assert!(caps.is_empty());
}

#[tokio::test]
async fn probe_reads_capabilities_from_capabilities_list_response() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("probe.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(serde_json::json!(1));
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "capabilities": ["cap.one", "cap.two"]
            }
        });
        let mut stream = reader.into_inner();
        let line = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(line.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert_eq!(caps, vec!["cap.one".to_string(), "cap.two".to_string()]);
}

#[tokio::test]
async fn probe_falls_back_to_capability_list_singular() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("fallback.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        // Connection 1: capabilities.list (plural) → return JSON-RPC error
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(serde_json::json!(1));
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {"code": -32601, "message": "Method not found"}
        });
        let mut stream = reader.into_inner();
        let resp = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(resp.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        // Connection 2: capability.list (singular) → return capabilities
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(serde_json::json!(1));
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": ["crypto", "security", "beacon"]
        });
        let mut stream = reader.into_inner();
        let resp = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(resp.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert_eq!(caps, vec!["crypto", "security", "beacon"]);
}

#[tokio::test]
async fn probe_invalid_json_line_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("bad-json.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        // Two connections: both return garbage
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let mut stream = reader.into_inner();
            stream.write_all(b"not-json\n").await.unwrap();
            stream.flush().await.unwrap();
        }
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert!(caps.is_empty());
}
