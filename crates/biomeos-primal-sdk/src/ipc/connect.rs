// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Transport connection — dispatch through [`TransportEndpoint`] to a connected stream.

use std::io;
use std::time::Duration;
use tokio::time::timeout;

use super::TransportEndpoint;
use super::TransportStream;

/// Connect to a [`TransportEndpoint`], returning a platform-appropriate stream.
///
/// - `UnixSocket` → UDS on Unix, TCP via port-file on Windows.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> io::Result<TransportStream> {
    match endpoint {
        TransportEndpoint::UnixSocket { path } => connect_unix_or_portfile(path).await,
    }
}

/// Connect with a timeout.
pub async fn connect_transport_timed(
    endpoint: &TransportEndpoint,
    duration: Duration,
) -> io::Result<TransportStream> {
    timeout(duration, connect_transport(endpoint))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "transport connect timeout"))?
}

/// On Unix: connect via UDS. On Windows: read `{path}.port` and connect via TCP.
#[cfg(unix)]
async fn connect_unix_or_portfile(path: &std::path::Path) -> io::Result<TransportStream> {
    let stream = tokio::net::UnixStream::connect(path).await?;
    Ok(TransportStream::Unix(stream))
}

#[cfg(windows)]
async fn connect_unix_or_portfile(path: &std::path::Path) -> io::Result<TransportStream> {
    use tokio::net::TcpStream;

    let port_file = path.with_extension("port");
    let port_str = tokio::fs::read_to_string(&port_file).await.map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "No Unix sockets on Windows and port-file not found: {}",
                port_file.display()
            ),
        )
    })?;

    let port: u16 = port_str.trim().parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid port in {}: {port_str:?}", port_file.display()),
        )
    })?;

    let stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
    Ok(TransportStream::Tcp(stream))
}
