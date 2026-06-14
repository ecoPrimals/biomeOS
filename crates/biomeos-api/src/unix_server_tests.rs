// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use axum::Router;
use axum::routing::get;
use biomeos_test_utils::ready_signal;

#[tokio::test]
async fn test_serve_unix_socket_binds_and_accepts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("test-api.sock");

    let app = Router::new().route("/health", get(|| async { "ok" }));

    // Spawn server in background (runs forever)
    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    // Wait for server to bind (serve_unix_socket signals after bind)
    ready_rx.wait().await.expect("server should signal");

    assert!(socket_path.exists(), "Socket should be created");

    // Connect and verify
    let stream = tokio::net::UnixStream::connect(&socket_path).await;
    assert!(stream.is_ok(), "Should connect to socket");

    // Abort server (it runs forever)
    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_serve_unix_socket_removes_stale_socket() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("stale.sock");

    // Create a stale socket file
    std::fs::write(&socket_path, "stale").expect("write stale");
    assert!(socket_path.exists());

    let app = Router::new().route("/", get(|| async { "ok" }));

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    // Wait for server to replace stale socket and bind
    ready_rx.wait().await.expect("server should signal");

    // Should be able to connect (stale was removed, new socket created)
    let result = tokio::net::UnixStream::connect(&socket_path).await;
    assert!(result.is_ok(), "Should connect after stale removal");

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_serve_unix_socket_handles_http_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("http-test.sock");

    let app = Router::new()
        .route("/health", get(|| async { "healthy" }))
        .route("/api/v1/status", get(|| async { "{\"ok\":true}" }));

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    ready_rx.wait().await.expect("server should signal");

    let stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    // Send riboCipher signal prefix (required by Wave 113 policy)
    use tokio::io::AsyncWriteExt;
    let mut raw = stream;
    raw.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix");
    let io = hyper_util::rt::TokioIo::new(raw);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .expect("handshake");
    tokio::spawn(conn);

    let req = hyper::Request::builder()
        .uri("/health")
        .header("host", "localhost")
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())
        .expect("build request");

    let resp = sender.send_request(req).await.expect("send request");
    assert_eq!(resp.status(), 200);

    use http_body_util::BodyExt;
    let body = resp.into_body().collect().await.expect("read body");
    let body_str = String::from_utf8(body.to_bytes().to_vec()).expect("utf8");
    assert_eq!(body_str, "healthy");

    let stream2 = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect2");
    let mut raw2 = stream2;
    raw2.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix2");
    let io2 = hyper_util::rt::TokioIo::new(raw2);
    let (mut sender2, conn2) = hyper::client::conn::http1::handshake(io2)
        .await
        .expect("handshake2");
    tokio::spawn(conn2);

    let req2 = hyper::Request::builder()
        .uri("/api/v1/status")
        .header("host", "localhost")
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())
        .expect("build request");

    let resp2 = sender2.send_request(req2).await.expect("send request");
    assert_eq!(resp2.status(), 200);

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_serve_unix_socket_on_ready_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("no-ready.sock");

    let app = Router::new().route("/", get(|| async { "ok" }));

    let path = socket_path.clone();
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, None).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        socket_path.exists(),
        "Socket should be created even without on_ready"
    );

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_raw_jsonrpc_health_check() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("jsonrpc-health.sock");

    let app = Router::new().route("/health", get(|| async { "ok" }));

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    ready_rx.wait().await.expect("server should signal");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    stream.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix");
    let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
    stream.write_all(req).await.expect("write");
    stream.flush().await.expect("flush");

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read");
    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["status"], "alive");
    assert_eq!(resp["result"]["primal"], "biomeos");

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_raw_jsonrpc_identity_get() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("jsonrpc-identity.sock");

    let app = Router::new();

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    ready_rx.wait().await.expect("server should signal");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    stream.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix");
    let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"identity.get\",\"params\":{},\"id\":42}\n";
    stream.write_all(req).await.expect("write");
    stream.flush().await.expect("flush");

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read");
    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 42);
    assert_eq!(resp["result"]["primal"], "biomeos");
    assert_eq!(resp["result"]["role"], "orchestrator");

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_raw_jsonrpc_capabilities_list() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("jsonrpc-caps.sock");

    let app = Router::new();

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    ready_rx.wait().await.expect("server should signal");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    stream.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix");
    let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"capabilities.list\",\"params\":{},\"id\":99}\n";
    stream.write_all(req).await.expect("write");
    stream.flush().await.expect("flush");

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read");
    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 99);
    assert!(resp["result"]["methods"].is_array());
    assert!(resp["result"]["http_api"].is_array());

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_raw_jsonrpc_unknown_method_returns_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join("jsonrpc-unknown.sock");

    let app = Router::new();

    let path = socket_path.clone();
    let (mut ready_tx, ready_rx) = ready_signal();
    let on_ready = Some(Box::new(move || ready_tx.signal()) as Box<dyn FnOnce() + Send>);
    let server_handle = tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

    ready_rx.wait().await.expect("server should signal");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    stream.write_all(&[0xEC, 0x01]).await.expect("riboCipher prefix");
    let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"nonexistent.method\",\"params\":{},\"id\":7}\n";
    stream.write_all(req).await.expect("write");
    stream.flush().await.expect("flush");

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read");
    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 7);
    assert_eq!(resp["error"]["code"], -32601);
    let msg = resp["error"]["message"].as_str().expect("message");
    assert!(
        msg.contains("Method not found"),
        "expected 'Method not found' in error, got: {msg}"
    );

    server_handle.abort();
    let _ = server_handle.await;
}

#[test]
fn test_dispatch_jsonrpc_line_parse_error() {
    let resp = dispatch_jsonrpc_line("not json at all");
    assert_eq!(resp["error"]["code"], -32700);
}

#[test]
fn test_dispatch_jsonrpc_line_btsp_client_hello_returns_redirect() {
    let hello = r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"AAAA"}"#;
    let resp = dispatch_jsonrpc_line(hello);
    assert_eq!(resp["error"]["code"], -32001);
    assert!(
        resp["error"]["message"]
            .as_str()
            .expect("message")
            .contains("neural-api")
    );
    assert_eq!(resp["error"]["data"]["protocol"], "btsp");
    assert_eq!(resp["error"]["data"]["redirect"], "neural-api");
}

#[test]
fn test_dispatch_jsonrpc_line_btsp_minimal_hello_returns_redirect() {
    let hello = r#"{"protocol":"btsp"}"#;
    let resp = dispatch_jsonrpc_line(hello);
    assert_eq!(resp["error"]["code"], -32001);
    assert_eq!(resp["error"]["data"]["redirect"], "neural-api");
}

#[test]
fn test_dispatch_jsonrpc_line_health_aliases() {
    for method in &["health", "health.check", "health.liveness", "health.status"] {
        let req = format!(
            r#"{{"jsonrpc":"2.0","method":"{}","params":{{}},"id":1}}"#,
            method
        );
        let resp = dispatch_jsonrpc_line(&req);
        assert_eq!(
            resp["result"]["status"], "alive",
            "{method} should return alive"
        );
    }
}

#[test]
fn test_dispatch_jsonrpc_line_primal_list() {
    let req = r#"{"jsonrpc":"2.0","method":"primal.list","params":{},"id":1}"#;
    let resp = dispatch_jsonrpc_line(req);
    assert!(resp["result"]["primals"].is_array());
    assert_eq!(resp["result"]["count"], 1);
}

#[test]
fn test_dispatch_jsonrpc_line_topology_primals_alias() {
    let req = r#"{"jsonrpc":"2.0","method":"topology.primals","params":{},"id":2}"#;
    let resp = dispatch_jsonrpc_line(req);
    assert!(resp["result"]["primals"].is_array());
}

#[test]
fn test_dispatch_jsonrpc_line_capabilities_list_includes_primal_list() {
    let req = r#"{"jsonrpc":"2.0","method":"capabilities.list","params":{},"id":1}"#;
    let resp = dispatch_jsonrpc_line(req);
    let methods = resp["result"]["methods"].as_array().expect("methods array");
    let has_primal_list = methods.iter().any(|m| m.as_str() == Some("primal.list"));
    assert!(
        has_primal_list,
        "capabilities.list should advertise primal.list"
    );
}
