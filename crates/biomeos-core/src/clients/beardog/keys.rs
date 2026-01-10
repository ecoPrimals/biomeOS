// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Key generation and lifecycle management
//!
//! This module manages cryptographic key operations:
//! - Key generation (symmetric and asymmetric)
//! - Key metadata retrieval
//! - Future: Key rotation, revocation, backup

use super::client::BearDogClient;
use super::types::KeyInfo;
use crate::clients::transport::TransportClient;
use anyhow::{Context, Result};
use serde_json::json;
use tracing::{debug, info};

impl BearDogClient {
    /// Generate a new cryptographic key
    ///
    /// Uses BearDog's JSON-RPC API: `keys.generate`
    ///
    /// # Arguments
    /// * `key_type` - Type of key to generate:
    ///   - "AES-256" - Symmetric encryption
    ///   - "Ed25519" - Digital signatures
    ///   - "RSA-2048" - Asymmetric encryption
    ///   - "RSA-4096" - High-security asymmetric
    /// * `key_id` - Identifier for the new key
    ///
    /// # Returns
    /// Key metadata including ID, type, and creation timestamp
    ///
    /// # Errors
    /// Returns an error if:
    /// - Key type is not supported
    /// - Key ID already exists
    /// - Key generation fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// 
    /// // Generate signing key
    /// let key = beardog.generate_key("Ed25519", "my-signing-key").await?;
    /// println!("Generated key: {} ({})", key.key_id, key.key_type);
    /// 
    /// // Generate encryption key
    /// let key2 = beardog.generate_key("AES-256", "my-encryption-key").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_key(&self, key_type: &str, key_id: &str) -> Result<KeyInfo> {
        info!("🔑 Generating {} key with ID '{}'", key_type, key_id);

        let response = self
            .transport
            .call(
                "keys.generate",
                Some(json!({
                    "key_type": key_type,
                    "key_id": key_id,
                    "family_id": self.family_id,
                    "options": {}
                })),
            )
            .await
            .context("Failed to call keys.generate")?;

        debug!("✅ Key generation successful");

        Ok(KeyInfo {
            key_id: response["key_id"]
                .as_str()
                .unwrap_or(key_id)
                .to_string(),
            key_type: response["key_type"]
                .as_str()
                .unwrap_or(key_type)
                .to_string(),
            status: response["status"]
                .as_str()
                .unwrap_or("active")
                .to_string(),
            created_at: response["created_at"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            metadata: response["metadata"].clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_generate_key() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let key = client
            .generate_key("Ed25519", "test-key")
            .await
            .unwrap();
        assert_eq!(key.key_id, "test-key");
        assert_eq!(key.key_type, "Ed25519");
    }
}
