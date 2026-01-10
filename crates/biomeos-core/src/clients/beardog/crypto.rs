// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Cryptography operations - Encryption, decryption, signing, verification
//!
//! This module provides secure cryptographic operations via the BearDog primal:
//! - **Encryption/Decryption**: Symmetric and asymmetric encryption
//! - **Digital Signatures**: Sign and verify data integrity
//! - **Key-based operations**: All operations use managed keys
//!
//! # Security
//!
//! All cryptographic operations are performed by the BearDog primal, which:
//! - Manages keys securely
//! - Provides auditing
//! - Enforces access policies
//! - Uses industry-standard algorithms

use super::client::BearDogClient;
use super::types::{EncryptedData, Signature};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use tracing::{debug, info};

impl BearDogClient {
    /// Encrypt data using a managed key
    ///
    /// Uses BearDog's JSON-RPC API: `encryption.encrypt`
    ///
    /// # Arguments
    /// * `data` - Plain text data to encrypt
    /// * `key_id` - ID of the encryption key to use
    ///
    /// # Returns
    /// Encrypted data with ciphertext and metadata
    ///
    /// # Errors
    /// Returns an error if:
    /// - Key doesn't exist
    /// - Encryption fails
    /// - Communication with BearDog fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let encrypted = beardog.encrypt("secret data", "my-key").await?;
    /// println!("Ciphertext: {}", encrypted.ciphertext);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        info!("🔐 Encrypting data with key '{}'", key_id);

        let response = self
            .transport
            .call(
                "encryption.encrypt",
                Some(json!({
                    "data": data,
                    "key_id": key_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call encryption.encrypt")?;

        debug!("✅ Encryption successful");

        Ok(EncryptedData {
            ciphertext: response["ciphertext"]
                .as_str()
                .context("Missing ciphertext in response")?
                .to_string(),
            key_id: key_id.to_string(),
            algorithm: response["algorithm"]
                .as_str()
                .unwrap_or("AES-256-GCM")
                .to_string(),
            metadata: response["metadata"].clone(),
        })
    }

    /// Decrypt data using a managed key
    ///
    /// Uses BearDog's JSON-RPC API: `encryption.decrypt`
    ///
    /// # Arguments
    /// * `ciphertext` - Encrypted data (typically from `encrypt()`)
    /// * `key_id` - ID of the decryption key
    ///
    /// # Returns
    /// Decrypted plaintext data
    ///
    /// # Errors
    /// Returns an error if:
    /// - Key doesn't exist or is wrong
    /// - Ciphertext is corrupted
    /// - Decryption fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let encrypted = beardog.encrypt("secret", "my-key").await?;
    /// let decrypted = beardog.decrypt(&encrypted.ciphertext, "my-key").await?;
    /// assert_eq!(decrypted, "secret");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn decrypt(&self, ciphertext: &str, key_id: &str) -> Result<String> {
        info!("🔓 Decrypting data with key '{}'", key_id);

        let response = self
            .transport
            .call(
                "encryption.decrypt",
                Some(json!({
                    "ciphertext": ciphertext,
                    "key_id": key_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call encryption.decrypt")?;

        debug!("✅ Decryption successful");

        response["plaintext"]
            .as_str()
            .context("Missing plaintext in response")
            .map(|s| s.to_string())
    }

    /// Sign data using a managed key
    ///
    /// Uses BearDog's JSON-RPC API: `signing.sign`
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `key_id` - ID of the signing key
    ///
    /// # Returns
    /// Digital signature with metadata
    ///
    /// # Errors
    /// Returns an error if:
    /// - Key doesn't exist
    /// - Signing fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let signature = beardog.sign("important data", "signing-key").await?;
    /// println!("Signature: {}", signature.signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign(&self, data: &str, key_id: &str) -> Result<Signature> {
        info!("✍️ Signing data with key '{}'", key_id);

        // Encode data as base64 for transport
        let data_b64 = STANDARD.encode(data.as_bytes());

        let response = self
            .transport
            .call(
                "signing.sign",
                Some(json!({
                    "data": data_b64,
                    "key_id": key_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call signing.sign")?;

        debug!("✅ Signing successful");

        Ok(Signature {
            signature: response["signature"]
                .as_str()
                .context("Missing signature in response")?
                .to_string(),
            key_id: key_id.to_string(),
            algorithm: response["algorithm"]
                .as_str()
                .unwrap_or("Ed25519")
                .to_string(),
        })
    }

    /// Verify a digital signature
    ///
    /// Uses BearDog's JSON-RPC API: `signing.verify`
    ///
    /// # Arguments
    /// * `data` - Original data that was signed
    /// * `signature` - Signature to verify
    /// * `key_id` - ID of the verification key (public key)
    ///
    /// # Returns
    /// `true` if signature is valid, `false` otherwise
    ///
    /// # Errors
    /// Returns an error if:
    /// - Key doesn't exist
    /// - Verification process fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let signature = beardog.sign("data", "key").await?;
    /// let valid = beardog.verify_signature("data", &signature.signature, "key").await?;
    /// assert!(valid);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn verify_signature(
        &self,
        data: &str,
        signature: &str,
        key_id: &str,
    ) -> Result<bool> {
        info!("🔍 Verifying signature with key '{}'", key_id);

        // Encode data as base64 for transport
        let data_b64 = STANDARD.encode(data.as_bytes());

        let response = self
            .transport
            .call(
                "signing.verify",
                Some(json!({
                    "data": data_b64,
                    "signature": signature,
                    "key_id": key_id,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call signing.verify")?;

        let valid = response["valid"]
            .as_bool()
            .context("Missing 'valid' field in response")?;

        if valid {
            debug!("✅ Signature is valid");
        } else {
            debug!("❌ Signature is invalid");
        }

        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_encrypt_decrypt() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let encrypted = client.encrypt("test data", "test-key").await.unwrap();
        let decrypted = client.decrypt(&encrypted.ciphertext, "test-key").await.unwrap();
        assert_eq!(decrypted, "test data");
    }

    #[tokio::test]
    #[ignore] // Requires live BearDog primal
    async fn test_sign_verify() {
        let client = BearDogClient::discover("nat0").await.unwrap();
        let sig = client.sign("test data", "test-key").await.unwrap();
        let valid = client
            .verify_signature("test data", &sig.signature, "test-key")
            .await
            .unwrap();
        assert!(valid);
    }
}
