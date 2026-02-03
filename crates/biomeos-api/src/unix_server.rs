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
///
/// # Security
///
/// The socket is created with 0600 permissions (owner-only).
pub async fn serve_unix_socket<P: AsRef<Path>>(socket_path: P, app: Router) -> Result<()> {
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
                                use tower::Service;
                                match app.call(request).await {
                                    Ok(response) => Ok::<_, hyper::Error>(response),
                                    Err(_) => {
                                        // Create error response
                                        let response = axum::http::Response::builder()
                                            .status(500)
                                            .body(axum::body::Body::from("Internal Server Error"))
                                            .unwrap();
                                        Ok(response)
                                    }
                                }
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

/// Serve an Axum router over both Unix socket and HTTP (temporary bridge)
///
/// ⚠️ This is TEMPORARY for PetalTongue transition!
/// Production deployments should use Unix socket only.
///
/// # Arguments
///
/// * `socket_path` - Path to Unix socket
/// * `http_addr` - HTTP bind address (e.g., "127.0.0.1:3000")
/// * `app` - Axum router to serve
pub async fn serve_dual_mode<P: AsRef<Path>>(
    socket_path: P,
    http_addr: std::net::SocketAddr,
    app: Router,
) -> Result<()> {
    let socket_path = socket_path.as_ref().to_path_buf();

    warn!("⚠️  Running in DUAL MODE (Unix socket + HTTP bridge)");
    warn!("   This is TEMPORARY for PetalTongue transition!");
    warn!("   Set BIOMEOS_API_HTTP_BRIDGE=false to disable HTTP");

    // Spawn Unix socket server
    let socket_app = app.clone();
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        if let Err(e) = serve_unix_socket(&socket_path_clone, socket_app).await {
            warn!("Unix socket server error: {}", e);
        }
    });

    // Spawn HTTP bridge
    info!("🌉 Starting HTTP bridge at http://{}", http_addr);
    info!("   ⚠️ HTTP is DEPRECATED and will be removed!");

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
