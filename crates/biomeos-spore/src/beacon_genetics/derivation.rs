//! Device Lineage Derivation
//!
//! Implements the genetic model for device-specific seed derivation.
//!
//! ## Key Principle: DERIVE, Don't Clone
//!
//! Instead of copying the family seed to each device (which is cloning),
//! we DERIVE a unique seed per device from the shared family root.
//!
//! ## Architecture
//!
//! ```text
//! Genesis Root (shared family seed)
//!     │
//!     │ HKDF(root, device_id, device_entropy)
//!     ├────────────────┬────────────────┐
//!     ▼                ▼                ▼
//! Tower Lineage   Pixel Lineage   Phone Lineage
//! (unique)        (unique)        (unique)
//! ```
//!
//! Each device has a UNIQUE seed but can prove shared ancestry.
//!
//! AGPL-3.0-only License

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

use super::capability::CapabilityCaller;
use crate::error::{SporeError, SporeResult};

// ============================================================================
// TYPES
// ============================================================================

/// Device lineage seed (derived, unique per device)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLineage {
    /// Device identifier
    pub device_id: String,

    /// Node identifier (human-friendly name)
    pub node_id: String,

    /// Family ID this device belongs to
    pub family_id: String,

    /// Generation (1 = direct child of genesis)
    pub generation: u32,

    /// Derived seed (base64, 32 bytes)
    /// This is UNIQUE to this device
    pub derived_seed: String,

    /// Timestamp of derivation
    pub derived_at: u64,

    /// Derivation method used
    pub derivation_method: String,

    /// Optional lineage certificate (when available)
    pub lineage_certificate: Option<String>,
}

/// Result of device enrollment
#[derive(Debug, Clone)]
pub struct EnrollmentResult {
    /// The derived device lineage
    pub lineage: DeviceLineage,

    /// Path where lineage seed was saved
    pub seed_path: std::path::PathBuf,
}

/// Derivation request parameters
#[derive(Debug, Clone, Serialize)]
pub struct DerivationParams {
    /// The root/family seed (base64)
    pub family_seed: String,

    /// Device ID for derivation
    pub device_id: String,

    /// Node ID for derivation
    pub node_id: String,

    /// Additional entropy (base64, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_entropy: Option<String>,

    /// Purpose/context string
    pub purpose: String,
}

// ============================================================================
// LINEAGE DERIVER
// ============================================================================

/// Device lineage deriver - orchestrates derivation via BearDog
pub struct LineageDeriver<C: CapabilityCaller> {
    caller: C,
}

impl<C: CapabilityCaller> LineageDeriver<C> {
    /// Create new lineage deriver with capability caller
    pub fn new(caller: C) -> Self {
        Self { caller }
    }

    /// Derive a unique device seed from the family root
    ///
    /// This is the core of the genetic model:
    /// - Takes the shared family seed as input
    /// - Derives a UNIQUE seed for this specific device
    /// - The derived seed can be used for all device-specific crypto
    ///
    /// ## Parameters
    ///
    /// - `family_seed`: The shared family seed (base64, 32 bytes)
    /// - `family_id`: Family identifier
    /// - `device_id`: Unique device identifier (e.g., hardware UUID)
    /// - `node_id`: Human-friendly node name (e.g., "tower", "pixel8a")
    /// - `device_entropy`: Optional additional entropy from device
    ///
    /// ## Returns
    ///
    /// A unique `DeviceLineage` with derived seed
    pub async fn derive_device_seed(
        &self,
        family_seed: &str,
        family_id: &str,
        device_id: &str,
        node_id: &str,
        device_entropy: Option<&[u8]>,
    ) -> SporeResult<DeviceLineage> {
        info!(
            "🧬 Deriving device lineage for {} ({})",
            node_id, device_id
        );

        // Build context for derivation
        let context = format!("ecoPrimals-device-lineage-v1:{}:{}", family_id, device_id);

        // Prepare parameters for BearDog
        let params = serde_json::json!({
            "our_family_id": format!("{}-{}", family_id, device_id),
            "peer_family_id": family_id, // Derive from family root
            "context": context,
            "lineage_seed": family_seed,
        });

        debug!("Calling genetic.derive_lineage_key with device-specific params");

        // Call BearDog to derive the key
        let result = self
            .caller
            .call("genetic.derive_lineage_key", params)
            .await
            .map_err(|e| {
                SporeError::SystemError(format!("Failed to derive lineage key: {}", e))
            })?;

        // Extract the derived key
        let derived_key = result
            .get("key")
            .and_then(|v: &serde_json::Value| v.as_str())
            .ok_or_else(|| {
                SporeError::SystemError("Missing 'key' in derivation result".to_string())
            })?;

        let method = result
            .get("method")
            .and_then(|v: &serde_json::Value| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        info!(
            "✅ Derived unique seed for {} using {}",
            node_id, method
        );

        // If additional device entropy provided, mix it in
        let final_seed = if let Some(entropy) = device_entropy {
            self.mix_device_entropy(derived_key, entropy, device_id)
                .await?
        } else {
            derived_key.to_string()
        };

        let derived_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // DEEP DEBT EVOLUTION: Sign the lineage certificate via security provider
        // BearDog provides Ed25519 signing via crypto.sign capability
        let lineage_certificate = self.sign_lineage_certificate(
            device_id, node_id, family_id, &final_seed, derived_at
        ).await;

        Ok(DeviceLineage {
            device_id: device_id.to_string(),
            node_id: node_id.to_string(),
            family_id: family_id.to_string(),
            generation: 1,
            derived_seed: final_seed,
            derived_at,
            derivation_method: method,
            lineage_certificate,
        })
    }

    /// Sign a lineage certificate via the security provider
    ///
    /// DEEP DEBT EVOLUTION (Feb 7, 2026): Implemented lineage signing.
    /// Uses Ed25519 signing via crypto.sign capability. If the security
    /// provider is unavailable, returns None (graceful degradation).
    async fn sign_lineage_certificate(
        &self,
        device_id: &str,
        node_id: &str,
        family_id: &str,
        derived_seed: &str,
        derived_at: u64,
    ) -> Option<String> {
        // Construct the data to sign
        let sign_data = format!(
            "lineage:{}:{}:{}:{}:{}",
            device_id, node_id, family_id, derived_seed, derived_at
        );

        let params = serde_json::json!({
            "data": sign_data,
            "purpose": "lineage_certificate",
            "family_id": family_id,
        });

        match self.caller.call("crypto.sign", params).await {
            Ok(result) => {
                if let Some(signature) = result.get("signature").and_then(|v| v.as_str()) {
                    debug!("✅ Lineage certificate signed for device {}", device_id);
                    Some(signature.to_string())
                } else {
                    debug!("⚠️  Signing succeeded but no signature in response");
                    None
                }
            }
            Err(e) => {
                debug!("⚠️  Lineage certificate signing unavailable: {}", e);
                debug!("   Proceeding without certificate (graceful degradation)");
                None
            }
        }
    }

    /// Mix additional device entropy into the derived seed
    async fn mix_device_entropy(
        &self,
        derived_key: &str,
        entropy: &[u8],
        device_id: &str,
    ) -> SporeResult<String> {
        debug!("Mixing {} bytes of device entropy", entropy.len());

        let params = serde_json::json!({
            "sources": [
                {"type": "derived", "data": derived_key},
                {"type": "device", "data": BASE64.encode(entropy)},
            ],
            "purpose": format!("device-entropy-mix:{}", device_id)
        });

        match self.caller.call("genetic.mix_entropy", params).await {
            Ok(result) => {
                let mixed = result
                    .get("mixed_seed")
                    .or_else(|| result.get("seed"))
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or(derived_key);
                Ok(mixed.to_string())
            }
            Err(e) => {
                warn!(
                    "Failed to mix device entropy, using derived key directly: {}",
                    e
                );
                Ok(derived_key.to_string())
            }
        }
    }

    /// Enroll a device - derive seed and save to file
    ///
    /// This is the high-level enrollment function that:
    /// 1. Reads the family seed from the specified path
    /// 2. Derives a unique device seed
    /// 3. Saves the lineage to `.lineage.seed` file
    pub async fn enroll_device(
        &self,
        family_seed_path: &Path,
        lineage_seed_path: &Path,
        family_id: &str,
        device_id: &str,
        node_id: &str,
    ) -> SporeResult<EnrollmentResult> {
        info!("📝 Enrolling device {} as {}", device_id, node_id);

        // Read family seed
        let family_seed_bytes = std::fs::read(family_seed_path).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read family seed: {}", e),
            ))
        })?;

        let family_seed = BASE64.encode(&family_seed_bytes);

        // Derive device seed
        let lineage = self
            .derive_device_seed(&family_seed, family_id, device_id, node_id, None)
            .await?;

        // Save lineage seed
        self.save_lineage(&lineage, lineage_seed_path)?;

        info!(
            "✅ Device {} enrolled, lineage saved to {}",
            node_id,
            lineage_seed_path.display()
        );

        Ok(EnrollmentResult {
            lineage,
            seed_path: lineage_seed_path.to_path_buf(),
        })
    }

    /// Save lineage to file
    fn save_lineage(&self, lineage: &DeviceLineage, path: &Path) -> SporeResult<()> {
        // Decode the base64 seed and write raw bytes
        let seed_bytes = BASE64.decode(&lineage.derived_seed).map_err(|e| {
            SporeError::DeserializationError(format!("Invalid base64 seed: {}", e))
        })?;

        std::fs::write(path, &seed_bytes).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write lineage seed: {}",
                e
            )))
        })?;

        // Also save metadata to .lineage.json
        let metadata_path = path.with_extension("json");
        let metadata = serde_json::to_string_pretty(lineage)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        std::fs::write(&metadata_path, metadata).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write lineage metadata: {}",
                e
            )))
        })?;

        debug!("Lineage metadata saved to {}", metadata_path.display());

        Ok(())
    }

    /// Load existing device lineage from file
    pub fn load_lineage(lineage_path: &Path) -> SporeResult<DeviceLineage> {
        let metadata_path = lineage_path.with_extension("json");

        if metadata_path.exists() {
            let contents = std::fs::read_to_string(&metadata_path).map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read lineage metadata: {}", e),
                ))
            })?;

            serde_json::from_str(&contents)
                .map_err(|e| SporeError::DeserializationError(format!("Invalid JSON: {}", e)))
        } else {
            // Fallback: read raw seed and construct minimal lineage
            let seed_bytes = std::fs::read(lineage_path).map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read lineage seed: {}", e),
                ))
            })?;

            Ok(DeviceLineage {
                device_id: "unknown".to_string(),
                node_id: "unknown".to_string(),
                family_id: "unknown".to_string(),
                generation: 1,
                derived_seed: BASE64.encode(&seed_bytes),
                derived_at: 0,
                derivation_method: "unknown".to_string(),
                lineage_certificate: None,
            })
        }
    }

    /// Check if device has existing lineage
    pub fn has_lineage(lineage_path: &Path) -> bool {
        lineage_path.exists()
    }

    /// Generate lineage proof for verification
    pub async fn generate_lineage_proof(
        &self,
        lineage: &DeviceLineage,
        peer_family_id: &str,
    ) -> SporeResult<String> {
        let params = serde_json::json!({
            "our_family_id": format!("{}-{}", lineage.family_id, lineage.device_id),
            "peer_family_id": peer_family_id,
            "lineage_seed": lineage.derived_seed,
        });

        let result = self
            .caller
            .call("genetic.generate_lineage_proof", params)
            .await
            .map_err(|e| {
                SporeError::SystemError(format!("Failed to generate proof: {}", e))
            })?;

        result
            .get("proof")
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s: &str| s.to_string())
            .ok_or_else(|| {
                SporeError::SystemError("Missing 'proof' in result".to_string())
            })
    }

    /// Verify lineage proof from a peer
    pub async fn verify_lineage_proof(
        &self,
        lineage: &DeviceLineage,
        peer_family_id: &str,
        peer_proof: &str,
    ) -> SporeResult<bool> {
        let params = serde_json::json!({
            "our_family_id": format!("{}-{}", lineage.family_id, lineage.device_id),
            "peer_family_id": peer_family_id,
            "lineage_proof": peer_proof,
            "lineage_seed": lineage.derived_seed,
        });

        let result = self
            .caller
            .call("genetic.verify_lineage", params)
            .await
            .map_err(|e| {
                SporeError::SystemError(format!("Failed to verify proof: {}", e))
            })?;

        Ok(result.get("valid").and_then(|v: &serde_json::Value| v.as_bool()).unwrap_or(false))
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Generate device entropy from available sources
pub fn generate_device_entropy() -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Mix in timestamp
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        .hash(&mut hasher);

    // Mix in process ID
    std::process::id().hash(&mut hasher);

    // Mix in thread ID
    std::thread::current().id().hash(&mut hasher);

    // Convert to bytes and expand with more hashing
    let hash = hasher.finish().to_le_bytes();

    // Expand to 32 bytes by hashing multiple times
    let mut entropy = Vec::with_capacity(32);
    for i in 0..4 {
        let mut h = DefaultHasher::new();
        hash.hash(&mut h);
        i.hash(&mut h);
        entropy.extend_from_slice(&h.finish().to_le_bytes());
    }

    entropy
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_lineage_serialization() {
        let lineage = DeviceLineage {
            device_id: "device-123".to_string(),
            node_id: "tower".to_string(),
            family_id: "1894e909e454".to_string(),
            generation: 1,
            derived_seed: "dGVzdHNlZWQ=".to_string(), // "testseed" in base64
            derived_at: 1738726800,
            derivation_method: "Blake3-Lineage-KDF".to_string(),
            lineage_certificate: None,
        };

        let json = serde_json::to_string(&lineage).unwrap();
        assert!(json.contains("device-123"));
        assert!(json.contains("tower"));
        assert!(json.contains("1894e909e454"));

        let parsed: DeviceLineage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.device_id, "device-123");
        assert_eq!(parsed.node_id, "tower");
    }

    #[test]
    fn test_generate_device_entropy() {
        let entropy1 = generate_device_entropy();
        let entropy2 = generate_device_entropy();

        // Should be 32 bytes
        assert_eq!(entropy1.len(), 32);
        assert_eq!(entropy2.len(), 32);

        // Should be different each time (high probability)
        // Note: In rare cases this could fail due to timing
        assert_ne!(entropy1, entropy2);
    }

    #[test]
    fn test_derivation_params_serialization() {
        let params = DerivationParams {
            family_seed: "c2VlZA==".to_string(),
            device_id: "dev-001".to_string(),
            node_id: "tower".to_string(),
            device_entropy: Some("ZW50cm9weQ==".to_string()),
            purpose: "device-lineage".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("family_seed"));
        assert!(json.contains("dev-001"));
        assert!(json.contains("device_entropy"));
    }

    #[test]
    fn test_derivation_params_without_entropy() {
        let params = DerivationParams {
            family_seed: "c2VlZA==".to_string(),
            device_id: "dev-001".to_string(),
            node_id: "tower".to_string(),
            device_entropy: None,
            purpose: "device-lineage".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("device_entropy")); // Should be skipped
    }
}
