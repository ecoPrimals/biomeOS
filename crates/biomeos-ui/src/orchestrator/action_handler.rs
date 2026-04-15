// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Action Handler Module
//!
//! Handles all user actions by coordinating between multiple primals.
//!
//! ## Network Effect in Action
//!
//! Each user action orchestrates multiple capability providers:
//! - Security: authorization
//! - Discovery: validation and registry
//! - Compute: capacity and process management
//! - Storage: persistence
//! - UI: visualization updates
//! - AI: suggestions

use crate::{
    actions::{ActionResult, UserAction},
    primal_client::{
        AiClient, ComputeClient, DiscoveryClient, PrimalConnections, SecurityClient, StorageClient,
        UiClient,
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

/// Context for device assignment coordination (bundles capability clients)
struct DeviceAssignmentCtx<'a> {
    ui: Option<&'a UiClient>,
    discovery: Option<&'a DiscoveryClient>,
    security: Option<&'a SecurityClient>,
    storage: Option<&'a StorageClient>,
    compute: Option<&'a ComputeClient>,
}

impl ActionHandler {
    /// Handle a user action
    ///
    /// Actions come from the UI (petalTongue) and are processed here.
    /// The orchestrator coordinates between multiple primals to fulfill the action.
    ///
    /// Takes `PrimalConnections` instead of
    /// individual primal references. Extracts typed clients internally.
    pub async fn handle_user_action(
        action: UserAction,
        family_id: &str,
        connections: &PrimalConnections,
    ) -> Result<ActionResult> {
        let ui = connections.get_by_capability("ui").cloned();
        let discovery = connections.get_by_capability("discovery").cloned();
        let security = connections.get_by_capability("encryption").cloned();
        let storage = connections.get_by_capability("storage").cloned();
        let compute = connections.get_by_capability("compute").cloned();
        let ai = connections.get_by_capability("ai").cloned();

        match action {
            UserAction::AssignDevice {
                device_id,
                primal_id,
            } => {
                let ctx = DeviceAssignmentCtx {
                    ui: ui.as_ref(),
                    discovery: discovery.as_ref(),
                    security: security.as_ref(),
                    storage: storage.as_ref(),
                    compute: compute.as_ref(),
                };
                Self::handle_assign_device(&device_id, &primal_id, family_id, &ctx).await
            }

            UserAction::UnassignDevice { device_id } => {
                Self::handle_unassign_device(
                    &device_id,
                    discovery.as_ref(),
                    storage.as_ref(),
                    ui.as_ref(),
                )
                .await
            }

            UserAction::StartPrimal { primal_name } => {
                Self::handle_start_primal(&primal_name, compute.as_ref()).await
            }

            UserAction::StopPrimal { primal_id } => {
                Self::handle_stop_primal(&primal_id, compute.as_ref()).await
            }

            UserAction::RestartPrimal { primal_id } => {
                Self::handle_restart_primal(&primal_id, compute.as_ref()).await
            }

            UserAction::AcceptSuggestion { suggestion_id } => {
                Self::handle_accept_suggestion(&suggestion_id, family_id, ai.as_ref()).await
            }

            UserAction::DismissSuggestion { suggestion_id } => {
                Self::handle_dismiss_suggestion(&suggestion_id, family_id, ai.as_ref()).await
            }

            UserAction::Refresh => {
                Self::handle_refresh(discovery.as_ref(), compute.as_ref(), ui.as_ref()).await
            }
        }
    }

    /// Handle device assignment
    ///
    /// Network effect: coordinates multiple capability providers for one user action.
    ///
    /// ## Coordination flow
    ///
    /// 1. **Security**: authorization (user permissions, primal policy)
    /// 2. **Discovery**: validation (device availability, primal health)
    /// 3. **Compute**: capacity check (resource availability)
    /// 4. **Discovery**: register assignment (service registry)
    /// 5. **Storage**: persist assignment (recovery after restart)
    /// 6. **UI**: update visualization (feedback)
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

        // Phase 1: Authorization via security provider
        let current_user = Authorization::get_current_user_id(ctx.security).await;
        let auth_result = Authorization::authorize_device_assignment(
            ctx.security,
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
                    "Authorization denied: {reason}"
                )));
            }
            Err(e) => {
                warn!("⚠️ Authorization check failed: {}", e);
                return Ok(ActionResult::error(format!(
                    "Authorization check failed: {e}"
                )));
            }
        }

        // Phase 2: Validation via discovery provider
        let validation_result =
            Validation::validate_device_assignment(ctx.discovery, device_id, primal_id).await;

        match validation_result {
            Ok(ValidationResult::Valid) => {
                info!("✅ Validation: Passed");
            }
            Ok(ValidationResult::Invalid(reason)) => {
                warn!("❌ Validation: Failed - {}", reason);
                return Ok(ActionResult::error(format!("Validation failed: {reason}")));
            }
            Err(e) => {
                warn!("⚠️ Validation check failed: {}", e);
                return Ok(ActionResult::error(format!("Validation check failed: {e}")));
            }
        }

        // Phase 3: Capacity check via compute provider
        let capacity_result =
            Capacity::check_primal_capacity(ctx.compute, device_id, primal_id).await;

        match capacity_result {
            Ok(CapacityResult::Available) => {
                info!("✅ Capacity: Available");
            }
            Ok(CapacityResult::Insufficient { reason }) => {
                warn!("❌ Capacity: Insufficient - {}", reason);
                return Ok(ActionResult::error(format!(
                    "Insufficient capacity: {reason}"
                )));
            }
            Err(e) => {
                warn!("⚠️ Capacity check failed: {}, proceeding anyway", e);
                // Non-critical: continue without capacity check
            }
        }

        // Phase 4: Register assignment via discovery provider
        let assignment_id =
            match Self::register_assignment(ctx.discovery, device_id, primal_id).await {
                Ok(id) => {
                    info!("✅ Assignment registered: {}", id);
                    id
                }
                Err(e) => {
                    warn!("❌ Failed to register assignment: {}", e);
                    return Ok(ActionResult::error(format!(
                        "Failed to register assignment: {e}"
                    )));
                }
            };

        // Phase 5: Persist assignment via storage provider (non-critical)
        if let Err(e) = Persistence::persist_assignment(
            ctx.storage,
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

        // Phase 6: Update UI via visualization provider (non-critical)
        if let Err(e) = UISync::update_ui_after_assignment(ctx.ui, device_id, primal_id).await {
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
            "Device {device_id} successfully assigned to primal {primal_id}"
        )))
    }

    /// Register assignment via discovery/registry provider
    ///
    /// Creates the assignment record in the service registry.
    /// Returns assignment ID for tracking.
    async fn register_assignment(
        discovery: Option<&DiscoveryClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<String> {
        if let Some(discovery) = discovery {
            info!("Discovery provider available — registering assignment");

            match discovery
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
                        .map_or_else(
                            || format!("registry-{device_id}-{primal_id}"),
                            std::string::ToString::to_string,
                        );
                    info!("✅ Registered via discovery provider: {}", assignment_id);
                    return Ok(assignment_id);
                }
                Err(e) => {
                    warn!(
                        "⚠️ Discovery registry registration failed: {} — using local ID",
                        e
                    );
                }
            }
        }

        let assignment_id = format!("local-{device_id}-{primal_id}");
        info!("✅ Registered locally: {}", assignment_id);
        Ok(assignment_id)
    }

    /// Handle device unassignment
    async fn handle_unassign_device(
        device_id: &str,
        discovery: Option<&DiscoveryClient>,
        storage: Option<&StorageClient>,
        ui: Option<&UiClient>,
    ) -> Result<ActionResult> {
        info!("Unassigning device {}", device_id);

        // Step 1: Remove from discovery/registry
        if let Some(discovery) = discovery {
            match discovery
                .call(
                    "registry.unassign_device",
                    serde_json::json!({ "device_id": device_id }),
                )
                .await
            {
                Ok(_) => info!("✅ Removed assignment from discovery registry"),
                Err(e) => warn!("⚠️ Discovery unassign failed: {}", e),
            }
        }

        // Step 2: Remove from storage provider
        let _ = Persistence::remove_assignment(storage, device_id).await;

        // Step 3: Update UI
        let _ = UISync::update_ui_after_unassignment(ui, device_id).await;

        Ok(ActionResult::success(format!(
            "Device {device_id} unassigned successfully"
        )))
    }

    /// Handle primal start
    async fn handle_start_primal(
        primal_name: &str,
        compute: Option<&ComputeClient>,
    ) -> Result<ActionResult> {
        info!("Starting primal {}", primal_name);

        if let Some(compute) = compute {
            match compute
                .call(
                    "compute.start_primal",
                    serde_json::json!({ "primal_name": primal_name }),
                )
                .await
            {
                Ok(result) => {
                    let pid = result
                        .get("pid")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0);
                    info!("✅ Primal {} started with PID {}", primal_name, pid);
                    return Ok(ActionResult::success(format!(
                        "Primal {primal_name} started (PID: {pid})"
                    )));
                }
                Err(e) => {
                    warn!("❌ Failed to start primal {}: {}", primal_name, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to start {primal_name}: {e}"
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute provider available to start primals".to_string(),
        ))
    }

    /// Handle primal stop
    async fn handle_stop_primal(
        primal_id: &str,
        compute: Option<&ComputeClient>,
    ) -> Result<ActionResult> {
        info!("Stopping primal {}", primal_id);

        if let Some(compute) = compute {
            match compute
                .call(
                    "compute.stop_primal",
                    serde_json::json!({ "primal_id": primal_id, "graceful": true }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Primal {} stopped", primal_id);
                    return Ok(ActionResult::success(format!("Primal {primal_id} stopped")));
                }
                Err(e) => {
                    warn!("❌ Failed to stop primal {}: {}", primal_id, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to stop {primal_id}: {e}"
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute provider available to stop primals".to_string(),
        ))
    }

    /// Handle primal restart
    async fn handle_restart_primal(
        primal_id: &str,
        compute: Option<&ComputeClient>,
    ) -> Result<ActionResult> {
        info!("Restarting primal {}", primal_id);

        if let Some(compute) = compute {
            match compute
                .call(
                    "compute.restart_primal",
                    serde_json::json!({ "primal_id": primal_id }),
                )
                .await
            {
                Ok(result) => {
                    let new_pid = result
                        .get("pid")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0);
                    info!("✅ Primal {} restarted with PID {}", primal_id, new_pid);
                    return Ok(ActionResult::success(format!(
                        "Primal {primal_id} restarted (new PID: {new_pid})"
                    )));
                }
                Err(e) => {
                    warn!("❌ Failed to restart primal {}: {}", primal_id, e);
                    return Ok(ActionResult::error(format!(
                        "Failed to restart {primal_id}: {e}"
                    )));
                }
            }
        }

        Ok(ActionResult::error(
            "No compute provider available to restart primals".to_string(),
        ))
    }

    /// Handle AI suggestion acceptance
    async fn handle_accept_suggestion(
        suggestion_id: &str,
        family_id: &str,
        ai: Option<&AiClient>,
    ) -> Result<ActionResult> {
        info!("Accepting suggestion {}", suggestion_id);

        if let Some(ai) = ai {
            match ai
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
                    info!("✅ AI provider notified of accepted suggestion");
                }
                Err(e) => {
                    warn!("⚠️ Failed to notify AI provider: {}", e);
                }
            }
        }

        Ok(ActionResult::success(format!(
            "Suggestion {suggestion_id} accepted"
        )))
    }

    /// Handle AI suggestion dismissal
    async fn handle_dismiss_suggestion(
        suggestion_id: &str,
        family_id: &str,
        ai: Option<&AiClient>,
    ) -> Result<ActionResult> {
        info!("Dismissing suggestion {}", suggestion_id);

        if let Some(ai) = ai {
            match ai
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
                    info!("✅ AI provider notified of dismissed suggestion");
                }
                Err(e) => {
                    warn!("⚠️ Failed to notify AI provider: {}", e);
                }
            }
        }

        Ok(ActionResult::success(format!(
            "Suggestion {suggestion_id} dismissed"
        )))
    }

    /// Handle UI refresh
    async fn handle_refresh(
        discovery: Option<&DiscoveryClient>,
        compute: Option<&ComputeClient>,
        ui: Option<&UiClient>,
    ) -> Result<ActionResult> {
        info!("Refreshing UI state");

        let mut refresh_results = Vec::new();

        // Refresh device list from discovery provider
        if let Some(discovery) = discovery {
            match discovery
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("devices"),
                Err(e) => warn!("Failed to refresh devices: {}", e),
            }

            match discovery
                .call("registry.list_primals", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("primals"),
                Err(e) => warn!("Failed to refresh primals: {}", e),
            }
        }

        // Refresh metrics from compute provider
        if let Some(compute) = compute {
            match compute
                .call("compute.get_metrics", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("metrics"),
                Err(e) => warn!("Failed to refresh metrics: {}", e),
            }
        }

        // Push refresh to UI
        let _ = UISync::push_refresh(ui, refresh_results.clone()).await;

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
#[path = "action_handler_tests/mod.rs"]
mod tests;
