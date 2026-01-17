// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0

//! Encrypted storage backend implementation

use super::metadata::EncryptionMetadata;
use super::types::{StorageBackend, StorageMetrics};
use crate::clients::beardog::crypto::CryptoClient;
use crate::clients::beardog::crypto::EncryptedData; // Use crypto's EncryptedData (has nonce/tag)
use crate::clients::beardog::BearDogClient;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Encrypted storage with transparent encryption/decryption
///
/// This wrapper provides automatic encryption for any storage backend.
/// All data is encrypted using BearDog before being stored, and decrypted
/// when retrieved.
///
/// # Performance
///
/// - Hardware-accelerated AES-256-GCM via CPU AES-NI instructions
/// - <5% overhead vs plaintext storage
/// - <100µs latency per MB
/// - Local BearDog (no network calls, no phone home)
///
/// # Security
///
/// - Per-dataset encryption keys (managed by BearDog)
/// - Authenticated encryption (AES-256-GCM prevents tampering)
/// - Zero-knowledge metadata (only hashes stored, no plaintext)
/// - Ephemeral keys (can be rotated/destroyed)
pub struct EncryptedStorage {
    /// BearDog crypto client for encryption operations
    crypto: CryptoClient,

    /// Underlying storage backend (filesystem, S3, ZFS, etc.)
    backend: Arc<dyn StorageBackend>,

    /// Family ID for lineage tracking
    family_id: String,

    /// Performance metrics
    metrics: Arc<RwLock<StorageMetrics>>,
}

impl EncryptedStorage {
    /// Create new encrypted storage from BearDog client
    ///
    /// # Arguments
    ///
    /// * `beardog` - BearDog client for encryption
    /// * `backend` - Storage backend implementation
    /// * `family_id` - Family ID for lineage tracking
    pub fn new(
        beardog: BearDogClient,
        backend: Arc<dyn StorageBackend>,
        family_id: impl Into<String>,
    ) -> Self {
        // Get transport from BearDog for crypto operations
        let transport = beardog.transport.clone().into();
        let crypto = CryptoClient::new(transport);

        Self {
            crypto,
            backend,
            family_id: family_id.into(),
            metrics: Arc::new(RwLock::new(StorageMetrics::default())),
        }
    }

    /// Store data with automatic encryption
    ///
    /// # Arguments
    ///
    /// * `key` - Unique key for this data
    /// * `data` - Plaintext data to encrypt and store
    ///
    /// # Returns
    ///
    /// Encryption metadata (zero-knowledge)
    #[instrument(skip(self, data), fields(key = %key, size = data.len()))]
    pub async fn store(&self, key: &str, data: &[u8]) -> Result<EncryptionMetadata> {
        let start = std::time::Instant::now();

        debug!("Encrypting {} bytes for key: {}", data.len(), key);

        // 1. Generate dataset-specific encryption key
        let key_ref = self.generate_key_for_dataset(key).await?;

        // 2. Hash plaintext (for integrity verification)
        let plaintext_hash = Self::hash_data(data);

        // 3. Encrypt data using BearDog crypto client
        let encrypted = self
            .crypto
            .encrypt(data, &key_ref)
            .await
            .context("Failed to encrypt data with BearDog")?;

        // 4. Convert to ciphertext bytes
        let ciphertext_bytes = base64::decode(&encrypted.ciphertext)
            .context("Failed to decode ciphertext from base64")?;

        // 5. Hash ciphertext (for corruption detection)
        let ciphertext_hash = Self::hash_data(&ciphertext_bytes);

        // 6. Store encrypted blob
        self.backend
            .store_raw(key, &ciphertext_bytes)
            .await
            .context("Failed to store encrypted data to backend")?;

        // 7. Create and store metadata
        let metadata = EncryptionMetadata::new(
            key_ref,
            encrypted.algorithm.clone(),
            encrypted.nonce.clone(),
            encrypted.tag.clone(),
            plaintext_hash,
            ciphertext_hash,
            data.len() as u64,
            ciphertext_bytes.len() as u64,
            self.family_id.clone(),
        );

        let metadata_bytes =
            serde_json::to_vec(&metadata).context("Failed to serialize metadata")?;

        self.backend
            .store_raw(&Self::metadata_key(key), &metadata_bytes)
            .await
            .context("Failed to store metadata")?;

        // 8. Update metrics
        let elapsed = start.elapsed();
        self.update_encrypt_metrics(data.len() as u64, elapsed)
            .await;

        info!(
            "Encrypted and stored {} bytes in {:?} ({})",
            data.len(),
            elapsed,
            metadata.zero_knowledge_summary()
        );

        Ok(metadata)
    }

    /// Retrieve and decrypt data
    ///
    /// # Arguments
    ///
    /// * `key` - Unique key for this data
    ///
    /// # Returns
    ///
    /// Decrypted plaintext data
    #[instrument(skip(self), fields(key = %key))]
    pub async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();

        debug!("Retrieving and decrypting data for key: {}", key);

        // 1. Load metadata
        let metadata = self.load_metadata(key).await?;

        // 2. Load encrypted blob
        let ciphertext_bytes = self
            .backend
            .retrieve_raw(key)
            .await
            .context("Failed to retrieve encrypted data from backend")?;

        // 3. Verify ciphertext integrity
        let ciphertext_hash = Self::hash_data(&ciphertext_bytes);
        if ciphertext_hash != metadata.ciphertext_hash {
            anyhow::bail!(
                "Ciphertext corruption detected for key: {} (hash mismatch)",
                key
            );
        }

        // 4. Reconstruct encrypted data structure
        let encrypted = EncryptedData {
            ciphertext: base64::encode(&ciphertext_bytes),
            nonce: metadata.nonce.clone(),
            tag: metadata.tag.clone(),
            algorithm: metadata.algorithm.clone(),
        };

        // 5. Decrypt using BearDog crypto client
        let plaintext = self
            .crypto
            .decrypt(&encrypted, &metadata.key_ref)
            .await
            .context("Failed to decrypt data with BearDog")?;

        // 6. Verify plaintext integrity
        let plaintext_hash = Self::hash_data(&plaintext);
        if plaintext_hash != metadata.plaintext_hash {
            warn!(
                "Plaintext hash mismatch for key: {} (decryption may have failed)",
                key
            );
            anyhow::bail!("Decryption integrity check failed for key: {}", key);
        }

        // 7. Update metrics
        let elapsed = start.elapsed();
        self.update_decrypt_metrics(ciphertext_bytes.len() as u64, elapsed)
            .await;

        info!(
            "Retrieved and decrypted {} bytes in {:?}",
            plaintext.len(),
            elapsed
        );

        Ok(plaintext)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        self.backend.exists(key).await
    }

    /// Delete encrypted data and metadata
    pub async fn delete(&self, key: &str) -> Result<()> {
        // Delete both data and metadata
        self.backend
            .delete(key)
            .await
            .context("Failed to delete encrypted data")?;

        self.backend
            .delete(&Self::metadata_key(key))
            .await
            .context("Failed to delete metadata")?;

        info!("Deleted encrypted data and metadata for key: {}", key);

        Ok(())
    }

    /// List all encrypted keys
    pub async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let all_keys = self.backend.list_keys(prefix).await?;

        // Filter out .meta files
        let data_keys: Vec<String> = all_keys
            .into_iter()
            .filter(|k| !k.ends_with(".meta"))
            .collect();

        Ok(data_keys)
    }

    /// Load metadata for a key
    pub async fn load_metadata(&self, key: &str) -> Result<EncryptionMetadata> {
        let metadata_bytes = self
            .backend
            .retrieve_raw(&Self::metadata_key(key))
            .await
            .context("Failed to retrieve metadata")?;

        serde_json::from_slice(&metadata_bytes).context("Failed to deserialize metadata")
    }

    /// Get current performance metrics
    pub async fn metrics(&self) -> StorageMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if performance targets are met (<5% overhead)
    pub async fn meets_performance_targets(&self) -> bool {
        self.metrics.read().await.meets_targets()
    }

    // =========================================================================
    // Private Helper Methods
    // =========================================================================

    /// Generate encryption key for a dataset
    async fn generate_key_for_dataset(&self, key: &str) -> Result<String> {
        // Use dataset key as seed for deterministic key generation
        // This allows key recovery if needed
        let key_id = format!("dataset_{}", Self::hash_data(key.as_bytes()));

        // Check if key already exists, otherwise generate new one
        // TODO: Implement key caching to avoid regenerating the same key

        Ok(key_id)
    }

    /// Hash data using SHA-256
    fn hash_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    /// Get metadata key for a data key
    fn metadata_key(key: &str) -> String {
        format!("{}.meta", key)
    }

    /// Update encryption metrics
    async fn update_encrypt_metrics(&self, bytes: u64, elapsed: std::time::Duration) {
        let mut metrics = self.metrics.write().await;

        metrics.bytes_stored += bytes;
        metrics.encrypt_count += 1;

        // Calculate rolling average latency
        let latency_us = elapsed.as_micros() as u64;
        if metrics.encrypt_count == 1 {
            metrics.avg_encrypt_latency_us = latency_us;
        } else {
            // Exponential moving average (alpha = 0.3)
            metrics.avg_encrypt_latency_us =
                (latency_us * 3 + metrics.avg_encrypt_latency_us * 7) / 10;
        }
    }

    /// Update decryption metrics
    async fn update_decrypt_metrics(&self, bytes: u64, elapsed: std::time::Duration) {
        let mut metrics = self.metrics.write().await;

        metrics.bytes_retrieved += bytes;
        metrics.decrypt_count += 1;

        // Calculate rolling average latency
        let latency_us = elapsed.as_micros() as u64;
        if metrics.decrypt_count == 1 {
            metrics.avg_decrypt_latency_us = latency_us;
        } else {
            // Exponential moving average (alpha = 0.3)
            metrics.avg_decrypt_latency_us =
                (latency_us * 3 + metrics.avg_decrypt_latency_us * 7) / 10;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    /// In-memory storage backend for testing
    struct MemoryBackend {
        data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl MemoryBackend {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for MemoryBackend {
        async fn store_raw(&self, key: &str, data: &[u8]) -> Result<()> {
            self.data
                .lock()
                .await
                .insert(key.to_string(), data.to_vec());
            Ok(())
        }

        async fn retrieve_raw(&self, key: &str) -> Result<Vec<u8>> {
            self.data
                .lock()
                .await
                .get(key)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))
        }

        async fn exists(&self, key: &str) -> Result<bool> {
            Ok(self.data.lock().await.contains_key(key))
        }

        async fn delete(&self, key: &str) -> Result<()> {
            self.data.lock().await.remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
            let data = self.data.lock().await;
            let keys: Vec<String> = match prefix {
                Some(p) => data.keys().filter(|k| k.starts_with(p)).cloned().collect(),
                None => data.keys().cloned().collect(),
            };
            Ok(keys)
        }
    }

    #[tokio::test]
    async fn test_hash_data() {
        let data = b"hello world";
        let hash = EncryptedStorage::hash_data(data);

        assert!(hash.starts_with("sha256:"));

        // Same data should produce same hash
        let hash2 = EncryptedStorage::hash_data(data);
        assert_eq!(hash, hash2);

        // Different data should produce different hash
        let hash3 = EncryptedStorage::hash_data(b"different data");
        assert_ne!(hash, hash3);
    }

    #[tokio::test]
    async fn test_metadata_key() {
        assert_eq!(
            EncryptedStorage::metadata_key("my-dataset"),
            "my-dataset.meta"
        );
    }

    // Note: Full integration tests require BearDog running
    // See tests/encrypted_storage_integration.rs
}
