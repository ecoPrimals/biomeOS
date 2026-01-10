// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Low-level BTSP tunnel management
//!
//! This module provides direct control over BTSP (BirdSong Tunnel Protocol) tunnels:
//! - Tunnel establishment
//! - Tunnel status monitoring
//! - Tunnel lifecycle management
//!
//! # BTSP Protocol
//!
//! BTSP is a secure, encrypted peer-to-peer tunneling protocol that provides:
//! - **End-to-end encryption** using BearDog's cryptographic primitives
//! - **UDP multicast discovery** via Songbird
//! - **NAT traversal** through hole-punching
//! - **P2P direct connections** for optimal latency
//!
//! # Architecture
//!
//! ```text
//! Node A                    BearDog                    Node B
//!   |                          |                          |
//!   |-- establish_tunnel ----->|                          |
//!   |                          |--- UDP discovery ------->|
//!   |                          |<-- peer response --------|
//!   |<-- tunnel_id -----------|                          |
//!   |                          |<======== P2P tunnel ========>
//!   |-- get_tunnel_status --->|                          |
//!   |<-- status: active -------|                          |
//!   |                          |                          |
//!   |-- close_tunnel --------->|                          |
//!   |<-- closed ---------------|                          |
//! ```

use super::client::BearDogClient;
use super::types::{TunnelInfo, TunnelStatus};
use crate::clients::transport::TransportClient;
use anyhow::{Context, Result};
use serde_json::json;
use tracing::{debug, info};

impl BearDogClient {
    /// Establish a BTSP tunnel to a peer
    ///
    /// Uses BearDog's JSON-RPC API: `btsp.tunnel_establish`
    ///
    /// # Arguments
    /// * `peer_id` - Peer node identifier
    /// * `endpoint` - Peer endpoint (e.g., "192.168.1.100:9091")
    ///
    /// # Returns
    /// Tunnel information including tunnel_id
    ///
    /// # Errors
    /// Returns an error if tunnel establishment fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// 
    /// // Establish tunnel to peer
    /// let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
    /// println!("✅ Tunnel established: {}", tunnel.tunnel_id);
    /// println!("   Peer: {}", tunnel.peer_id);
    /// 
    /// // Use tunnel for secure communication...
    /// 
    /// // Close when done
    /// beardog.close_tunnel(&tunnel.tunnel_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish_tunnel(&self, peer_id: &str, endpoint: &str) -> Result<TunnelInfo> {
        info!("🔗 Establishing BTSP tunnel to peer '{}' at {}", peer_id, endpoint);

        let response = self
            .transport
            .call(
                "btsp.tunnel_establish",
                Some(json!({
                    "peer_id": peer_id,
                    "endpoint": endpoint,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call btsp.tunnel_establish")?;

        let tunnel_info: TunnelInfo = serde_json::from_value(response)
            .context("Failed to parse tunnel info from response")?;

        debug!("✅ Tunnel established: {}", tunnel_info.tunnel_id);

        Ok(tunnel_info)
    }

    /// Get BTSP tunnel status
    ///
    /// Uses BearDog's JSON-RPC API: `btsp.tunnel_status`
    ///
    /// # Arguments
    /// * `tunnel_id` - Tunnel identifier
    ///
    /// # Returns
    /// Detailed tunnel status including state, statistics, and security info
    ///
    /// # Errors
    /// Returns an error if the tunnel doesn't exist or status check fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
    /// 
    /// // Check tunnel status
    /// let status = beardog.get_tunnel_status(&tunnel.tunnel_id).await?;
    /// println!("Tunnel state: {}", status.state);
    /// println!("Bytes sent: {}", status.bytes_sent);
    /// println!("Bytes received: {}", status.bytes_received);
    /// 
    /// // Monitor tunnel health
    /// match status.state.as_str() {
    ///     "active" => println!("✅ Tunnel is healthy"),
    ///     "failed" => println!("❌ Tunnel has failed"),
    ///     _ => println!("⏳ Tunnel state: {}", status.state),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tunnel_status(&self, tunnel_id: &str) -> Result<TunnelStatus> {
        debug!("📊 Querying status for tunnel '{}'", tunnel_id);

        let response = self
            .transport
            .call(
                "btsp.tunnel_status",
                Some(json!({
                    "tunnel_id": tunnel_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .with_context(|| format!("Failed to get status for tunnel {}", tunnel_id))?;

        let status: TunnelStatus = serde_json::from_value(response)
            .context("Failed to parse tunnel status from response")?;

        debug!("✅ Tunnel status: {} ({})", tunnel_id, status.state);

        Ok(status)
    }

    /// Close a BTSP tunnel
    ///
    /// Uses BearDog's JSON-RPC API: `btsp.tunnel_close`
    ///
    /// # Arguments
    /// * `tunnel_id` - Tunnel identifier
    ///
    /// # Errors
    /// Returns an error if tunnel closure fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
    /// 
    /// // Use tunnel...
    /// 
    /// // Gracefully close tunnel
    /// beardog.close_tunnel(&tunnel.tunnel_id).await?;
    /// println!("✅ Tunnel closed gracefully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()> {
        info!("🔒 Closing BTSP tunnel '{}'", tunnel_id);

        self.transport
            .call(
                "btsp.tunnel_close",
                Some(json!({
                    "tunnel_id": tunnel_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .with_context(|| format!("Failed to close tunnel {}", tunnel_id))?;

        debug!("✅ Tunnel closed: {}", tunnel_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_establish_tunnel() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let tunnel = client
            .establish_tunnel("test-peer", "192.168.1.100:9091")
            .await
            .unwrap();
        assert!(!tunnel.tunnel_id.is_empty());
        assert_eq!(tunnel.peer_id, "test-peer");
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_get_tunnel_status() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let tunnel = client
            .establish_tunnel("test-peer", "192.168.1.100:9091")
            .await
            .unwrap();
        let status = client.get_tunnel_status(&tunnel.tunnel_id).await.unwrap();
        assert!(!status.state.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_close_tunnel() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let tunnel = client
            .establish_tunnel("test-peer", "192.168.1.100:9091")
            .await
            .unwrap();
        client.close_tunnel(&tunnel.tunnel_id).await.unwrap();
    }
}
