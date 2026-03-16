// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! tarpc Transport Helpers for Unix Socket IPC
//!
//! Thin convenience layer over `tarpc::serde_transport::unix` that adds
//! biomeOS-specific conventions: stale-socket cleanup, parent-directory
//! creation, and the `.tarpc.sock` naming scheme.
//!
//! # Protocol Escalation
//!
//! Primals start with JSON-RPC and can upgrade hot paths to tarpc for
//! zero-overhead binary RPC. A primal exposes both sockets side by side:
//!
//! ```text
//! beardog-family123.sock          ← JSON-RPC  (always present)
//! beardog-family123.tarpc.sock    ← tarpc      (optional, high-perf)
//! ```
//!
//! # Server Example
//!
//! ```rust,ignore
//! use biomeos_primal_sdk::tarpc_transport;
//! use biomeos_types::tarpc_types::HealthRpc;
//!
//! let service = MyHealthServiceImpl;
//! tarpc_transport::serve_tarpc_health("/tmp/primal.tarpc.sock", service).await?;
//! ```
//!
//! # Client Example
//!
//! ```rust,ignore
//! use biomeos_primal_sdk::tarpc_transport;
//!
//! let transport = tarpc::serde_transport::unix::connect(
//!     "/tmp/primal.tarpc.sock",
//!     tokio_serde::formats::Bincode::default,
//! ).await?;
//! let client = HealthRpcClient::new(tarpc::client::Config::default(), transport).spawn();
//! ```

use std::path::Path;

use anyhow::{Context, Result};
use biomeos_types::tarpc_types::HealthRpc;
use futures::StreamExt;
use tarpc::serde_transport::unix;
use tarpc::server::{BaseChannel, Channel};
use tokio_serde::formats::Bincode;

/// Prepare a socket path for tarpc listening.
///
/// Removes stale sockets and creates parent directories.  After calling
/// this, pass the path to `tarpc::serde_transport::unix::listen()`.
pub async fn prepare_socket(path: impl AsRef<Path>) -> Result<std::path::PathBuf> {
    let path = path.as_ref();

    if path.exists() {
        tokio::fs::remove_file(path)
            .await
            .with_context(|| format!("Failed to remove stale socket: {}", path.display()))?;
    }

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create socket directory: {}", parent.display()))?;
    }

    tracing::info!(socket = %path.display(), "tarpc socket path prepared");
    Ok(path.to_path_buf())
}

/// Derive a tarpc socket name from a JSON-RPC socket name.
///
/// Convention: `beardog-family123.sock` → `beardog-family123.tarpc.sock`.
#[must_use]
pub fn tarpc_socket_name(jsonrpc_socket: &str) -> String {
    if let Some(base) = jsonrpc_socket.strip_suffix(".sock") {
        format!("{base}.tarpc.sock")
    } else {
        format!("{jsonrpc_socket}.tarpc")
    }
}

/// Derive a tarpc socket path from a JSON-RPC socket path.
#[must_use]
pub fn tarpc_socket_path(jsonrpc_socket: &Path) -> std::path::PathBuf {
    let name = jsonrpc_socket.to_string_lossy();
    std::path::PathBuf::from(tarpc_socket_name(&name))
}

/// Spawn a tarpc HealthRpc server on a Unix socket.
///
/// Listens on the given socket path using the tarpc binary protocol (Bincode).
/// Each incoming connection is served sequentially until the client disconnects.
/// Runs until the process exits or the listener is dropped.
///
/// # Example
///
/// ```rust,ignore
/// use biomeos_primal_sdk::tarpc_transport;
/// use biomeos_types::tarpc_types::{HealthRpc, HealthStatus, HealthMetrics, VersionInfo};
///
/// #[derive(Clone)]
/// struct MyHealthService;
/// impl HealthRpc for MyHealthService {
///     async fn health_check(self, _: tarpc::context::Context) -> HealthStatus { ... }
///     async fn health_metrics(self, _: tarpc::context::Context) -> HealthMetrics { ... }
///     async fn version(self, _: tarpc::context::Context) -> VersionInfo { ... }
/// }
///
/// tarpc_transport::serve_tarpc_health("/tmp/primal.tarpc.sock", MyHealthService).await?;
/// ```
pub async fn serve_tarpc_health(
    socket_path: impl AsRef<Path>,
    service: impl HealthRpc + Clone + Send + 'static,
) -> Result<()> {
    let path = socket_path.as_ref();
    let prepared = prepare_socket(path).await?;

    let mut incoming = unix::listen(&prepared, Bincode::default)
        .await
        .with_context(|| format!("Failed to listen on tarpc socket: {}", path.display()))?;

    tracing::info!(socket = %path.display(), "tarpc HealthRpc server listening");

    while let Some(transport_result) = incoming.next().await {
        match transport_result {
            Ok(transport) => {
                let service = service.clone();
                let channel = BaseChannel::with_defaults(transport);
                let requests = channel.execute(service.serve());
                // Box::pin makes the stream Unpin so we can use StreamExt::next
                let mut requests = Box::pin(requests);
                while let Some(fut) = futures::StreamExt::next(&mut requests).await {
                    fut.await;
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "tarpc accept error");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_socket_name_standard() {
        assert_eq!(
            tarpc_socket_name("beardog-family123.sock"),
            "beardog-family123.tarpc.sock"
        );
    }

    #[test]
    fn tarpc_socket_name_no_sock_suffix() {
        assert_eq!(tarpc_socket_name("beardog"), "beardog.tarpc");
    }

    #[test]
    fn tarpc_socket_name_nested_path() {
        assert_eq!(
            tarpc_socket_name("/run/biomeos/songbird-f1.sock"),
            "/run/biomeos/songbird-f1.tarpc.sock"
        );
    }

    #[test]
    fn tarpc_socket_path_conversion() {
        let jsonrpc = std::path::Path::new("/run/biomeos/beardog-f1.sock");
        let tarpc = tarpc_socket_path(jsonrpc);
        assert_eq!(
            tarpc,
            std::path::PathBuf::from("/run/biomeos/beardog-f1.tarpc.sock")
        );
    }

    #[tokio::test]
    async fn prepare_socket_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir
            .path()
            .join("nested")
            .join("deep")
            .join("test.tarpc.sock");
        let prepared = prepare_socket(&sock).await.unwrap();
        assert_eq!(prepared, sock);
        assert!(sock.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn prepare_socket_removes_stale() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("stale.tarpc.sock");
        tokio::fs::write(&sock, b"stale").await.unwrap();
        assert!(sock.exists());
        prepare_socket(&sock).await.unwrap();
        assert!(!sock.exists()); // file removed by prepare
    }

    #[tokio::test]
    async fn listen_and_connect_roundtrip() {
        use tokio::net::{UnixListener, UnixStream};

        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("roundtrip.tarpc.sock");

        let prepared = prepare_socket(&sock).await.unwrap();

        // Use raw UnixListener to verify socket path works
        let listener = UnixListener::bind(&prepared).unwrap();

        let sock_clone = sock.clone();
        let server = tokio::spawn(async move { listener.accept().await.is_ok() });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _stream = UnixStream::connect(&sock_clone).await.unwrap();

        assert!(server.await.unwrap());
    }
}
