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
use tarpc::serde_transport::unix;
use tokio_serde::formats::Bincode;

/// Connect to a primal's tarpc socket and return a `HealthRpcClient`.
///
/// Uses Bincode for binary serialization. Returns error if socket doesn't exist
/// or connection fails — caller should fall back to JSON-RPC.
pub async fn connect_tarpc_health(socket_path: &Path) -> Result<HealthRpcClient> {
    let transport = unix::connect(socket_path, Bincode::default)
        .await
        .with_context(|| {
            format!(
                "Failed to connect to tarpc socket: {}",
                socket_path.display()
            )
        })?;

    let client = HealthRpcClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

/// Connect to a primal's tarpc socket and return a `DiscoveryRpcClient`.
pub async fn connect_tarpc_discovery(socket_path: &Path) -> Result<DiscoveryRpcClient> {
    let transport = unix::connect(socket_path, Bincode::default)
        .await
        .with_context(|| {
            format!(
                "Failed to connect to tarpc socket: {}",
                socket_path.display()
            )
        })?;

    let client = DiscoveryRpcClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

/// Connect to a primal's tarpc socket and return a `SecurityRpcClient`.
pub async fn connect_tarpc_security(socket_path: &Path) -> Result<SecurityRpcClient> {
    let transport = unix::connect(socket_path, Bincode::default)
        .await
        .with_context(|| {
            format!(
                "Failed to connect to tarpc socket: {}",
                socket_path.display()
            )
        })?;

    let client = SecurityRpcClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

#[cfg(test)]
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
