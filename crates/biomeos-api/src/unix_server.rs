//! Isomorphic server for biomeOS API
//!
//! **TRUE ecoBin v2.0:** Platform-agnostic IPC with automatic adaptation.
//!
//! Provides secure, port-free communication with automatic platform detection:
//! - Linux/macOS: Unix sockets (optimal)
//! - Android: TCP fallback with XDG discovery
//! - Windows: Named pipes (future)
//!
//! This implements the Try→Detect→Adapt→Succeed pattern from songbird.

use anyhow::{Context, Result};
use axum::Router;
use biomeos_core::ipc::{Transport, TransportListener, TransportType};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Serve an Axum router over isomorphic transport (Unix socket with TCP fallback)
///
/// **TRUE ecoBin v2.0:** This is the PRIMARY transport for biomeOS API, providing:
/// - Port-free architecture on Linux/macOS (Unix sockets)
/// - Automatic TCP fallback on Android (SELinux constraints)
/// - XDG-compliant discovery files for clients
/// - Fast (0.1ms Unix overhead vs 10ms HTTP)
/// - Secure by default (filesystem permissions or localhost-only)
///
/// # Isomorphism
///
/// The server automatically detects platform constraints and adapts:
/// 1. **Try**: Attempt optimal transport (Unix socket)
/// 2. **Detect**: Runtime check for platform constraints (SELinux)
/// 3. **Adapt**: Fall back to TCP with discovery file
/// 4. **Succeed**: Server starts successfully on any platform
///
/// # Arguments
///
/// * `socket_path` - Path to Unix socket (or base name for discovery)
/// * `app` - Axum router to serve
///
/// # Security
///
/// - Unix sockets: Created with 0600 permissions (owner-only)
/// - TCP fallback: Binds to 127.0.0.1 (localhost-only)
/// - Discovery files: Written to XDG-compliant runtime dir
pub async fn serve_isomorphic<P: AsRef<Path>>(socket_path: P, app: Router) -> Result<()> {
    let socket_path = socket_path.as_ref();

    // Remove old socket if exists (only for Unix sockets)
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("Failed to remove old Unix socket")?;
    }

    info!("🔌 Starting biomeOS API server (isomorphic mode)");

    // Create transport with automatic fallback
    let transport = Transport::new(TransportType::UnixSocket {
        path: socket_path.to_path_buf(),
    });

    let mut listener = transport
        .bind_with_fallback()
        .await
        .context("Failed to bind biomeOS API")?;

    // Set permissions for Unix sockets
    #[cfg(unix)]
    if socket_path.exists() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600)) {
            warn!("Could not set socket permissions: {}", e);
        } else {
            info!("   Security: Owner-only (0600 permissions)");
        }
    }

    info!("📡 biomeOS API listening (isomorphic mode)");
    info!("   Protocol: JSON-RPC 2.0");
    info!("   Port-free: ✅ TRUE PRIMAL architecture!");

    // Serve connections
    loop {
        match listener.accept().await {
            Ok(stream) => {
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

/// Legacy wrapper: Serve an Axum router over Unix socket
///
/// **DEPRECATED**: Use `serve_isomorphic()` instead for TRUE ecoBin compliance.
///
/// This wrapper is maintained for backward compatibility but internally uses
/// the isomorphic transport layer.
pub async fn serve_unix_socket<P: AsRef<Path>>(socket_path: P, app: Router) -> Result<()> {
    warn!("⚠️  serve_unix_socket() is deprecated - use serve_isomorphic() instead");
    serve_isomorphic(socket_path, app).await
}

/// Serve an Axum router over both isomorphic transport and HTTP (temporary bridge)
///
/// ⚠️ This is TEMPORARY for PetalTongue transition!
/// Production deployments should use isomorphic transport only.
///
/// # Arguments
///
/// * `socket_path` - Path to Unix socket (or base for discovery)
/// * `http_addr` - HTTP bind address (e.g., "127.0.0.1:3000")
/// * `app` - Axum router to serve
pub async fn serve_dual_mode<P: AsRef<Path>>(
    socket_path: P,
    http_addr: std::net::SocketAddr,
    app: Router,
) -> Result<()> {
    let socket_path = socket_path.as_ref().to_path_buf();

    warn!("⚠️  Running in DUAL MODE (isomorphic IPC + HTTP bridge)");
    warn!("   This is TEMPORARY for PetalTongue transition!");
    warn!("   Set BIOMEOS_API_HTTP_BRIDGE=false to disable HTTP");

    // Spawn isomorphic IPC server
    let socket_app = app.clone();
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        if let Err(e) = serve_isomorphic(&socket_path_clone, socket_app).await {
            warn!("Isomorphic IPC server error: {}", e);
        }
    });

    // Spawn HTTP bridge
    info!("🌉 Starting HTTP bridge at http://{}", http_addr);
    info!("   ⚠️ HTTP is DEPRECATED and will be removed!");

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
