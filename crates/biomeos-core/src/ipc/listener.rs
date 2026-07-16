// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! [`TransportListener`] — a platform-agnostic server socket.

use std::io;
use std::path::Path;
use tokio::net::TcpListener;

use super::TransportStream;

/// A bound listener that accepts incoming [`TransportStream`] connections.
pub enum TransportListener {
    /// Unix domain socket listener (Tier 1).
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),

    /// TCP listener (Tier 2 — universal).
    Tcp(TcpListener),
}

impl TransportListener {
    /// Bind a Unix socket. On Windows, binds TCP on localhost and writes a port-file.
    #[cfg(unix)]
    pub async fn bind_unix(path: &Path) -> io::Result<Self> {
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
        let listener = tokio::net::UnixListener::bind(path)?;
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
                .map(|a| format!("{a:?}"))
                .unwrap_or_else(|_| "unix://<unknown>".to_string()),
            Self::Tcp(l) => l
                .local_addr()
                .map(|a| format!("tcp://{a}"))
                .unwrap_or_else(|_| "tcp://<unknown>".to_string()),
        }
    }
}
