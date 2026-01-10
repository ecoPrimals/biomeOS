// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! High-level BTSP API
//!
//! This module provides ergonomic, user-friendly wrappers around the low-level
//! BTSP tunnel operations. It includes:
//! - Simplified tunnel management
//! - Automatic tunnel monitoring
//! - Graceful error handling
//! - Fluent API design
//!
//! # Philosophy
//!
//! While `tunnels.rs` provides direct, low-level access to BTSP operations,
//! this module offers a **high-level, developer-friendly API** for common
//! use cases.
//!
//! # Example Workflows
//!
//! ## Simple tunnel establishment:
//! ```no_run
//! # use biomeos_core::clients::beardog::BearDogClient;
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let beardog = BearDogClient::discover("nat0").await?;
//! let tunnel = beardog.btsp().connect("peer-node-1", "192.168.1.100:9091").await?;
//! println!("Connected via: {}", tunnel.tunnel_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Tunnel lifecycle:
//! ```no_run
//! # use biomeos_core::clients::beardog::BearDogClient;
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let beardog = BearDogClient::discover("nat0").await?;
//! let btsp = beardog.btsp();
//!
//! // Establish
//! let tunnel = btsp.connect("peer", "192.168.1.100:9091").await?;
//!
//! // Monitor
//! if btsp.is_active(&tunnel.tunnel_id).await? {
//!     println!("Tunnel is active!");
//! }
//!
//! // Close
//! btsp.disconnect(&tunnel.tunnel_id).await?;
//! # Ok(())
//! # }
//! ```

use super::client::BearDogClient;
use super::types::{TunnelInfo, TunnelStatus};
use crate::clients::transport::TransportClient;
use anyhow::{Context, Result};
use tracing::{debug, info};

/// High-level BTSP API client
#[derive(Debug, Clone)]
pub struct BtspClient {
    beardog: BearDogClient,
}

impl BtspClient {
    /// Creates a new BTSP client wrapping a BearDog client
    pub(crate) fn new(beardog: BearDogClient) -> Self {
        Self { beardog }
    }

    /// Establish a connection to a peer via BTSP
    ///
    /// This is an ergonomic wrapper around `establish_tunnel` with simpler naming.
    ///
    /// # Arguments
    /// * `peer_id` - Identifier of the peer node
    /// * `endpoint` - Network endpoint of the peer
    ///
    /// # Returns
    /// Tunnel information for the established connection
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// 
    /// // Simple, intuitive API
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// println!("Connected! Tunnel: {}", tunnel.tunnel_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&self, peer_id: &str, endpoint: &str) -> Result<TunnelInfo> {
        info!("🌐 BTSP: Connecting to peer '{}' at {}", peer_id, endpoint);
        self.beardog.establish_tunnel(peer_id, endpoint).await
    }

    /// Disconnect a BTSP tunnel
    ///
    /// This is an ergonomic wrapper around `close_tunnel` with simpler naming.
    ///
    /// # Arguments
    /// * `tunnel_id` - Identifier of the tunnel to close
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// 
    /// // Simple disconnect
    /// beardog.btsp().disconnect(&tunnel.tunnel_id).await?;
    /// println!("Disconnected!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&self, tunnel_id: &str) -> Result<()> {
        info!("🔌 BTSP: Disconnecting tunnel '{}'", tunnel_id);
        self.beardog.close_tunnel(tunnel_id).await
    }

    /// Check if a tunnel is currently active
    ///
    /// # Arguments
    /// * `tunnel_id` - Identifier of the tunnel to check
    ///
    /// # Returns
    /// `true` if the tunnel state is "active", `false` otherwise
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// 
    /// // Simple health check
    /// if beardog.btsp().is_active(&tunnel.tunnel_id).await? {
    ///     println!("✅ Tunnel is active");
    /// } else {
    ///     println!("⚠️  Tunnel is not active");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_active(&self, tunnel_id: &str) -> Result<bool> {
        let status = self.beardog.get_tunnel_status(tunnel_id).await?;
        Ok(status.state == "active")
    }

    /// Get detailed tunnel status
    ///
    /// # Arguments
    /// * `tunnel_id` - Identifier of the tunnel
    ///
    /// # Returns
    /// Complete tunnel status information
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// 
    /// // Get detailed status
    /// let status = beardog.btsp().status(&tunnel.tunnel_id).await?;
    /// println!("State: {}", status.state);
    /// println!("Bytes sent: {}", status.bytes_sent);
    /// println!("Bytes received: {}", status.bytes_received);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn status(&self, tunnel_id: &str) -> Result<TunnelStatus> {
        self.beardog.get_tunnel_status(tunnel_id).await
    }

    /// Get tunnel statistics as a formatted string
    ///
    /// # Arguments
    /// * `tunnel_id` - Identifier of the tunnel
    ///
    /// # Returns
    /// Human-readable statistics string
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// 
    /// // Print formatted stats
    /// let stats = beardog.btsp().stats(&tunnel.tunnel_id).await?;
    /// println!("{}", stats);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stats(&self, tunnel_id: &str) -> Result<String> {
        let status = self.beardog.get_tunnel_status(tunnel_id).await?;
        
        Ok(format!(
            "BTSP Tunnel Stats ({})\n\
             State: {}\n\
             Peer: {}\n\
             Bytes Sent: {} ({} MB)\n\
             Bytes Received: {} ({} MB)\n\
             Encryption: {}",
            tunnel_id,
            status.state,
            status.peer_id,
            status.bytes_sent,
            status.bytes_sent / 1_000_000,
            status.bytes_received,
            status.bytes_received / 1_000_000,
            status.encryption_algorithm.as_deref().unwrap_or("unknown")
        ))
    }

    /// Wait for a tunnel to become active
    ///
    /// Polls the tunnel status until it reaches "active" state or times out.
    ///
    /// # Arguments
    /// * `tunnel_id` - Identifier of the tunnel
    /// * `max_attempts` - Maximum number of status checks (default: 10)
    /// * `interval_ms` - Milliseconds between checks (default: 500)
    ///
    /// # Returns
    /// `Ok(())` if tunnel becomes active, error if timeout or failure
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let tunnel = beardog.btsp().connect("peer-alpha", "192.168.1.10:9091").await?;
    /// 
    /// // Wait for tunnel to be ready
    /// beardog.btsp().wait_for_active(&tunnel.tunnel_id, 20, 250).await?;
    /// println!("✅ Tunnel is active and ready!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_active(
        &self,
        tunnel_id: &str,
        max_attempts: usize,
        interval_ms: u64,
    ) -> Result<()> {
        for attempt in 1..=max_attempts {
            let status = self.beardog.get_tunnel_status(tunnel_id).await?;
            
            match status.state.as_str() {
                "active" => {
                    debug!("✅ Tunnel '{}' is active after {} attempt(s)", tunnel_id, attempt);
                    return Ok(());
                }
                "failed" => {
                    return Err(anyhow::anyhow!(
                        "Tunnel '{}' failed to establish",
                        tunnel_id
                    ));
                }
                "closed" => {
                    return Err(anyhow::anyhow!(
                        "Tunnel '{}' was closed before becoming active",
                        tunnel_id
                    ));
                }
                state => {
                    debug!(
                        "⏳ Tunnel '{}' state: {} (attempt {}/{})",
                        tunnel_id, state, attempt, max_attempts
                    );
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
        }
        
        Err(anyhow::anyhow!(
            "Timeout waiting for tunnel '{}' to become active",
            tunnel_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_btsp_connect() {
        let beardog = BearDogClient::discover("nat0").await.unwrap();
        let btsp = beardog.btsp();
        let tunnel = btsp.connect("test-peer", "192.168.1.100:9091").await.unwrap();
        assert!(!tunnel.tunnel_id.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_btsp_is_active() {
        let beardog = BearDogClient::discover("nat0").await.unwrap();
        let btsp = beardog.btsp();
        let tunnel = btsp.connect("test-peer", "192.168.1.100:9091").await.unwrap();
        let is_active = btsp.is_active(&tunnel.tunnel_id).await.unwrap();
        // State may vary, just verify we can check
        println!("Tunnel active: {}", is_active);
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_btsp_stats() {
        let beardog = BearDogClient::discover("nat0").await.unwrap();
        let btsp = beardog.btsp();
        let tunnel = btsp.connect("test-peer", "192.168.1.100:9091").await.unwrap();
        let stats = btsp.stats(&tunnel.tunnel_id).await.unwrap();
        assert!(stats.contains("BTSP Tunnel Stats"));
        println!("{}", stats);
    }
}
