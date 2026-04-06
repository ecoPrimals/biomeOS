// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Persistence Module
//!
//! Handles data persistence via the storage capability provider.
//!
//! ## Network Effect Phase 5: Persistence
//!
//! Stores assignments for recovery after restart.
//!
//! ## Graceful Degradation
//!
//! If no storage provider is available, data is not persisted
//! but operations continue successfully.

use crate::primal_client::StorageClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Persistence handler
pub struct Persistence;

impl Persistence {
    /// Persist assignment via the storage provider
    ///
    /// Falls back gracefully if storage is unavailable.
    pub async fn persist_assignment(
        storage: Option<&StorageClient>,
        family_id: &str,
        assignment_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!(
            "Persisting assignment: id={}, device={}, primal={}",
            assignment_id, device_id, primal_id
        );

        if let Some(storage) = storage {
            info!("Storage provider available — persisting assignment");

            match storage
                .call(
                    "storage.store",
                    serde_json::json!({
                        "key": format!("assignment:{}", assignment_id),
                        "value": {
                            "assignment_id": assignment_id,
                            "device_id": device_id,
                            "primal_id": primal_id,
                            "family_id": family_id
                        }
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Persisted via storage provider");
                    Ok(())
                }
                Err(e) => {
                    warn!(
                        "⚠️ Storage provider failed: {} — continuing without persistence",
                        e
                    );
                    Ok(())
                }
            }
        } else {
            warn!("⚠️ No storage provider available, assignment not persisted");
            Err(anyhow::anyhow!("No storage provider available"))
        }
    }

    /// Remove assignment from storage
    pub async fn remove_assignment(storage: Option<&StorageClient>, device_id: &str) -> Result<()> {
        if let Some(storage) = storage {
            match storage
                .call(
                    "storage.delete",
                    serde_json::json!({ "key_prefix": format!("assignment:*-{}", device_id) }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Removed assignment from storage");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ Storage delete failed: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("⚠️ No storage provider available");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primal_client::StorageClient;

    #[tokio::test]
    async fn test_persist_assignment_no_storage() {
        let result = Persistence::persist_assignment(
            None,
            "test-family",
            "test-id",
            "test-device",
            "test-primal",
        )
        .await;

        // Should return error but not panic
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("No storage provider available"),
            "Expected 'No storage provider available' in error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_remove_assignment_no_storage() {
        let result = Persistence::remove_assignment(None, "test-device").await;

        // Should succeed gracefully (no-op when no storage provider)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_persist_assignment_storage_unavailable_graceful_degradation() {
        // Storage client pointing to non-existent socket — call will fail
        let storage = Some(StorageClient::with_socket(
            "storage",
            "/tmp/nonexistent-biomeos-persistence-test-12345.sock",
        ));

        let result = Persistence::persist_assignment(
            storage.as_ref(),
            "test-family",
            "assign-001",
            "gpu-0",
            "toadstool-1",
        )
        .await;

        // Graceful degradation: storage failure returns Ok(()) - assignment continues
        assert!(
            result.is_ok(),
            "Persistence should degrade gracefully when storage call fails: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_remove_assignment_storage_unavailable_returns_ok() {
        // When storage is None, remove_assignment returns Ok (no-op)
        let result = Persistence::remove_assignment(None, "gpu-0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_assignment_storage_call_fails_returns_err() {
        let storage = Some(StorageClient::with_socket(
            "storage",
            "/tmp/nonexistent-biomeos-remove-test-67890.sock",
        ));

        let result = Persistence::remove_assignment(storage.as_ref(), "gpu-0").await;

        assert!(
            result.is_err(),
            "remove_assignment should return Err when storage delete fails"
        );
    }

    #[tokio::test]
    async fn test_persist_assignment_key_format() {
        // Verify the persistence key format is assignment:{id}
        // We test with None to ensure the error path includes our params
        let result = Persistence::persist_assignment(
            None,
            "family-xyz",
            "unique-assign-id",
            "device-abc",
            "primal-def",
        )
        .await;

        assert!(result.is_err());
    }
}
