// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Beacon Genetics Manager - Orchestration Layer
//!
//! Manages beacon genetics (address book, seeds, meetings) using capability.call.

#[cfg(test)]
mod tests;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{SporeError, SporeResult};

use super::capability::{CapabilityCaller, NeuralApiCapabilityCaller};
use super::types::{
    BeaconGeneticsManifest, BeaconId, MeetingRecord, MeetingRelationship, MeetingVisibility,
    SyncResult, current_timestamp,
};

/// Manages beacon genetics (address book, seeds, meetings)
pub struct BeaconGeneticsManager {
    /// Root path for beacon storage
    pub(crate) root_path: PathBuf,
    /// Capability caller (neuralAPI or mock)
    capability_caller: Box<dyn CapabilityCaller>,
    /// Cached manifest
    pub(crate) manifest: Option<BeaconGeneticsManifest>,
}

impl BeaconGeneticsManager {
    /// Create new manager with neuralAPI
    pub fn new(root_path: &Path) -> Self {
        let neural_socket = NeuralApiCapabilityCaller::default_socket();
        Self::with_capability_caller(
            root_path,
            Box::new(NeuralApiCapabilityCaller::new(&neural_socket)),
        )
    }

    /// Create with custom capability caller (for testing)
    pub fn with_capability_caller(
        root_path: &Path,
        capability_caller: Box<dyn CapabilityCaller>,
    ) -> Self {
        Self {
            root_path: root_path.to_path_buf(),
            capability_caller,
            manifest: None,
        }
    }

    /// Initialize beacon genetics (generate or load)
    pub async fn initialize(&mut self) -> SporeResult<()> {
        let manifest_path = self.root_path.join(".beacon.genetics.json");
        let seeds_dir = self.root_path.join(".beacon_seeds");

        std::fs::create_dir_all(&seeds_dir).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to create .beacon_seeds directory: {e}"
            )))
        })?;

        if manifest_path.exists() {
            info!("📖 Loading existing beacon genetics");
            self.manifest = Some(BeaconGeneticsManifest::load(&manifest_path)?);
        } else {
            info!("🧬 Generating new beacon genetics");
            let beacon_id = self.generate_beacon_seed().await?;
            let lineage_hint = self.get_lineage_hint().unwrap_or_else(|_| {
                warn!("⚠️ Could not get lineage hint, using default");
                "0000000000000000".to_string()
            });
            let manifest = BeaconGeneticsManifest::new(beacon_id, &lineage_hint);
            manifest.save(&manifest_path)?;
            self.manifest = Some(manifest);
            info!("✅ Beacon genetics initialized");
        }
        Ok(())
    }

    async fn generate_beacon_seed(&self) -> SporeResult<BeaconId> {
        let response = self
            .capability_caller
            .call("beacon.generate", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.generate failed: {e}")))?;

        let beacon_id = response
            .get("beacon_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to generate beacon seed".to_string())
            })?;

        let seed_path = self.root_path.join(".beacon.seed");
        if let Some(seed_hex) = response.get("seed_hex").and_then(|s| s.as_str()) {
            let seed_bytes = hex::decode(seed_hex)
                .map_err(|e| SporeError::DeserializationError(format!("Invalid hex: {e}")))?;
            std::fs::write(&seed_path, seed_bytes).map_err(|e| {
                SporeError::IoError(std::io::Error::other(format!(
                    "Failed to write beacon seed: {e}"
                )))
            })?;
        }
        Ok(BeaconId::from_hex(beacon_id))
    }

    pub(crate) fn get_lineage_hint(&self) -> SporeResult<String> {
        let family_seed_path = self.root_path.join(".family.seed");
        if family_seed_path.exists() {
            let seed_data = std::fs::read(&family_seed_path).map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read family seed: {e}"),
                ))
            })?;
            if seed_data.len() >= 8 {
                return Ok(hex::encode(&seed_data[0..8]));
            }
        }
        Err(SporeError::ValidationFailed(
            "No family seed found".to_string(),
        ))
    }

    /// Returns our beacon ID if beacon genetics has been initialized.
    pub fn our_beacon_id(&self) -> Option<&BeaconId> {
        self.manifest.as_ref().map(|m| &m.own_beacon_id)
    }

    /// Initiate a meeting with a peer: exchange encrypted seeds and store for future decryption.
    #[expect(
        clippy::too_many_lines,
        reason = "orchestrates multi-step beacon exchange with BearDog"
    )]
    pub async fn initiate_meeting(
        &mut self,
        peer_endpoint: &str,
        node_name: &str,
    ) -> SporeResult<BeaconId> {
        info!(
            "🤝 Initiating meeting with {} at {}",
            node_name, peer_endpoint
        );

        let our_id_response = self
            .capability_caller
            .call("beacon.get_id", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.get_id failed: {e}")))?;

        let our_beacon_id = our_id_response
            .get("beacon_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing our beacon_id".to_string()))?;

        let our_seed_response = self
            .capability_caller
            .call("beacon.get_seed", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.get_seed failed: {e}")))?;

        let our_seed_hex = our_seed_response
            .get("seed_hex")
            .and_then(|s| s.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing seed_hex".to_string()))?;

        let encrypted_response = self
            .capability_caller
            .call(
                "crypto.encrypt",
                serde_json::json!({
                    "plaintext": our_seed_hex,
                    "context": "beacon-exchange-v1"
                }),
            )
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("crypto.encrypt failed: {e}")))?;

        let our_seed_encrypted = encrypted_response
            .get("ciphertext")
            .and_then(|c| c.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing ciphertext".to_string()))?;

        let exchange_payload = serde_json::json!({
            "beacon_id": our_beacon_id,
            "encrypted_seed": our_seed_encrypted
        });

        let exchange_response = self
            .capability_caller
            .call(
                "network.beacon_exchange",
                serde_json::json!({
                    "endpoint": peer_endpoint,
                    "payload": exchange_payload
                }),
            )
            .await
            .map_err(|e| {
                SporeError::ValidationFailed(format!("network.beacon_exchange failed: {e}"))
            })?;

        let peer_beacon_id = exchange_response
            .get("peer_beacon_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing peer_beacon_id".to_string()))?;

        let peer_seed_encrypted = exchange_response
            .get("peer_encrypted_seed")
            .and_then(|s| s.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Missing peer_encrypted_seed".to_string())
            })?;

        let decrypt_response = self
            .capability_caller
            .call(
                "crypto.decrypt",
                serde_json::json!({
                    "ciphertext": peer_seed_encrypted,
                    "context": "beacon-exchange-v1"
                }),
            )
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("crypto.decrypt failed: {e}")))?;

        let peer_seed_hex = decrypt_response
            .get("plaintext")
            .and_then(|p| p.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to decrypt peer seed".to_string())
            })?;

        self.store_met_seed_local(&BeaconId::from_hex(peer_beacon_id), peer_seed_hex)
            .await?;

        let now = current_timestamp();
        let beacon_id_short = if peer_beacon_id.len() >= 8 {
            &peer_beacon_id[..8]
        } else {
            peer_beacon_id
        };

        let record = MeetingRecord {
            node_name: node_name.to_string(),
            first_met: now,
            last_seen: now,
            endpoints: vec![peer_endpoint.to_string()],
            capabilities_hint: vec![],
            notes: "Met via direct exchange".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: format!("{beacon_id_short}.seed"),
        };

        if let Some(ref mut manifest) = self.manifest {
            manifest.add_meeting(BeaconId::from_hex(peer_beacon_id), record);
            manifest.shared_with.insert(peer_beacon_id.to_string());
            self.save_manifest()?;
        }

        info!("✅ Meeting complete with {}", node_name);
        Ok(BeaconId::from_hex(peer_beacon_id))
    }

    async fn store_met_seed_local(&self, beacon_id: &BeaconId, seed_hex: &str) -> SporeResult<()> {
        let seeds_dir = self.root_path.join(".beacon_seeds");
        let seed_file = seeds_dir.join(format!("{}.seed", beacon_id.short()));

        let encrypted_response = self
            .capability_caller
            .call(
                "crypto.encrypt_with_lineage",
                serde_json::json!({
                    "plaintext": seed_hex,
                    "context": "beacon-seed-storage-v1"
                }),
            )
            .await
            .map_err(|e| {
                SporeError::ValidationFailed(format!("crypto.encrypt_with_lineage failed: {e}"))
            })?;

        let encrypted = encrypted_response
            .get("ciphertext")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to encrypt seed for storage".to_string())
            })?;

        std::fs::write(&seed_file, encrypted).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write met seed: {e}"
            )))
        })?;
        debug!("Stored met seed: {}", seed_file.display());
        Ok(())
    }

    async fn load_met_seed(&self, beacon_id: &BeaconId) -> SporeResult<String> {
        let seeds_dir = self.root_path.join(".beacon_seeds");
        let seed_file = seeds_dir.join(format!("{}.seed", beacon_id.short()));

        let encrypted = std::fs::read_to_string(&seed_file).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Met seed not found: {e}"),
            ))
        })?;

        let decrypt_response = self
            .capability_caller
            .call(
                "crypto.decrypt_with_lineage",
                serde_json::json!({
                    "ciphertext": encrypted,
                    "context": "beacon-seed-storage-v1"
                }),
            )
            .await
            .map_err(|e| {
                SporeError::ValidationFailed(format!("crypto.decrypt_with_lineage failed: {e}"))
            })?;

        decrypt_response
            .get("plaintext")
            .and_then(|p| p.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| SporeError::ValidationFailed("Failed to decrypt met seed".to_string()))
    }

    /// Attempt to decrypt an encrypted beacon using seeds from previously met peers.
    pub async fn try_decrypt_with_met_seeds(
        &self,
        encrypted_beacon: &[u8],
    ) -> SporeResult<Option<(serde_json::Value, BeaconId)>> {
        let manifest = self.manifest.as_ref().ok_or_else(|| {
            SporeError::ValidationFailed("Beacon genetics not initialized".to_string())
        })?;

        for beacon_id_str in manifest.meetings.keys() {
            let beacon_id = BeaconId::from_hex(beacon_id_str);
            let Ok(seed_hex) = self.load_met_seed(&beacon_id).await else {
                continue;
            };

            let decrypt_response = self
                .capability_caller
                .call(
                    "beacon.try_decrypt",
                    serde_json::json!({
                        "ciphertext": BASE64.encode(encrypted_beacon),
                        "seed_hex": seed_hex
                    }),
                )
                .await;

            if let Ok(result) = decrypt_response
                && result.get("decrypted").and_then(serde_json::Value::as_bool) == Some(true)
                && let Some(plaintext) = result.get("payload")
            {
                info!("✅ Beacon decrypted - met peer: {}", beacon_id.short());
                return Ok(Some((plaintext.clone(), beacon_id)));
            }
        }
        Ok(None)
    }

    /// Sync meeting records and shared state with a lineage peer.
    pub async fn sync_with_lineage_peer(
        &mut self,
        peer_manifest: &BeaconGeneticsManifest,
    ) -> SporeResult<SyncResult> {
        info!("🔄 Syncing beacon genetics with lineage peer");

        let manifest = self.manifest.as_mut().ok_or_else(|| {
            SporeError::ValidationFailed("Beacon genetics not initialized".to_string())
        })?;

        if manifest.lineage_hint != peer_manifest.lineage_hint {
            return Err(SporeError::ValidationFailed(
                "Cannot sync with different lineage".to_string(),
            ));
        }

        let mut added = 0;
        let mut updated = 0;

        for (beacon_id, peer_record) in &peer_manifest.meetings {
            if let Some(local_record) = manifest.meetings.get_mut(beacon_id) {
                if peer_record.last_seen > local_record.last_seen {
                    local_record.last_seen = peer_record.last_seen;
                    updated += 1;
                }
                for ep in &peer_record.endpoints {
                    if !local_record.endpoints.contains(ep) {
                        local_record.endpoints.push(ep.clone());
                    }
                }
            } else {
                manifest
                    .meetings
                    .insert(beacon_id.clone(), peer_record.clone());
                added += 1;
            }
        }

        manifest
            .shared_with
            .extend(peer_manifest.shared_with.iter().cloned());

        manifest.sync_token = Uuid::new_v4().to_string();
        manifest.last_sync = current_timestamp();

        self.save_manifest()?;

        info!("✅ Sync complete: {} added, {} updated", added, updated);
        Ok(SyncResult { added, updated })
    }

    pub(crate) fn save_manifest(&self) -> SporeResult<()> {
        if let Some(ref manifest) = self.manifest {
            let path = self.root_path.join(".beacon.genetics.json");
            manifest.save(&path)?;
        }
        Ok(())
    }

    /// List all meetings (peers we've exchanged seeds with).
    pub fn list_meetings(&self) -> Vec<(BeaconId, &MeetingRecord)> {
        self.manifest
            .as_ref()
            .map(|m| {
                m.meetings
                    .iter()
                    .map(|(k, v)| (BeaconId::from_hex(k), v))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Override manifest (test-only: inject manifest for unit tests).
    #[cfg(test)]
    pub fn set_manifest(&mut self, manifest: BeaconGeneticsManifest) {
        self.manifest = Some(manifest);
    }
}
