// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unix socket server for biomeOS API
//!
//! Provides secure, port-free communication via Unix sockets.
//! Supports both HTTP (axum) and raw JSON-RPC (NDJSON) transports via
//! first-byte auto-detection: if the first byte is `{`, the connection
//! is handled as newline-delimited JSON-RPC; otherwise it is passed to
//! hyper/axum as HTTP.

use anyhow::{Context, Result};
use axum::Router;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, info, warn};

/// Serve an Axum router over a Unix socket
///
/// This is the PRIMARY transport for biomeOS API, providing:
/// - Port-free architecture (no TCP ports!)
/// - Secure by default (filesystem permissions)
/// - Fast (0.1ms overhead vs 10ms HTTP)
/// - Dual-protocol: HTTP (axum) and raw JSON-RPC via first-byte auto-detect
///
/// # Arguments
///
/// * `socket_path` - Path to Unix socket
/// * `app` - Axum router to serve
/// * `on_ready` - Optional callback invoked after bind (for tests)
///
/// # Security
///
/// The socket is created with 0600 permissions (owner-only).
pub async fn serve_unix_socket<P: AsRef<Path>>(
    socket_path: P,
    app: Router,
    on_ready: Option<Box<dyn FnOnce() + Send>>,
) -> Result<()> {
    let socket_path = socket_path.as_ref();

    // Remove old socket if exists
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("Failed to remove old Unix socket")?;
    }

    // Create Unix listener
    let listener = UnixListener::bind(socket_path).context("Failed to bind Unix socket")?;

    // Set permissions (0600 - owner only)
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))
            .context("Failed to set socket permissions")?;
    }

    info!(
        "📡 biomeOS API listening on Unix socket: {}",
        socket_path.display()
    );
    info!("   Security: Owner-only (0600 permissions)");
    info!("   Protocol: HTTP + raw JSON-RPC (auto-detect)");
    info!("   Port-free: ✅ TRUE PRIMAL architecture!");

    if let Some(f) = on_ready {
        f();
    }

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let app = app.clone();

                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);

                    let is_jsonrpc = match reader.fill_buf().await {
                        Ok(buf) if !buf.is_empty() => buf[0] == b'{' || buf[0] == b'[',
                        _ => false,
                    };

                    if is_jsonrpc {
                        debug!("UDS auto-detect: raw JSON-RPC connection");
                        if let Err(e) = handle_raw_jsonrpc(reader).await {
                            debug!("Raw JSON-RPC connection ended: {e}");
                        }
                    } else {
                        // BufReader<UnixStream> implements AsyncRead + AsyncWrite,
                        // so any bytes already buffered by fill_buf are replayed to
                        // hyper transparently.
                        serve_http_connection(reader, app).await;
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn serve_http_connection(stream: BufReader<tokio::net::UnixStream>, app: Router) {
    let stream = hyper_util::rt::TokioIo::new(stream);
    let hyper_service =
        hyper::service::service_fn(move |request: hyper::Request<hyper::body::Incoming>| {
            let (parts, body) = request.into_parts();
            let body = axum::body::Body::new(body);
            let request = axum::http::Request::from_parts(parts, body);
            let mut app = app.clone();

            async move {
                use tower::Service;
                let response = match app.call(request).await {
                    Ok(resp) => resp,
                    Err(infallible) => match infallible {},
                };
                Ok::<_, hyper::Error>(response)
            }
        });

    if let Err(e) =
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(stream, hyper_service)
            .await
    {
        warn!("Error serving HTTP connection: {}", e);
    }
}

/// Handle a raw JSON-RPC (NDJSON) connection.
///
/// Responds to standard primal discovery methods (`health.check`,
/// `health.liveness`, `identity.get`, `capabilities.list`) so that
/// spring probes and discovery sweeps see biomeOS as alive rather than
/// receiving HTTP 400.
async fn handle_raw_jsonrpc(mut reader: BufReader<tokio::net::UnixStream>) -> Result<()> {
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = dispatch_jsonrpc_line(trimmed);
        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        reader.get_mut().write_all(out.as_bytes()).await?;
        reader.get_mut().flush().await?;
    }

    Ok(())
}

fn dispatch_jsonrpc_line(line: &str) -> serde_json::Value {
    let null = serde_json::Value::Null;
    let Ok(req) = serde_json::from_str::<serde_json::Value>(line) else {
        return jsonrpc_error(&null, -32700, "Parse error");
    };

    let id = req.get("id").unwrap_or(&null);
    let method = req
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match method {
        "health" | "health.check" | "health.liveness" | "health.status" => {
            let result = serde_json::json!({ "status": "healthy", "primal": "biomeos" });
            jsonrpc_ok(id, &result)
        }
        "identity.get" | "identity" => {
            let result = serde_json::json!({
                "primal": "biomeos",
                "role": "orchestrator",
                "version": env!("CARGO_PKG_VERSION"),
                "transport": "http+jsonrpc",
                "note": "biomeOS is an HTTP API server; use HTTP POST for full API, or the neural-api socket for capability.call routing"
            });
            jsonrpc_ok(id, &result)
        }
        "capabilities.list" | "capability.list" => {
            let result = serde_json::json!({
                "primal": "biomeos",
                "methods": [
                    "health.check",
                    "health.liveness",
                    "identity.get",
                    "capabilities.list"
                ],
                "http_api": [
                    "/api/v1/health",
                    "/api/v1/topology",
                    "/api/v1/genome/list",
                    "/api/v1/trust/identity",
                    "/api/v1/events/stream"
                ],
                "note": "Full API available via HTTP over this socket. Neural API (capability.call, graph.execute) is on the neural-api socket."
            });
            jsonrpc_ok(id, &result)
        }
        _ => jsonrpc_error(
            id,
            -32601,
            &format!(
                "Method not found: {method}. biomeOS API uses HTTP transport; \
                 for capability.call routing use the neural-api socket"
            ),
        ),
    }
}

fn jsonrpc_ok(id: &serde_json::Value, result: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

fn jsonrpc_error(id: &serde_json::Value, code: i32, message: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": code, "message": message },
        "id": id
    })
}

#[cfg(test)]
#[cfg(unix)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

        ready_rx.wait().await.expect("server should signal");

        let stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");
        let io = hyper_util::rt::TokioIo::new(stream);

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
        let io2 = hyper_util::rt::TokioIo::new(stream2);
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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

        ready_rx.wait().await.expect("server should signal");

        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        stream.write_all(req).await.expect("write");
        stream.flush().await.expect("flush");

        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.expect("read");
        let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["status"], "healthy");
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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

        ready_rx.wait().await.expect("server should signal");

        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

        ready_rx.wait().await.expect("server should signal");

        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let req =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"capabilities.list\",\"params\":{},\"id\":99}\n";
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
        let server_handle =
            tokio::spawn(async move { serve_unix_socket(&path, app, on_ready).await });

        ready_rx.wait().await.expect("server should signal");

        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let req = b"{\"jsonrpc\":\"2.0\",\"method\":\"graph.execute\",\"params\":{},\"id\":7}\n";
        stream.write_all(req).await.expect("write");
        stream.flush().await.expect("flush");

        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.expect("read");
        let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse response");

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 7);
        assert_eq!(resp["error"]["code"], -32601);
        assert!(
            resp["error"]["message"]
                .as_str()
                .expect("message")
                .contains("neural-api socket")
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
    fn test_dispatch_jsonrpc_line_health_aliases() {
        for method in &["health", "health.check", "health.liveness", "health.status"] {
            let req = format!(
                r#"{{"jsonrpc":"2.0","method":"{}","params":{{}},"id":1}}"#,
                method
            );
            let resp = dispatch_jsonrpc_line(&req);
            assert_eq!(
                resp["result"]["status"], "healthy",
                "{method} should return healthy"
            );
        }
    }
}
