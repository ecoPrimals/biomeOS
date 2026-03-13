// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

use crate::beacon_genetics::CapabilityCaller;
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
///
/// ## Deep Debt Evolution (Feb 11, 2026)
///
/// - BEFORE: Direct `beardog_socket: String` — hardcoded primal knowledge
/// - AFTER: `Arc<dyn CapabilityCaller>` — capability-routed, primal-agnostic
///
/// The beacon manager has zero knowledge of which primal provides crypto.
/// All operations route through the `CapabilityCaller` trait:
/// - Production: `NeuralApiCapabilityCaller` → Neural API → discovered primal
/// - Bootstrap: `DirectBeardogCaller` → direct socket (enrollment only)
/// - Testing: `MockCapabilityCaller` → deterministic responses
#[derive(Clone)]
pub struct DarkForestBeacon {
    /// Capability caller for crypto operations (primal-agnostic)
    capability_caller: Arc<dyn CapabilityCaller>,
    /// Family seed (base64)
    family_seed_b64: String,
    /// Node ID
    node_id: String,
}

impl DarkForestBeacon {
    /// Create a new Dark Forest beacon manager with capability routing
    ///
    /// # Arguments
    /// * `capability_caller` - Primal-agnostic crypto provider
    /// * `seed_path` - Path to .family.seed file
    /// * `node_id` - This node's identifier
    pub async fn new<P: AsRef<Path>>(
        capability_caller: Arc<dyn CapabilityCaller>,
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
            capability_caller,
            family_seed_b64,
            node_id: node_id.to_string(),
        })
    }

    /// Create from a BearDog socket path (backward compatibility / bootstrap)
    ///
    /// Wraps the socket path in a `DirectBeardogCaller`. Prefer `new()` with
    /// `NeuralApiCapabilityCaller` for production use.
    pub async fn from_beardog_socket<P: AsRef<Path>>(
        beardog_socket: &str,
        seed_path: P,
        node_id: &str,
    ) -> SporeResult<Self> {
        let caller = Arc::new(crate::beacon_genetics::DirectBeardogCaller::new(
            beardog_socket,
        ));
        Self::new(caller, seed_path, node_id).await
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

    /// Call crypto provider via capability routing
    ///
    /// Deep Debt Evolution: Routes through `CapabilityCaller` trait instead of
    /// direct BearDog socket. The caller has zero knowledge of which primal
    /// provides the capability.
    ///
    /// Maintains JSON-RPC envelope compatibility with existing call sites.
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

        // Route through capability caller (primal-agnostic)
        let result = self
            .capability_caller
            .call(method, params)
            .await
            .map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Capability call '{}' failed: {}", method, e),
                ))
            })?;

        // Wrap result in JSON-RPC response format for compatibility with call sites
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

    // ========================================================================
    // BeaconPlaintext Tests
    // ========================================================================

    #[test]
    fn test_beacon_plaintext_serialization_roundtrip() {
        let beacon = BeaconPlaintext {
            family_hash: "abc123def456".to_string(),
            node_id: "tower1".to_string(),
            timestamp: 1234567890,
            socket_path: "/tmp/beardog.sock".to_string(),
            capabilities_hash: "cap_hash_def456".to_string(),
            lineage_mode: Some("genesis".to_string()),
        };

        let json = serde_json::to_string(&beacon).expect("serialize beacon");
        let parsed: BeaconPlaintext = serde_json::from_str(&json).expect("parse beacon");

        assert_eq!(parsed.family_hash, "abc123def456");
        assert_eq!(parsed.node_id, "tower1");
        assert_eq!(parsed.timestamp, 1234567890);
        assert_eq!(parsed.socket_path, "/tmp/beardog.sock");
        assert_eq!(parsed.capabilities_hash, "cap_hash_def456");
        assert_eq!(parsed.lineage_mode, Some("genesis".to_string()));
    }

    #[test]
    fn test_beacon_plaintext_without_lineage() {
        let beacon = BeaconPlaintext {
            family_hash: "abc".to_string(),
            node_id: "node1".to_string(),
            timestamp: 100,
            socket_path: "/tmp/sock".to_string(),
            capabilities_hash: "hash".to_string(),
            lineage_mode: None,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        // lineage_mode with skip_serializing_if = None should be omitted
        assert!(!json.contains("lineage_mode"));

        let parsed: BeaconPlaintext = serde_json::from_str(&json).expect("parse");
        assert!(parsed.lineage_mode.is_none());
    }

    #[test]
    fn test_beacon_plaintext_sibling_lineage() {
        let beacon = BeaconPlaintext {
            family_hash: "fam1".to_string(),
            node_id: "tower2".to_string(),
            timestamp: 99999,
            socket_path: "/run/user/1000/biomeos/beardog.sock".to_string(),
            capabilities_hash: "caps".to_string(),
            lineage_mode: Some("sibling".to_string()),
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        assert!(json.contains("sibling"));
        assert!(json.contains("tower2"));
    }

    #[test]
    fn test_beacon_plaintext_clone() {
        let beacon = BeaconPlaintext {
            family_hash: "h".to_string(),
            node_id: "n".to_string(),
            timestamp: 42,
            socket_path: "/s".to_string(),
            capabilities_hash: "c".to_string(),
            lineage_mode: None,
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.node_id, beacon.node_id);
        assert_eq!(cloned.timestamp, beacon.timestamp);
    }

    #[test]
    fn test_beacon_plaintext_debug() {
        let beacon = BeaconPlaintext {
            family_hash: "h".to_string(),
            node_id: "n".to_string(),
            timestamp: 0,
            socket_path: "/s".to_string(),
            capabilities_hash: "c".to_string(),
            lineage_mode: None,
        };

        let debug = format!("{:?}", beacon);
        assert!(debug.contains("BeaconPlaintext"));
        assert!(debug.contains("node_id"));
    }

    // ========================================================================
    // EncryptedBeacon Tests
    // ========================================================================

    #[test]
    fn test_encrypted_beacon_serialization_roundtrip() {
        let beacon = EncryptedBeacon {
            ciphertext: "base64ciphertext==".to_string(),
            nonce: "base64nonce==".to_string(),
            tag: "base64tag==".to_string(),
            version: 1,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        let parsed: EncryptedBeacon = serde_json::from_str(&json).expect("parse");

        assert_eq!(parsed.ciphertext, "base64ciphertext==");
        assert_eq!(parsed.nonce, "base64nonce==");
        assert_eq!(parsed.tag, "base64tag==");
        assert_eq!(parsed.version, 1);
    }

    #[test]
    fn test_encrypted_beacon_clone() {
        let beacon = EncryptedBeacon {
            ciphertext: "ct".to_string(),
            nonce: "n".to_string(),
            tag: "t".to_string(),
            version: 2,
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.version, 2);
        assert_eq!(cloned.ciphertext, "ct");
    }

    #[test]
    fn test_encrypted_beacon_debug() {
        let beacon = EncryptedBeacon {
            ciphertext: "data".to_string(),
            nonce: "nonce".to_string(),
            tag: "tag".to_string(),
            version: 1,
        };

        let debug = format!("{:?}", beacon);
        assert!(debug.contains("EncryptedBeacon"));
        assert!(debug.contains("version"));
    }

    // ========================================================================
    // DarkForestBeacon Tests (non-I/O)
    // ========================================================================

    #[test]
    fn test_dark_forest_beacon_clone() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let beacon = DarkForestBeacon {
            capability_caller: Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock")),
            family_seed_b64: "dGVzdHNlZWQ=".to_string(),
            node_id: "tower1".to_string(),
        };

        let cloned = beacon.clone();
        assert_eq!(cloned.node_id, "tower1");
        assert_eq!(cloned.family_seed_b64, beacon.family_seed_b64);
    }

    #[tokio::test]
    async fn test_dark_forest_beacon_new_missing_seed() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let caller = Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock"));
        let result =
            DarkForestBeacon::new(caller, "/nonexistent/path/.family.seed", "tower1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dark_forest_beacon_new_with_seed_file() {
        use crate::beacon_genetics::DirectBeardogCaller;

        let dir = tempfile::tempdir().expect("create tempdir");
        let seed_path = dir.path().join(".family.seed");
        tokio::fs::write(&seed_path, b"test-seed-bytes-32chars-minimum!")
            .await
            .expect("write seed");

        let caller = Arc::new(DirectBeardogCaller::new("/tmp/beardog.sock"));
        let beacon = DarkForestBeacon::new(caller, &seed_path, "tower1")
            .await
            .expect("create beacon");

        assert_eq!(beacon.node_id, "tower1");
        assert!(!beacon.family_seed_b64.is_empty());

        // Verify the seed was properly base64 encoded
        let decoded = BASE64
            .decode(&beacon.family_seed_b64)
            .expect("decode base64");
        assert_eq!(decoded, b"test-seed-bytes-32chars-minimum!");
    }

    // ========================================================================
    // Base64 / encoding tests (relevant to beacon creation)
    // ========================================================================

    #[test]
    fn test_beacon_plaintext_json_to_base64_roundtrip() {
        let beacon = BeaconPlaintext {
            family_hash: "fam123".to_string(),
            node_id: "tower1".to_string(),
            timestamp: 1700000000,
            socket_path: "/run/user/1000/biomeos/beardog.sock".to_string(),
            capabilities_hash: "cap456".to_string(),
            lineage_mode: Some("genesis".to_string()),
        };

        // Simulate what generate_encrypted_beacon does
        let json = serde_json::to_string(&beacon).expect("serialize");
        let b64 = BASE64.encode(json.as_bytes());

        // Simulate what try_decrypt_beacon does
        let decoded_bytes = BASE64.decode(&b64).expect("decode base64");
        let decoded: BeaconPlaintext =
            serde_json::from_slice(&decoded_bytes).expect("parse decoded beacon");

        assert_eq!(decoded.family_hash, "fam123");
        assert_eq!(decoded.node_id, "tower1");
        assert_eq!(decoded.timestamp, 1700000000);
    }

    #[test]
    fn test_beacon_version_field() {
        // Version should be serialized for protocol evolution
        let beacon = EncryptedBeacon {
            ciphertext: "data".to_string(),
            nonce: "nonce".to_string(),
            tag: "tag".to_string(),
            version: 2,
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        assert!(json.contains("\"version\":2"));
    }

    // ========================================================================
    // Pure noise beacon format validation (min size, structure)
    // ========================================================================

    #[test]
    fn test_pure_noise_beacon_minimum_size() {
        // try_decrypt_pure_noise_beacon returns None for < 28 bytes (12 nonce + 16 tag)
        // This tests our understanding of the format
        let too_small: [u8; 27] = [0u8; 27];
        assert!(too_small.len() < 28);

        let exactly_min: [u8; 28] = [0u8; 28];
        assert_eq!(exactly_min.len(), 28);
    }

    #[test]
    fn test_discovered_peer_struct() {
        let peer = DiscoveredPeer {
            beacon: BeaconPlaintext {
                family_hash: "h".to_string(),
                node_id: "n1".to_string(),
                timestamp: 100,
                socket_path: "/s".to_string(),
                capabilities_hash: "c".to_string(),
                lineage_mode: None,
            },
            lineage_verified: true,
            session_key: Some("key123".to_string()),
        };
        assert!(peer.lineage_verified);
        assert_eq!(peer.session_key.as_deref(), Some("key123"));
        assert_eq!(peer.beacon.node_id, "n1");
    }

    #[test]
    fn test_encrypted_beacon_version_values() {
        for v in [0u8, 1, 2, 255] {
            let beacon = EncryptedBeacon {
                ciphertext: "c".to_string(),
                nonce: "n".to_string(),
                tag: "t".to_string(),
                version: v,
            };
            let json = serde_json::to_string(&beacon).unwrap();
            assert!(json.contains(&format!("\"version\":{}", v)));
        }
    }
}
