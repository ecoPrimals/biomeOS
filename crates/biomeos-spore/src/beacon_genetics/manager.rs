//! Beacon Genetics Manager - Orchestration Layer
//!
//! Manages beacon genetics (address book, seeds, meetings) using capability.call.
//!
//! ## Architectural Principle: Primals Have Self-Knowledge Only
//!
//! **biomeOS orchestrates. Primals execute primitives.**
//!
//! This module handles ecosystem-level concepts:
//! - "Meetings" (social graph of beacon exchanges)
//! - "Address book" (collection of met beacon seeds)
//! - "Beacon vs Lineage" (discovery vs permissions)
//! - Orchestration of multi-step workflows
//!
//! Primals provide primitives via capability.call:
//! - BearDog: `beacon.encrypt`, `beacon.decrypt`, `beacon.generate`
//! - Nestgate: `storage.write`, `storage.read`
//! - Songbird: `network.send`, `network.receive`

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{SporeError, SporeResult};

use super::capability::{CapabilityCaller, NeuralApiCapabilityCaller};
use super::types::{
    current_timestamp, BeaconGeneticsManifest, BeaconId, MeetingRecord, MeetingRelationship,
    MeetingVisibility, SyncResult,
};

/// Manages beacon genetics (address book, seeds, meetings)
///
/// Uses capability.call to orchestrate primals:
/// - BearDog for crypto (beacon.encrypt, beacon.decrypt)
/// - Nestgate for storage (storage.write, storage.read)
/// - Songbird for network (network.send, network.receive)
pub struct BeaconGeneticsManager {
    /// Root path for beacon storage
    root_path: PathBuf,

    /// Capability caller (neuralAPI or mock)
    capability_caller: Box<dyn CapabilityCaller>,

    /// Cached manifest
    manifest: Option<BeaconGeneticsManifest>,
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

        // Ensure seeds directory exists
        std::fs::create_dir_all(&seeds_dir).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to create .beacon_seeds directory: {}",
                e
            )))
        })?;

        // Check if manifest exists
        if manifest_path.exists() {
            info!("📖 Loading existing beacon genetics");
            self.manifest = Some(BeaconGeneticsManifest::load(&manifest_path)?);
        } else {
            info!("🧬 Generating new beacon genetics");

            // Generate beacon seed via capability.call("beacon.generate")
            let beacon_id = self.generate_beacon_seed().await?;

            // Get lineage hint from family seed (local file operation)
            let lineage_hint = self.get_lineage_hint().unwrap_or_else(|_| {
                warn!("⚠️ Could not get lineage hint, using default");
                "0000000000000000".to_string()
            });

            // Create manifest
            let manifest = BeaconGeneticsManifest::new(beacon_id, &lineage_hint);
            manifest.save(&manifest_path)?;

            self.manifest = Some(manifest);
            info!("✅ Beacon genetics initialized");
        }

        Ok(())
    }

    /// Generate new beacon seed via capability.call
    ///
    /// Uses semantic capability "beacon.generate" which gets translated
    /// to BearDog's actual method by the CapabilityTranslationRegistry.
    async fn generate_beacon_seed(&self) -> SporeResult<BeaconId> {
        // Call semantic capability - biomeOS doesn't know this goes to BearDog
        let response = self
            .capability_caller
            .call("beacon.generate", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.generate failed: {}", e)))?;

        let beacon_id = response
            .get("beacon_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to generate beacon seed".to_string())
            })?;

        // Save beacon seed to local file (filesystem is local, not a capability)
        let seed_path = self.root_path.join(".beacon.seed");
        if let Some(seed_hex) = response.get("seed_hex").and_then(|s| s.as_str()) {
            let seed_bytes = hex::decode(seed_hex)
                .map_err(|e| SporeError::DeserializationError(format!("Invalid hex: {}", e)))?;
            std::fs::write(&seed_path, seed_bytes).map_err(|e| {
                SporeError::IoError(std::io::Error::other(format!(
                    "Failed to write beacon seed: {}",
                    e
                )))
            })?;
        }

        Ok(BeaconId::from_hex(beacon_id))
    }

    /// Get lineage hint from family seed (local file operation)
    ///
    /// This is a LOCAL operation - reading the family seed file.
    /// It doesn't need capability.call because it's not calling a primal.
    fn get_lineage_hint(&self) -> SporeResult<String> {
        let family_seed_path = self.root_path.join(".family.seed");
        if family_seed_path.exists() {
            let seed_data = std::fs::read(&family_seed_path).map_err(|e| {
                SporeError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read family seed: {}", e),
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

    /// Get our own beacon ID
    pub fn our_beacon_id(&self) -> Option<&BeaconId> {
        self.manifest.as_ref().map(|m| &m.own_beacon_id)
    }

    /// Initiate a meeting with another node
    ///
    /// This orchestrates a 6-step protocol using capability.call:
    /// 1. Get our beacon ID
    /// 2. Get our seed for exchange
    /// 3. Encrypt our seed for transport
    /// 4. Exchange via network
    /// 5. Decrypt peer's seed
    /// 6. Store locally
    pub async fn initiate_meeting(
        &mut self,
        peer_endpoint: &str,
        node_name: &str,
    ) -> SporeResult<BeaconId> {
        info!(
            "🤝 Initiating meeting with {} at {}",
            node_name, peer_endpoint
        );

        // Step 1: Get our beacon ID (capability.call to BearDog primitive)
        let our_id_response = self
            .capability_caller
            .call("beacon.get_id", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.get_id failed: {}", e)))?;

        let our_beacon_id = our_id_response
            .get("beacon_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing our beacon_id".to_string()))?;

        // Step 2: Get our seed for exchange (capability.call to BearDog primitive)
        let our_seed_response = self
            .capability_caller
            .call("beacon.get_seed", serde_json::json!({}))
            .await
            .map_err(|e| SporeError::ValidationFailed(format!("beacon.get_seed failed: {}", e)))?;

        let our_seed_hex = our_seed_response
            .get("seed_hex")
            .and_then(|s| s.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing seed_hex".to_string()))?;

        // Step 3: Encrypt our seed for transport (capability.call to BearDog primitive)
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
            .map_err(|e| SporeError::ValidationFailed(format!("crypto.encrypt failed: {}", e)))?;

        let our_seed_encrypted = encrypted_response
            .get("ciphertext")
            .and_then(|c| c.as_str())
            .ok_or_else(|| SporeError::ValidationFailed("Missing ciphertext".to_string()))?;

        // Step 4: Exchange via network (capability.call to Songbird primitive)
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
                SporeError::ValidationFailed(format!("network.beacon_exchange failed: {}", e))
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

        // Step 5: Decrypt peer's seed (capability.call to BearDog primitive)
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
            .map_err(|e| SporeError::ValidationFailed(format!("crypto.decrypt failed: {}", e)))?;

        let peer_seed_hex = decrypt_response
            .get("plaintext")
            .and_then(|p| p.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to decrypt peer seed".to_string())
            })?;

        // Step 6: Store locally (filesystem - NOT a capability call)
        self.store_met_seed_local(&BeaconId::from_hex(peer_beacon_id), peer_seed_hex)
            .await?;

        // Step 7: Update manifest
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
            seed_file: format!("{}.seed", beacon_id_short),
        };

        if let Some(ref mut manifest) = self.manifest {
            manifest.add_meeting(BeaconId::from_hex(peer_beacon_id), record);
            manifest.shared_with.insert(peer_beacon_id.to_string());
            self.save_manifest()?;
        }

        info!("✅ Meeting complete with {}", node_name);
        Ok(BeaconId::from_hex(peer_beacon_id))
    }

    /// Store a met beacon seed locally (encrypted at rest)
    ///
    /// This encrypts the seed using a capability call, then writes to local filesystem.
    /// The filesystem write is LOCAL - not a capability (we don't need a primal for that).
    async fn store_met_seed_local(&self, beacon_id: &BeaconId, seed_hex: &str) -> SporeResult<()> {
        let seeds_dir = self.root_path.join(".beacon_seeds");
        let seed_file = seeds_dir.join(format!("{}.seed", beacon_id.short()));

        // Encrypt with lineage key (capability.call to BearDog primitive)
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
                SporeError::ValidationFailed(format!("crypto.encrypt_with_lineage failed: {}", e))
            })?;

        let encrypted = encrypted_response
            .get("ciphertext")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                SporeError::ValidationFailed("Failed to encrypt seed for storage".to_string())
            })?;

        // Write to local filesystem (NOT a capability call)
        std::fs::write(&seed_file, encrypted).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write met seed: {}",
                e
            )))
        })?;

        debug!("Stored met seed: {}", seed_file.display());
        Ok(())
    }

    /// Load a met beacon seed (decrypt from storage)
    ///
    /// Reads from local filesystem, then decrypts via capability.call.
    async fn load_met_seed(&self, beacon_id: &BeaconId) -> SporeResult<String> {
        let seeds_dir = self.root_path.join(".beacon_seeds");
        let seed_file = seeds_dir.join(format!("{}.seed", beacon_id.short()));

        // Read from local filesystem (NOT a capability call)
        let encrypted = std::fs::read_to_string(&seed_file).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Met seed not found: {}", e),
            ))
        })?;

        // Decrypt with lineage key (capability.call to BearDog primitive)
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
                SporeError::ValidationFailed(format!("crypto.decrypt_with_lineage failed: {}", e))
            })?;

        decrypt_response
            .get("plaintext")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SporeError::ValidationFailed("Failed to decrypt met seed".to_string()))
    }

    /// Try to decrypt a beacon using all met seeds
    ///
    /// This is ecosystem-level logic - we iterate through our address book
    /// and try each seed using BearDog's primitive decrypt capability.
    pub async fn try_decrypt_with_met_seeds(
        &self,
        encrypted_beacon: &[u8],
    ) -> SporeResult<Option<(serde_json::Value, BeaconId)>> {
        let manifest = self.manifest.as_ref().ok_or_else(|| {
            SporeError::ValidationFailed("Beacon genetics not initialized".to_string())
        })?;

        // Try each met seed (ecosystem-level iteration)
        for beacon_id_str in manifest.meetings.keys() {
            let beacon_id = BeaconId::from_hex(beacon_id_str);

            // Load the met seed (includes decrypt via capability.call)
            let seed_hex = match self.load_met_seed(&beacon_id).await {
                Ok(s) => s,
                Err(_) => continue, // Skip if can't load
            };

            // Try to decrypt with this seed (capability.call to BearDog primitive)
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

            // Check if decryption succeeded
            if let Ok(result) = decrypt_response {
                if let Some(true) = result.get("decrypted").and_then(|d| d.as_bool()) {
                    if let Some(plaintext) = result.get("payload") {
                        info!("✅ Beacon decrypted - met peer: {}", beacon_id.short());
                        return Ok(Some((plaintext.clone(), beacon_id)));
                    }
                }
            }
        }

        // Couldn't decrypt with any met seed (TRUE Dark Forest - silent failure)
        Ok(None)
    }

    /// Sync address books with a same-lineage device
    pub async fn sync_with_lineage_peer(
        &mut self,
        peer_manifest: &BeaconGeneticsManifest,
    ) -> SporeResult<SyncResult> {
        info!("🔄 Syncing beacon genetics with lineage peer");

        let manifest = self.manifest.as_mut().ok_or_else(|| {
            SporeError::ValidationFailed("Beacon genetics not initialized".to_string())
        })?;

        // Verify same lineage
        if manifest.lineage_hint != peer_manifest.lineage_hint {
            return Err(SporeError::ValidationFailed(
                "Cannot sync with different lineage".to_string(),
            ));
        }

        let mut added = 0;
        let mut updated = 0;

        // Merge meetings
        for (beacon_id, peer_record) in &peer_manifest.meetings {
            if let Some(local_record) = manifest.meetings.get_mut(beacon_id) {
                // Update existing
                if peer_record.last_seen > local_record.last_seen {
                    local_record.last_seen = peer_record.last_seen;
                    updated += 1;
                }

                // Union endpoints
                for ep in &peer_record.endpoints {
                    if !local_record.endpoints.contains(ep) {
                        local_record.endpoints.push(ep.clone());
                    }
                }
            } else {
                // Add new
                manifest
                    .meetings
                    .insert(beacon_id.clone(), peer_record.clone());
                added += 1;
            }
        }

        // Union shared_with
        manifest
            .shared_with
            .extend(peer_manifest.shared_with.iter().cloned());

        // Update sync token
        manifest.sync_token = Uuid::new_v4().to_string();
        manifest.last_sync = current_timestamp();

        self.save_manifest()?;

        info!("✅ Sync complete: {} added, {} updated", added, updated);
        Ok(SyncResult { added, updated })
    }

    /// Save manifest to file
    fn save_manifest(&self) -> SporeResult<()> {
        if let Some(ref manifest) = self.manifest {
            let path = self.root_path.join(".beacon.genetics.json");
            manifest.save(&path)?;
        }
        Ok(())
    }

    /// List all known meetings
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

    /// Set manifest directly (for testing)
    #[cfg(test)]
    pub fn set_manifest(&mut self, manifest: BeaconGeneticsManifest) {
        self.manifest = Some(manifest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock capability caller for testing
    struct MockCapabilityCaller {
        responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    impl MockCapabilityCaller {
        fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Set mock response for a capability (used by tests)
        async fn set_response(&self, capability: &str, response: serde_json::Value) {
            self.responses
                .lock()
                .await
                .insert(capability.to_string(), response);
        }
    }

    #[async_trait::async_trait]
    impl CapabilityCaller for MockCapabilityCaller {
        async fn call(
            &self,
            capability: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            let responses = self.responses.lock().await;
            responses
                .get(capability)
                .cloned()
                .ok_or_else(|| format!("No mock response for {}", capability))
        }
    }

    #[tokio::test]
    async fn test_sync_with_lineage_peer_same_lineage() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        // Create initial manifest
        let mut local_manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "same_lineage");

        // Add a local meeting
        local_manifest.add_meeting(
            BeaconId::from_hex("peer_a"),
            MeetingRecord {
                node_name: "peer-a".to_string(),
                first_met: 1000,
                last_seen: 1000,
                endpoints: vec!["192.168.1.1:9900".to_string()],
                capabilities_hint: vec![],
                notes: "Local meeting".to_string(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "peer_a.seed".to_string(),
            },
        );

        manager.set_manifest(local_manifest);

        // Create remote manifest (same lineage)
        let mut remote_manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "same_lineage");

        // Add a remote meeting
        remote_manifest.add_meeting(
            BeaconId::from_hex("peer_b"),
            MeetingRecord {
                node_name: "peer-b".to_string(),
                first_met: 2000,
                last_seen: 2000,
                endpoints: vec!["192.168.1.2:9900".to_string()],
                capabilities_hint: vec!["compute".to_string()],
                notes: "Remote meeting".to_string(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "peer_b.seed".to_string(),
            },
        );

        // Sync
        let result = manager
            .sync_with_lineage_peer(&remote_manifest)
            .await
            .unwrap();

        assert_eq!(result.added, 1);
        assert_eq!(result.updated, 0);

        // Verify merged manifest
        let manifest = manager.manifest.as_ref().unwrap();
        assert_eq!(manifest.meetings.len(), 2);
        assert!(manifest.meetings.contains_key("peer_a"));
        assert!(manifest.meetings.contains_key("peer_b"));
    }

    #[tokio::test]
    async fn test_sync_with_different_lineage_fails() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        // Create local manifest
        let local_manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "lineage_a");
        manager.set_manifest(local_manifest);

        // Create remote manifest with DIFFERENT lineage
        let remote_manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("remote456"), "lineage_b");

        // Sync should fail
        let result = manager.sync_with_lineage_peer(&remote_manifest).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("different lineage"));
    }

    #[test]
    fn test_list_meetings_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let meetings = manager.list_meetings();
        assert!(meetings.is_empty());
    }

    #[test]
    fn test_list_meetings_with_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let mut manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("local123"), "lineage_hint");

        manifest.add_meeting(
            BeaconId::from_hex("peer123"),
            MeetingRecord {
                node_name: "peer-node".to_string(),
                first_met: 1234567890,
                last_seen: 1234567890,
                endpoints: vec![],
                capabilities_hint: vec![],
                notes: "Test".to_string(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "peer123.seed".to_string(),
            },
        );

        manager.set_manifest(manifest);

        let meetings = manager.list_meetings();
        assert_eq!(meetings.len(), 1);
        assert_eq!(meetings[0].0 .0, "peer123");
        assert_eq!(meetings[0].1.node_name, "peer-node");
    }

    #[test]
    fn test_our_beacon_id() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        // Before initialization
        assert!(manager.our_beacon_id().is_none());

        // After setting manifest
        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("our_beacon_123"), "lineage");
        manager.set_manifest(manifest);

        let id = manager.our_beacon_id().expect("should have ID");
        assert_eq!(id.0, "our_beacon_123");
    }

    // ═══════════════════════════════════════════════════════════════
    // initialize tests
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_initialize_loads_existing_manifest() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // Pre-create a manifest file
        let manifest =
            BeaconGeneticsManifest::new(BeaconId::from_hex("existing-beacon"), "lineage-hint");
        let manifest_path = temp_dir.path().join(".beacon.genetics.json");
        manifest.save(&manifest_path).expect("save manifest");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        manager.initialize().await.expect("init should succeed");

        let id = manager.our_beacon_id().expect("should have loaded ID");
        assert_eq!(id.0, "existing-beacon");
    }

    #[tokio::test]
    async fn test_initialize_generates_new_manifest() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        mock_caller
            .set_response(
                "beacon.generate",
                serde_json::json!({
                    "beacon_id": "new-beacon-456",
                    "seed_hex": "deadbeefcafebabe"
                }),
            )
            .await;

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        manager.initialize().await.expect("init should succeed");

        let id = manager.our_beacon_id().expect("should have new ID");
        assert_eq!(id.0, "new-beacon-456");

        // Manifest file should have been saved
        assert!(temp_dir.path().join(".beacon.genetics.json").exists());
        // Beacon seed should have been saved
        assert!(temp_dir.path().join(".beacon.seed").exists());
        // Seeds directory should have been created
        assert!(temp_dir.path().join(".beacon_seeds").is_dir());
    }

    #[tokio::test]
    async fn test_initialize_generate_fails() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        // No mock response set for "beacon.generate" — will fail

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let result = manager.initialize().await;
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    // get_lineage_hint tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_get_lineage_hint_with_family_seed() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // Write a family seed (at least 8 bytes)
        let seed_data = b"abcdefghijklmnop"; // 16 bytes
        std::fs::write(temp_dir.path().join(".family.seed"), seed_data).expect("write seed");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let hint = manager.get_lineage_hint().expect("should succeed");
        // First 8 bytes hex-encoded
        assert_eq!(hint, hex::encode(&seed_data[0..8]));
    }

    #[test]
    fn test_get_lineage_hint_short_seed() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // Write a seed that's too short (< 8 bytes)
        std::fs::write(temp_dir.path().join(".family.seed"), b"short").expect("write seed");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let result = manager.get_lineage_hint();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_lineage_hint_no_seed_file() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        let mock_caller = Box::new(MockCapabilityCaller::new());
        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let result = manager.get_lineage_hint();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    // save_manifest tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_save_manifest_no_manifest() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());
        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        // Should be a no-op when there's no manifest
        manager.save_manifest().expect("no-op save should succeed");
        assert!(!temp_dir.path().join(".beacon.genetics.json").exists());
    }

    #[test]
    fn test_save_manifest_with_manifest() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        manager.set_manifest(BeaconGeneticsManifest::new(
            BeaconId::from_hex("save-test"),
            "hint",
        ));

        manager.save_manifest().expect("save should succeed");
        assert!(temp_dir.path().join(".beacon.genetics.json").exists());
    }

    // ═══════════════════════════════════════════════════════════════
    // sync_with_lineage_peer - additional edge cases
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_sync_updates_existing_meetings() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let mut local = BeaconGeneticsManifest::new(BeaconId::from_hex("local"), "same_hint");
        local.add_meeting(
            BeaconId::from_hex("shared"),
            MeetingRecord {
                node_name: "shared-node".into(),
                first_met: 100,
                last_seen: 200,
                endpoints: vec!["10.0.0.1:9000".into()],
                capabilities_hint: vec![],
                notes: "local note".into(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "shared.seed".into(),
            },
        );
        manager.set_manifest(local);

        // Remote has same meeting but newer last_seen and new endpoint
        let mut remote = BeaconGeneticsManifest::new(BeaconId::from_hex("remote"), "same_hint");
        remote.add_meeting(
            BeaconId::from_hex("shared"),
            MeetingRecord {
                node_name: "shared-node".into(),
                first_met: 100,
                last_seen: 500, // newer
                endpoints: vec!["10.0.0.1:9000".into(), "10.0.0.2:9000".into()],
                capabilities_hint: vec![],
                notes: "remote note".into(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "shared.seed".into(),
            },
        );

        let result = manager.sync_with_lineage_peer(&remote).await.expect("sync");
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 1);

        let manifest = manager.manifest.as_ref().expect("manifest");
        let shared_record = manifest.meetings.get("shared").expect("shared meeting");
        assert_eq!(shared_record.last_seen, 500);
        assert_eq!(shared_record.endpoints.len(), 2);
    }

    #[tokio::test]
    async fn test_sync_not_initialized() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);
        // Don't set manifest

        let remote = BeaconGeneticsManifest::new(BeaconId::from_hex("remote"), "hint");

        let result = manager.sync_with_lineage_peer(&remote).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_sync_merges_shared_with() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let mut local = BeaconGeneticsManifest::new(BeaconId::from_hex("local"), "same");
        local.shared_with.insert("peer-a".into());
        manager.set_manifest(local);

        let mut remote = BeaconGeneticsManifest::new(BeaconId::from_hex("remote"), "same");
        remote.shared_with.insert("peer-b".into());

        let result = manager.sync_with_lineage_peer(&remote).await.expect("sync");
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 0);

        let manifest = manager.manifest.as_ref().expect("manifest");
        assert!(manifest.shared_with.contains("peer-a"));
        assert!(manifest.shared_with.contains("peer-b"));
    }

    // ═══════════════════════════════════════════════════════════════
    // try_decrypt_with_met_seeds tests
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_try_decrypt_not_initialized() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let result = manager.try_decrypt_with_met_seeds(b"encrypted-data").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_try_decrypt_no_meetings() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock_caller = Box::new(MockCapabilityCaller::new());

        let mut manager =
            BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock_caller);

        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("local"), "hint");
        manager.set_manifest(manifest);

        let result = manager
            .try_decrypt_with_met_seeds(b"encrypted-data")
            .await
            .expect("should succeed");

        assert!(result.is_none(), "no meetings, so nothing to decrypt with");
    }

    // ═══════════════════════════════════════════════════════════════
    // manager construction tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_new_manager() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let manager = BeaconGeneticsManager::new(temp_dir.path());
        assert!(manager.our_beacon_id().is_none());
        assert!(manager.list_meetings().is_empty());
    }

    #[test]
    fn test_with_capability_caller() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let mock = Box::new(MockCapabilityCaller::new());
        let manager = BeaconGeneticsManager::with_capability_caller(temp_dir.path(), mock);

        assert!(manager.our_beacon_id().is_none());
        assert_eq!(manager.root_path, temp_dir.path());
    }
}
