// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0

//! Types for encrypted storage

use serde::{Deserialize, Serialize};

/// Encrypted data blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    /// Ciphertext (base64-encoded)
    pub ciphertext: String,

    /// Nonce/IV (base64-encoded)
    pub nonce: String,

    /// Authentication tag (base64-encoded)
    pub tag: Option<String>,

    /// Encryption algorithm
    pub algorithm: String,
}

/// Abstract storage backend trait
///
/// This allows swapping storage implementations (filesystem, S3, ZFS, etc.)
/// while maintaining the same encryption layer.
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store raw bytes under a key
    async fn store_raw(&self, key: &str, data: &[u8]) -> anyhow::Result<()>;

    /// Retrieve raw bytes for a key
    async fn retrieve_raw(&self, key: &str) -> anyhow::Result<Vec<u8>>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;

    /// Delete a key
    async fn delete(&self, key: &str) -> anyhow::Result<()>;

    /// List all keys with optional prefix filter
    async fn list_keys(&self, prefix: Option<&str>) -> anyhow::Result<Vec<String>>;
}

/// Performance metrics for encrypted storage operations
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    /// Total bytes stored (encrypted)
    pub bytes_stored: u64,

    /// Total bytes retrieved (encrypted)
    pub bytes_retrieved: u64,

    /// Total encrypt operations
    pub encrypt_count: u64,

    /// Total decrypt operations
    pub decrypt_count: u64,

    /// Average encryption latency (microseconds)
    pub avg_encrypt_latency_us: u64,

    /// Average decryption latency (microseconds)
    pub avg_decrypt_latency_us: u64,
}

impl StorageMetrics {
    /// Calculate encryption overhead percentage
    pub fn encryption_overhead(&self) -> f64 {
        if self.encrypt_count == 0 {
            return 0.0;
        }
        // Assuming 95µs target for 1MB (5% overhead on 100µs baseline)
        let baseline_us = 100.0;
        ((self.avg_encrypt_latency_us as f64 / baseline_us) - 1.0) * 100.0
    }

    /// Check if performance targets are met (<5% overhead)
    pub fn meets_targets(&self) -> bool {
        self.encryption_overhead() < 5.0
    }
}
