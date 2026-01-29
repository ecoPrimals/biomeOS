//! Dark Forest Encrypted Beacon System
//!
//! Implements the BirdSong Dark Forest trust model:
//! - Encrypted beacons that reveal nothing to outsiders
//! - Only family members can decrypt and understand broadcasts
//! - Lineage verification after discovery
//!
//! ## Architecture
//!
//! ```text
//! Broadcast: [encrypted_beacon]
//!     │
//!     ├── Family member: Decrypt → See node, socket, capabilities
//!     └── Attacker: Decryption fails → See only noise
//! ```
//!
//! ## Protocol Flow
//!
//! 1. Node derives broadcast key from family seed
//! 2. Node encrypts beacon with ChaCha20-Poly1305
//! 3. Node broadcasts encrypted beacon (reveals nothing)
//! 4. Family nodes try to decrypt with their key
//! 5. Successful decrypt → proceed to lineage verification
//! 6. Failed decrypt → ignore (not family)

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

use crate::error::{SporeError, SporeResult};

/// Plaintext beacon data (before encryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPlaintext {
    /// Hash of family_id (not the actual ID)
    pub family_hash: String,
    /// Node identifier
    pub node_id: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// BearDog socket path
    pub socket_path: String,
    /// Capabilities (hashed)
    pub capabilities_hash: String,
    /// Optional: lineage mode (genesis/sibling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_mode: Option<String>,
}

/// Encrypted beacon (what gets broadcast)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBeacon {
    /// Encrypted payload (base64)
    pub ciphertext: String,
    /// Nonce used for encryption (base64)
    pub nonce: String,
    /// Authentication tag (base64)
    pub tag: String,
    /// Version for protocol evolution
    pub version: u8,
}

/// Dark Forest beacon manager
#[derive(Clone)]
pub struct DarkForestBeacon {
    /// BearDog socket path for crypto operations
    beardog_socket: String,
    /// Family seed (base64)
    family_seed_b64: String,
    /// Node ID
    node_id: String,
}

impl DarkForestBeacon {
    /// Create a new Dark Forest beacon manager
    ///
    /// # Arguments
    /// * `beardog_socket` - Path to BearDog Unix socket
    /// * `seed_path` - Path to .family.seed file
    /// * `node_id` - This node's identifier
    pub async fn new<P: AsRef<Path>>(
        beardog_socket: &str,
        seed_path: P,
        node_id: &str,
    ) -> SporeResult<Self> {
        // Read and encode family seed
        let seed_bytes = tokio::fs::read(seed_path.as_ref()).await.map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read seed file: {}", e),
            ))
        })?;

        let family_seed_b64 = BASE64.encode(&seed_bytes);

        Ok(Self {
            beardog_socket: beardog_socket.to_string(),
            family_seed_b64,
            node_id: node_id.to_string(),
        })
    }

    /// Derive family broadcast key from seed
    async fn derive_broadcast_key(&self) -> SporeResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "genetic.derive_lineage_key",
            "params": {
                "our_family_id": "family",
                "peer_family_id": "broadcast",
                "context": "birdsong-broadcast-v1",
                "lineage_seed": self.family_seed_b64
            },
            "id": 1
        });

        let response = self.call_beardog(&request).await?;

        response
            .get("result")
            .and_then(|r| r.get("key"))
            .and_then(|k| k.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to derive broadcast key".to_string())
            })
    }

    /// Generate an encrypted beacon
    ///
    /// Returns an encrypted beacon that reveals nothing to non-family.
    pub async fn generate_encrypted_beacon(
        &self,
        socket_path: &str,
        capabilities: &[&str],
        lineage_mode: Option<&str>,
    ) -> SporeResult<EncryptedBeacon> {
        info!("🌲 Generating encrypted Dark Forest beacon");

        // Derive broadcast key
        let broadcast_key = self.derive_broadcast_key().await?;
        debug!("Derived broadcast key");

        // Create beacon plaintext
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SporeError::SystemError(format!("Time error: {}", e)))?
            .as_secs();

        // Hash family ID (don't reveal actual ID)
        let family_hash = self.hash_string("family").await?;

        // Hash capabilities
        let caps_str = capabilities.join(",");
        let capabilities_hash = self.hash_string(&caps_str).await?;

        let beacon = BeaconPlaintext {
            family_hash: family_hash[..16].to_string(), // First 16 chars
            node_id: self.node_id.clone(),
            timestamp,
            socket_path: socket_path.to_string(),
            capabilities_hash: capabilities_hash[..16].to_string(),
            lineage_mode: lineage_mode.map(|s| s.to_string()),
        };

        // Serialize and encode
        let beacon_json = serde_json::to_string(&beacon)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;
        let beacon_b64 = BASE64.encode(beacon_json.as_bytes());

        // Encrypt with ChaCha20-Poly1305
        let encrypt_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.chacha20_poly1305_encrypt",
            "params": {
                "key": broadcast_key,
                "plaintext": beacon_b64
            },
            "id": 2
        });

        let response = self.call_beardog(&encrypt_request).await?;
        let result = response.get("result").ok_or_else(|| {
            SporeError::ValidationFailed("No result in encrypt response".to_string())
        })?;

        let ciphertext = result
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing ciphertext".to_string()))?;
        let nonce = result
            .get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing nonce".to_string()))?;
        let tag = result
            .get("tag")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing tag".to_string()))?;

        info!("✅ Encrypted beacon generated (reveals nothing to outsiders)");

        Ok(EncryptedBeacon {
            ciphertext: ciphertext.to_string(),
            nonce: nonce.to_string(),
            tag: tag.to_string(),
            version: 1,
        })
    }

    /// Try to decrypt a beacon
    ///
    /// Returns `Some(BeaconPlaintext)` if we're family, `None` if not.
    /// This is the "gate" - only family can decrypt.
    pub async fn try_decrypt_beacon(
        &self,
        beacon: &EncryptedBeacon,
    ) -> SporeResult<Option<BeaconPlaintext>> {
        debug!("🔓 Attempting to decrypt Dark Forest beacon");

        // Derive our broadcast key
        let broadcast_key = self.derive_broadcast_key().await?;

        // Try to decrypt
        let decrypt_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.chacha20_poly1305_decrypt",
            "params": {
                "key": broadcast_key,
                "ciphertext": beacon.ciphertext,
                "nonce": beacon.nonce,
                "tag": beacon.tag
            },
            "id": 3
        });

        let response = self.call_beardog(&decrypt_request).await?;

        // Check if decryption failed (not family)
        if response.get("error").is_some() {
            debug!("❌ Beacon decryption failed - not family");
            return Ok(None);
        }

        // Decryption succeeded - we're family!
        let plaintext_b64 = response
            .get("result")
            .and_then(|r| r.get("plaintext"))
            .and_then(|p| p.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Missing plaintext in response".to_string())
            })?;

        let plaintext_bytes = BASE64
            .decode(plaintext_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid base64: {}", e)))?;

        let beacon: BeaconPlaintext = serde_json::from_slice(&plaintext_bytes)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid beacon JSON: {}", e)))?;

        info!(
            "✅ Beacon decrypted - family member found: {}",
            beacon.node_id
        );

        Ok(Some(beacon))
    }

    /// Hash a string using Blake3 (via BearDog)
    async fn hash_string(&self, input: &str) -> SporeResult<String> {
        let input_b64 = BASE64.encode(input.as_bytes());

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.blake3_hash",
            "params": {
                "data": input_b64
            },
            "id": 10
        });

        let response = self.call_beardog(&request).await?;

        response
            .get("result")
            .and_then(|r| r.get("hash"))
            .and_then(|h| h.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SporeError::ValidationFailed("Failed to hash string".to_string()))
    }

    /// Call BearDog via Unix socket
    async fn call_beardog(&self, request: &serde_json::Value) -> SporeResult<serde_json::Value> {
        let mut stream = UnixStream::connect(&self.beardog_socket)
            .await
            .map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!(
                        "Failed to connect to BearDog at {}: {}",
                        self.beardog_socket, e
                    ),
                ))
            })?;

        let request_str = serde_json::to_string(request)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        stream.write_all(request_str.as_bytes()).await?;
        stream.shutdown().await?;

        // Read with timeout to prevent hangs (30s for JSON-RPC)
        let mut response_str = String::new();
        timeout(Duration::from_secs(30), stream.read_to_string(&mut response_str))
            .await
            .map_err(|_| SporeError::SystemError("Socket read timeout (30s)".to_string()))?
            .map_err(|e| SporeError::SystemError(format!("Read error: {e}")))?;

        serde_json::from_str(&response_str)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid JSON response: {}", e)))
    }

    /// Verify lineage after successful beacon decryption
    ///
    /// This is the independent validation step - even after introduction,
    /// we verify the peer's lineage directly.
    pub async fn verify_peer_lineage(
        &self,
        peer_family_id: &str,
        peer_proof: &str,
    ) -> SporeResult<bool> {
        info!("🔍 Verifying peer lineage (independent validation)");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "genetic.verify_lineage",
            "params": {
                "our_family_id": "family",
                "peer_family_id": peer_family_id,
                "lineage_proof": peer_proof,
                "lineage_seed": self.family_seed_b64
            },
            "id": 4
        });

        let response = self.call_beardog(&request).await?;

        let valid = response
            .get("result")
            .and_then(|r| r.get("valid"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if valid {
            info!("✅ Peer lineage verified");
        } else {
            warn!("❌ Peer lineage verification failed");
        }

        Ok(valid)
    }

    /// Generate a lineage proof for this node
    pub async fn generate_lineage_proof(&self, peer_family_id: &str) -> SporeResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "genetic.generate_lineage_proof",
            "params": {
                "our_family_id": "family",
                "peer_family_id": peer_family_id,
                "lineage_seed": self.family_seed_b64
            },
            "id": 5
        });

        let response = self.call_beardog(&request).await?;

        response
            .get("result")
            .and_then(|r| r.get("proof"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to generate lineage proof".to_string())
            })
    }

    /// Derive a session key for encrypted communication with a verified peer
    pub async fn derive_session_key(&self, peer_id: &str, context: &str) -> SporeResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "genetic.derive_lineage_key",
            "params": {
                "our_family_id": "family",
                "peer_family_id": peer_id,
                "context": context,
                "lineage_seed": self.family_seed_b64
            },
            "id": 6
        });

        let response = self.call_beardog(&request).await?;

        response
            .get("result")
            .and_then(|r| r.get("key"))
            .and_then(|k| k.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SporeError::ValidationFailed("Failed to derive session key".to_string()))
    }
}

/// Discovery result from scanning for encrypted beacons
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Decrypted beacon data
    pub beacon: BeaconPlaintext,
    /// Whether lineage has been verified
    pub lineage_verified: bool,
    /// Session key (if derived)
    pub session_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_serialization() {
        let beacon = BeaconPlaintext {
            family_hash: "abc123".to_string(),
            node_id: "test-node".to_string(),
            timestamp: 1234567890,
            socket_path: "/tmp/test.sock".to_string(),
            capabilities_hash: "def456".to_string(),
            lineage_mode: Some("genesis".to_string()),
        };

        let json = serde_json::to_string(&beacon).unwrap();
        let parsed: BeaconPlaintext = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.node_id, "test-node");
        assert_eq!(parsed.lineage_mode, Some("genesis".to_string()));
    }

    #[test]
    fn test_encrypted_beacon_serialization() {
        let beacon = EncryptedBeacon {
            ciphertext: "encrypted_data".to_string(),
            nonce: "nonce_value".to_string(),
            tag: "auth_tag".to_string(),
            version: 1,
        };

        let json = serde_json::to_string(&beacon).unwrap();
        assert!(json.contains("encrypted_data"));
        assert!(json.contains("version"));
    }
}
