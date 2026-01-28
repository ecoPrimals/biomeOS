//! Interactive UI Orchestrator
//!
//! Coordinates between multiple primals to create the interactive UI network effect.
//!
//! ## TRUE PRIMAL Principles
//!
//! - **No hardcoding**: All primals discovered via capabilities
//! - **Runtime discovery**: No compile-time dependencies
//! - **Network effect**: Value emerges from cooperation
//! - **Capability-based**: Query capabilities, don't assume

use crate::{
    actions::{ActionResult, UserAction},
    events::EventBroadcaster,
    state::UIState,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Result of authorization check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Authorization granted
    Authorized,
    /// Authorization denied with reason
    Denied(String),
}

/// Result of validation check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Validation passed
    Valid,
    /// Validation failed with reason
    Invalid(String),
}

/// Result of capacity check
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityResult {
    /// Capacity available
    Available,
    /// Capacity insufficient with details
    Insufficient { reason: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// PRIMAL CLIENTS - Using AtomicClient for Pure Rust JSON-RPC
// ═══════════════════════════════════════════════════════════════════════════
//
// ✅ EVOLVED (Jan 27, 2026): Extracted to primal_client module for reuse
//
// All primal communication uses:
// - Pure Rust Unix socket JSON-RPC (no C dependencies)
// - Capability-based discovery via SystemPaths
// - Runtime primal discovery (no hardcoded paths)
// ═══════════════════════════════════════════════════════════════════════════

use crate::primal_client::{
    BearDogClient, NestGateClient, PetalTongueClient, PrimalClient, SongbirdClient, SquirrelClient,
    ToadStoolClient,
};

/// Interactive UI Orchestrator
///
/// This is the main coordinator that creates the network effect by connecting
/// multiple primals together to provide an interactive UI.
///
/// ## Network Effect Architecture
///
/// This orchestrator doesn't "own" the UI functionality. Instead, it creates
/// an emergent capability by coordinating between 7 primals:
///
/// - **petalTongue**: UI framework and rendering
/// - **Songbird**: Device/primal discovery and registry
/// - **BearDog**: Authorization and security
/// - **NestGate**: Configuration persistence
/// - **ToadStool**: Resource metrics
/// - **Squirrel**: AI suggestions
/// - **biomeOS** (this orchestrator): Coordination layer
///
/// Value = n² (Metcalfe's Law) = 7² = 49 potential interactions!
pub struct InteractiveUIOrchestrator {
    /// UI state
    state: Arc<RwLock<UIState>>,

    /// Event broadcaster
    events: EventBroadcaster,

    /// Primal clients (discovered at runtime via capabilities)
    petaltongue: Option<PetalTongueClient>,
    songbird: Option<SongbirdClient>,
    beardog: Option<BearDogClient>,
    nestgate: Option<NestGateClient>,
    toadstool: Option<ToadStoolClient>,
    squirrel: Option<SquirrelClient>,

    /// Family ID for primal discovery
    family_id: String,
}

impl InteractiveUIOrchestrator {
    /// Create a new orchestrator
    ///
    /// This will discover and connect to all available primals using
    /// capability-based discovery. No hardcoded dependencies!
    ///
    /// ## TRUE PRIMAL Compliance
    ///
    /// - Discovers primals at runtime
    /// - Uses capabilities, not names
    /// - Gracefully handles missing primals
    /// - No compile-time coupling
    pub async fn new(family_id: impl Into<String>) -> Result<Self> {
        let family_id = family_id.into();
        let state = Arc::new(RwLock::new(UIState::new()));
        let events = EventBroadcaster::new();

        info!(
            "Creating Interactive UI Orchestrator for family: {}",
            family_id
        );

        Ok(Self {
            state,
            events,
            petaltongue: None,
            songbird: None,
            beardog: None,
            nestgate: None,
            toadstool: None,
            squirrel: None,
            family_id,
        })
    }

    /// Discover and connect to all primals
    ///
    /// Uses capability-based discovery to find primals. No hardcoded assumptions!
    async fn discover_primals(&mut self) -> Result<()> {
        info!("Discovering primals via capability-based discovery...");

        // Try to discover each primal by capability
        // Note: These discoveries are independent and fail gracefully
        // TRUE PRIMAL: Uses XDG-compliant Unix socket discovery via AtomicClient

        // 1. Discover visualization primal (petalTongue)
        info!("Attempting to discover visualization primal...");
        self.petaltongue = PrimalClient::discover("petaltongue").await.ok();

        // 2. Discover service registry primal (Songbird)
        info!("Attempting to discover service registry primal...");
        self.songbird = PrimalClient::discover("songbird").await.ok();

        // 3. Discover security primal (BearDog)
        info!("Attempting to discover security primal...");
        self.beardog = PrimalClient::discover("beardog").await.ok();

        // 4. Discover storage primal (NestGate)
        info!("Attempting to discover storage primal...");
        self.nestgate = PrimalClient::discover("nestgate").await.ok();

        // 5. Discover compute primal (ToadStool)
        info!("Attempting to discover compute primal...");
        self.toadstool = PrimalClient::discover("toadstool").await.ok();

        // 6. Discover AI primal (Squirrel)
        info!("Attempting to discover AI primal...");
        self.squirrel = PrimalClient::discover("squirrel").await.ok();

        let discovered_count = [
            self.petaltongue.is_some(),
            self.songbird.is_some(),
            self.beardog.is_some(),
            self.nestgate.is_some(),
            self.toadstool.is_some(),
            self.squirrel.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        info!("Discovered {}/6 primals", discovered_count);

        if discovered_count == 0 {
            warn!("No primals discovered! UI will have limited functionality.");
        }

        Ok(())
    }

    /// Discover devices from available primals
    ///
    /// Uses Songbird's device registry if available. Falls back gracefully.
    async fn discover_devices(&self) -> Result<()> {
        info!("Discovering devices...");

        if let Some(ref songbird) = self.songbird {
            // Query Songbird for registered devices using JSON-RPC
            match songbird
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                Ok(devices) => {
                    debug!("Discovered devices: {:?}", devices);
                    info!("Successfully discovered devices from Songbird");
                }
                Err(e) => {
                    warn!("Device discovery failed: {} - Songbird may not support device registry yet", e);
                }
            }
        } else {
            info!("No Songbird available for device discovery");
        }

        Ok(())
    }

    /// Discover active primals
    ///
    /// Uses Songbird's primal registry to get list of active primals.
    async fn discover_active_primals(&self) -> Result<()> {
        info!("Discovering active primals...");

        if let Some(ref songbird) = self.songbird {
            // Query Songbird for all registered primals using JSON-RPC
            match songbird
                .call("registry.list_primals", serde_json::json!({}))
                .await
            {
                Ok(primals) => {
                    debug!("Discovered primals: {:?}", primals);
                    info!("Successfully queried Songbird for active primals");
                }
                Err(e) => {
                    warn!("Primal discovery failed: {} - check Songbird connection", e);
                }
            }
        } else {
            info!("No Songbird available, cannot discover other primals");
        }

        Ok(())
    }

    /// Load saved state from NestGate
    async fn load_saved_state(&self) -> Result<()> {
        info!("Loading saved UI state...");

        if let Some(ref nestgate) = self.nestgate {
            // Try to load previous UI state from NestGate using JSON-RPC
            match nestgate
                .call(
                    "storage.retrieve",
                    serde_json::json!({
                        "key": format!("ui_state:{}", self.family_id)
                    }),
                )
                .await
            {
                Ok(state) => {
                    debug!("Loaded saved state: {:?}", state);
                    info!("Successfully loaded saved UI state from NestGate");
                }
                Err(e) => {
                    debug!("No saved state found or error: {}", e);
                    info!("Starting with fresh state (no previous state found)");
                }
            }
        } else {
            info!("No storage primal available, starting with fresh state");
        }

        Ok(())
    }

    /// Start the orchestrator
    ///
    /// This will:
    /// 1. Discover all primals (capability-based, no hardcoding)
    /// 2. Discover devices and active primals
    /// 3. Load saved state if available
    /// 4. Launch petalTongue UI if available
    /// 5. Sync initial state to UI
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting Interactive UI Orchestrator...");

        // Phase 1: Discover all primals (TRUE PRIMAL - runtime discovery!)
        self.discover_primals().await?;

        // Phase 2: Discover devices and primals
        self.discover_devices().await?;
        self.discover_active_primals().await?;

        // Phase 3: Load saved state
        self.load_saved_state().await?;

        // Phase 4: Launch UI if petalTongue is available
        if self.petaltongue.is_some() {
            info!("✅ petalTongue available - UI will be rendered");
        } else {
            warn!("⚠️  No petalTongue available - running headless");
        }

        // Phase 5: Sync initial state to petalTongue
        if let Some(ref petaltongue) = self.petaltongue {
            let initial_state = self.build_initial_ui_state().await;
            match petaltongue.call("ui.initialize", initial_state).await {
                Ok(_) => info!("✅ Initial UI state pushed to petalTongue"),
                Err(e) => warn!("⚠️ Failed to push initial state: {}", e),
            }
        }

        info!("✅ Interactive UI Orchestrator started successfully!");

        Ok(())
    }

    /// Get the current user ID from BearDog session or environment
    ///
    /// Falls back to "anonymous" if no session is available.
    async fn get_current_user_id(&self) -> String {
        // Try to get from BearDog session
        if let Some(ref beardog) = self.beardog {
            if let Ok(result) = beardog
                .call("auth.get_current_user", serde_json::json!({}))
                .await
            {
                if let Some(user_id) = result.get("user_id").and_then(|v| v.as_str()) {
                    return user_id.to_string();
                }
            }
        }

        // Fall back to environment variable
        if let Ok(user) = std::env::var("BIOMEOS_USER") {
            return user;
        }

        // Fall back to system user
        if let Ok(user) = std::env::var("USER") {
            return user;
        }

        // Default to anonymous
        "anonymous".to_string()
    }

    /// Build the initial UI state from discovered primals
    async fn build_initial_ui_state(&self) -> serde_json::Value {
        let mut state = serde_json::json!({
            "family_id": self.family_id,
            "primals": {
                "petaltongue": self.petaltongue.is_some(),
                "songbird": self.songbird.is_some(),
                "beardog": self.beardog.is_some(),
                "nestgate": self.nestgate.is_some(),
                "toadstool": self.toadstool.is_some(),
                "squirrel": self.squirrel.is_some()
            },
            "devices": [],
            "assignments": []
        });

        // Fetch devices from Songbird if available
        if let Some(ref songbird) = self.songbird {
            if let Ok(devices) = songbird
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                state["devices"] = devices;
            }
        }

        // Fetch assignments from NestGate if available
        if let Some(ref nestgate) = self.nestgate {
            if let Ok(assignments) = nestgate
                .call(
                    "storage.list",
                    serde_json::json!({ "key_prefix": "assignment:" }),
                )
                .await
            {
                state["assignments"] = assignments;
            }
        }

        state
    }

    /// Handle a user action
    ///
    /// Actions come from the UI (petalTongue) and are processed here.
    /// The orchestrator coordinates between multiple primals to fulfill the action.
    pub async fn handle_user_action(&self, action: UserAction) -> Result<ActionResult> {
        debug!(?action, "Handling user action");

        match action {
            UserAction::AssignDevice {
                device_id,
                primal_id,
            } => self.handle_assign_device(&device_id, &primal_id).await,

            UserAction::UnassignDevice { device_id } => {
                self.handle_unassign_device(&device_id).await
            }

            UserAction::StartPrimal { primal_name } => self.handle_start_primal(&primal_name).await,

            UserAction::StopPrimal { primal_id } => self.handle_stop_primal(&primal_id).await,

            UserAction::RestartPrimal { primal_id } => self.handle_restart_primal(&primal_id).await,

            UserAction::AcceptSuggestion { suggestion_id } => {
                self.handle_accept_suggestion(&suggestion_id).await
            }

            UserAction::DismissSuggestion { suggestion_id } => {
                self.handle_dismiss_suggestion(&suggestion_id).await
            }

            UserAction::Refresh => self.handle_refresh().await,
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
    ///
    /// This is the network effect in action!
    async fn handle_assign_device(&self, device_id: &str, primal_id: &str) -> Result<ActionResult> {
        info!(
            "🎯 Device assignment requested: {} → {}",
            device_id, primal_id
        );

        // Phase 1: Authorization via BearDog
        let current_user = self.get_current_user_id().await;
        let auth_result = self
            .authorize_device_assignment(&current_user, device_id, primal_id)
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
        let validation_result = self.validate_device_assignment(device_id, primal_id).await;

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
        let capacity_result = self.check_primal_capacity(device_id, primal_id).await;

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
        let assignment_id = match self.register_assignment(device_id, primal_id).await {
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
        if let Err(e) = self
            .persist_assignment(&assignment_id, device_id, primal_id)
            .await
        {
            warn!("⚠️ Failed to persist assignment: {}, continuing", e);
            // Non-critical: assignment still works, just won't survive restart
        } else {
            info!("✅ Assignment persisted");
        }

        // Phase 6: Update UI via petalTongue (non-critical)
        if let Err(e) = self.update_ui_after_assignment(device_id, primal_id).await {
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

    /// Authorize device assignment via BearDog
    ///
    /// ## Network Effect Phase 1: Authorization
    ///
    /// Checks:
    /// - User has permission to assign this device
    /// - Primal accepts this device type
    ///
    /// ## Graceful Degradation
    ///
    /// If BearDog is not available, authorization is granted by default.
    /// This allows the system to function without security, useful for:
    /// - Development environments
    /// - Single-user systems
    /// - Degraded operation mode
    async fn authorize_device_assignment(
        &self,
        user_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<AuthorizationResult> {
        debug!(
            "Authorizing device assignment: user={}, device={}, primal={}",
            user_id, device_id, primal_id
        );

        // Check if BearDog is available
        if let Some(ref beardog) = self.beardog {
            info!("🔒 BearDog available - checking authorization");

            // Call BearDog to check authorization
            match beardog
                .call(
                    "auth.check_device_assignment",
                    serde_json::json!({
                        "user_id": user_id,
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("authorized")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        info!("✅ BearDog authorization: Approved");
                        Ok(AuthorizationResult::Authorized)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Authorization denied")
                            .to_string();
                        info!("❌ BearDog authorization: Denied - {}", reason);
                        Ok(AuthorizationResult::Denied(reason))
                    }
                }
                Err(e) => {
                    // BearDog might not support this method yet
                    warn!("⚠️ BearDog call failed: {} - falling back to allow", e);
                    info!("✅ BearDog authorization: Approved (fallback)");
                    Ok(AuthorizationResult::Authorized)
                }
            }
        } else {
            warn!("⚠️ No security primal (BearDog) available");
            warn!("⚠️ Allowing assignment without authorization (graceful degradation)");
            info!("✅ Authorization: Approved (no security primal)");
            Ok(AuthorizationResult::Authorized)
        }
    }

    /// Validate device assignment via Songbird
    ///
    /// ## Network Effect Phase 2: Validation
    ///
    /// Checks:
    /// - Device is available (not already assigned)
    /// - Primal is healthy and running
    /// - No conflicts with existing assignments
    ///
    /// ## Graceful Degradation
    ///
    /// If Songbird is not available, validation passes by default.
    async fn validate_device_assignment(
        &self,
        device_id: &str,
        primal_id: &str,
    ) -> Result<ValidationResult> {
        debug!(
            "Validating device assignment: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(ref songbird) = self.songbird {
            info!("🎵 Songbird available - checking validation");

            // Call Songbird to validate the assignment
            match songbird
                .call(
                    "registry.validate_assignment",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("valid")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                    {
                        info!("✅ Songbird validation: Passed");
                        Ok(ValidationResult::Valid)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Validation failed")
                            .to_string();
                        info!("❌ Songbird validation: Failed - {}", reason);
                        Ok(ValidationResult::Invalid(reason))
                    }
                }
                Err(e) => {
                    // Songbird might not support this method yet
                    warn!("⚠️ Songbird call failed: {} - falling back to valid", e);
                    info!("✅ Songbird validation: Passed (fallback)");
                    Ok(ValidationResult::Valid)
                }
            }
        } else {
            warn!("⚠️ No service registry (Songbird) available");
            warn!("⚠️ Allowing assignment without validation (graceful degradation)");
            info!("✅ Validation: Passed (no service registry)");
            Ok(ValidationResult::Valid)
        }
    }

    /// Check primal capacity via ToadStool
    ///
    /// ## Network Effect Phase 3: Capacity Check
    ///
    /// Checks:
    /// - Primal has capacity for device
    /// - Resource requirements can be met
    ///
    /// ## Graceful Degradation
    ///
    /// If ToadStool is not available, capacity check passes by default.
    async fn check_primal_capacity(
        &self,
        device_id: &str,
        primal_id: &str,
    ) -> Result<CapacityResult> {
        debug!(
            "Checking primal capacity: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(ref toadstool) = self.toadstool {
            info!("🍄 ToadStool available - checking capacity");

            // Call ToadStool to check resource capacity
            match toadstool
                .call(
                    "compute.check_capacity",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("available")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                    {
                        info!("✅ ToadStool capacity: Available");
                        Ok(CapacityResult::Available)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Insufficient capacity")
                            .to_string();
                        info!("❌ ToadStool capacity: Insufficient - {}", reason);
                        Ok(CapacityResult::Insufficient { reason })
                    }
                }
                Err(e) => {
                    // ToadStool might not support this method yet
                    warn!(
                        "⚠️ ToadStool call failed: {} - falling back to available",
                        e
                    );
                    info!("✅ ToadStool capacity: Available (fallback)");
                    Ok(CapacityResult::Available)
                }
            }
        } else {
            warn!("⚠️ No compute primal (ToadStool) available");
            warn!("⚠️ Allowing assignment without capacity check (graceful degradation)");
            info!("✅ Capacity: Available (no compute primal)");
            Ok(CapacityResult::Available)
        }
    }

    /// Register assignment via Songbird
    ///
    /// ## Network Effect Phase 4: Register Assignment
    ///
    /// Creates the assignment record in the service registry.
    /// Returns assignment ID for tracking.
    ///
    /// ## Graceful Degradation
    ///
    /// If Songbird is not available, generates local assignment ID.
    async fn register_assignment(&self, device_id: &str, primal_id: &str) -> Result<String> {
        debug!(
            "Registering assignment: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(ref songbird) = self.songbird {
            info!("🎵 Songbird available - registering assignment");

            // Call Songbird to register the assignment
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

            let assignment_id = format!("local-{}-{}", device_id, primal_id);
            info!(
                "✅ Registered locally (Songbird fallback): {}",
                assignment_id
            );
            Ok(assignment_id)
        } else {
            warn!("⚠️ No service registry available");
            let assignment_id = format!("local-{}-{}", device_id, primal_id);
            info!("✅ Registered locally: {}", assignment_id);
            Ok(assignment_id)
        }
    }

    /// Persist assignment via NestGate
    ///
    /// ## Network Effect Phase 5: Persist Assignment
    ///
    /// Stores assignment for recovery after restart.
    ///
    /// ## Graceful Degradation
    ///
    /// If NestGate is not available, assignment is not persisted
    /// but the operation continues successfully.
    async fn persist_assignment(
        &self,
        assignment_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<()> {
        debug!(
            "Persisting assignment: id={}, device={}, primal={}",
            assignment_id, device_id, primal_id
        );

        if let Some(ref nestgate) = self.nestgate {
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
                            "family_id": self.family_id
                        }
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Persisted via NestGate");
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "⚠️ NestGate storage failed: {} - continuing without persistence",
                        e
                    );
                    return Ok(());
                }
            }
        } else {
            warn!("⚠️ No storage primal available, assignment not persisted");
            Err(anyhow::anyhow!("No storage primal available"))
        }
    }

    /// Update UI via petalTongue
    ///
    /// ## Network Effect Phase 6: Update UI
    ///
    /// Pushes topology update and shows success notification.
    ///
    /// ## Graceful Degradation
    ///
    /// If petalTongue is not available, UI is not updated
    /// but the operation continues successfully.
    async fn update_ui_after_assignment(&self, device_id: &str, primal_id: &str) -> Result<()> {
        debug!("Updating UI: device={}, primal={}", device_id, primal_id);

        if let Some(ref petaltongue) = self.petaltongue {
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
                    return Ok(());
                }
                Err(e) => {
                    warn!("⚠️ petalTongue update failed: {} - continuing", e);
                    return Ok(());
                }
            }
        } else {
            warn!("⚠️ No visualization primal available, UI not updated");
            Err(anyhow::anyhow!("No visualization primal available"))
        }
    }

    /// Handle device unassignment
    ///
    /// Removes device-primal assignment from registry and persistence.
    async fn handle_unassign_device(&self, device_id: &str) -> Result<ActionResult> {
        info!("Unassigning device {}", device_id);

        // Step 1: Remove from Songbird registry
        if let Some(ref songbird) = self.songbird {
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
        if let Some(ref nestgate) = self.nestgate {
            match nestgate
                .call(
                    "storage.delete",
                    serde_json::json!({ "key_prefix": format!("assignment:*-{}", device_id) }),
                )
                .await
            {
                Ok(_) => info!("✅ Removed assignment from NestGate"),
                Err(e) => warn!("⚠️ NestGate delete failed: {}", e),
            }
        }

        // Step 3: Update UI
        if let Some(ref petaltongue) = self.petaltongue {
            let _ = petaltongue
                .call(
                    "ui.update_topology",
                    serde_json::json!({
                        "event": "device_unassigned",
                        "device_id": device_id
                    }),
                )
                .await;
        }

        Ok(ActionResult::success(format!(
            "Device {} unassigned successfully",
            device_id
        )))
    }

    /// Handle primal start
    ///
    /// Requests ToadStool to start a primal process.
    async fn handle_start_primal(&self, primal_name: &str) -> Result<ActionResult> {
        info!("Starting primal {}", primal_name);

        if let Some(ref toadstool) = self.toadstool {
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
    ///
    /// Requests ToadStool to stop a primal process gracefully.
    async fn handle_stop_primal(&self, primal_id: &str) -> Result<ActionResult> {
        info!("Stopping primal {}", primal_id);

        if let Some(ref toadstool) = self.toadstool {
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
    ///
    /// Stops then starts a primal via ToadStool.
    async fn handle_restart_primal(&self, primal_id: &str) -> Result<ActionResult> {
        info!("Restarting primal {}", primal_id);

        if let Some(ref toadstool) = self.toadstool {
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
    ///
    /// Notifies Squirrel that a suggestion was accepted for learning.
    async fn handle_accept_suggestion(&self, suggestion_id: &str) -> Result<ActionResult> {
        info!("Accepting suggestion {}", suggestion_id);

        if let Some(ref squirrel) = self.squirrel {
            match squirrel
                .call(
                    "ai.accept_suggestion",
                    serde_json::json!({
                        "suggestion_id": suggestion_id,
                        "family_id": self.family_id
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
    ///
    /// Notifies Squirrel that a suggestion was dismissed for learning.
    async fn handle_dismiss_suggestion(&self, suggestion_id: &str) -> Result<ActionResult> {
        info!("Dismissing suggestion {}", suggestion_id);

        if let Some(ref squirrel) = self.squirrel {
            match squirrel
                .call(
                    "ai.dismiss_suggestion",
                    serde_json::json!({
                        "suggestion_id": suggestion_id,
                        "family_id": self.family_id
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
    ///
    /// Re-discovers all primals and refreshes UI state.
    async fn handle_refresh(&self) -> Result<ActionResult> {
        info!("Refreshing UI state");

        // Gather fresh data from all primals
        let mut refresh_results = Vec::new();

        // Refresh device list from Songbird
        if let Some(ref songbird) = self.songbird {
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
        if let Some(ref toadstool) = self.toadstool {
            match toadstool
                .call("compute.get_metrics", serde_json::json!({}))
                .await
            {
                Ok(_) => refresh_results.push("metrics"),
                Err(e) => warn!("Failed to refresh metrics: {}", e),
            }
        }

        // Push refresh to UI
        if let Some(ref petaltongue) = self.petaltongue {
            let _ = petaltongue
                .call(
                    "ui.refresh",
                    serde_json::json!({ "refreshed": refresh_results }),
                )
                .await;
        }

        Ok(ActionResult::success(format!(
            "UI refreshed ({} sources updated)",
            refresh_results.len()
        )))
    }

    /// Run the orchestrator event loop
    ///
    /// This listens for events from primals and pushes updates to the UI.
    pub async fn run(&mut self) -> Result<()> {
        info!("Running Interactive UI Orchestrator event loop...");

        // Subscribe to Songbird events if available
        if let Some(ref songbird) = self.songbird {
            match songbird
                .call(
                    "events.subscribe",
                    serde_json::json!({
                        "events": ["primal.started", "primal.stopped", "device.connected", "device.disconnected"]
                    }),
                )
                .await
            {
                Ok(_) => info!("✅ Subscribed to Songbird events"),
                Err(e) => warn!("⚠️ Failed to subscribe to events: {}", e),
            }
        }

        // Main event loop - poll for updates periodically
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Check for pending events from Songbird
            if let Some(ref songbird) = self.songbird {
                if let Ok(events) = songbird.call("events.poll", serde_json::json!({})).await {
                    if let Some(event_list) = events.as_array() {
                        for event in event_list {
                            self.handle_primal_event(event);
                        }
                    }
                }
            }

            // Push any state updates to petalTongue
            if let Some(ref petaltongue) = self.petaltongue {
                // Signal that state sync is available
                let _ = petaltongue
                    .call("ui.heartbeat", serde_json::json!({ "status": "running" }))
                    .await;
            }
        }
    }

    /// Handle an event from a primal
    fn handle_primal_event(&self, event: &serde_json::Value) {
        use crate::events::UIEvent;

        let event_type = event
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match event_type {
            "primal.started" => {
                if let Some(name) = event.get("primal_name").and_then(|v| v.as_str()) {
                    info!("📢 Primal started: {}", name);
                    self.events.emit(UIEvent::PrimalStatusChanged {
                        primal_id: name.to_string(),
                        status: "started".to_string(),
                    });
                }
            }
            "primal.stopped" => {
                if let Some(name) = event.get("primal_name").and_then(|v| v.as_str()) {
                    info!("📢 Primal stopped: {}", name);
                    self.events.emit(UIEvent::PrimalStatusChanged {
                        primal_id: name.to_string(),
                        status: "stopped".to_string(),
                    });
                }
            }
            "device.connected" => {
                if let Some(id) = event.get("device_id").and_then(|v| v.as_str()) {
                    info!("📢 Device connected: {}", id);
                    self.events.emit(UIEvent::DeviceStatusChanged {
                        device_id: id.to_string(),
                        status: "connected".to_string(),
                    });
                }
            }
            "device.disconnected" => {
                if let Some(id) = event.get("device_id").and_then(|v| v.as_str()) {
                    info!("📢 Device disconnected: {}", id);
                    self.events.emit(UIEvent::DeviceStatusChanged {
                        device_id: id.to_string(),
                        status: "disconnected".to_string(),
                    });
                }
            }
            _ => {
                debug!("Unknown event type: {}", event_type);
            }
        }
    }

    /// Get a reference to the UI state
    pub fn state(&self) -> &Arc<RwLock<UIState>> {
        &self.state
    }

    /// Get a reference to the event broadcaster
    pub fn events(&self) -> &EventBroadcaster {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = InteractiveUIOrchestrator::new("test-family").await;
        assert!(orchestrator.is_ok());
    }

    #[tokio::test]
    async fn test_orchestrator_start_graceful_degradation() {
        // Should start even with no primals available
        let mut orchestrator = InteractiveUIOrchestrator::new("test-family").await.unwrap();

        let result = orchestrator.start().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_user_action_assign_device() {
        let orchestrator = InteractiveUIOrchestrator::new("test-family").await.unwrap();

        let result = orchestrator
            .handle_user_action(UserAction::AssignDevice {
                device_id: "test-device".to_string(),
                primal_id: "test-primal".to_string(),
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }
}
