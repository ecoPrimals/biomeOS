// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! BearDog client for security and cryptography
//!
//! BearDog is the security and cryptography primal. It provides:
//! - Encryption and decryption
//! - Key management
//! - Digital signatures
//! - Access control validation

use crate::clients::base::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// BearDog security and cryptography client
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::beardog::BearDogClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let beardog = BearDogClient::new("http://localhost:9000");
///
///     // Encrypt data
///     let encrypted = beardog.encrypt("my-data", "my-key-id").await?;
///     println!("Encrypted: {:?}", encrypted);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BearDogClient {
    http: PrimalHttpClient,
    endpoint: String,
}

impl BearDogClient {
    /// Create a new BearDog client
    ///
    /// # Arguments
    /// * `endpoint` - BearDog endpoint URL (discovered via capability query for "security")
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }

    /// Encrypt data
    ///
    /// # Arguments
    /// * `data` - Data to encrypt
    /// * `key_id` - Encryption key identifier
    ///
    /// # Returns
    /// Encrypted data
    ///
    /// # Errors
    /// Returns an error if encryption fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::new("http://localhost:9000");
    /// let encrypted = beardog.encrypt("secret data", "my-key").await?;
    /// println!("Encrypted: {}", encrypted.ciphertext);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        let body = serde_json::json!({
            "data": data,
            "key_id": key_id
        });

        let response = self.http.post("/api/v1/crypto/encrypt", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse encrypted data: {}", e))
    }

    /// Decrypt data
    ///
    /// # Arguments
    /// * `ciphertext` - Encrypted data
    /// * `key_id` - Decryption key identifier
    ///
    /// # Returns
    /// Decrypted plaintext
    ///
    /// # Errors
    /// Returns an error if decryption fails.
    pub async fn decrypt(&self, ciphertext: &str, key_id: &str) -> Result<String> {
        let body = serde_json::json!({
            "ciphertext": ciphertext,
            "key_id": key_id
        });

        let response = self.http.post("/api/v1/crypto/decrypt", body).await?;

        response["plaintext"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No plaintext in response"))
    }

    /// Sign data
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `key_id` - Signing key identifier
    ///
    /// # Returns
    /// Digital signature
    ///
    /// # Errors
    /// Returns an error if signing fails.
    pub async fn sign(&self, data: &str, key_id: &str) -> Result<Signature> {
        let body = serde_json::json!({
            "data": data,
            "key_id": key_id
        });

        let response = self.http.post("/api/v1/crypto/sign", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse signature: {}", e))
    }

    /// Verify signature
    ///
    /// # Arguments
    /// * `data` - Original data
    /// * `signature` - Signature to verify
    /// * `key_id` - Verification key identifier
    ///
    /// # Returns
    /// True if signature is valid
    ///
    /// # Errors
    /// Returns an error if verification fails.
    pub async fn verify_signature(
        &self,
        data: &str,
        signature: &str,
        key_id: &str,
    ) -> Result<bool> {
        let body = serde_json::json!({
            "data": data,
            "signature": signature,
            "key_id": key_id
        });

        let response = self.http.post("/api/v1/crypto/verify", body).await?;

        response["valid"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("No valid field in response"))
    }

    /// Generate a new cryptographic key
    ///
    /// # Arguments
    /// * `key_type` - Type of key to generate (e.g., "rsa", "ed25519")
    /// * `key_id` - Identifier for the new key
    ///
    /// # Returns
    /// Key generation result
    ///
    /// # Errors
    /// Returns an error if key generation fails.
    pub async fn generate_key(&self, key_type: &str, key_id: &str) -> Result<KeyInfo> {
        let body = serde_json::json!({
            "key_type": key_type,
            "key_id": key_id
        });

        let response = self.http.post("/api/v1/crypto/generate-key", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse key info: {}", e))
    }

    /// Validate access control
    ///
    /// # Arguments
    /// * `request` - Access control request
    ///
    /// # Returns
    /// Access decision
    ///
    /// # Errors
    /// Returns an error if validation fails.
    pub async fn validate_access(&self, request: &AccessRequest) -> Result<AccessDecision> {
        let response = self
            .http
            .post("/api/v1/security/access", serde_json::to_value(request)?)
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse access decision: {}", e))
    }

    /// Get security audit log
    ///
    /// # Arguments
    /// * `filters` - Optional filters for the audit log
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn get_audit_log(&self, filters: Option<&Value>) -> Result<Vec<AuditEntry>> {
        let path = "/api/v1/security/audit";
        let response = if let Some(filters) = filters {
            self.http.post(path, filters.clone()).await?
        } else {
            self.http.get(path).await?
        };

        serde_json::from_value(response["entries"].clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse audit log: {}", e))
    }
}

#[async_trait]
impl PrimalClient for BearDogClient {
    fn name(&self) -> &str {
        "beardog"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
    }
}

/// Encrypted data result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncryptedData {
    /// Encrypted ciphertext
    pub ciphertext: String,

    /// Key ID used for encryption
    pub key_id: String,

    /// Encryption algorithm
    pub algorithm: String,

    /// Initialization vector (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iv: Option<String>,
}

/// Digital signature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signature {
    /// Signature value
    pub signature: String,

    /// Key ID used for signing
    pub key_id: String,

    /// Signature algorithm
    pub algorithm: String,
}

/// Cryptographic key information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyInfo {
    /// Key identifier
    pub key_id: String,

    /// Key type
    pub key_type: String,

    /// Public key (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// Key creation timestamp
    pub created_at: String,
}

/// Access control request
#[derive(Debug, Clone, Serialize)]
pub struct AccessRequest {
    /// Subject requesting access
    pub subject: String,

    /// Resource being accessed
    pub resource: String,

    /// Action being performed
    pub action: String,

    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

/// Access control decision
#[derive(Debug, Clone, Deserialize)]
pub struct AccessDecision {
    /// Whether access is allowed
    pub allowed: bool,

    /// Reason for the decision
    pub reason: String,

    /// Applied policies
    #[serde(default)]
    pub policies: Vec<String>,
}

/// Security audit log entry
#[derive(Debug, Clone, Deserialize)]
pub struct AuditEntry {
    /// Event timestamp
    pub timestamp: String,

    /// Event type
    pub event_type: String,

    /// Subject involved
    pub subject: String,

    /// Resource involved
    pub resource: String,

    /// Action performed
    pub action: String,

    /// Event outcome
    pub outcome: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let client = BearDogClient::new("http://localhost:9000");
        assert_eq!(client.name(), "beardog");
        assert_eq!(client.endpoint(), "http://localhost:9000");
    }

    #[test]
    fn test_encrypted_data_deserialization() {
        let json = serde_json::json!({
            "ciphertext": "abc123",
            "key_id": "my-key",
            "algorithm": "AES-256-GCM",
            "iv": "xyz789"
        });

        let encrypted: EncryptedData = serde_json::from_value(json).unwrap();
        assert_eq!(encrypted.ciphertext, "abc123");
        assert_eq!(encrypted.key_id, "my-key");
    }

    #[test]
    fn test_access_request_serialization() {
        let request = AccessRequest {
            subject: "user123".to_string(),
            resource: "/api/data".to_string(),
            action: "read".to_string(),
            context: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["subject"], "user123");
        assert_eq!(json["action"], "read");
    }
}
