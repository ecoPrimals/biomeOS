// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Device lineage deriver - orchestrates derivation via `BearDog`

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::path::Path;
use tracing::{debug, info, warn};

use super::types::{DeviceLineage, EnrollmentResult};
use crate::error::{SporeError, SporeResult};

use crate::beacon_genetics::capability::CapabilityCaller;

/// Device lineage deriver - orchestrates derivation via `BearDog`
pub struct LineageDeriver<C: CapabilityCaller> {
    caller: C,
}

impl<C: CapabilityCaller> LineageDeriver<C> {
    /// Create new lineage deriver with capability caller
    pub const fn new(caller: C) -> Self {
        Self { caller }
    }

    /// Derive a unique device seed from the family root
    pub async fn derive_device_seed(
        &self,
        family_seed: &str,
        family_id: &str,
        device_id: &str,
        node_id: &str,
        device_entropy: Option<&[u8]>,
    ) -> SporeResult<DeviceLineage> {
        info!("🧬 Deriving device lineage for {} ({})", node_id, device_id);

        let context = format!("ecoPrimals-device-lineage-v1:{family_id}:{device_id}");

        let params = serde_json::json!({
            "our_family_id": format!("{}-{}", family_id, device_id),
            "peer_family_id": family_id,
            "context": context,
            "lineage_seed": family_seed,
        });

        debug!("Calling genetic.derive_lineage_key with device-specific params");

        let result = self
            .caller
            .call("genetic.derive_lineage_key", params)
            .await
            .map_err(|e| SporeError::SystemError(format!("Failed to derive lineage key: {e}")))?;

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

    async fn sign_lineage_certificate(
        &self,
        device_id: &str,
        node_id: &str,
        family_id: &str,
        derived_seed: &str,
        derived_at: u64,
    ) -> Option<String> {
        let sign_data =
            format!("lineage:{device_id}:{node_id}:{family_id}:{derived_seed}:{derived_at}");

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
    pub async fn enroll_device(
        &self,
        family_seed_path: &Path,
        lineage_seed_path: &Path,
        family_id: &str,
        device_id: &str,
        node_id: &str,
    ) -> SporeResult<EnrollmentResult> {
        info!("📝 Enrolling device {} as {}", device_id, node_id);

        let family_seed_bytes = std::fs::read(family_seed_path).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read family seed: {e}"),
            ))
        })?;

        let family_seed = BASE64.encode(&family_seed_bytes);

        let lineage = self
            .derive_device_seed(&family_seed, family_id, device_id, node_id, None)
            .await?;

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
    pub fn save_lineage(&self, lineage: &DeviceLineage, path: &Path) -> SporeResult<()> {
        let seed_bytes = BASE64
            .decode(&lineage.derived_seed)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid base64 seed: {e}")))?;

        std::fs::write(path, &seed_bytes).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write lineage seed: {e}"
            )))
        })?;

        let metadata_path = path.with_extension("json");
        let metadata = serde_json::to_string_pretty(lineage)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        std::fs::write(&metadata_path, metadata).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write lineage metadata: {e}"
            )))
        })?;

        debug!("Lineage metadata saved to {}", metadata_path.display());

        Ok(())
    }

    /// Load existing device lineage from file.
    ///
    /// Delegates to the free function [`load_lineage`].
    pub fn load_lineage(lineage_path: &Path) -> SporeResult<DeviceLineage> {
        load_lineage(lineage_path)
    }

    /// Check if device has existing lineage.
    ///
    /// Delegates to the free function [`has_lineage`].
    #[must_use]
    pub fn has_lineage(lineage_path: &Path) -> bool {
        has_lineage(lineage_path)
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
            .map_err(|e| SporeError::SystemError(format!("Failed to generate proof: {e}")))?;

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
            .map_err(|e| SporeError::SystemError(format!("Failed to verify proof: {e}")))?;

        Ok(result
            .get("valid")
            .and_then(|v: &serde_json::Value| v.as_bool())
            .unwrap_or(false))
    }
}

/// Load existing device lineage from file (caller-agnostic, no generic needed).
///
/// Prefer this over `LineageDeriver::<C>::load_lineage()` when you don't have
/// a `LineageDeriver` instance, to avoid a phantom type parameter.
pub fn load_lineage(lineage_path: &Path) -> SporeResult<DeviceLineage> {
    let metadata_path = lineage_path.with_extension("json");

    if metadata_path.exists() {
        let contents = std::fs::read_to_string(&metadata_path).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read lineage metadata: {e}"),
            ))
        })?;

        serde_json::from_str(&contents)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid JSON: {e}")))
    } else {
        let seed_bytes = std::fs::read(lineage_path).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read lineage seed: {e}"),
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

/// Check if device has existing lineage (caller-agnostic).
#[must_use]
pub fn has_lineage(lineage_path: &Path) -> bool {
    lineage_path.exists()
}
