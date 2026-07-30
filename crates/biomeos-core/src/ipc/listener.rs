// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! [`TransportListener`] — a platform-agnostic server socket.

use std::io;
use std::path::Path;
use tokio::net::TcpListener;

use super::TransportStream;

/// Environment variable to override the socket group (default: `membrane`).
#[cfg(unix)]
const MEMBRANE_SOCKET_GROUP_ENV: &str = "MEMBRANE_SOCKET_GROUP";

/// Default group name applied to sockets for multi-user access.
#[cfg(unix)]
const DEFAULT_SOCKET_GROUP: &str = "membrane";

/// Apply ownership and permissions to a freshly-bound Unix socket.
///
/// Sets mode `0o660` (owner+group rw) and attempts to `chown` the socket
/// to the group specified by `MEMBRANE_SOCKET_GROUP` (default: `membrane`).
/// This enables multi-user access on gates where Sovereign CI or other
/// services run as different users within the `membrane` group.
///
/// Failures are logged but do not abort — the socket remains functional
/// for single-user deployments where the calling user owns the process.
#[cfg(unix)]
pub fn apply_socket_ownership(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let perms = std::fs::Permissions::from_mode(0o660);
    if let Err(e) = std::fs::set_permissions(path, perms) {
        tracing::warn!(
            "Failed to set socket permissions on {}: {e}",
            path.display()
        );
    }

    apply_membrane_group(path);
}

/// Apply ownership and permissions to a socket directory.
///
/// Sets mode `0o770` (owner+group rwx) for directory traversal and `chown`s
/// to the `MEMBRANE_SOCKET_GROUP` (default: `membrane`).
#[cfg(unix)]
pub fn apply_dir_ownership(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let perms = std::fs::Permissions::from_mode(0o770);
    if let Err(e) = std::fs::set_permissions(path, perms) {
        tracing::debug!("Failed to set dir permissions on {}: {e}", path.display());
    }

    apply_membrane_group(path);
}

/// `chown :<MEMBRANE_SOCKET_GROUP>` on a path. Non-fatal on failure.
#[cfg(unix)]
fn apply_membrane_group(path: &Path) {
    let group_name = std::env::var(MEMBRANE_SOCKET_GROUP_ENV)
        .unwrap_or_else(|_| DEFAULT_SOCKET_GROUP.to_string());

    if let Some(gid) = resolve_group_id(&group_name) {
        if let Err(e) = rustix::fs::chown(path, None, Some(gid)) {
            tracing::debug!(
                "chown :{group_name} on {} failed (non-fatal): {e}",
                path.display()
            );
        }
    } else {
        tracing::debug!(
            "Group '{group_name}' not found — {} retains default ownership",
            path.display()
        );
    }
}

/// Resolve a group name to a GID via `/etc/group` parsing.
#[cfg(unix)]
fn resolve_group_id(name: &str) -> Option<rustix::fs::Gid> {
    use std::io::BufRead;

    let file = std::fs::File::open("/etc/group").ok()?;
    let reader = std::io::BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 && parts[0] == name {
            let gid: u32 = parts[2].parse().ok()?;
            return Some(rustix::fs::Gid::from_raw(gid));
        }
    }
    None
}

/// A bound listener that accepts incoming [`TransportStream`] connections.
pub enum TransportListener {
    /// Unix domain socket listener (Tier 1).
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),

    /// TCP listener (Tier 2 — universal).
    Tcp(TcpListener),
}

impl TransportListener {
    /// Bind a Unix socket with proper group ownership for multi-user access.
    ///
    /// Post-bind: sets `0o660` and `chown :<MEMBRANE_SOCKET_GROUP>` (default `membrane`).
    #[cfg(unix)]
    pub async fn bind_unix(path: &Path) -> io::Result<Self> {
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
        let listener = tokio::net::UnixListener::bind(path)?;
        apply_socket_ownership(path);
        Ok(Self::Unix(listener))
    }

    /// Bind a Unix socket. On Windows, binds TCP on localhost and writes a port-file.
    #[cfg(windows)]
    pub async fn bind_unix(path: &Path) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let port_file = path.with_extension("port");
        tokio::fs::write(&port_file, port.to_string()).await?;

        Ok(Self::Tcp(listener))
    }

    /// Bind a TCP listener on the given address.
    pub async fn bind_tcp(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self::Tcp(listener))
    }

    /// Accept the next incoming connection.
    pub async fn accept(&self) -> io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Unix(stream))
            }
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Tcp(stream))
            }
        }
    }

    /// Get the local address description for logging.
    #[must_use]
    pub fn local_addr_display(&self) -> String {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => l
                .local_addr()
                .map_or_else(|_| "unix://<unknown>".to_string(), |a| format!("{a:?}")),
            Self::Tcp(l) => l
                .local_addr()
                .map_or_else(|_| "tcp://<unknown>".to_string(), |a| format!("tcp://{a}")),
        }
    }
}
