// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! tarpc Client Connector for Neural Router
//!
//! Connects to primal tarpc sockets using the native binary protocol.
//! Used by `forward_via_tarpc()` for health.check, discovery.*, security.* methods.
//!
//! Falls back to JSON-RPC if tarpc socket doesn't exist or connection fails.

use anyhow::{Context, Result};
use biomeos_types::tarpc_types::{DiscoveryRpcClient, HealthRpcClient, SecurityRpcClient};
use std::path::Path;
use tarpc::client;
#[cfg(windows)]
use tarpc::serde_transport::tcp;
#[cfg(unix)]
use tarpc::serde_transport::unix;
use tokio_serde::formats::Bincode;

/// Connect to a primal's tarpc socket and return a `HealthRpcClient`.
///
/// Uses Bincode for binary serialization. Returns error if socket doesn't exist
/// or connection fails — caller should fall back to JSON-RPC.
#[expect(clippy::needless_return, reason = "cfg(unix) vs cfg(windows) branches both return")]
pub async fn connect_tarpc_health(socket_path: &Path) -> Result<HealthRpcClient> {
    #[cfg(unix)]
    {
        let transport = unix::connect(socket_path, Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc socket: {}",
                    socket_path.display()
                )
            })?;
        return Ok(HealthRpcClient::new(client::Config::default(), transport).spawn());
    }

    #[cfg(windows)]
    {
        let port = read_tarpc_port(socket_path).await?;
        let transport = tcp::connect(format!("127.0.0.1:{port}"), Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc TCP endpoint for {}",
                    socket_path.display()
                )
            })?;
        Ok(HealthRpcClient::new(client::Config::default(), transport).spawn())
    }
}

/// Connect to a primal's tarpc socket and return a `DiscoveryRpcClient`.
#[expect(clippy::needless_return, reason = "cfg(unix) vs cfg(windows) branches both return")]
pub async fn connect_tarpc_discovery(socket_path: &Path) -> Result<DiscoveryRpcClient> {
    #[cfg(unix)]
    {
        let transport = unix::connect(socket_path, Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc socket: {}",
                    socket_path.display()
                )
            })?;
        return Ok(DiscoveryRpcClient::new(client::Config::default(), transport).spawn());
    }

    #[cfg(windows)]
    {
        let port = read_tarpc_port(socket_path).await?;
        let transport = tcp::connect(format!("127.0.0.1:{port}"), Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc TCP endpoint for {}",
                    socket_path.display()
                )
            })?;
        Ok(DiscoveryRpcClient::new(client::Config::default(), transport).spawn())
    }
}

/// Connect to a primal's tarpc socket and return a `SecurityRpcClient`.
#[expect(clippy::needless_return, reason = "cfg(unix) vs cfg(windows) branches both return")]
pub async fn connect_tarpc_security(socket_path: &Path) -> Result<SecurityRpcClient> {
    #[cfg(unix)]
    {
        let transport = unix::connect(socket_path, Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc socket: {}",
                    socket_path.display()
                )
            })?;
        return Ok(SecurityRpcClient::new(client::Config::default(), transport).spawn());
    }

    #[cfg(windows)]
    {
        let port = read_tarpc_port(socket_path).await?;
        let transport = tcp::connect(format!("127.0.0.1:{port}"), Bincode::default)
            .await
            .with_context(|| {
                format!(
                    "Failed to connect to tarpc TCP endpoint for {}",
                    socket_path.display()
                )
            })?;
        Ok(SecurityRpcClient::new(client::Config::default(), transport).spawn())
    }
}

#[cfg(windows)]
async fn read_tarpc_port(socket_path: &Path) -> Result<u16> {
    let port_file = socket_path.with_extension("port");
    let port_str = tokio::fs::read_to_string(&port_file)
        .await
        .with_context(|| {
            format!(
                "No Unix tarpc on Windows and port-file not found: {}",
                port_file.display()
            )
        })?;
    port_str
        .trim()
        .parse()
        .with_context(|| format!("Invalid port in {}: {port_str:?}", port_file.display()))
}

#[cfg(all(test, unix))]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn connect_tarpc_health_nonexistent_socket_reports_path() {
        let p = Path::new("/nonexistent/biomeos/tarpc-health-missing.sock");
        let err = connect_tarpc_health(p).await.unwrap_err();
        let s = format!("{err:#}");
        assert!(
            s.contains("Failed to connect") || s.contains("connect"),
            "{s}"
        );
        assert!(s.contains("tarpc-health-missing") || s.contains("nonexistent"));
    }

    #[tokio::test]
    async fn connect_tarpc_discovery_nonexistent_socket_is_error() {
        let p = Path::new("/nonexistent/biomeos/tarpc-discovery-missing.sock");
        assert!(connect_tarpc_discovery(p).await.is_err());
    }

    #[tokio::test]
    async fn connect_tarpc_security_nonexistent_socket_is_error() {
        let p = Path::new("/nonexistent/biomeos/tarpc-security-missing.sock");
        assert!(connect_tarpc_security(p).await.is_err());
    }
}
