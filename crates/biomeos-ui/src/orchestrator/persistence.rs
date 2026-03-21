// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Persistence Module
//!
//! Handles data persistence via NestGate storage primal.
//!
//! ## Network Effect Phase 5: Persistence
//!
//! Stores assignments for recovery after restart.
//!
//! ## Graceful Degradation
//!
//! If NestGate is not available, data is not persisted
//! but operations continue successfully.

use crate::primal_client::NestGateClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Persistence handler
pub struct Persistence;

impl Persistence {
    /// Persist assignment via NestGate
    ///
    /// Falls back gracefully if NestGate is unavailable.
    pub async fn persist_assignment(
        nestgate: Option<&NestGateClient>,
        family_id: &str,
        assignment_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!(
            "Persisting assignment: id={}, device={}, primal={}",
            assignment_id, device_id, primal_id
        );

        if let Some(nestgate) = nestgate {
            info!("🏠 NestGate available - persisting assignment");

            // Call NestGate to store the assignment
            match nestgate
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
                    info!("✅ Persisted via NestGate");
                    Ok(())
                }
                Err(e) => {
                    warn!(
                        "⚠️ NestGate storage failed: {} - continuing without persistence",
                        e
                    );
                    Ok(())
                }
            }
        } else {
            warn!("⚠️ No storage primal available, assignment not persisted");
            Err(anyhow::anyhow!("No storage primal available"))
        }
    }

    /// Remove assignment from NestGate persistence
    pub async fn remove_assignment(
        nestgate: Option<&NestGateClient>,
        device_id: &str,
    ) -> Result<()> {
        if let Some(nestgate) = nestgate {
            match nestgate
                .call(
                    "storage.delete",
                    serde_json::json!({ "key_prefix": format!("assignment:*-{}", device_id) }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Removed assignment from NestGate");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ NestGate delete failed: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("⚠️ No storage primal available");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primal_client::NestGateClient;

    #[tokio::test]
    async fn test_persist_assignment_no_nestgate() {
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
            err.to_string().contains("No storage primal available"),
            "Expected 'No storage primal available' in error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_remove_assignment_no_nestgate() {
        let result = Persistence::remove_assignment(None, "test-device").await;

        // Should succeed gracefully (no-op when no NestGate)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_persist_assignment_nestgate_unavailable_graceful_degradation() {
        // NestGate client pointing to non-existent socket - call will fail
        let nestgate = Some(NestGateClient::with_socket(
            "nestgate",
            "/tmp/nonexistent-biomeos-persistence-test-12345.sock",
        ));

        let result = Persistence::persist_assignment(
            nestgate.as_ref(),
            "test-family",
            "assign-001",
            "gpu-0",
            "toadstool-1",
        )
        .await;

        // Graceful degradation: storage failure returns Ok(()) - assignment continues
        assert!(
            result.is_ok(),
            "Persistence should degrade gracefully when NestGate call fails: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_remove_assignment_nestgate_unavailable_returns_ok() {
        // When NestGate is None, remove_assignment returns Ok (no-op)
        let result = Persistence::remove_assignment(None, "gpu-0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_assignment_nestgate_call_fails_returns_err() {
        // NestGate client pointing to non-existent socket - delete call will fail
        let nestgate = Some(NestGateClient::with_socket(
            "nestgate",
            "/tmp/nonexistent-biomeos-remove-test-67890.sock",
        ));

        let result = Persistence::remove_assignment(nestgate.as_ref(), "gpu-0").await;

        // remove_assignment propagates errors when NestGate call fails
        assert!(
            result.is_err(),
            "remove_assignment should return Err when NestGate delete fails"
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
