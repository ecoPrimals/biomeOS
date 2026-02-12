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
        info!("🧬 Deriving device lineage for {} ({})", node_id, device_id);

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
            .map_err(|e| SporeError::SystemError(format!("Failed to derive lineage key: {}", e)))?;

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

        info!("✅ Derived unique seed for {} using {}", node_id, method);

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
        let lineage_certificate = self
            .sign_lineage_certificate(device_id, node_id, family_id, &final_seed, derived_at)
            .await;

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
        let seed_bytes = BASE64
            .decode(&lineage.derived_seed)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid base64 seed: {}", e)))?;

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
            .map_err(|e| SporeError::SystemError(format!("Failed to generate proof: {}", e)))?;

        result
            .get("proof")
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s: &str| s.to_string())
            .ok_or_else(|| SporeError::SystemError("Missing 'proof' in result".to_string()))
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
            .map_err(|e| SporeError::SystemError(format!("Failed to verify proof: {}", e)))?;

        Ok(result
            .get("valid")
            .and_then(|v: &serde_json::Value| v.as_bool())
            .unwrap_or(false))
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
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // ═══════════════════════════════════════════════════════════════
    // Mock capability caller
    // ═══════════════════════════════════════════════════════════════

    struct MockCaller {
        responses: Arc<Mutex<HashMap<String, Result<serde_json::Value, String>>>>,
    }

    impl MockCaller {
        fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        async fn set_ok(&self, cap: &str, val: serde_json::Value) {
            self.responses.lock().await.insert(cap.to_string(), Ok(val));
        }

        async fn set_err(&self, cap: &str, msg: &str) {
            self.responses
                .lock()
                .await
                .insert(cap.to_string(), Err(msg.to_string()));
        }
    }

    #[async_trait::async_trait]
    impl CapabilityCaller for MockCaller {
        async fn call(
            &self,
            capability: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            let responses = self.responses.lock().await;
            responses
                .get(capability)
                .cloned()
                .unwrap_or_else(|| Err(format!("No mock for {}", capability)))
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // DeviceLineage tests
    // ═══════════════════════════════════════════════════════════════

    fn sample_lineage() -> DeviceLineage {
        DeviceLineage {
            device_id: "device-123".to_string(),
            node_id: "tower".to_string(),
            family_id: "1894e909e454".to_string(),
            generation: 1,
            derived_seed: "dGVzdHNlZWQ=".to_string(), // "testseed" in base64
            derived_at: 1738726800,
            derivation_method: "Blake3-Lineage-KDF".to_string(),
            lineage_certificate: None,
        }
    }

    #[test]
    fn test_device_lineage_serialization() {
        let lineage = sample_lineage();

        let json = serde_json::to_string(&lineage).expect("serialize");
        assert!(json.contains("device-123"));
        assert!(json.contains("tower"));
        assert!(json.contains("1894e909e454"));

        let parsed: DeviceLineage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.device_id, "device-123");
        assert_eq!(parsed.node_id, "tower");
    }

    #[test]
    fn test_device_lineage_serde_roundtrip() {
        let lineage = DeviceLineage {
            lineage_certificate: Some("signed-cert-data".to_string()),
            ..sample_lineage()
        };
        let json = serde_json::to_string(&lineage).expect("serialize");
        let restored: DeviceLineage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            restored.lineage_certificate.as_deref(),
            Some("signed-cert-data")
        );
        assert_eq!(restored.generation, 1);
        assert_eq!(restored.derivation_method, "Blake3-Lineage-KDF");
    }

    #[test]
    fn test_device_lineage_clone_and_debug() {
        let lineage = sample_lineage();
        let cloned = lineage.clone();
        assert_eq!(cloned.device_id, "device-123");
        let dbg = format!("{:?}", lineage);
        assert!(dbg.contains("DeviceLineage"));
        assert!(dbg.contains("device-123"));
    }

    // ═══════════════════════════════════════════════════════════════
    // EnrollmentResult tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_enrollment_result_clone_and_debug() {
        let result = EnrollmentResult {
            lineage: sample_lineage(),
            seed_path: std::path::PathBuf::from("/tmp/test.seed"),
        };
        let cloned = result.clone();
        assert_eq!(cloned.seed_path, Path::new("/tmp/test.seed"));
        assert_eq!(cloned.lineage.node_id, "tower");

        let dbg = format!("{:?}", result);
        assert!(dbg.contains("EnrollmentResult"));
    }

    // ═══════════════════════════════════════════════════════════════
    // DerivationParams tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_derivation_params_serialization() {
        let params = DerivationParams {
            family_seed: "c2VlZA==".to_string(),
            device_id: "dev-001".to_string(),
            node_id: "tower".to_string(),
            device_entropy: Some("ZW50cm9weQ==".to_string()),
            purpose: "device-lineage".to_string(),
        };

        let json = serde_json::to_string(&params).expect("serialize");
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

        let json = serde_json::to_string(&params).expect("serialize");
        assert!(!json.contains("device_entropy")); // Should be skipped
    }

    #[test]
    fn test_derivation_params_clone_and_debug() {
        let params = DerivationParams {
            family_seed: "seed".into(),
            device_id: "dev".into(),
            node_id: "node".into(),
            device_entropy: None,
            purpose: "test".into(),
        };
        let cloned = params.clone();
        assert_eq!(cloned.purpose, "test");
        let dbg = format!("{:?}", params);
        assert!(dbg.contains("DerivationParams"));
    }

    // ═══════════════════════════════════════════════════════════════
    // generate_device_entropy tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_generate_device_entropy() {
        let entropy1 = generate_device_entropy();
        let entropy2 = generate_device_entropy();

        // Should be 32 bytes
        assert_eq!(entropy1.len(), 32);
        assert_eq!(entropy2.len(), 32);

        // Should be different each time (high probability)
        assert_ne!(entropy1, entropy2);
    }

    #[test]
    fn test_generate_device_entropy_non_zero() {
        let entropy = generate_device_entropy();
        // All zeros is astronomically unlikely
        assert!(entropy.iter().any(|&b| b != 0));
    }

    // ═══════════════════════════════════════════════════════════════
    // LineageDeriver tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_lineage_deriver_new() {
        let mock = MockCaller::new();
        let _deriver = LineageDeriver::new(mock);
        // Just verify construction succeeds
    }

    #[tokio::test]
    async fn test_derive_device_seed_success() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "key": "ZGVyaXZlZC1zZWVkLWRhdGE=", // "derived-seed-data" in base64
                "method": "Blake3-KDF"
            }),
        )
        .await;
        mock.set_err("crypto.sign", "not available").await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .derive_device_seed("family-seed-b64", "family-01", "dev-001", "tower", None)
            .await;

        let lineage = result.expect("derivation should succeed");
        assert_eq!(lineage.device_id, "dev-001");
        assert_eq!(lineage.node_id, "tower");
        assert_eq!(lineage.family_id, "family-01");
        assert_eq!(lineage.generation, 1);
        assert_eq!(lineage.derived_seed, "ZGVyaXZlZC1zZWVkLWRhdGE=");
        assert_eq!(lineage.derivation_method, "Blake3-KDF");
        assert!(lineage.derived_at > 0);
        // Certificate is None because crypto.sign returned an error
        assert!(lineage.lineage_certificate.is_none());
    }

    #[tokio::test]
    async fn test_derive_device_seed_with_certificate() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "key": "c2VlZA==",
                "method": "HKDF"
            }),
        )
        .await;
        mock.set_ok(
            "crypto.sign",
            serde_json::json!({
                "signature": "sig-abc123"
            }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let lineage = deriver
            .derive_device_seed("family-b64", "fam-01", "dev-02", "pixel", None)
            .await
            .expect("should succeed");

        assert_eq!(lineage.lineage_certificate.as_deref(), Some("sig-abc123"));
    }

    #[tokio::test]
    async fn test_derive_device_seed_with_device_entropy() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "key": "ZGVyaXZlZA==",
                "method": "KDF"
            }),
        )
        .await;
        mock.set_ok(
            "genetic.mix_entropy",
            serde_json::json!({
                "mixed_seed": "bWl4ZWQ="
            }),
        )
        .await;
        mock.set_err("crypto.sign", "unavailable").await;

        let deriver = LineageDeriver::new(mock);
        let entropy = b"extra-device-entropy-data-32byte";
        let lineage = deriver
            .derive_device_seed("family", "fam", "dev", "node", Some(entropy))
            .await
            .expect("should succeed");

        assert_eq!(lineage.derived_seed, "bWl4ZWQ=");
    }

    #[tokio::test]
    async fn test_derive_device_seed_entropy_mix_fallback() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "key": "b3JpZ2luYWw=",
                "method": "KDF"
            }),
        )
        .await;
        // Mix entropy fails — should fallback to the original key
        mock.set_err("genetic.mix_entropy", "mix failed").await;
        mock.set_err("crypto.sign", "unavailable").await;

        let deriver = LineageDeriver::new(mock);
        let lineage = deriver
            .derive_device_seed("fam", "fam", "dev", "node", Some(b"entropy"))
            .await
            .expect("should succeed with fallback");

        assert_eq!(lineage.derived_seed, "b3JpZ2luYWw=");
    }

    #[tokio::test]
    async fn test_derive_device_seed_derivation_fails() {
        let mock = MockCaller::new();
        mock.set_err("genetic.derive_lineage_key", "service down")
            .await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .derive_device_seed("fam", "fam", "dev", "node", None)
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("derive lineage key"),
            "unexpected error: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_derive_device_seed_missing_key_in_response() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({ "not_key": "value" }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .derive_device_seed("fam", "fam", "dev", "node", None)
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Missing 'key'"), "unexpected: {}", err_msg);
    }

    // ═══════════════════════════════════════════════════════════════
    // save_lineage / load_lineage / has_lineage tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_save_and_load_lineage_roundtrip() {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let seed_path = tmp.path().join("device.lineage");

        let mock = MockCaller::new();
        let deriver = LineageDeriver::new(mock);

        let lineage = DeviceLineage {
            derived_seed: BASE64.encode(b"32-bytes-of-derived-seed-data!!"),
            ..sample_lineage()
        };

        deriver
            .save_lineage(&lineage, &seed_path)
            .expect("save should succeed");

        assert!(seed_path.exists(), "seed file should exist");
        assert!(
            seed_path.with_extension("json").exists(),
            "metadata file should exist"
        );

        let loaded =
            LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("load should succeed");
        assert_eq!(loaded.device_id, "device-123");
        assert_eq!(loaded.node_id, "tower");
        assert_eq!(loaded.family_id, "1894e909e454");
    }

    #[test]
    fn test_load_lineage_raw_seed_fallback() {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let seed_path = tmp.path().join("raw.lineage");

        // Write raw seed bytes with NO .json metadata
        std::fs::write(&seed_path, b"raw-seed-32-bytes-of-data!!!!!!").expect("write raw seed");

        let loaded =
            LineageDeriver::<MockCaller>::load_lineage(&seed_path).expect("load should succeed");

        assert_eq!(loaded.device_id, "unknown");
        assert_eq!(loaded.node_id, "unknown");
        assert_eq!(loaded.generation, 1);
        assert!(loaded.lineage_certificate.is_none());
        // derived_seed should be base64 of the raw bytes
        assert!(!loaded.derived_seed.is_empty());
    }

    #[test]
    fn test_load_lineage_nonexistent() {
        let result = LineageDeriver::<MockCaller>::load_lineage(Path::new("/nonexistent/file"));
        assert!(result.is_err());
    }

    #[test]
    fn test_has_lineage() {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let path = tmp.path().join("exists.lineage");

        assert!(!LineageDeriver::<MockCaller>::has_lineage(&path));

        std::fs::write(&path, b"data").expect("write");
        assert!(LineageDeriver::<MockCaller>::has_lineage(&path));
    }

    // ═══════════════════════════════════════════════════════════════
    // enroll_device tests
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_enroll_device_success() {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let family_seed_path = tmp.path().join(".family.seed");
        let lineage_path = tmp.path().join(".lineage.seed");

        // Write a fake family seed
        std::fs::write(&family_seed_path, b"fake-family-seed-32bytes!!!!!!")
            .expect("write family seed");

        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.derive_lineage_key",
            serde_json::json!({
                "key": BASE64.encode(b"derived-device-specific-key!!!!"),
                "method": "HKDF-SHA256"
            }),
        )
        .await;
        mock.set_err("crypto.sign", "not running").await;

        let deriver = LineageDeriver::new(mock);
        let enrollment = deriver
            .enroll_device(
                &family_seed_path,
                &lineage_path,
                "fam-01",
                "dev-01",
                "tower",
            )
            .await
            .expect("enrollment should succeed");

        assert_eq!(enrollment.lineage.device_id, "dev-01");
        assert_eq!(enrollment.lineage.node_id, "tower");
        assert_eq!(enrollment.seed_path, lineage_path);
        assert!(lineage_path.exists());
    }

    #[tokio::test]
    async fn test_enroll_device_missing_family_seed() {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        let family_path = tmp.path().join("nonexistent.seed");
        let lineage_path = tmp.path().join("lineage.seed");

        let mock = MockCaller::new();
        let deriver = LineageDeriver::new(mock);

        let result = deriver
            .enroll_device(&family_path, &lineage_path, "fam", "dev", "node")
            .await;

        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    // generate_lineage_proof / verify_lineage_proof tests
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_generate_lineage_proof_success() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.generate_lineage_proof",
            serde_json::json!({ "proof": "proof-data-abc" }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let lineage = sample_lineage();
        let proof = deriver
            .generate_lineage_proof(&lineage, "peer-family")
            .await
            .expect("proof should succeed");

        assert_eq!(proof, "proof-data-abc");
    }

    #[tokio::test]
    async fn test_generate_lineage_proof_missing_proof() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.generate_lineage_proof",
            serde_json::json!({ "no_proof": true }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .generate_lineage_proof(&sample_lineage(), "peer")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_lineage_proof_call_fails() {
        let mock = MockCaller::new();
        mock.set_err("genetic.generate_lineage_proof", "service down")
            .await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .generate_lineage_proof(&sample_lineage(), "peer")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_lineage_proof_valid() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.verify_lineage",
            serde_json::json!({ "valid": true }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let valid = deriver
            .verify_lineage_proof(&sample_lineage(), "peer-fam", "proof-data")
            .await
            .expect("verify should succeed");

        assert!(valid);
    }

    #[tokio::test]
    async fn test_verify_lineage_proof_invalid() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.verify_lineage",
            serde_json::json!({ "valid": false }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let valid = deriver
            .verify_lineage_proof(&sample_lineage(), "peer-fam", "bad-proof")
            .await
            .expect("verify should succeed");

        assert!(!valid);
    }

    #[tokio::test]
    async fn test_verify_lineage_proof_missing_field() {
        let mock = MockCaller::new();
        mock.set_ok(
            "genetic.verify_lineage",
            serde_json::json!({ "something_else": 42 }),
        )
        .await;

        let deriver = LineageDeriver::new(mock);
        let valid = deriver
            .verify_lineage_proof(&sample_lineage(), "peer-fam", "proof")
            .await
            .expect("verify should succeed (defaults to false)");

        assert!(!valid); // Missing "valid" field defaults to false
    }

    #[tokio::test]
    async fn test_verify_lineage_proof_call_fails() {
        let mock = MockCaller::new();
        mock.set_err("genetic.verify_lineage", "service down").await;

        let deriver = LineageDeriver::new(mock);
        let result = deriver
            .verify_lineage_proof(&sample_lineage(), "peer", "proof")
            .await;
        assert!(result.is_err());
    }
}
