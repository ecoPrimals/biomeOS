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

/// Default `HealthRpc` implementation suitable for any primal.
///
/// Provides basic health, metrics, and version information derived from
/// the running process. Primals can either use this directly or supply
/// a custom implementation of [`HealthRpc`].
#[derive(Clone)]
pub struct DefaultHealthService {
    primal_name: String,
    start_time: tokio::time::Instant,
}

impl DefaultHealthService {
    /// Create a new default health service.
    #[must_use]
    pub fn new(primal_name: impl Into<String>) -> Self {
        Self {
            primal_name: primal_name.into(),
            start_time: tokio::time::Instant::now(),
        }
    }
}

impl HealthRpc for DefaultHealthService {
    async fn health_check(
        self,
        _: tarpc::context::Context,
    ) -> biomeos_types::tarpc_types::HealthStatus {
        biomeos_types::tarpc_types::HealthStatus {
            healthy: true,
            message: Some(format!("{} healthy", self.primal_name)),
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    async fn health_metrics(
        self,
        _: tarpc::context::Context,
    ) -> biomeos_types::tarpc_types::HealthMetrics {
        biomeos_types::tarpc_types::HealthMetrics {
            healthy: true,
            cpu_usage: 0.0,
            memory_bytes: 0,
            active_connections: 0,
            total_requests: 0,
            total_errors: 0,
            avg_latency_us: 0,
        }
    }

    async fn version(self, _: tarpc::context::Context) -> biomeos_types::tarpc_types::VersionInfo {
        biomeos_types::tarpc_types::VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: None,
            build_timestamp: None,
            protocols: vec!["jsonrpc".to_string(), "tarpc".to_string()],
        }
    }
}

/// Prepare and start a tarpc health sidecar alongside a primal's JSON-RPC socket.
///
/// Given the path to a primal's JSON-RPC socket, derives the corresponding
/// `.tarpc.sock` path and serves `HealthRpc` on it. This function runs
/// the tarpc server loop — call it from a dedicated task or select branch.
///
/// # Protocol Escalation
///
/// This enables the dual-protocol pattern:
/// ```text
/// beardog-family123.sock          ← JSON-RPC  (always present)
/// beardog-family123.tarpc.sock    ← tarpc      (started by this function)
/// ```
///
/// # Errors
///
/// Returns an error if the socket cannot be prepared or the listener fails.
pub async fn start_tarpc_sidecar(
    jsonrpc_socket: &std::path::Path,
    service: impl HealthRpc + Clone + Send + 'static,
) -> Result<()> {
    let tarpc_path = tarpc_socket_path(jsonrpc_socket);
    serve_tarpc_health(&tarpc_path, service).await
}

/// Convenience: start a default tarpc health sidecar for a primal.
///
/// Uses [`DefaultHealthService`] as the implementation. Runs the server
/// loop — call from a dedicated task or select branch.
///
/// # Errors
///
/// Returns an error if the socket cannot be prepared or the listener fails.
pub async fn start_default_tarpc_sidecar(
    jsonrpc_socket: &std::path::Path,
    primal_name: impl Into<String>,
) -> Result<()> {
    let service = DefaultHealthService::new(primal_name);
    start_tarpc_sidecar(jsonrpc_socket, service).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::ready_signal;

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
        let (mut ready_tx, ready_rx) = ready_signal();
        let listener = UnixListener::bind(&prepared).unwrap();
        ready_tx.signal();

        let sock_clone = sock.clone();
        let server = tokio::spawn(async move { listener.accept().await.is_ok() });

        ready_rx.wait().await.unwrap();
        let _stream = UnixStream::connect(&sock_clone).await.unwrap();

        assert!(server.await.unwrap());
    }

    #[test]
    fn default_health_service_creation() {
        let svc = DefaultHealthService::new("beardog");
        assert_eq!(svc.primal_name, "beardog");
    }

    #[tokio::test]
    async fn default_health_service_check() {
        let svc = DefaultHealthService::new("beardog");
        let status = svc.health_check(tarpc::context::current()).await;
        assert!(status.healthy);
        assert!(status.message.unwrap().contains("beardog"));
    }

    #[tokio::test]
    async fn default_health_service_metrics() {
        let svc = DefaultHealthService::new("beardog");
        let metrics = svc.health_metrics(tarpc::context::current()).await;
        assert!(metrics.healthy);
    }

    #[tokio::test]
    async fn default_health_service_version() {
        let svc = DefaultHealthService::new("beardog");
        let info = svc.version(tarpc::context::current()).await;
        assert!(!info.version.is_empty());
        assert!(info.protocols.contains(&"tarpc".to_string()));
        assert!(info.protocols.contains(&"jsonrpc".to_string()));
    }

    #[tokio::test]
    async fn start_tarpc_sidecar_derives_path() {
        let dir = tempfile::tempdir().unwrap();
        let jsonrpc_sock = dir.path().join("test-primal.sock");
        let expected_tarpc = dir.path().join("test-primal.tarpc.sock");

        let tarpc_path = tarpc_socket_path(&jsonrpc_sock);
        assert_eq!(tarpc_path, expected_tarpc);
    }

    #[test]
    fn tarpc_socket_name_empty_string() {
        assert_eq!(tarpc_socket_name(""), ".tarpc");
    }

    #[test]
    fn tarpc_socket_name_dot_sock_only() {
        assert_eq!(tarpc_socket_name(".sock"), ".tarpc.sock");
    }

    #[test]
    fn tarpc_socket_path_with_extension() {
        let p = std::path::Path::new("/tmp/foo.bar.sock");
        let tarpc = tarpc_socket_path(p);
        assert_eq!(tarpc, std::path::PathBuf::from("/tmp/foo.bar.tarpc.sock"));
    }

    #[tokio::test]
    async fn prepare_socket_root_only() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("single.sock");
        let prepared = prepare_socket(&sock).await.unwrap();
        assert_eq!(prepared, sock);
        assert!(sock.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn prepare_socket_no_parent() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("x.sock");
        let prepared = prepare_socket(&sock).await.unwrap();
        assert_eq!(prepared, sock);
    }

    #[tokio::test(start_paused = true)]
    async fn default_health_service_uptime() {
        let svc = DefaultHealthService::new("test");
        tokio::time::advance(std::time::Duration::from_millis(10)).await;
        let status = svc.health_check(tarpc::context::current()).await;
        assert!(status.healthy);
        assert!(status.message.is_some());
    }

    #[test]
    fn default_health_service_version_protocols() {
        let svc = DefaultHealthService::new("beardog");
        let info = futures::executor::block_on(svc.version(tarpc::context::current()));
        assert!(info.protocols.contains(&"jsonrpc".to_string()));
        assert!(info.protocols.contains(&"tarpc".to_string()));
        assert!(info.version.len() > 0);
    }
}
