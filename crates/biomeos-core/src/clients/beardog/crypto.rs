//! BearDog Cryptography Client
//!
//! Provides encryption, decryption, signing, and verification using BearDog's JSON-RPC API.
//!
//! Uses the real BearDog `encryption.*` and `signing.*` methods discovered from v0.9.0+

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::Value;

use crate::clients::transport::PrimalTransport;

/// Cryptography operations client for BearDog
///
/// Provides real encryption, decryption, signing, and verification via JSON-RPC
pub struct CryptoClient {
    transport: PrimalTransport,
}

impl CryptoClient {
    /// Create a new crypto client with the given transport
    pub fn new(transport: PrimalTransport) -> Self {
        Self { transport }
    }

    /// Encrypt data using BearDog's `encryption.encrypt` method
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt (will be base64 encoded)
    /// * `key_ref` - Key reference/ID to use for encryption
    ///
    /// # Returns
    /// * Encrypted data with ciphertext, nonce, and tag
    pub async fn encrypt(&self, plaintext: &[u8], key_ref: &str) -> Result<EncryptedData> {
        let plaintext_b64 = BASE64.encode(plaintext);

        let response = self
            .transport
            .call_method(
                "encryption.encrypt",
                Some(serde_json::json!({
                    "plaintext": plaintext_b64,
                    "key_ref": key_ref,
                    "algorithm": "AES-256-GCM"
                })),
            )
            .await
            .context("Failed to call encryption.encrypt")?;

        let ciphertext = response["ciphertext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing ciphertext in response"))?
            .to_string();

        let nonce = response["nonce"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing nonce in response"))?
            .to_string();

        let tag = response
            .get("tag")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(EncryptedData {
            ciphertext,
            nonce,
            tag,
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Decrypt data using BearDog's `encryption.decrypt` method
    ///
    /// # Arguments
    /// * `encrypted` - Encrypted data structure
    /// * `key_ref` - Key reference/ID to use for decryption
    ///
    /// # Returns
    /// * Decrypted plaintext as bytes
    pub async fn decrypt(&self, encrypted: &EncryptedData, key_ref: &str) -> Result<Vec<u8>> {
        let mut params = serde_json::json!({
            "ciphertext": encrypted.ciphertext,
            "nonce": encrypted.nonce,
            "key_ref": key_ref,
        });

        if let Some(tag) = &encrypted.tag {
            params["tag"] = Value::String(tag.clone());
        }

        let response = self
            .transport
            .call_method("encryption.decrypt", Some(params))
            .await
            .context("Failed to call encryption.decrypt")?;

        let plaintext_b64 = response["plaintext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing plaintext in response"))?;

        BASE64
            .decode(plaintext_b64)
            .context("Failed to decode base64 plaintext")
    }

    /// Sign data using BearDog's `signing.sign` method
    ///
    /// # Arguments
    /// * `message` - Message to sign (will be base64 encoded)
    /// * `key_ref` - Key reference/ID to use for signing
    ///
    /// # Returns
    /// * Digital signature
    pub async fn sign(&self, message: &[u8], key_ref: &str) -> Result<Signature> {
        let message_b64 = BASE64.encode(message);

        let response = self
            .transport
            .call_method(
                "signing.sign",
                Some(serde_json::json!({
                    "message": message_b64,
                    "key_ref": key_ref,
                    "algorithm": "Ed25519"
                })),
            )
            .await
            .context("Failed to call signing.sign")?;

        let signature = response["signature"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing signature in response"))?
            .to_string();

        let algorithm = response["algorithm"]
            .as_str()
            .unwrap_or("Ed25519")
            .to_string();

        Ok(Signature {
            signature,
            key_id: key_ref.to_string(),
            algorithm,
        })
    }

    /// Verify a signature using BearDog's `signing.verify` method
    ///
    /// # Arguments
    /// * `message` - Original message that was signed
    /// * `signature` - Signature to verify
    /// * `public_key` - Public key to verify against
    ///
    /// # Returns
    /// * true if signature is valid, false otherwise
    pub async fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
        public_key: &str,
    ) -> Result<bool> {
        let message_b64 = BASE64.encode(message);

        let response = self
            .transport
            .call_method(
                "signing.verify",
                Some(serde_json::json!({
                    "message": message_b64,
                    "signature": signature.signature,
                    "public_key": public_key,
                })),
            )
            .await
            .context("Failed to call signing.verify")?;

        Ok(response["valid"].as_bool().unwrap_or(false))
    }
}

/// Encrypted data structure
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Ciphertext (base64 encoded)
    pub ciphertext: String,
    /// Nonce/IV (base64 encoded)
    pub nonce: String,
    /// Authentication tag (base64 encoded, for AEAD modes)
    pub tag: Option<String>,
    /// Algorithm used
    pub algorithm: String,
}

/// Digital signature
#[derive(Debug, Clone)]
pub struct Signature {
    /// Signature bytes (base64 encoded)
    pub signature: String,
    /// Public key ID
    pub key_id: String,
    /// Algorithm used
    pub algorithm: String,
}
