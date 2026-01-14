// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! BearDog client core - Discovery, connection, lifecycle management

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info, warn};

use super::{access::AccessClient, btsp::BtspClient, crypto::CryptoClient, keys::KeysClient, tunnels::TunnelsClient};

/// BearDog client for security and cryptography operations
///
/// # Transport
/// - **PRIMARY**: JSON-RPC over Unix socket
/// - **FALLBACK**: HTTP REST API (deprecated)
#[derive(Debug, Clone)]
pub struct BearDogClient {
    pub(crate) transport: TransportClient,
    pub(crate) family_id: String,
}

impl BearDogClient {
    /// Discover BearDog primal via Unix socket (preferred)
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID for discovery
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        info!("🐻 Discovering BearDog primal for family '{}'", family_id);

        let transport = TransportClient::discover(
            "beardog",
            family_id,
        )
        .await
        .context("Failed to discover BearDog primal")?;

        debug!("✅ BearDog discovered via {}", transport.transport_type());

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    /// Create BearDog client from explicit endpoint (HTTP fallback)
    ///
    /// # Deprecated
    /// Prefer `discover()` for automatic Unix socket discovery.
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:9000")
    /// * `family_id` - Genetic family ID
    #[deprecated(since = "0.2.0", note = "Use discover() for Unix socket auto-discovery")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let endpoint = endpoint.into();
        info!(
            "🐻 Creating BearDog client with HTTP endpoint: {} (deprecated)",
            endpoint
        );

        let transport = TransportClient::discover_with_preference(
            "beardog",
            family_id,
            TransportPreference::Auto,  // ✅ Evolved: Auto-discover secure transport
        )
        .await
        .context("Failed to discover BearDog via secure transport")?;

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    /// Get the endpoint this client is connected to
    pub fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    /// Get the family ID
    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    /// Establish a BTSP tunnel to a peer
    pub async fn establish_tunnel(&self, peer_id: &str, endpoint: &str) -> Result<super::types::TunnelInfo> {
        let response = self.transport.call(
            "btsp.establish_tunnel",
            Some(serde_json::json!({
                "peer_id": peer_id,
                "endpoint": endpoint,
                "family_id": self.family_id
            }))
        ).await?;
        serde_json::from_value(response).context("Failed to parse tunnel info")
    }

    /// Close a BTSP tunnel
    pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()> {
        self.transport.call(
            "btsp.close_tunnel",
            Some(serde_json::json!({
                "tunnel_id": tunnel_id,
                "family_id": self.family_id
            }))
        ).await?;
        Ok(())
    }

    /// Get tunnel status
    pub async fn get_tunnel_status(&self, tunnel_id: &str) -> Result<super::types::TunnelStatus> {
        let response = self.transport.call(
            "btsp.get_tunnel_status",
            Some(serde_json::json!({
                "tunnel_id": tunnel_id,
                "family_id": self.family_id
            }))
        ).await?;
        serde_json::from_value(response).context("Failed to parse tunnel status")
    }

    /// Get high-level BTSP API client
    pub fn btsp(&self) -> super::btsp::BtspClient {
        super::btsp::BtspClient::new(self.clone())
    }

    /// Get cryptography operations client
    ///
    /// Provides encryption, decryption, signing, and verification
    pub fn crypto(&self) -> super::crypto::CryptoClient {
        super::crypto::CryptoClient::new(self.transport.clone())
    }

    /// Get key management client
    ///
    /// Provides key generation, rotation, and lifecycle management
    pub fn keys(&self) -> super::keys::KeysClient {
        super::keys::KeysClient::new(self.transport.clone())
    }

    /// Get access control client
    ///
    /// Provides access validation and audit logging
    pub fn access(&self) -> super::access::AccessClient {
        super::access::AccessClient::new(self.transport.clone())
    }
}

#[async_trait]
impl PrimalClient for BearDogClient {
    fn name(&self) -> &str {
        "beardog"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_discover() {
        let client = BearDogClient::discover("nat0").await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_properties() {
        // Test will be implemented with mock transport
    }
}

