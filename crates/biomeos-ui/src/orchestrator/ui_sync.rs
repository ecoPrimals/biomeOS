//! UI Sync Module
//!
//! Handles UI updates and event synchronization via petalTongue.
//!
//! ## Network Effect Phase 6: UI Updates
//!
//! Pushes topology updates and shows notifications.
//!
//! ## Graceful Degradation
//!
//! If petalTongue is not available, UI is not updated
//! but operations continue successfully.

use crate::primal_client::PetalTongueClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// UI synchronization handler
pub struct UISync;

impl UISync {
    /// Update UI via petalTongue after assignment
    ///
    /// Falls back gracefully if petalTongue is unavailable.
    pub async fn update_ui_after_assignment(
        petaltongue: &Option<PetalTongueClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!("Updating UI: device={}, primal={}", device_id, primal_id);

        if let Some(ref petaltongue) = petaltongue {
            info!("🌸 petalTongue available - updating UI");

            // Call petalTongue to update the topology display
            match petaltongue
                .call(
                    "ui.update_topology",
                    serde_json::json!({
                        "event": "device_assigned",
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ UI updated via petalTongue");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ petalTongue update failed: {} - continuing", e);
                    Ok(())
                }
            }
        } else {
            warn!("⚠️ No visualization primal available, UI not updated");
            Err(anyhow::anyhow!("No visualization primal available"))
        }
    }

    /// Update UI after device unassignment
    pub async fn update_ui_after_unassignment(
        petaltongue: &Option<PetalTongueClient>,
        device_id: &str,
    ) -> Result<()> {
        if let Some(ref petaltongue) = petaltongue {
            match petaltongue
                .call(
                    "ui.update_topology",
                    serde_json::json!({
                        "event": "device_unassigned",
                        "device_id": device_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ UI updated after unassignment");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ Failed to update UI: {}", e);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Initialize UI state
    pub async fn initialize_ui(
        petaltongue: &Option<PetalTongueClient>,
        initial_state: serde_json::Value,
    ) -> Result<()> {
        if let Some(ref petaltongue) = petaltongue {
            match petaltongue.call("ui.initialize", initial_state).await {
                Ok(_) => {
                    info!("✅ Initial UI state pushed to petalTongue");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ Failed to push initial state: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("⚠️ No petalTongue available - running headless");
            Ok(())
        }
    }

    /// Push UI refresh
    pub async fn push_refresh(
        petaltongue: &Option<PetalTongueClient>,
        refresh_results: Vec<&str>,
    ) -> Result<()> {
        if let Some(ref petaltongue) = petaltongue {
            let _ = petaltongue
                .call(
                    "ui.refresh",
                    serde_json::json!({ "refreshed": refresh_results }),
                )
                .await;
        }
        Ok(())
    }

    /// Send UI heartbeat
    pub async fn send_heartbeat(petaltongue: &Option<PetalTongueClient>) -> Result<()> {
        if let Some(ref petaltongue) = petaltongue {
            let _ = petaltongue
                .call("ui.heartbeat", serde_json::json!({ "status": "running" }))
                .await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_ui_no_petaltongue() {
        let result = UISync::update_ui_after_assignment(&None, "test-device", "test-primal").await;

        // Should return error but not panic
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_ui_no_petaltongue() {
        let state = serde_json::json!({"test": "data"});
        let result = UISync::initialize_ui(&None, state).await;

        // Should succeed gracefully
        assert!(result.is_ok());
    }
}
