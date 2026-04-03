// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! UI Sync Module
//!
//! Handles UI updates and event synchronization via the UI / visualization provider.
//!
//! ## Network Effect Phase 6: UI Updates
//!
//! Pushes topology updates and shows notifications.
//!
//! ## Graceful Degradation
//!
//! If no UI provider is available, visualization is not updated
//! but operations continue successfully.

use crate::primal_client::UiClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// UI synchronization handler
pub struct UISync;

impl UISync {
    /// Update UI after assignment via the visualization provider
    ///
    /// Falls back gracefully if the UI client is unavailable.
    pub async fn update_ui_after_assignment(
        ui: Option<&UiClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!("Updating UI: device={}, primal={}", device_id, primal_id);

        if let Some(ui) = ui {
            info!("UI provider available — updating visualization");

            match ui
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
                    info!("✅ UI updated via visualization provider");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ UI update failed: {} — continuing", e);
                    Ok(())
                }
            }
        } else {
            warn!("⚠️ No visualization provider available, UI not updated");
            Err(anyhow::anyhow!("No visualization provider available"))
        }
    }

    /// Update UI after device unassignment
    pub async fn update_ui_after_unassignment(
        ui: Option<&UiClient>,
        device_id: &str,
    ) -> Result<()> {
        if let Some(ui) = ui {
            match ui
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
        ui: Option<&UiClient>,
        initial_state: serde_json::Value,
    ) -> Result<()> {
        if let Some(ui) = ui {
            match ui.call("ui.initialize", initial_state).await {
                Ok(_) => {
                    info!("✅ Initial UI state pushed to visualization provider");
                    Ok(())
                }
                Err(e) => {
                    warn!("⚠️ Failed to push initial state: {}", e);
                    Err(e)
                }
            }
        } else {
            warn!("⚠️ No UI provider available — running headless");
            Ok(())
        }
    }

    /// Push UI refresh
    pub async fn push_refresh(ui: Option<&UiClient>, refresh_results: Vec<&str>) -> Result<()> {
        if let Some(ui) = ui {
            let _ = ui
                .call(
                    "ui.refresh",
                    serde_json::json!({ "refreshed": refresh_results }),
                )
                .await;
        }
        Ok(())
    }

    /// Send UI heartbeat
    pub async fn send_heartbeat(ui: Option<&UiClient>) -> Result<()> {
        if let Some(ui) = ui {
            let _ = ui
                .call("ui.heartbeat", serde_json::json!({ "status": "running" }))
                .await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primal_client::{PrimalClient, UiClient};

    #[tokio::test]
    async fn test_update_ui_no_ui_provider() {
        let result = UISync::update_ui_after_assignment(None, "test-device", "test-primal").await;

        // Should return error but not panic
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No visualization provider"));
    }

    #[tokio::test]
    async fn test_update_ui_after_assignment_with_client_call_fails() {
        let client: UiClient = PrimalClient::with_socket("ui", "/nonexistent/ui.sock");
        let ui = Some(client);
        let result =
            UISync::update_ui_after_assignment(ui.as_ref(), "device-123", "primal-456").await;
        assert!(
            result.is_ok(),
            "graceful degradation when UI provider call fails"
        );
    }

    #[tokio::test]
    async fn test_initialize_ui_no_ui_provider() {
        let state = serde_json::json!({"test": "data"});
        let result = UISync::initialize_ui(None, state).await;

        // Should succeed gracefully (running headless)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_ui_with_client_call_fails() {
        let client: UiClient = PrimalClient::with_socket("ui", "/nonexistent/ui.sock");
        let ui = Some(client);
        let state = serde_json::json!({"family_id": "test", "primals": {}});
        let result = UISync::initialize_ui(ui.as_ref(), state).await;
        assert!(
            result.is_err(),
            "initialize_ui propagates error when call fails"
        );
    }

    #[tokio::test]
    async fn test_update_ui_after_unassignment_no_client() {
        let result = UISync::update_ui_after_unassignment(None, "device-123").await;

        // Should succeed (graceful degradation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_ui_after_unassignment_with_client_call_fails() {
        let client: UiClient = PrimalClient::with_socket("ui", "/nonexistent/ui.sock");
        let ui = Some(client);
        let result = UISync::update_ui_after_unassignment(ui.as_ref(), "device-123").await;
        assert!(result.is_ok(), "graceful degradation when call fails");
    }

    #[tokio::test]
    async fn test_push_refresh_no_client() {
        let refresh_results = vec!["devices", "primals", "metrics"];
        let result = UISync::push_refresh(None, refresh_results).await;

        // Should succeed (graceful degradation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_refresh_with_client() {
        let client: UiClient = PrimalClient::with_socket("ui", "/nonexistent/ui.sock");
        let ui = Some(client);
        let result = UISync::push_refresh(ui.as_ref(), vec!["devices", "primals"]).await;
        assert!(result.is_ok(), "push_refresh always returns Ok");
    }

    #[tokio::test]
    async fn test_push_refresh_empty_results() {
        let result = UISync::push_refresh(None, vec![]).await;

        // Should succeed with empty results
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_heartbeat_no_client() {
        let result = UISync::send_heartbeat(None).await;

        // Should succeed (graceful degradation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_heartbeat_with_client() {
        let client: UiClient = PrimalClient::with_socket("ui", "/nonexistent/ui.sock");
        let ui = Some(client);
        let result = UISync::send_heartbeat(ui.as_ref()).await;
        assert!(result.is_ok(), "send_heartbeat always returns Ok");
    }

    #[tokio::test]
    async fn test_update_ui_after_assignment_with_device_and_primal() {
        // Test that the function handles device and primal IDs properly
        let result =
            UISync::update_ui_after_assignment(None, "device-abc-123", "primal-xyz-456").await;

        assert!(result.is_err());
        // Error message should be consistent
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("visualization provider")
        );
    }
}
