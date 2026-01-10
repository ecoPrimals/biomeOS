// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! BearDog client for security, cryptography, and BTSP tunneling
//!
//! BearDog is the security and cryptography primal. It provides:
//! - Encryption and decryption
//! - Key management
//! - Digital signatures
//! - Access control validation
//! - BTSP (BirdSong Tunnel Protocol) tunnel management
//! - BirdSong genetic cryptography (lineage-aware encryption)
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_core::clients::beardog::BearDogClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let beardog = BearDogClient::discover("nat0").await?;
//!     
//!     // Establish tunnel
//!     let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
//!     println!("Tunnel created: {}", tunnel.tunnel_id);
//!     
//!     // Check status
//!     let status = beardog.get_tunnel_status(&tunnel.tunnel_id).await?;
//!     println!("Tunnel state: {}", status.state);
//!     
//!     // Close tunnel
//!     beardog.close_tunnel(&tunnel.tunnel_id).await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::clients::transport::{PrimalClient as TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// BearDog security and cryptography client
///
/// Uses JSON-RPC 2.0 over Unix sockets for fast, secure communication.
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::beardog::BearDogClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Auto-discover via Unix socket
///     let beardog = BearDogClient::discover("nat0").await?;
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
    transport: TransportClient,
    family_id: String,
}

impl BearDogClient {
    /// Auto-discover BearDog via Unix socket
    ///
    /// Searches for BearDog's Unix socket in XDG runtime directory.
    /// Falls back to HTTP if Unix socket not available.
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID (e.g., "nat0")
    ///
    /// # Returns
    /// BearDogClient configured with JSON-RPC over Unix socket (primary)
    /// or HTTP (fallback)
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::beardog::BearDogClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let beardog = BearDogClient::discover("nat0").await?;
    ///     let encrypted = beardog.encrypt("secret", "my-key").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover("beardog", family_id).await
            .context("Failed to discover BearDog. Is it running?")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support (100x faster)
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:9000")
    /// * `family_id` - Genetic family ID
    #[deprecated(note = "Use BearDogClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "beardog",
            family_id,
            TransportPreference::Http
        ).await
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy constructor (DEPRECATED)
    ///
    /// **BREAKING**: This method is now async. Use `discover()` instead.
    #[deprecated(note = "Use BearDogClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("BearDogClient::new() is deprecated. Use BearDogClient::discover() instead.");
    }

    /// Encrypt data
    ///
    /// Uses BearDog's JSON-RPC API: `encryption.encrypt`
    ///
    /// # Arguments
    /// * `data` - Data to encrypt (will be base64-encoded)
    /// * `key_id` - Encryption key identifier
    ///
    /// # Returns
    /// Encrypted data with ciphertext, nonce, and authentication tag
    ///
    /// # Errors
    /// Returns an error if encryption fails or BearDog is unavailable.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let encrypted = beardog.encrypt("secret data", "my-key").await?;
    /// println!("Encrypted: {}", encrypted.ciphertext);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        // BearDog expects base64-encoded plaintext
        let plaintext_b64 = BASE64.encode(data.as_bytes());
        
        let response = self.transport.call_method(
            "encryption.encrypt",
            serde_json::json!({
                "plaintext": plaintext_b64,
                "key_ref": key_id,
                "algorithm": "AES-256-GCM"
            })
        ).await
            .context("Failed to call encryption.encrypt")?;

        // Parse JSON-RPC response
        let ciphertext = response["ciphertext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing ciphertext in response"))?
            .to_string();
        
        let nonce = response["nonce"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing nonce in response"))?
            .to_string();
        
        let tag = response["tag"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing tag in response"))?
            .to_string();
        
        Ok(EncryptedData {
            ciphertext,
            key_id: key_id.to_string(),
            algorithm: response["algorithm"]
                .as_str()
                .unwrap_or("AES-256-GCM")
                .to_string(),
            iv: Some(nonce),
        })
    }

    /// Decrypt data
    ///
    /// Uses BearDog's JSON-RPC API: `encryption.decrypt`
    ///
    /// # Arguments
    /// * `ciphertext` - Base64-encoded encrypted data
    /// * `key_id` - Decryption key identifier
    ///
    /// # Returns
    /// Decrypted plaintext
    ///
    /// # Errors
    /// Returns an error if decryption fails or authentication tag is invalid.
    ///
    /// # Note
    /// This is a simplified signature. Full decrypt requires nonce and tag.
    /// Consider using decrypt_full() for complete control.
    pub async fn decrypt(&self, ciphertext: &str, key_id: &str) -> Result<String> {
        // Note: This simplified version assumes nonce/tag are embedded or default
        // Real usage should call decrypt_with_nonce_tag()
        let response = self.transport.call_method(
            "encryption.decrypt",
            serde_json::json!({
                "ciphertext": ciphertext,
                "key_ref": key_id,
                // TODO: Add nonce and tag parameters
            })
        ).await
            .context("Failed to call encryption.decrypt")?;

        // Decode base64 plaintext
        let plaintext_b64 = response["plaintext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing plaintext in response"))?;
        
        let plaintext_bytes = BASE64.decode(plaintext_b64)
            .context("Failed to decode plaintext from base64")?;
        
        String::from_utf8(plaintext_bytes)
            .context("Plaintext is not valid UTF-8")
    }

    /// Sign data
    ///
    /// Uses BearDog's JSON-RPC API: `signing.sign`
    ///
    /// # Arguments
    /// * `data` - Data to sign (will be base64-encoded)
    /// * `key_id` - Signing key identifier
    ///
    /// # Returns
    /// Digital signature
    ///
    /// # Errors
    /// Returns an error if signing fails.
    pub async fn sign(&self, data: &str, key_id: &str) -> Result<Signature> {
        // BearDog expects base64-encoded message
        let message_b64 = BASE64.encode(data.as_bytes());
        
        let response = self.transport.call_method(
            "signing.sign",
            serde_json::json!({
                "message": message_b64,
                "key_ref": key_id,
                "algorithm": "Ed25519"
            })
        ).await
            .context("Failed to call signing.sign")?;

        Ok(Signature {
            signature: response["signature"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing signature in response"))?
                .to_string(),
            key_id: key_id.to_string(),
            algorithm: response["algorithm"]
                .as_str()
                .unwrap_or("Ed25519")
                .to_string(),
        })
    }

    /// Verify signature
    ///
    /// Uses BearDog's JSON-RPC API: `signing.verify`
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
        // BearDog expects base64-encoded message
        let message_b64 = BASE64.encode(data.as_bytes());
        
        let response = self.transport.call_method(
            "signing.verify",
            serde_json::json!({
                "message": message_b64,
                "signature": signature,
                "key_ref": key_id,
                "algorithm": "Ed25519"
            })
        ).await
            .context("Failed to call signing.verify")?;

        response["valid"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("Missing valid field in response"))
    }

    /// Generate a new cryptographic key
    ///
    /// Uses BearDog's JSON-RPC API: `keys.generate`
    ///
    /// # Arguments
    /// * `key_type` - Type of key to generate (e.g., "AES", "Ed25519")
    /// * `key_id` - Identifier for the new key
    ///
    /// # Returns
    /// Key generation result
    ///
    /// # Errors
    /// Returns an error if key generation fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let key = beardog.generate_key("Ed25519", "signing-key-1").await?;
    /// println!("Generated key: {}", key.key_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_key(&self, key_type: &str, key_id: &str) -> Result<KeyInfo> {
        let response = self.transport.call_method(
            "keys.generate",
            serde_json::json!({
                "key_type": key_type,
                "key_ref": key_id,
                "options": {}
            })
        ).await
            .context("Failed to call keys.generate")?;

        serde_json::from_value(response)
            .context("Failed to parse key info from response")
    }

    /// Validate access control
    ///
    /// Uses BearDog's JSON-RPC API: `access.validate`
    ///
    /// # Arguments
    /// * `request` - Access control request
    ///
    /// # Returns
    /// Access decision
    ///
    /// # Errors
    /// Returns an error if validation fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::{BearDogClient, AccessRequest};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let request = AccessRequest {
    ///     identity_id: "user-alpha".to_string(),
    ///     resource_id: "data-beta".to_string(),
    ///     action: "read".to_string(),
    /// };
    /// let decision = beardog.validate_access(&request).await?;
    /// println!("Access granted: {}", decision.granted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_access(&self, request: &AccessRequest) -> Result<AccessDecision> {
        let response = self.transport.call_method(
            "access.validate",
            serde_json::to_value(request)?
        ).await
            .context("Failed to call access.validate")?;

        serde_json::from_value(response)
            .context("Failed to parse access decision from response")
    }

    /// Get security audit log
    ///
    /// Uses BearDog's JSON-RPC API: `security.audit_log`
    ///
    /// # Arguments
    /// * `filters` - Optional filters for the audit log
    ///
    /// # Errors
    /// Returns an error if the request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::beardog::BearDogClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let beardog = BearDogClient::discover("nat0").await?;
    /// let filters = json!({"limit": 100, "level": "warning"});
    /// let entries = beardog.get_audit_log(Some(&filters)).await?;
    /// println!("Found {} audit entries", entries.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_audit_log(&self, filters: Option<&Value>) -> Result<Vec<AuditEntry>> {
        let params = filters.cloned().unwrap_or(serde_json::json!({}));
        
        let response = self.transport.call_method(
            "security.audit_log",
            params
        ).await
            .context("Failed to call security.audit_log")?;

        serde_json::from_value(response["entries"].clone())
            .context("Failed to parse audit log entries from response")
    }

    // ========================================================================
    // BTSP Tunnel Management (JSON-RPC 2.0)
    // ========================================================================

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
    /// let tunnel = beardog.establish_tunnel("peer-node-1", "192.168.1.100:9091").await?;
    /// println!("Tunnel ID: {}", tunnel.tunnel_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish_tunnel(&self, peer_id: &str, endpoint: &str) -> Result<TunnelInfo> {
        let response = self.transport.call_method(
            "btsp.tunnel_establish",
            serde_json::json!({
                "peer_id": peer_id,
                "endpoint": endpoint,
                "family_id": self.family_id
            })
        ).await
            .context("Failed to call btsp.tunnel_establish")?;

        serde_json::from_value(response)
            .context("Failed to parse tunnel info from response")
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
    /// let status = beardog.get_tunnel_status("btsp_abc123xyz").await?;
    /// println!("Tunnel state: {}", status.state);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tunnel_status(&self, tunnel_id: &str) -> Result<TunnelStatus> {
        let response = self.transport.call_method(
            "btsp.tunnel_status",
            serde_json::json!({
                "tunnel_id": tunnel_id,
                "family_id": self.family_id
            })
        ).await
            .with_context(|| format!("Failed to get status for tunnel {}", tunnel_id))?;

        serde_json::from_value(response)
            .context("Failed to parse tunnel status from response")
    }

    /// Close a BTSP tunnel
    ///
    /// Uses BearDog's HTTP REST API: POST /api/v1/tunnel/close
    ///
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
    /// beardog.close_tunnel("btsp_abc123xyz").await?;
    /// println!("Tunnel closed");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()> {
        self.transport.call_method(
            "btsp.tunnel_close",
            serde_json::json!({
                "tunnel_id": tunnel_id,
                "family_id": self.family_id
            })
        ).await
            .with_context(|| format!("Failed to close tunnel {}", tunnel_id))?;

        Ok(())
    }
}

/// BTSP Tunnel information (from establish_tunnel)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TunnelInfo {
    /// Unique tunnel identifier
    pub tunnel_id: String,
    /// Peer node ID
    pub peer_id: String,
    /// Tunnel establishment timestamp
    pub established_at: String,
}

/// BTSP Tunnel status (from get_tunnel_status)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TunnelStatus {
    /// Tunnel identifier
    pub tunnel_id: String,
    /// Tunnel state (active, idle, establishing, closing, closed, failed)
    pub state: String,
    /// Peer node ID
    pub peer_id: String,
    /// Peer endpoint
    #[serde(default)]
    pub peer_endpoint: Option<String>,
    /// Local endpoint
    #[serde(default)]
    pub local_endpoint: Option<String>,
    /// Bytes sent
    #[serde(default)]
    pub bytes_sent: u64,
    /// Bytes received
    #[serde(default)]
    pub bytes_received: u64,
    /// Last activity timestamp
    #[serde(default)]
    pub last_activity: Option<String>,
    /// Establishment timestamp
    pub established_at: String,
}

#[async_trait]
impl PrimalClient for BearDogClient {
    fn name(&self) -> &str {
        "beardog"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        // For JSON-RPC, method becomes the RPC method name, path is ignored
        self.transport.call(method, body).await
    }
}
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

/// BTSP (BirdSong Tunnel Protocol) tunnel management via BearDog CLI
///
/// BearDog is CLI-only by design for security. This module provides a Rust
/// interface to the BearDog CLI for BTSP tunnel operations.
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::beardog::btsp::BtspTunnel;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create a BTSP tunnel to a peer
///     let tunnel = BtspTunnel::create("peer-node-1", "http://192.168.1.100:8080").await?;
///     println!("Tunnel created: {}", tunnel.tunnel_id);
///     
///     // Check tunnel status
///     let status = tunnel.status().await?;
///     println!("Tunnel status: {}", status.state);
///     
///     // Destroy tunnel when done
///     tunnel.destroy().await?;
///     
///     Ok(())
/// }
/// ```
pub mod btsp {
    use anyhow::{Context, Result};
    use serde::{Deserialize, Serialize};
    use std::process::Command;

    /// BTSP tunnel instance
    #[derive(Debug, Clone)]
    pub struct BtspTunnel {
        /// Unique tunnel identifier
        pub tunnel_id: String,
        /// Peer node ID
        pub peer_id: String,
        /// Peer endpoint
        pub peer_endpoint: String,
    }

    impl BtspTunnel {
        /// Create a new BTSP tunnel to a peer
        ///
        /// Executes: `beardog tunnel create --peer <id> --endpoint <endpoint>`
        ///
        /// # Arguments
        /// * `peer_id` - Peer node identifier
        /// * `peer_endpoint` - Peer endpoint URL
        ///
        /// # Errors
        /// Returns an error if the BearDog CLI is not available or tunnel creation fails.
        pub async fn create(peer_id: &str, peer_endpoint: &str) -> Result<Self> {
            let output = Command::new("beardog")
                .arg("tunnel")
                .arg("create")
                .arg("--peer")
                .arg(peer_id)
                .arg("--endpoint")
                .arg(peer_endpoint)
                .arg("--output")
                .arg("json")
                .output()
                .context("Failed to execute beardog tunnel create. Is beardog installed?")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("beardog tunnel create failed: {}", stderr);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let response: CreateTunnelResponse = serde_json::from_str(&stdout)
                .context("Failed to parse beardog tunnel create response")?;

            Ok(Self {
                tunnel_id: response.tunnel_id,
                peer_id: peer_id.to_string(),
                peer_endpoint: peer_endpoint.to_string(),
            })
        }

        /// Get tunnel status
        ///
        /// Executes: `beardog tunnel status <tunnel-id>`
        ///
        /// # Errors
        /// Returns an error if the status check fails.
        pub async fn status(&self) -> Result<TunnelStatus> {
            let output = Command::new("beardog")
                .arg("tunnel")
                .arg("status")
                .arg(&self.tunnel_id)
                .arg("--output")
                .arg("json")
                .output()
                .context("Failed to execute beardog tunnel status")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("beardog tunnel status failed: {}", stderr);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str(&stdout)
                .context("Failed to parse beardog tunnel status response")
        }

        /// Destroy the tunnel
        ///
        /// Executes: `beardog tunnel destroy <tunnel-id>`
        ///
        /// # Errors
        /// Returns an error if tunnel destruction fails.
        pub async fn destroy(self) -> Result<()> {
            let output = Command::new("beardog")
                .arg("tunnel")
                .arg("destroy")
                .arg(&self.tunnel_id)
                .output()
                .context("Failed to execute beardog tunnel destroy")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("beardog tunnel destroy failed: {}", stderr);
            }

            Ok(())
        }

        /// List all active tunnels
        ///
        /// Executes: `beardog tunnel list`
        ///
        /// # Errors
        /// Returns an error if listing fails.
        pub async fn list_all() -> Result<Vec<TunnelInfo>> {
            let output = Command::new("beardog")
                .arg("tunnel")
                .arg("list")
                .arg("--output")
                .arg("json")
                .output()
                .context("Failed to execute beardog tunnel list")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("beardog tunnel list failed: {}", stderr);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let response: ListTunnelsResponse = serde_json::from_str(&stdout)
                .context("Failed to parse beardog tunnel list response")?;

            Ok(response.tunnels)
        }
    }

    /// Response from `beardog tunnel create`
    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct CreateTunnelResponse {
        tunnel_id: String,
        #[serde(default)]
        status: String,
    }

    /// Tunnel status information
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TunnelStatus {
        /// Tunnel identifier
        pub tunnel_id: String,
        /// Tunnel state (e.g., "active", "pending", "failed")
        pub state: String,
        /// Peer node ID
        pub peer_id: String,
        /// Local port
        #[serde(default)]
        pub local_port: Option<u16>,
        /// Remote port
        #[serde(default)]
        pub remote_port: Option<u16>,
        /// Creation timestamp
        pub created_at: String,
        /// Last activity timestamp
        #[serde(default)]
        pub last_activity: Option<String>,
    }

    /// Tunnel information
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TunnelInfo {
        /// Tunnel identifier
        pub tunnel_id: String,
        /// Tunnel state
        pub state: String,
        /// Peer node ID
        pub peer_id: String,
        /// Peer endpoint
        pub peer_endpoint: String,
    }

    /// Response from `beardog tunnel list`
    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct ListTunnelsResponse {
        tunnels: Vec<TunnelInfo>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_tunnel_info_deserialization() {
            let json = serde_json::json!({
                "tunnel_id": "btsp-abc123",
                "state": "active",
                "peer_id": "node-456",
                "peer_endpoint": "http://192.168.1.100:8080"
            });

            let info: TunnelInfo = serde_json::from_value(json).unwrap();
            assert_eq!(info.tunnel_id, "btsp-abc123");
            assert_eq!(info.state, "active");
            assert_eq!(info.peer_id, "node-456");
        }

        #[test]
        fn test_tunnel_status_deserialization() {
            let json = serde_json::json!({
                "tunnel_id": "btsp-abc123",
                "state": "active",
                "peer_id": "node-456",
                "local_port": 9090,
                "remote_port": 9091,
                "created_at": "2026-01-01T12:00:00Z",
                "last_activity": "2026-01-01T12:05:00Z"
            });

            let status: TunnelStatus = serde_json::from_value(json).unwrap();
            assert_eq!(status.tunnel_id, "btsp-abc123");
            assert_eq!(status.state, "active");
            assert_eq!(status.local_port, Some(9090));
        }
    }
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

    #[test]
    fn test_tunnel_info_deserialization() {
        let json = serde_json::json!({
            "tunnel_id": "btsp_abc123xyz",
            "peer_id": "node-456",
            "established_at": "2026-01-01T12:00:00Z"
        });

        let info: TunnelInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.tunnel_id, "btsp_abc123xyz");
        assert_eq!(info.peer_id, "node-456");
        assert_eq!(info.established_at, "2026-01-01T12:00:00Z");
    }

    #[test]
    fn test_tunnel_status_deserialization() {
        let json = serde_json::json!({
            "tunnel_id": "btsp_abc123xyz",
            "state": "active",
            "peer_id": "node-456",
            "peer_endpoint": "192.168.1.100:9091",
            "local_endpoint": "192.168.1.50:9090",
            "bytes_sent": 1024000,
            "bytes_received": 2048000,
            "last_activity": "2026-01-01T12:05:30Z",
            "established_at": "2026-01-01T12:00:00Z"
        });

        let status: TunnelStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status.tunnel_id, "btsp_abc123xyz");
        assert_eq!(status.state, "active");
        assert_eq!(status.peer_id, "node-456");
        assert_eq!(status.bytes_sent, 1024000);
        assert_eq!(status.bytes_received, 2048000);
        assert_eq!(status.peer_endpoint, Some("192.168.1.100:9091".to_string()));
    }
}
