// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Action Handler Module
//!
//! Handles all user actions by coordinating between multiple primals.
//!
//! ## Network Effect in Action
//!
//! Each user action orchestrates multiple primals:
//! - BearDog: Authorization
//! - Songbird: Validation and registry
//! - ToadStool: Capacity and process management
//! - NestGate: Persistence
//! - petalTongue: UI updates
//! - Squirrel: AI suggestions

use crate::{
    actions::{ActionResult, UserAction},
    primal_client::{
        BearDogClient, NestGateClient, PetalTongueClient, PrimalConnections, SongbirdClient,
        SquirrelClient, ToadStoolClient,
    },
};
use anyhow::Result;
use tracing::{info, warn};

use super::{
    authorization::{Authorization, AuthorizationResult},
    capacity::{Capacity, CapacityResult},
    persistence::Persistence,
    ui_sync::UISync,
    validation::{Validation, ValidationResult},
};

/// Action handler
pub struct ActionHandler;

/// Context for device assignment coordination (bundles primal clients)
struct DeviceAssignmentCtx<'a> {
    petaltongue: &'a Option<PetalTongueClient>,
    songbird: &'a Option<SongbirdClient>,
    beardog: &'a Option<BearDogClient>,
    nestgate: &'a Option<NestGateClient>,
    toadstool: &'a Option<ToadStoolClient>,
}

impl ActionHandler {
    /// Handle a user action
    ///
    /// Actions come from the UI (petalTongue) and are processed here.
    /// The orchestrator coordinates between multiple primals to fulfill the action.
    /// Handle a user action
    ///
    /// DEEP DEBT EVOLUTION (Feb 7, 2026): Takes `PrimalConnections` instead of
    /// individual primal references. Extracts typed clients internally.
    pub async fn handle_user_action(
        action: UserAction,
        family_id: &str,
        connections: &PrimalConnections,
    ) -> Result<ActionResult> {
        // Extract typed clients from dynamic registry (backward compatible)
        let petaltongue = connections.petaltongue().cloned();
        let songbird = connections.songbird().cloned();
        let beardog = connections.beardog().cloned();
        let nestgate = connections.nestgate().cloned();
        let toadstool = connections.toadstool().cloned();
        let squirrel = connections.squirrel().cloned();

        match action {
            UserAction::AssignDevice {
                device_id,
                primal_id,
            } => {
                let ctx = DeviceAssignmentCtx {
                    petaltongue: &petaltongue,
                    songbird: &songbird,
                    beardog: &beardog,
                    nestgate: &nestgate,
                    toadstool: &toadstool,
                };
                Self::handle_assign_device(&device_id, &primal_id, family_id, &ctx).await
            }

            UserAction::UnassignDevice { device_id } => {
                Self::handle_unassign_device(&device_id, &songbird, &nestgate, &petaltongue).await
            }

            UserAction::StartPrimal { primal_name } => {
                Self::handle_start_primal(&primal_name, &toadstool).await
            }

            UserAction::StopPrimal { primal_id } => {
                Self::handle_stop_primal(&primal_id, &toadstool).await
            }

            UserAction::RestartPrimal { primal_id } => {
                Self::handle_restart_primal(&primal_id, &toadstool).await
            }

            UserAction::AcceptSuggestion { suggestion_id } => {
                Self::handle_accept_suggestion(&suggestion_id, family_id, &squirrel).await
            }

            UserAction::DismissSuggestion { suggestion_id } => {
                Self::handle_dismiss_suggestion(&suggestion_id, family_id, &squirrel).await
            }

            UserAction::Refresh => Self::handle_refresh(&songbird, &toadstool, &petaltongue).await,
        }
    }

    /// Handle device assignment
    ///
    /// Network effect: Coordinates 6 primals for a single user action!
    ///
    /// ## Multi-Primal Coordination Flow
    ///
    /// 1. **BearDog**: Authorization (user permissions, primal policy)
    /// 2. **Songbird**: Validation (device availability, primal health)
    /// 3. **ToadStool**: Capacity check (resource availability)
    /// 4. **Songbird**: Register assignment (service registry)
    /// 5. **NestGate**: Persist assignment (recovery after restart)
    /// 6. **petalTongue**: Update UI (visual feedback)
    async fn handle_assign_device(
        device_id: &str,
        primal_id: &str,
        family_id: &str,
        ctx: &DeviceAssignmentCtx<'_>,
    ) -> Result<ActionResult> {
        info!(
            "🎯 Device assignment requested: {} → {}",
            device_id, primal_id
        );

        // Phase 1: Authorization via BearDog
        let current_user = Authorization::get_current_user_id(ctx.beardog).await;
        let auth_result = Authorization::authorize_device_assignment(
            ctx.beardog,
            &current_user,
            device_id,
            primal_id,
        )
        .await;

        match auth_result {
            Ok(AuthorizationResult::Authorized) => {
                info!("✅ Authorization: Approved");
            }
            Ok(AuthorizationResult::Denied(reason)) => {
                warn!("❌ Authorization: Denied - {}", reason);
                return Ok(ActionResult::error(format!(
                    "Authorization denied: {}",
                    reason
                )));
            }
            Err(e) => {
                warn!("⚠️ Authorization check failed: {}", e);
                return Ok(ActionResult::error(format!(
                    "Authorization check failed: {}",
                    e
                )));
            }
        }

        // Phase 2: Validation via Songbird
        let validation_result =
            Validation::validate_device_assignment(ctx.songbird, device_id, primal_id).await;

        match validation_result {
            Ok(ValidationResult::Valid) => {
                info!("✅ Validation: Passed");
            }
            Ok(ValidationResult::Invalid(reason)) => {
                warn!("❌ Validation: Failed - {}", reason);
                return Ok(ActionResult::error(format!(
                    "Validation failed: {}",
                    reason
                )));
            }
            Err(e) => {
                warn!("⚠️ Validation check failed: {}", e);
                return Ok(ActionResult::error(format!(
                    "Validation check failed: {}",
                    e
                )));
            }
        }

        // Phase 3: Capacity check via ToadStool
        let capacity_result =
            Capacity::check_primal_capacity(ctx.toadstool, device_id, primal_id).await;

        match capacity_result {
            Ok(CapacityResult::Available) => {
                info!("✅ Capacity: Available");
            }
            Ok(CapacityResult::Insufficient { reason }) => {
                warn!("❌ Capacity: Insufficient - {}", reason);
                return Ok(ActionResult::error(format!(
                    "Insufficient capacity: {}",
                    reason
                )));
            }
            Err(e) => {
                warn!("⚠️ Capacity check failed: {}, proceeding anyway", e);
                // Non-critical: continue without capacity check
            }
        }

        // Phase 4: Register assignment via Songbird
        let assignment_id =
            match Self::register_assignment(ctx.songbird, device_id, primal_id).await {
                Ok(id) => {
                    info!("✅ Assignment registered: {}", id);
                    id
                }
                Err(e) => {
                    warn!("❌ Failed to register assignment: {}", e);
                    return Ok(ActionResult::error(format!(
                        "Failed to register assignment: {}",
                        e
                    )));
                }
            };

        // Phase 5: Persist assignment via NestGate (non-critical)
        if let Err(e) = Persistence::persist_assignment(
            ctx.nestgate,
            family_id,
            &assignment_id,
            device_id,
            primal_id,
        )
        .await
        {
            warn!("⚠️ Failed to persist assignment: {}, continuing", e);
            // Non-critical: assignment still works, just won't survive restart
        } else {
            info!("✅ Assignment persisted");
        }

        // Phase 6: Update UI via petalTongue (non-critical)
        if let Err(e) =
            UISync::update_ui_after_assignment(ctx.petaltongue, device_id, primal_id).await
        {
            warn!("⚠️ Failed to update UI: {}, continuing", e);
            // Non-critical: assignment succeeded, UI just not updated
        } else {
            info!("✅ UI updated");
        }

        info!(
            "🎉 Device assignment complete: {} → {}",
            device_id, primal_id
        );

        Ok(ActionResult::success(format!(
            "Device {} successfully assigned to primal {}",
            device_id, primal_id
        )))
    }

    /// Register assignment via Songbird
    ///
    /// Creates the assignment record in the service registry.
    /// Returns assignment ID for tracking.
    async fn register_assignment(
        songbird: &Option<SongbirdClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<String> {
        if let Some(ref songbird) = songbird {
            info!("🎵 Songbird available - registering assignment");

            match songbird
                .call(
                    "registry.register_assignment",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    let assignment_id = result
                        .get("assignment_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("songbird-{}-{}", device_id, primal_id));
                    info!("✅ Registered via Songbird: {}", assignment_id);
                    return Ok(assignment_id);
                }
                Err(e) => {
                    warn!("⚠️ Songbird registration failed: {} - using local ID", e);
                }
            }
        }

        let assignment_id = format!("local-{}-{}", device_id, primal_id);
        info!("✅ Registered locally: {}", assignment_id);
        Ok(assignment_id)
    }

    /// Handle device unassignment
    async fn handle_unassign_device(
        device_id: &str,
        songbird: &Option<SongbirdClient>,
        nestgate: &Option<NestGateClient>,
        petaltongue: &Option<PetalTongueClient>,
    ) -> Result<ActionResult> {
        info!("Unassigning device {}", device_id);

        // Step 1: Remove from Songbird registry
        if let Some(ref songbird) = songbird {
            match songbird
                .call(
                    "registry.unassign_device",
                    serde_json::json!({ "device_id": device_id }),
                )
                .await
            {
                Ok(_) => info!("✅ Removed assignment from Songbird registry"),
                Err(e) => warn!("⚠️ Songbird unassign failed: {}", e),
            }
        }

        // Step 2: Remove from NestGate persistence
        let _ = Persistence::remove_assignment(nestgate, device_id).await;

        // Step 3: Update UI
        let _ = UISync::update_ui_after_unassignment(petaltongue, device_id).await;

        Ok(ActionResult::success(format!(
            "Device {} unassigned successfully",
            device_id
        )))
    }

    /// Handle primal start
    async fn handle_start_primal(
        primal_name: &str,
        toadstool: &Option<ToadStoolClient>,
    ) -> Result<ActionResult> {
        info!("Starting primal {}", primal_name);

        if let Some(ref toadstool) = toadstool {
            match toadstool
                .call(
                    "compute.start_primal",
                    serde_json::json!({ "primal_name": primal_name }),
                )
                .await
            {
                Ok(result) => {
                    let pid = result.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                    info!("✅ Primal {} started with PID {}", primal_name, pid);
                    return Ok(ActionResult::success(format!(
                        "Primal {} started (PID: {})",
                        primal_name, pid
                    )));
                }
                Err(e) => {
                    warn!("❌ Failed to start primal {}: {}", primal_name, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to start {}: {}",
                        primal_name, e
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute primal (ToadStool) available to start primals".to_string(),
        ))
    }

    /// Handle primal stop
    async fn handle_stop_primal(
        primal_id: &str,
        toadstool: &Option<ToadStoolClient>,
    ) -> Result<ActionResult> {
        info!("Stopping primal {}", primal_id);

        if let Some(ref toadstool) = toadstool {
            match toadstool
                .call(
                    "compute.stop_primal",
                    serde_json::json!({ "primal_id": primal_id, "graceful": true }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Primal {} stopped", primal_id);
                    return Ok(ActionResult::success(format!(
                        "Primal {} stopped",
                        primal_id
                    )));
                }
                Err(e) => {
                    warn!("❌ Failed to stop primal {}: {}", primal_id, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to stop {}: {}",
                        primal_id, e
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute primal (ToadStool) available to stop primals".to_string(),
        ))
    }

    /// Handle primal restart
    async fn handle_restart_primal(
        primal_id: &str,
        toadstool: &Option<ToadStoolClient>,
    ) -> Result<ActionResult> {
        info!("Restarting primal {}", primal_id);

        if let Some(ref toadstool) = toadstool {
            match toadstool
                .call(
                    "compute.restart_primal",
                    serde_json::json!({ "primal_id": primal_id }),
                )
                .await
            {
                Ok(result) => {
                    let new_pid = result.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                    info!("✅ Primal {} restarted with PID {}", primal_id, new_pid);
                    return Ok(ActionResult::success(format!(
                        "Primal {} restarted (new PID: {})",
                        primal_id, new_pid
                    )));
                }
                Err(e) => {
                    warn!("❌ Failed to restart primal {}: {}", primal_id, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to restart {}: {}",
                        primal_id, e
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute primal (ToadStool) available to restart primals".to_string(),
        ))
    }

    /// Handle AI suggestion acceptance
    async fn handle_accept_suggestion(
        suggestion_id: &str,
        family_id: &str,
        squirrel: &Option<SquirrelClient>,
    ) -> Result<ActionResult> {
        info!("Accepting suggestion {}", suggestion_id);

        if let Some(ref squirrel) = squirrel {
            match squirrel
                .call(
                    "ai.accept_suggestion",
                    serde_json::json!({
                        "suggestion_id": suggestion_id,
                        "family_id": family_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Squirrel notified of accepted suggestion");
                }
                Err(e) => {
                    warn!("⚠️ Failed to notify Squirrel: {}", e);
                }
            }
        }

        Ok(ActionResult::success(format!(
            "Suggestion {} accepted",
            suggestion_id
        )))
    }

    /// Handle AI suggestion dismissal
    async fn handle_dismiss_suggestion(
        suggestion_id: &str,
        family_id: &str,
        squirrel: &Option<SquirrelClient>,
    ) -> Result<ActionResult> {
        info!("Dismissing suggestion {}", suggestion_id);

        if let Some(ref squirrel) = squirrel {
            match squirrel
                .call(
                    "ai.dismiss_suggestion",
                    serde_json::json!({
                        "suggestion_id": suggestion_id,
                        "family_id": family_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Squirrel notified of dismissed suggestion");
                }
                Err(e) => {
                    warn!("⚠️ Failed to notify Squirrel: {}", e);
                }
            }
        }

        Ok(ActionResult::success(format!(
            "Suggestion {} dismissed",
            suggestion_id
        )))
    }

    /// Handle UI refresh
    async fn handle_refresh(
        songbird: &Option<SongbirdClient>,
        toadstool: &Option<ToadStoolClient>,
        petaltongue: &Option<PetalTongueClient>,
    ) -> Result<ActionResult> {
        info!("Refreshing UI state");

        let mut refresh_results = Vec::new();

        // Refresh device list from Songbird
        if let Some(ref songbird) = songbird {
            match songbird
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("devices"),
                Err(e) => warn!("Failed to refresh devices: {}", e),
            }

            match songbird
                .call("registry.list_primals", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("primals"),
                Err(e) => warn!("Failed to refresh primals: {}", e),
            }
        }

        // Refresh metrics from ToadStool
        if let Some(ref toadstool) = toadstool {
            match toadstool
                .call("compute.get_metrics", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("metrics"),
                Err(e) => warn!("Failed to refresh metrics: {}", e),
            }
        }

        // Push refresh to UI
        let _ = UISync::push_refresh(petaltongue, refresh_results.clone()).await;

        Ok(ActionResult::success(format!(
            "UI refreshed ({} sources updated)",
            refresh_results.len()
        )))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_handle_start_primal_no_toadstool() {
        let result = ActionHandler::handle_start_primal("beardog", &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_error());
    }

    #[tokio::test]
    async fn test_handle_stop_primal_no_toadstool() {
        let result = ActionHandler::handle_stop_primal("primal-123", &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_error());
    }

    #[tokio::test]
    async fn test_handle_restart_primal_no_toadstool() {
        let result = ActionHandler::handle_restart_primal("primal-456", &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_error());
    }

    #[tokio::test]
    async fn test_handle_accept_suggestion_no_squirrel() {
        let result =
            ActionHandler::handle_accept_suggestion("suggestion-001", "family-123", &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        // Should succeed even without Squirrel (non-critical)
        assert!(action_result.is_success());
    }

    #[tokio::test]
    async fn test_handle_dismiss_suggestion_no_squirrel() {
        let result =
            ActionHandler::handle_dismiss_suggestion("suggestion-002", "family-123", &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        // Should succeed even without Squirrel (non-critical)
        assert!(action_result.is_success());
    }

    #[tokio::test]
    async fn test_handle_unassign_device_no_clients() {
        let result = ActionHandler::handle_unassign_device("device-123", &None, &None, &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_success());
    }

    #[tokio::test]
    async fn test_handle_refresh_no_clients() {
        let result = ActionHandler::handle_refresh(&None, &None, &None).await;

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_success());
    }

    #[tokio::test]
    async fn test_register_assignment_no_songbird() {
        let result = ActionHandler::register_assignment(&None, "device-001", "primal-001").await;

        assert!(result.is_ok());
        let assignment_id = result.unwrap();
        // Should generate local ID
        assert!(assignment_id.starts_with("local-"));
        assert!(assignment_id.contains("device-001"));
        assert!(assignment_id.contains("primal-001"));
    }

    #[tokio::test]
    async fn test_handle_user_action_assign_device() {
        let connections = PrimalConnections::default();
        let action = UserAction::AssignDevice {
            device_id: "dev-123".to_string(),
            primal_id: "primal-456".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_user_action_unassign_device() {
        let connections = PrimalConnections::default();
        let action = UserAction::UnassignDevice {
            device_id: "dev-123".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[tokio::test]
    async fn test_handle_user_action_start_primal() {
        let connections = PrimalConnections::default();
        let action = UserAction::StartPrimal {
            primal_name: "beardog".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        // Fails without ToadStool
        assert!(result.unwrap().is_error());
    }

    #[tokio::test]
    async fn test_handle_user_action_stop_primal() {
        let connections = PrimalConnections::default();
        let action = UserAction::StopPrimal {
            primal_id: "primal-123".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_error());
    }

    #[tokio::test]
    async fn test_handle_user_action_restart_primal() {
        let connections = PrimalConnections::default();
        let action = UserAction::RestartPrimal {
            primal_id: "primal-789".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_error());
    }

    #[tokio::test]
    async fn test_handle_user_action_accept_suggestion() {
        let connections = PrimalConnections::default();
        let action = UserAction::AcceptSuggestion {
            suggestion_id: "suggestion-001".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[tokio::test]
    async fn test_handle_user_action_dismiss_suggestion() {
        let connections = PrimalConnections::default();
        let action = UserAction::DismissSuggestion {
            suggestion_id: "suggestion-002".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[tokio::test]
    async fn test_handle_user_action_refresh() {
        let connections = PrimalConnections::default();
        let action = UserAction::Refresh;
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[tokio::test]
    async fn test_handle_start_primal_toadstool_connection_fails() {
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "toadstool",
            ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
        );
        let action = UserAction::StartPrimal {
            primal_name: "beardog".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_error());
    }

    #[tokio::test]
    async fn test_handle_stop_primal_toadstool_connection_fails() {
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "toadstool",
            ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
        );
        let action = UserAction::StopPrimal {
            primal_id: "primal-123".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_error());
    }

    #[tokio::test]
    async fn test_handle_refresh_with_failing_clients() {
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "songbird",
            SongbirdClient::with_socket("songbird", "/nonexistent/songbird.sock"),
        );
        connections.add_client(
            "toadstool",
            ToadStoolClient::with_socket("toadstool", "/nonexistent/toadstool.sock"),
        );
        let action = UserAction::Refresh;
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.is_success());
        if let ActionResult::Success { message } = &action_result {
            assert!(message.contains("0 sources"));
        }
    }

    #[tokio::test]
    async fn test_register_assignment_songbird_returns_fallback_id() {
        let result = ActionHandler::register_assignment(&None, "device-001", "primal-001").await;
        assert!(result.is_ok());
        let id = result.unwrap();
        assert!(id.starts_with("local-"));
        assert!(id.contains("device-001"));
        assert!(id.contains("primal-001"));
    }

    #[tokio::test]
    async fn test_handle_unassign_device_with_songbird_failing() {
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "songbird",
            SongbirdClient::with_socket("songbird", "/nonexistent/songbird.sock"),
        );
        let action = UserAction::UnassignDevice {
            device_id: "dev-123".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-123", &connections).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[tokio::test]
    async fn test_handle_assign_device_success_with_empty_connections() {
        let connections = PrimalConnections::default();
        let action = UserAction::AssignDevice {
            device_id: "gpu-0".to_string(),
            primal_id: "toadstool".to_string(),
        };
        let result = ActionHandler::handle_user_action(action, "family-1", &connections).await;
        assert!(result.is_ok());
        let ar = result.unwrap();
        assert!(ar.is_success());
        if let ActionResult::Success { message } = &ar {
            assert!(message.contains("gpu-0"));
            assert!(message.contains("toadstool"));
        }
    }

    #[tokio::test]
    async fn test_register_assignment_songbird_returns_id() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("songbird.sock");
        let path = socket_path.clone();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.expect("read");
                let _req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"assignment_id": "songbird-abc-123"},
                    "id": 1
                });
                let line = format!("{}\n", resp);
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let songbird = SongbirdClient::with_socket("songbird", &socket_path);
        let result =
            ActionHandler::register_assignment(&Some(songbird), "device-1", "primal-1").await;
        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id, "songbird-abc-123");
        server.abort();
    }
}
