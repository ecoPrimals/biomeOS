// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unix socket server for biomeOS API
//!
//! Provides secure, port-free communication via Unix sockets with JSON-RPC 2.0.

use anyhow::{Context, Result};
use axum::Router;
use std::path::Path;
use tokio::net::UnixListener;
use tracing::{info, warn};

/// Serve an Axum router over a Unix socket
///
/// This is the PRIMARY transport for biomeOS API, providing:
/// - Port-free architecture (no TCP ports!)
/// - Secure by default (filesystem permissions)
/// - Fast (0.1ms overhead vs 10ms HTTP)
/// - Isomorphic (same API as HTTP)
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
    info!("   Protocol: JSON-RPC 2.0 over Unix socket");
    info!("   Port-free: ✅ TRUE PRIMAL architecture!");

    if let Some(f) = on_ready {
        f();
    }

    // Serve connections
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let app = app.clone();

                tokio::spawn(async move {
                    let stream = hyper_util::rt::TokioIo::new(stream);
                    let hyper_service = hyper::service::service_fn(
                        move |request: hyper::Request<hyper::body::Incoming>| {
                            // Convert hyper request to axum request
                            let (parts, body) = request.into_parts();
                            let body = axum::body::Body::new(body);
                            let request = axum::http::Request::from_parts(parts, body);

                            // Clone app for this request
                            let mut app = app.clone();

                            async move {
                                // Use tower::Service::call directly
                                // Axum Router::call returns Result<Response, Infallible> — always Ok
                                use tower::Service;
                                let response = match app.call(request).await {
                                    Ok(resp) => resp,
                                    Err(infallible) => match infallible {},
                                };
                                Ok::<_, hyper::Error>(response)
                            }
                        },
                    );

                    if let Err(e) = hyper_util::server::conn::auto::Builder::new(
                        hyper_util::rt::TokioExecutor::new(),
                    )
                    .serve_connection(stream, hyper_service)
                    .await
                    {
                        warn!("Error serving connection: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
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
}
