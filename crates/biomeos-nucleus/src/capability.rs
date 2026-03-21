// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Layer 3: Capability Verification
//!
//! Verifies that discovered primals actually have the capabilities they claim.
//! This is done by direct query to the primal's Unix socket.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{Error, Result, discovery::DiscoveredPrimal, identity::IdentityProof};

/// Capability information (from primal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Primal name
    pub primal: String,
    /// Version
    pub version: String,
    /// Family ID
    pub family_id: String,
    /// Node ID
    pub node_id: String,
    /// Capabilities
    pub capabilities: Vec<String>,
}

/// Capability verification result
#[derive(Debug, Clone)]
pub struct CapabilityVerification {
    /// Whether verification succeeded
    pub verified: bool,
    /// Expected capabilities
    pub expected: Vec<String>,
    /// Actual capabilities
    pub actual: Vec<String>,
    /// Message
    pub message: String,
}

/// Capability verification layer
#[async_trait]
pub trait CapabilityLayer: Send + Sync {
    /// Query capabilities from a primal
    async fn query_capabilities(&self, endpoint: &str) -> Result<CapabilityInfo>;

    /// Verify capabilities match expected
    async fn verify_capabilities(
        &self,
        discovered: &DiscoveredPrimal,
        _identity: &IdentityProof,
    ) -> Result<CapabilityVerification> {
        // Get primary endpoint
        let endpoint = discovered
            .endpoints
            .first()
            .ok_or_else(|| Error::invalid_response(&discovered.primal, "No endpoints available"))?;

        // Query actual capabilities
        let actual_caps = self.query_capabilities(&endpoint.address).await?;

        // Verify capabilities match
        let expected = &discovered.capabilities;
        let actual = &actual_caps.capabilities;

        // Check if all expected capabilities are present
        if expected.iter().any(|cap| !actual.contains(cap)) {
            return Err(Error::capability_mismatch(expected.clone(), actual.clone()));
        }

        info!(
            primal = %discovered.primal,
            capabilities = ?actual,
            "Capability verification successful"
        );

        Ok(CapabilityVerification {
            verified: true,
            expected: expected.clone(),
            actual: actual.clone(),
            message: "All capabilities verified".to_string(),
        })
    }
}

/// Capability layer implementation
pub struct CapabilityLayerImpl;

impl CapabilityLayerImpl {
    /// Create a new capability layer
    pub fn new() -> Self {
        info!("Initializing NUCLEUS Capability Layer");
        Self
    }
}

impl Default for CapabilityLayerImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityLayer for CapabilityLayerImpl {
    async fn query_capabilities(&self, endpoint: &str) -> Result<CapabilityInfo> {
        debug!(endpoint = %endpoint, "Querying capabilities from primal");

        let params = serde_json::json!({});

        let response: serde_json::Value =
            crate::client::call_unix_socket_rpc(endpoint, "get_capabilities", params).await?;

        // Parse capability info
        let cap_info: CapabilityInfo = serde_json::from_value(response)?;

        debug!(
            primal = %cap_info.primal,
            capabilities = ?cap_info.capabilities,
            "Received capability info"
        );

        Ok(cap_info)
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_info_parsing() {
        let json = r#"{
            "primal": "beardog",
            "version": "0.15.2",
            "family_id": "1894e909e454",
            "node_id": "node-alpha",
            "capabilities": ["encryption", "identity", "trust"]
        }"#;

        let info: CapabilityInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.primal, "beardog");
        assert_eq!(info.capabilities.len(), 3);
        assert!(info.capabilities.contains(&"encryption".to_string()));
    }

    #[test]
    fn test_capability_verification_logic() {
        let expected = ["encryption".to_string(), "identity".to_string()];
        let actual = [
            "encryption".to_string(),
            "identity".to_string(),
            "trust".to_string(),
        ];

        // All expected capabilities are present
        assert!(expected.iter().all(|cap| actual.contains(cap)));
    }

    #[test]
    fn test_capability_verification_missing() {
        let expected = ["encryption".to_string(), "identity".to_string()];
        let actual = ["encryption".to_string()]; // Missing "identity"

        let missing: Vec<_> = expected
            .iter()
            .filter(|cap| !actual.contains(cap))
            .collect();

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], &"identity".to_string());
    }
}
