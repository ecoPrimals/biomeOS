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
        nestgate: &Option<NestGateClient>,
        family_id: &str,
        assignment_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!(
            "Persisting assignment: id={}, device={}, primal={}",
            assignment_id, device_id, primal_id
        );

        if let Some(ref nestgate) = nestgate {
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
        nestgate: &Option<NestGateClient>,
        device_id: &str,
    ) -> Result<()> {
        if let Some(ref nestgate) = nestgate {
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

    #[tokio::test]
    async fn test_persist_assignment_no_nestgate() {
        let result = Persistence::persist_assignment(
            &None,
            "test-family",
            "test-id",
            "test-device",
            "test-primal",
        )
        .await;

        // Should return error but not panic
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_assignment_no_nestgate() {
        let result = Persistence::remove_assignment(&None, "test-device").await;

        // Should succeed gracefully
        assert!(result.is_ok());
    }
}
