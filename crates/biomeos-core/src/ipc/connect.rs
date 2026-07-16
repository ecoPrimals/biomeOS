// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Transport connection — dispatch through [`TransportEndpoint`] to a connected stream.

use std::io;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::TransportStream;
use crate::TransportEndpoint;

/// Connect to a [`TransportEndpoint`], returning a platform-appropriate stream.
///
/// - `UnixSocket` → UDS on Unix, TCP via port-file on Windows.
/// - `AbstractSocket` → Abstract UDS on Linux, error on other platforms.
/// - `TcpSocket` → TCP everywhere.
/// - `HttpJsonRpc` → TCP (HTTP framing handled at a higher layer).
pub async fn connect_transport(endpoint: &TransportEndpoint) -> io::Result<TransportStream> {
    match endpoint {
        TransportEndpoint::UnixSocket { path } => connect_unix_or_portfile(path).await,

        TransportEndpoint::AbstractSocket { name } => connect_abstract(name).await,

        TransportEndpoint::TcpSocket { host, port } | TransportEndpoint::HttpJsonRpc { host, port } => {
            let addr = format!("{host}:{port}");
            let stream = TcpStream::connect(&addr).await?;
            Ok(TransportStream::Tcp(stream))
        }
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

/// Abstract sockets — Linux only.
#[cfg(target_os = "linux")]
async fn connect_abstract(name: &str) -> io::Result<TransportStream> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;

    let addr = SocketAddr::from_abstract_name(name).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid abstract socket name @{name}: {e}"),
        )
    })?;

    let std_stream = std::os::unix::net::UnixStream::connect_addr(&addr)?;
    std_stream.set_nonblocking(true)?;
    let stream = tokio::net::UnixStream::from_std(std_stream)?;
    Ok(TransportStream::Unix(stream))
}

#[cfg(all(unix, not(target_os = "linux")))]
async fn connect_abstract(name: &str) -> io::Result<TransportStream> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("Abstract sockets only supported on Linux (requested: @{name})"),
    ))
}

#[cfg(windows)]
async fn connect_abstract(name: &str) -> io::Result<TransportStream> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("Abstract sockets not supported on Windows (requested: @{name})"),
    ))
}
