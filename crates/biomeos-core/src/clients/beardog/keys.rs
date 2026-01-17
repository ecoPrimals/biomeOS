//! BearDog Key Management Client
//!
//! Provides key generation, rotation, and lifecycle management using BearDog's JSON-RPC API.
//!
//! Uses the real BearDog `keys.*` methods discovered from v0.9.0+

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::clients::transport::PrimalTransport;

/// Key management client for BearDog
///
/// Provides real key generation, listing, and rotation via JSON-RPC
pub struct KeysClient {
    transport: PrimalTransport,
}

impl KeysClient {
    /// Create a new keys client with the given transport
    pub fn new(transport: PrimalTransport) -> Self {
        Self { transport }
    }

    /// Generate a new key using BearDog's `keys.generate` method
    ///
    /// # Arguments
    /// * `key_type` - Type of key to generate ("Ed25519", "RSA-4096", etc.)
    /// * `key_id` - Optional key ID (BearDog will generate if not provided)
    ///
    /// # Returns
    /// * Key information for the newly generated key
    pub async fn generate(&self, key_type: &str, key_id: Option<&str>) -> Result<KeyInfo> {
        let mut params = serde_json::json!({
            "key_type": key_type,
        });

        if let Some(id) = key_id {
            params["key_id"] = serde_json::Value::String(id.to_string());
        }

        let response = self
            .transport
            .call_method("keys.generate", Some(params))
            .await
            .context("Failed to call keys.generate")?;

        Ok(KeyInfo {
            key_id: response["key_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing key_id in response"))?
                .to_string(),
            key_type: response["key_type"]
                .as_str()
                .unwrap_or(key_type)
                .to_string(),
            created_at: response["created_at"].as_u64().unwrap_or(0),
            expires_at: response["expires_at"].as_u64(),
            status: response["status"].as_str().unwrap_or("active").to_string(),
        })
    }

    /// List all keys using BearDog's `keys.list` method
    ///
    /// # Returns
    /// * Vector of key information structures
    pub async fn list(&self) -> Result<Vec<KeyInfo>> {
        let response = self
            .transport
            .call_method("keys.list", None)
            .await
            .context("Failed to call keys.list")?;

        let keys_array = response["keys"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected 'keys' array in response"))?;

        let mut keys = Vec::new();
        for key in keys_array {
            keys.push(KeyInfo {
                key_id: key["key_id"].as_str().unwrap_or("unknown").to_string(),
                key_type: key["key_type"].as_str().unwrap_or("unknown").to_string(),
                created_at: key["created_at"].as_u64().unwrap_or(0),
                expires_at: key["expires_at"].as_u64(),
                status: key["status"].as_str().unwrap_or("unknown").to_string(),
            });
        }

        Ok(keys)
    }

    /// Get information about a specific key using BearDog's `keys.info` method
    ///
    /// # Arguments
    /// * `key_id` - Key ID to query
    ///
    /// # Returns
    /// * Key information structure
    pub async fn info(&self, key_id: &str) -> Result<KeyInfo> {
        let response = self
            .transport
            .call_method(
                "keys.info",
                Some(serde_json::json!({
                    "key_id": key_id,
                })),
            )
            .await
            .context("Failed to call keys.info")?;

        Ok(KeyInfo {
            key_id: response["key_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing key_id in response"))?
                .to_string(),
            key_type: response["key_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            created_at: response["created_at"].as_u64().unwrap_or(0),
            expires_at: response["expires_at"].as_u64(),
            status: response["status"].as_str().unwrap_or("unknown").to_string(),
        })
    }

    /// Rotate a key using BearDog's `keys.rotate` method
    ///
    /// # Arguments
    /// * `key_id` - Key ID to rotate
    ///
    /// # Returns
    /// * New key information after rotation
    pub async fn rotate(&self, key_id: &str) -> Result<KeyInfo> {
        let response = self
            .transport
            .call_method(
                "keys.rotate",
                Some(serde_json::json!({
                    "key_id": key_id,
                })),
            )
            .await
            .context("Failed to call keys.rotate")?;

        Ok(KeyInfo {
            key_id: response["new_key_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing new_key_id in response"))?
                .to_string(),
            key_type: response["key_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            created_at: response["created_at"].as_u64().unwrap_or(0),
            expires_at: response["expires_at"].as_u64(),
            status: response["status"].as_str().unwrap_or("active").to_string(),
        })
    }

    /// Revoke a key using BearDog's `keys.revoke` method
    ///
    /// # Arguments
    /// * `key_id` - Key ID to revoke
    ///
    /// # Returns
    /// * Success indicator
    pub async fn revoke(&self, key_id: &str) -> Result<bool> {
        let response = self
            .transport
            .call_method(
                "keys.revoke",
                Some(serde_json::json!({
                    "key_id": key_id,
                })),
            )
            .await
            .context("Failed to call keys.revoke")?;

        Ok(response["revoked"].as_bool().unwrap_or(false))
    }
}

/// Key information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key ID
    pub key_id: String,
    /// Key type (e.g., "Ed25519", "RSA-4096")
    pub key_type: String,
    /// Creation timestamp (Unix epoch)
    pub created_at: u64,
    /// Expiration timestamp (if any)
    pub expires_at: Option<u64>,
    /// Key status ("active", "expired", "revoked")
    pub status: String,
}
