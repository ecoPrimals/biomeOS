// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Dark Forest beacon management
//!
//! Encrypted beacon generation/decryption and lineage verification via
//! capability-routed crypto (primal-agnostic).

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use biomeos_types::{JsonRpcRequest, JSONRPC_VERSION};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

use crate::beacon_genetics::CapabilityCaller;
use crate::error::{SporeError, SporeResult};

use super::types::{BeaconPlaintext, EncryptedBeacon};

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
    pub(crate) capability_caller: Arc<dyn CapabilityCaller>,
    /// Family seed (base64)
    pub(crate) family_seed_b64: String,
    /// Node ID
    pub(crate) node_id: String,
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
                format!("Failed to read seed file: {e}"),
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
        let request = JsonRpcRequest::new(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "our_family_id": "family",
                "peer_family_id": "broadcast",
                "context": "birdsong-broadcast-v1",
                "lineage_seed": self.family_seed_b64
            }),
        );
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

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
            .map_err(|e| SporeError::SystemError(format!("Time error: {e}")))?
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
        let encrypt_request = JsonRpcRequest::new(
            "crypto.chacha20_poly1305_encrypt",
            serde_json::json!({
                "key": broadcast_key,
                "plaintext": beacon_b64
            }),
        );
        let encrypt_value = serde_json::to_value(&encrypt_request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&encrypt_value).await?;
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
        let decrypt_request = JsonRpcRequest::new(
            "crypto.chacha20_poly1305_decrypt",
            serde_json::json!({
                "key": broadcast_key,
                "ciphertext": beacon.ciphertext,
                "nonce": beacon.nonce,
                "tag": beacon.tag
            }),
        );
        let decrypt_value = serde_json::to_value(&decrypt_request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&decrypt_value).await?;

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
            .map_err(|e| SporeError::DeserializationError(format!("Invalid base64: {e}")))?;

        let beacon: BeaconPlaintext = serde_json::from_slice(&plaintext_bytes)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid beacon JSON: {e}")))?;

        info!(
            "✅ Beacon decrypted - family member found: {}",
            beacon.node_id
        );

        Ok(Some(beacon))
    }

    /// Hash a string using Blake3 (via BearDog)
    async fn hash_string(&self, input: &str) -> SporeResult<String> {
        let input_b64 = BASE64.encode(input.as_bytes());

        let request = JsonRpcRequest::new(
            "crypto.blake3_hash",
            serde_json::json!({ "data": input_b64 }),
        );
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

        response
            .get("result")
            .and_then(|r| r.get("hash"))
            .and_then(|h| h.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SporeError::ValidationFailed("Failed to hash string".to_string()))
    }

    /// Call crypto provider via capability routing
    async fn call_beardog(&self, request: &serde_json::Value) -> SporeResult<serde_json::Value> {
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing method in request".to_string()))?;

        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let result = self
            .capability_caller
            .call(method, params)
            .await
            .map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("Capability call '{method}' failed: {e}"),
                ))
            })?;

        Ok(serde_json::json!({
            "jsonrpc": JSONRPC_VERSION,
            "result": result,
            "id": request.get("id").cloned().unwrap_or(serde_json::json!(1))
        }))
    }

    /// Verify lineage after successful beacon decryption
    pub async fn verify_peer_lineage(
        &self,
        peer_family_id: &str,
        peer_proof: &str,
    ) -> SporeResult<bool> {
        info!("🔍 Verifying peer lineage (independent validation)");

        let request = JsonRpcRequest::new(
            "genetic.verify_lineage",
            serde_json::json!({
                "our_family_id": "family",
                "peer_family_id": peer_family_id,
                "lineage_proof": peer_proof,
                "lineage_seed": self.family_seed_b64
            }),
        );
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

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
        let request = JsonRpcRequest::new(
            "genetic.generate_lineage_proof",
            serde_json::json!({
                "our_family_id": "family",
                "peer_family_id": peer_family_id,
                "lineage_seed": self.family_seed_b64
            }),
        );
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

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
        let request = JsonRpcRequest::new(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "our_family_id": "family",
                "peer_family_id": peer_id,
                "context": context,
                "lineage_seed": self.family_seed_b64
            }),
        );
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

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

    /// Derive dedicated beacon key from lineage (TRUE Dark Forest)
    async fn derive_dedicated_beacon_key(&self) -> SporeResult<String> {
        let request =
            JsonRpcRequest::new("genetic.derive_lineage_beacon_key", serde_json::json!({}));
        let request_value = serde_json::to_value(&request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = self.call_beardog(&request_value).await?;

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
    pub async fn generate_pure_noise_beacon(
        &self,
        socket_path: &str,
        capabilities: &[&str],
        lineage_mode: Option<&str>,
    ) -> SporeResult<Vec<u8>> {
        info!("🌑 Generating pure noise Dark Forest beacon (A++ security)");

        let beacon_key = self.derive_dedicated_beacon_key().await?;
        debug!("   Derived dedicated beacon key (HKDF-SHA256)");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SporeError::SystemError(format!("Time error: {e}")))?
            .as_secs();

        let beacon = serde_json::json!({
            "node_id": self.node_id,
            "timestamp": timestamp,
            "socket_path": socket_path,
            "capabilities": capabilities,
            "lineage_mode": lineage_mode
        });

        let beacon_json = serde_json::to_string(&beacon)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;
        let beacon_b64 = BASE64.encode(beacon_json.as_bytes());

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

        let nonce = BASE64
            .decode(nonce_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid nonce: {e}")))?;
        let ciphertext = BASE64
            .decode(ciphertext_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid ciphertext: {e}")))?;
        let tag = BASE64
            .decode(tag_b64)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid tag: {e}")))?;

        let mut beacon_bytes = Vec::with_capacity(nonce.len() + ciphertext.len() + tag.len());
        beacon_bytes.extend_from_slice(&nonce);
        beacon_bytes.extend_from_slice(&ciphertext);
        beacon_bytes.extend_from_slice(&tag);

        info!(
            "✅ Pure noise beacon generated: {} bytes (zero metadata)",
            beacon_bytes.len()
        );

        Ok(beacon_bytes)
    }

    /// Try to decrypt pure noise beacon (TRUE Dark Forest)
    pub async fn try_decrypt_pure_noise_beacon(
        &self,
        noise_bytes: &[u8],
    ) -> SporeResult<Option<serde_json::Value>> {
        if noise_bytes.len() < 28 {
            return Ok(None);
        }

        let beacon_key = match self.derive_dedicated_beacon_key().await {
            Ok(key) => key,
            Err(_) => return Ok(None),
        };

        let nonce = &noise_bytes[0..12];
        let ciphertext_and_tag = &noise_bytes[12..];

        if ciphertext_and_tag.len() < 16 {
            return Ok(None);
        }

        let ciphertext = &ciphertext_and_tag[..ciphertext_and_tag.len() - 16];
        let tag = &ciphertext_and_tag[ciphertext_and_tag.len() - 16..];

        let nonce_b64 = BASE64.encode(nonce);
        let ciphertext_b64 = BASE64.encode(ciphertext);
        let tag_b64 = BASE64.encode(tag);

        let decrypt_request = JsonRpcRequest::new(
            "crypto.chacha20_poly1305_decrypt",
            serde_json::json!({
                "key": beacon_key,
                "ciphertext": ciphertext_b64,
                "nonce": nonce_b64,
                "tag": tag_b64
            }),
        );
        let decrypt_value = serde_json::to_value(&decrypt_request)
            .map_err(|e| SporeError::SerializationError(format!("JSON error: {e}")))?;
        let response = match self.call_beardog(&decrypt_value).await {
            Ok(resp) => resp,
            Err(_) => return Ok(None),
        };

        if response.get("error").is_some() {
            return Ok(None);
        }

        let plaintext_b64 = match response
            .get("result")
            .and_then(|r| r.get("plaintext"))
            .and_then(|p| p.as_str())
        {
            Some(p) => p,
            None => return Ok(None),
        };

        let plaintext_bytes = match BASE64.decode(plaintext_b64) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };

        let beacon: serde_json::Value = match serde_json::from_slice(&plaintext_bytes) {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        info!("✅ Pure noise beacon decrypted - family member found");

        Ok(Some(beacon))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[path = "beacon_tests.rs"]
mod tests;
