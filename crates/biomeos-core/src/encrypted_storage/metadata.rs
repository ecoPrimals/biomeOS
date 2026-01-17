// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0

//! Encryption metadata for stored data

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Encryption metadata stored alongside encrypted blobs
///
/// This metadata is stored in `.meta` files alongside the encrypted data.
/// It contains everything needed to decrypt the data (except the plaintext key itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Reference to the encryption key in BearDog
    ///
    /// This is NOT the key itself, just a reference/ID that BearDog uses
    /// to look up the actual key material.
    pub key_ref: String,

    /// Encryption algorithm used
    ///
    /// Currently always "AES-256-GCM" for authenticated encryption.
    pub algorithm: String,

    /// Nonce/IV (base64-encoded)
    ///
    /// Used for encryption/decryption. Must be unique per encryption operation.
    pub nonce: String,

    /// Authentication tag (base64-encoded)
    ///
    /// Ensures data integrity and authenticity (prevents tampering).
    pub tag: Option<String>,

    /// SHA-256 hash of the PLAINTEXT data
    ///
    /// Used for integrity verification and lineage tracking.
    /// This is zero-knowledge: you can verify the data matches without
    /// exposing the plaintext.
    pub plaintext_hash: String,

    /// SHA-256 hash of the CIPHERTEXT data
    ///
    /// Used for detecting corruption in the encrypted blob.
    pub ciphertext_hash: String,

    /// Size of plaintext data in bytes
    pub plaintext_size: u64,

    /// Size of ciphertext data in bytes
    pub ciphertext_size: u64,

    /// When this data was encrypted
    pub encrypted_at: DateTime<Utc>,

    /// Family ID that encrypted this data
    ///
    /// Used for lineage tracking and access control.
    pub family_id: String,

    /// Optional user-provided tags/labels
    #[serde(default)]
    pub tags: Vec<String>,
}

impl EncryptionMetadata {
    /// Create new encryption metadata
    pub fn new(
        key_ref: String,
        algorithm: String,
        nonce: String,
        tag: Option<String>,
        plaintext_hash: String,
        ciphertext_hash: String,
        plaintext_size: u64,
        ciphertext_size: u64,
        family_id: String,
    ) -> Self {
        Self {
            key_ref,
            algorithm,
            nonce,
            tag,
            plaintext_hash,
            ciphertext_hash,
            plaintext_size,
            ciphertext_size,
            encrypted_at: Utc::now(),
            family_id,
            tags: Vec::new(),
        }
    }

    /// Add a tag to this metadata
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Check if this metadata has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Calculate encryption overhead (ciphertext vs plaintext size)
    pub fn size_overhead_percent(&self) -> f64 {
        if self.plaintext_size == 0 {
            return 0.0;
        }
        let overhead = self.ciphertext_size as f64 - self.plaintext_size as f64;
        (overhead / self.plaintext_size as f64) * 100.0
    }

    /// Get a zero-knowledge summary (no key material)
    pub fn zero_knowledge_summary(&self) -> String {
        format!(
            "Encrypted {} bytes → {} bytes ({:.1}% overhead) at {} using {}",
            self.plaintext_size,
            self.ciphertext_size,
            self.size_overhead_percent(),
            self.encrypted_at.format("%Y-%m-%d %H:%M:%S"),
            self.algorithm
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = EncryptionMetadata::new(
            "key_123".to_string(),
            "AES-256-GCM".to_string(),
            "nonce_abc".to_string(),
            Some("tag_def".to_string()),
            "plaintext_hash_123".to_string(),
            "ciphertext_hash_456".to_string(),
            1000,
            1050,
            "nat0".to_string(),
        );

        assert_eq!(meta.key_ref, "key_123");
        assert_eq!(meta.algorithm, "AES-256-GCM");
        assert_eq!(meta.plaintext_size, 1000);
        assert_eq!(meta.ciphertext_size, 1050);
        assert_eq!(meta.family_id, "nat0");
    }

    #[test]
    fn test_size_overhead() {
        let meta = EncryptionMetadata::new(
            "key_123".to_string(),
            "AES-256-GCM".to_string(),
            "nonce_abc".to_string(),
            None,
            "hash1".to_string(),
            "hash2".to_string(),
            1000,
            1050,
            "nat0".to_string(),
        );

        // 50 bytes overhead on 1000 bytes = 5%
        assert_eq!(meta.size_overhead_percent(), 5.0);
    }

    #[test]
    fn test_tags() {
        let mut meta = EncryptionMetadata::new(
            "key_123".to_string(),
            "AES-256-GCM".to_string(),
            "nonce_abc".to_string(),
            None,
            "hash1".to_string(),
            "hash2".to_string(),
            1000,
            1000,
            "nat0".to_string(),
        );

        assert!(!meta.has_tag("medical"));

        meta.add_tag("medical");
        meta.add_tag("pii");

        assert!(meta.has_tag("medical"));
        assert!(meta.has_tag("pii"));
        assert!(!meta.has_tag("public"));
    }
}
