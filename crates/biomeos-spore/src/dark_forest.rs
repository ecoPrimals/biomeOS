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
use biomeos_core::AtomicClient;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
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

    /// Call BearDog via AtomicClient (Universal IPC v3.0)
    ///
    /// Uses AtomicClient for consistent, multi-transport JSON-RPC communication.
    /// Supports Unix sockets, abstract sockets, and TCP fallback.
    async fn call_beardog(&self, request: &serde_json::Value) -> SporeResult<serde_json::Value> {
        // Extract method and params from JSON-RPC request
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing method in request".to_string()))?;

        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Create AtomicClient with 30s timeout (consistent with original)
        let client = AtomicClient::unix(&self.beardog_socket)
            .with_timeout(std::time::Duration::from_secs(30));

        // Make the call via AtomicClient
        let result = client.call(method, params).await.map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("BearDog call failed: {}", e),
            ))
        })?;

        // Wrap result in JSON-RPC response format for compatibility
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": request.get("id").cloned().unwrap_or(serde_json::json!(1))
        }))
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

    // ═══════════════════════════════════════════════════════════════════
    // TRUE DARK FOREST - Pure Noise Beacons (Feb 2, 2026)
    // ═══════════════════════════════════════════════════════════════════
    //
    // Evolution from structured beacons to pure noise (A → A++ security):
    //
    // Old (A grade):
    //   {"ciphertext":"...","nonce":"...","tag":"...","version":1}
    //   Problem: JSON structure is metadata (identifiable, traceable)
    //
    // New (A++ LEGENDARY):
    //   [nonce (12 bytes)] + [ciphertext + tag (N+16 bytes)]
    //   Indistinguishable from random noise to outsiders
    //
    // User Insight: "Birds communicate via encrypted noise. Family lineage
    //                mixes beacon to noise, relatives can hear and understand.
    //                No plaintext leaks."
    //
    // Result: Zero metadata leaks, true Dark Forest communication

    /// Derive dedicated beacon key from lineage (TRUE Dark Forest)
    ///
    /// Calls BearDog's new `genetic.derive_lineage_beacon_key` method.
    /// This is domain-separated from other genetic keys.
    ///
    /// All family members derive identical keys from their shared lineage,
    /// enabling pure noise beacons with genetic decryption.
    ///
    /// # Returns
    /// 32-byte ChaCha20-Poly1305 key (hex-encoded)
    async fn derive_dedicated_beacon_key(&self) -> SporeResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "genetic.derive_lineage_beacon_key",
            "params": {},
            "id": 101
        });

        let response = self.call_beardog(&request).await?;

        response
            .get("result")
            .and_then(|r| r.get("beacon_key"))
            .and_then(|k| k.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to derive dedicated beacon key".to_string())
            })
    }

    /// Generate pure noise beacon (TRUE Dark Forest - A++ security)
    ///
    /// Output is indistinguishable from random bytes to outsiders.
    /// Only family members with correct lineage can decrypt.
    ///
    /// # Format
    /// `[nonce (12 bytes)] + [ciphertext (N bytes)] + [tag (16 bytes)]`
    ///
    /// No JSON, no structure, no version, NO metadata.
    ///
    /// # Arguments
    /// * `socket_path` - BearDog socket path to advertise
    /// * `capabilities` - Capabilities to advertise
    /// * `lineage_mode` - Optional lineage mode
    ///
    /// # Returns
    /// Pure noise bytes (indistinguishable from random)
    pub async fn generate_pure_noise_beacon(
        &self,
        socket_path: &str,
        capabilities: &[&str],
        lineage_mode: Option<&str>,
    ) -> SporeResult<Vec<u8>> {
        info!("🌑 Generating pure noise Dark Forest beacon (A++ security)");

        // Derive dedicated beacon key (domain-separated)
        let beacon_key = self.derive_dedicated_beacon_key().await?;
        debug!("   Derived dedicated beacon key (HKDF-SHA256)");

        // Create beacon plaintext (NO family_hash, NO version, NO metadata)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SporeError::SystemError(format!("Time error: {}", e)))?
            .as_secs();

        // Pure discovery info (no hashes, no obscuration)
        let beacon = serde_json::json!({
            "node_id": self.node_id,
            "timestamp": timestamp,
            "socket_path": socket_path,
            "capabilities": capabilities,
            "lineage_mode": lineage_mode
        });

        // Serialize to JSON
        let beacon_json = serde_json::to_string(&beacon)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        // Encode for transmission
        let beacon_b64 = BASE64.encode(beacon_json.as_bytes());

        // Encrypt with ChaCha20-Poly1305
        let encrypt_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.chacha20_poly1305_encrypt",
            "params": {
                "key": beacon_key,
                "plaintext": beacon_b64
            },
            "id": 102
        });

        let response = self.call_beardog(&encrypt_request).await?;
        let result = response.get("result").ok_or_else(|| {
            SporeError::ValidationFailed("No result in encrypt response".to_string())
        })?;

        // Extract encryption components
        let nonce_b64 = result
            .get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing nonce".to_string()))?;
        let ciphertext_b64 = result
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing ciphertext".to_string()))?;
        let tag_b64 = result
            .get("tag")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing tag".to_string()))?;

        // Decode from base64 to raw bytes
        let nonce = BASE64
            .decode(nonce_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid nonce: {}", e)))?;
        let ciphertext = BASE64
            .decode(ciphertext_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid ciphertext: {}", e)))?;
        let tag = BASE64
            .decode(tag_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid tag: {}", e)))?;

        // Concatenate: nonce + ciphertext + tag (PURE BYTES, NO STRUCTURE)
        let mut beacon_bytes = Vec::with_capacity(nonce.len() + ciphertext.len() + tag.len());
        beacon_bytes.extend_from_slice(&nonce);
        beacon_bytes.extend_from_slice(&ciphertext);
        beacon_bytes.extend_from_slice(&tag);

        info!(
            "✅ Pure noise beacon generated: {} bytes (zero metadata)",
            beacon_bytes.len()
        );
        debug!(
            "   Components: nonce={} bytes, ciphertext={} bytes, tag={} bytes",
            nonce.len(),
            ciphertext.len(),
            tag.len()
        );

        Ok(beacon_bytes)
    }

    /// Try to decrypt pure noise beacon (TRUE Dark Forest)
    ///
    /// Returns `Some(beacon)` if same family, `None` if different family/noise.
    /// Failures are SILENT (no logs) - true Dark Forest principle.
    ///
    /// # Format
    /// Input: `[nonce (12 bytes)] + [ciphertext (N bytes)] + [tag (16 bytes)]`
    ///
    /// # Returns
    /// - `Some(Value)` if same family (successful decrypt)
    /// - `None` if different family or actual noise (silent failure)
    ///
    /// # Security
    /// - No error logs on decrypt failure (silent)
    /// - Indistinguishable from handling random noise
    /// - Zero metadata extraction possible
    pub async fn try_decrypt_pure_noise_beacon(
        &self,
        noise_bytes: &[u8],
    ) -> SporeResult<Option<serde_json::Value>> {
        // Validate minimum size: nonce (12) + tag (16) = 28 bytes minimum
        if noise_bytes.len() < 28 {
            // SILENT - could be noise, ignore without logging
            return Ok(None);
        }

        // Derive OUR dedicated beacon key
        let beacon_key = match self.derive_dedicated_beacon_key().await {
            Ok(key) => key,
            Err(_) => {
                // SILENT - if we can't get key, treat as noise
                return Ok(None);
            }
        };

        // Split beacon: nonce (12 bytes) + ciphertext (variable) + tag (16 bytes)
        // ChaCha20-Poly1305 uses 12-byte nonce and 16-byte auth tag
        let nonce = &noise_bytes[0..12];
        let ciphertext_and_tag = &noise_bytes[12..];

        // Tag is last 16 bytes
        if ciphertext_and_tag.len() < 16 {
            // SILENT - malformed, treat as noise
            return Ok(None);
        }

        let ciphertext = &ciphertext_and_tag[..ciphertext_and_tag.len() - 16];
        let tag = &ciphertext_and_tag[ciphertext_and_tag.len() - 16..];

        // Encode for BearDog JSON-RPC
        let nonce_b64 = BASE64.encode(nonce);
        let ciphertext_b64 = BASE64.encode(ciphertext);
        let tag_b64 = BASE64.encode(tag);

        // Try to decrypt (SILENT failure on error)
        let decrypt_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "crypto.chacha20_poly1305_decrypt",
            "params": {
                "key": beacon_key,
                "ciphertext": ciphertext_b64,
                "nonce": nonce_b64,
                "tag": tag_b64
            },
            "id": 103
        });

        let response = match self.call_beardog(&decrypt_request).await {
            Ok(resp) => resp,
            Err(_) => {
                // SILENT - beardog communication failed, treat as noise
                return Ok(None);
            }
        };

        // Check if decryption failed (not family)
        if response.get("error").is_some() {
            // SILENT - different family or wrong key, this is noise to us
            return Ok(None);
        }

        // Decryption succeeded - we're family!
        let plaintext_b64 = match response
            .get("result")
            .and_then(|r| r.get("plaintext"))
            .and_then(|p| p.as_str())
        {
            Some(p) => p,
            None => {
                // SILENT - malformed response, treat as noise
                return Ok(None);
            }
        };

        // Decode plaintext
        let plaintext_bytes = match BASE64.decode(plaintext_b64) {
            Ok(bytes) => bytes,
            Err(_) => {
                // SILENT - invalid base64, treat as noise
                return Ok(None);
            }
        };

        // Parse beacon JSON
        let beacon: serde_json::Value = match serde_json::from_slice(&plaintext_bytes) {
            Ok(b) => b,
            Err(_) => {
                // SILENT - invalid JSON, treat as noise
                return Ok(None);
            }
        };

        info!("✅ Pure noise beacon decrypted - family member found");
        debug!(
            "   Node: {}",
            beacon
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        );

        Ok(Some(beacon))
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
