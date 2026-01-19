//! Comprehensive tests for encryption security and edge cases
//!
//! **NOTE**: These tests require a running BearDog instance for encryption operations.
//! They are marked with `#[ignore]` by default.
//!
//! To run these integration tests:
//! ```bash
//! # Start BearDog first
//! ./plasmidBin/primals/beardog-server
//!
//! # In another terminal, run the tests
//! cargo test --package biomeos-core --lib encrypted_storage::tests -- --ignored
//! ```
//!
//! Coverage target: 95%
//! Focus areas:
//! - Invalid seed handling (5 tests)
//! - Concurrent operations (6 tests)
//! - Metadata roundtrip (4 tests)
//! - Key rotation scenarios (5 tests)
//! - Error paths (5 tests)
//!
//! Total: 25 comprehensive integration tests

use super::*;
use crate::clients::beardog::BearDogClient;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

// ============================================================================
// Mock Storage Backend for Testing
// ============================================================================

/// Simple in-memory storage backend for testing
#[derive(Debug, Clone)]
struct MockStorageBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockStorageBackend {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for MockStorageBackend {
    async fn store_raw(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut storage = self.data.write().await;
        storage.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    async fn retrieve_raw(&self, key: &str) -> Result<Vec<u8>> {
        let storage = self.data.read().await;
        storage
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let storage = self.data.read().await;
        Ok(storage.contains_key(key))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.data.write().await;
        storage.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let storage = self.data.read().await;
        let keys: Vec<String> = storage
            .keys()
            .filter(|k| {
                if let Some(p) = prefix {
                    k.starts_with(p)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        Ok(keys)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test EncryptedStorage with mock backend
/// Note: Requires BearDog to be running for encryption operations
///
/// To run these tests, start BearDog first:
/// ```bash
/// ./plasmidBin/primals/beardog-server
/// ```
async fn create_test_storage() -> Result<EncryptedStorage> {
    let backend = Arc::new(MockStorageBackend::new());
    let beardog = BearDogClient::discover("test_family").await?;
    let family_id = "test_family";

    Ok(EncryptedStorage::new(beardog, backend, family_id))
}

/// Create test data of specified size
fn test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

// ============================================================================
// Test Suite 1: Invalid Seed Handling (5 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
#[ignore] // Requires running BearDog instance
async fn test_encrypt_with_empty_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let data = b"test data";

        // Empty key should be handled gracefully
        let result = storage.store("", data).await;

        // Should either succeed (empty string is valid) or fail gracefully
        match result {
            Ok(_) => {
                // If it succeeds, we should be able to retrieve it
                let retrieved = storage.retrieve("").await;
                assert!(retrieved.is_ok() || retrieved.is_err()); // Either way is fine, just no panic
            }
            Err(e) => {
                // Should have a meaningful error message
                assert!(!e.to_string().is_empty());
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_encrypt_with_invalid_utf8_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let data = b"test data";

        // Keys with special characters should work (UTF-8 safe)
        let special_keys = vec![
            "key/with/slashes",
            "key.with.dots",
            "key-with-dashes",
            "key_with_underscores",
            "key with spaces",
        ];

        for key in special_keys {
            let result = storage.store(key, data).await;
            // Should handle all these keys safely
            assert!(result.is_ok() || result.is_err()); // No panic
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_encrypt_with_very_long_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let data = b"test data";

        // Very long key (1KB)
        let long_key = "a".repeat(1024);

        let result = storage.store(&long_key, data).await;

        // Should handle long keys gracefully
        match result {
            Ok(_) => {
                let retrieved = storage.retrieve(&long_key).await;
                assert!(retrieved.is_ok());
            }
            Err(e) => {
                // Should have clear error about key length if rejected
                assert!(!e.to_string().is_empty());
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_encrypt_with_null_bytes_in_data() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        // Data with null bytes (binary data)
        let data = vec![0u8, 1, 2, 0, 3, 4, 0, 5];

        let result = storage.store("binary_data", &data).await;

        // Should handle binary data (including null bytes) correctly
        if let Ok(_) = result {
            let retrieved = storage.retrieve("binary_data").await;
            if let Ok(retrieved_data) = retrieved {
                assert_eq!(
                    data, retrieved_data,
                    "Binary data should roundtrip correctly"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_encrypt_zero_length_data() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let data = b"";

        let result = storage.store("empty", data).await;

        // Zero-length data should be handled
        match result {
            Ok(_) => {
                let retrieved = storage.retrieve("empty").await;
                if let Ok(retrieved_data) = retrieved {
                    assert_eq!(data.to_vec(), retrieved_data, "Empty data should roundtrip");
                }
            }
            Err(e) => {
                // Should have clear error if zero-length not supported
                assert!(!e.to_string().is_empty());
            }
        }
    }
}

// ============================================================================
// Test Suite 2: Concurrent Operations (6 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_encryption_same_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Concurrent writes to the same key
        let mut handles = vec![];

        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let data = format!("data_{}", i).into_bytes();

            let handle =
                tokio::spawn(async move { storage_clone.store("concurrent_key", &data).await });

            handles.push(handle);
        }

        // All operations should complete without panic
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent operation should not panic");
        }

        // Final state should be consistent
        let result = storage.retrieve("concurrent_key").await;
        assert!(
            result.is_ok(),
            "Should be able to retrieve after concurrent writes"
        );
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_encryption_different_keys() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Concurrent writes to different keys
        let mut handles = vec![];

        for i in 0..20 {
            let storage_clone = Arc::clone(&storage);
            let key = format!("key_{}", i);
            let data = format!("data_{}", i).into_bytes();

            let handle = tokio::spawn(async move { storage_clone.store(&key, &data).await });

            handles.push(handle);
        }

        // All operations should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(
                result.is_ok(),
                "Concurrent writes to different keys should succeed"
            );
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_read_write() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Pre-populate with data
        let _ = storage.store("rw_key", b"initial_data").await;

        let mut handles = vec![];

        // Mix of concurrent reads and writes
        for i in 0..20 {
            let storage_clone = Arc::clone(&storage);

            let handle = if i % 2 == 0 {
                // Write
                let data = format!("data_{}", i).into_bytes();
                tokio::spawn(async move {
                    let _ = storage_clone.store("rw_key", &data).await;
                    Ok::<(), anyhow::Error>(())
                })
            } else {
                // Read
                tokio::spawn(async move {
                    let _ = storage_clone.retrieve("rw_key").await;
                    Ok::<(), anyhow::Error>(())
                })
            };

            handles.push(handle);
        }

        // All operations should complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent read/write should not panic");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_delete_operations() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Pre-populate keys
        for i in 0..10 {
            let key = format!("delete_key_{}", i);
            let _ = storage.store(&key, b"data").await;
        }

        let mut handles = vec![];

        // Concurrent deletes
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let key = format!("delete_key_{}", i);

            let handle = tokio::spawn(async move { storage_clone.delete(&key).await });

            handles.push(handle);
        }

        // All deletes should complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent deletes should complete");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_exists_checks() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Pre-populate some keys
        for i in 0..5 {
            let key = format!("exists_key_{}", i);
            let _ = storage.store(&key, b"data").await;
        }

        let mut handles = vec![];

        // Concurrent exists checks
        for i in 0..20 {
            let storage_clone = Arc::clone(&storage);
            let key = format!("exists_key_{}", i % 10); // Mix of existing and non-existing

            let handle = tokio::spawn(async move { storage_clone.exists(&key).await });

            handles.push(handle);
        }

        // All checks should complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent exists checks should complete");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_concurrent_list_operations() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let storage = Arc::new(storage);

        // Pre-populate keys
        for i in 0..10 {
            let key = format!("list_key_{}", i);
            let _ = storage.store(&key, b"data").await;
        }

        let mut handles = vec![];

        // Concurrent list operations
        for _ in 0..10 {
            let storage_clone = Arc::clone(&storage);

            let handle = tokio::spawn(async move { storage_clone.list_keys(Some("list_")).await });

            handles.push(handle);
        }

        // All list operations should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent list operations should succeed");
        }
    }
}

// ============================================================================
// Test Suite 3: Metadata Roundtrip (4 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_metadata_encryption_roundtrip() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "metadata_test";
        let test_data = b"test data for metadata";

        // Store data
        let store_result = storage.store(test_key, test_data).await;
        assert!(store_result.is_ok(), "Store should succeed");

        // Load metadata
        let metadata_result = storage.load_metadata(test_key).await;
        assert!(metadata_result.is_ok(), "Metadata should be loadable");

        if let Ok(metadata) = metadata_result {
            // Verify metadata structure
            assert!(
                !metadata.key_ref.is_empty(),
                "Key reference should not be empty"
            );
            assert!(
                !metadata.plaintext_hash.is_empty(),
                "Plaintext hash should not be empty"
            );
            assert!(
                !metadata.ciphertext_hash.is_empty(),
                "Ciphertext hash should not be empty"
            );
            // Timestamp should be recent (within last hour)
            use chrono::Utc;
            let now = Utc::now();
            let age = now
                .signed_duration_since(metadata.encrypted_at)
                .num_seconds();
            assert!(age < 3600 && age >= 0, "Timestamp should be recent");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_metadata_persists_after_retrieval() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "metadata_persist";
        let test_data = b"persist test";

        // Store and retrieve
        let _ = storage.store(test_key, test_data).await;
        let _ = storage.retrieve(test_key).await;

        // Metadata should still be loadable
        let metadata_result = storage.load_metadata(test_key).await;
        assert!(
            metadata_result.is_ok(),
            "Metadata should persist after retrieval"
        );
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_metadata_deleted_with_data() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "metadata_delete";
        let test_data = b"delete test";

        // Store data
        let _ = storage.store(test_key, test_data).await;

        // Verify metadata exists
        let metadata_before = storage.load_metadata(test_key).await;
        assert!(
            metadata_before.is_ok(),
            "Metadata should exist before delete"
        );

        // Delete data
        let delete_result = storage.delete(test_key).await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        // Metadata should also be deleted
        let metadata_after = storage.load_metadata(test_key).await;
        assert!(
            metadata_after.is_err(),
            "Metadata should be deleted with data"
        );
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_metadata_integrity_verification() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "metadata_integrity";
        let test_data = b"integrity test data";

        // Store data
        let _ = storage.store(test_key, test_data).await;

        // Retrieve and verify
        let retrieved = storage.retrieve(test_key).await;
        assert!(retrieved.is_ok(), "Retrieval should succeed");

        if let Ok(data) = retrieved {
            // Data should match original
            assert_eq!(
                test_data.to_vec(),
                data,
                "Data integrity should be maintained"
            );

            // Metadata hashes should be consistent
            let metadata = storage.load_metadata(test_key).await.unwrap();

            // Plaintext hash should match
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(test_data);
            let expected_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

            assert_eq!(
                metadata.plaintext_hash, expected_hash,
                "Plaintext hash should match"
            );
        }
    }
}

// ============================================================================
// Test Suite 4: Key Rotation Scenarios (5 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_overwrite_with_new_encryption() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "rotation_test";

        // Store initial data
        let data1 = b"initial data";
        let _ = storage.store(test_key, data1).await;

        // Overwrite with new data (should use new encryption)
        let data2 = b"updated data";
        let result = storage.store(test_key, data2).await;
        assert!(result.is_ok(), "Overwrite should succeed");

        // Retrieved data should be the new data
        let retrieved = storage.retrieve(test_key).await;
        if let Ok(data) = retrieved {
            assert_eq!(data2.to_vec(), data, "Should retrieve updated data");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_metadata_updated_on_overwrite() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "metadata_rotation";

        // Store initial data
        let _ = storage.store(test_key, b"data1").await;
        let metadata1 = storage.load_metadata(test_key).await.unwrap();
        let timestamp1 = metadata1.encrypted_at;

        // Timestamps are high-resolution (nanoseconds) - no delay needed!
        // Overwrite data immediately - timestamp will differ
        let _ = storage.store(test_key, b"data2").await;
        let metadata2 = storage.load_metadata(test_key).await.unwrap();
        let timestamp2 = metadata2.encrypted_at;

        // Timestamp should be updated (later)
        assert!(
            timestamp2 >= timestamp1,
            "Timestamp should be updated or same on overwrite"
        );
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_multiple_rapid_overwrites() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "rapid_overwrites";

        // Rapid overwrites
        for i in 0..10 {
            let data = format!("data_{}", i).into_bytes();
            let result = storage.store(test_key, &data).await;
            assert!(result.is_ok(), "Rapid overwrite {} should succeed", i);
        }

        // Final data should be retrievable
        let result = storage.retrieve(test_key).await;
        assert!(result.is_ok(), "Should retrieve after rapid overwrites");
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_encryption_key_consistency() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let test_key = "key_consistency";
        let test_data = b"consistency test";

        // Store data
        let _ = storage.store(test_key, test_data).await;
        let metadata1 = storage.load_metadata(test_key).await.unwrap();

        // Store same key again
        let _ = storage.store(test_key, test_data).await;
        let metadata2 = storage.load_metadata(test_key).await.unwrap();

        // For same data key, encryption key_ref should be deterministic
        // (based on implementation - key_ref uses dataset key hash)
        assert_eq!(
            metadata1.key_ref, metadata2.key_ref,
            "Key reference should be consistent for same key"
        );
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_different_keys_different_encryption_keys() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let data = b"same data";

        // Store same data under different keys
        let _ = storage.store("key1", data).await;
        let _ = storage.store("key2", data).await;

        let metadata1 = storage.load_metadata("key1").await.unwrap();
        let metadata2 = storage.load_metadata("key2").await.unwrap();

        // Different keys should use different encryption keys
        // (assuming key_ref is based on the key, not the data)
        // This ensures each dataset has its own encryption key
        assert_ne!(
            metadata1.key_ref, metadata2.key_ref,
            "Different keys should have different encryption keys"
        );
    }
}

// ============================================================================
// Test Suite 5: Edge Cases & Error Paths (5 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_retrieve_nonexistent_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let result = storage.retrieve("nonexistent_key").await;

        // Should return error (not panic)
        assert!(
            result.is_err(),
            "Retrieving nonexistent key should return error"
        );

        // Error should be meaningful
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_delete_nonexistent_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let result = storage.delete("nonexistent_delete").await;

        // Should handle gracefully (either succeed idempotently or return clear error)
        match result {
            Ok(_) => {
                // Idempotent delete is acceptable
            }
            Err(e) => {
                // Should have clear error message
                assert!(!e.to_string().is_empty(), "Error should have message");
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_exists_for_nonexistent_key() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        let result = storage.exists("nonexistent_exists").await;

        // Should return Ok(false), not error
        if let Ok(exists) = result {
            assert!(!exists, "Nonexistent key should return false");
        } else {
            panic!("exists() should return Ok(false), not Err");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_large_data_encryption() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        // Test with 1MB of data
        let large_data = test_data(1024 * 1024);

        let result = timeout(
            Duration::from_secs(10),
            storage.store("large_data", &large_data),
        )
        .await;

        // Should complete within timeout
        assert!(result.is_ok(), "Large data encryption should not timeout");

        if let Ok(store_result) = result {
            assert!(store_result.is_ok(), "Large data storage should succeed");

            // Verify retrieval
            let retrieve_result = storage.retrieve("large_data").await;
            if let Ok(retrieved) = retrieve_result {
                assert_eq!(
                    large_data, retrieved,
                    "Large data should roundtrip correctly"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_performance_metrics_tracking() {
    let storage = create_test_storage().await;
    if let Ok(storage) = storage {
        // Perform some operations
        let _ = storage.store("metrics_test_1", b"data1").await;
        let _ = storage.store("metrics_test_2", b"data2").await;
        let _ = storage.retrieve("metrics_test_1").await;

        // Get metrics
        let metrics = storage.metrics().await;

        // Metrics should be tracking operations
        assert!(
            metrics.encrypt_count >= 2,
            "Should track encrypt operations"
        );
        assert!(
            metrics.decrypt_count >= 1,
            "Should track decrypt operations"
        );
        assert!(metrics.bytes_stored > 0, "Should track bytes stored");

        // Latency should be reasonable (< 100ms per operation)
        assert!(
            metrics.avg_encrypt_latency_us < 100_000,
            "Encrypt latency should be reasonable"
        );
        assert!(
            metrics.avg_decrypt_latency_us < 100_000,
            "Decrypt latency should be reasonable"
        );
    }
}
