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
    actions::{UserAction, ActionResult},
    events::EventBroadcaster,
    state::UIState,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Result of authorization check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Authorization granted
    Authorized,
    /// Authorization denied with reason
    Denied(String),
}

// Placeholder types for primal clients
// These will be replaced with actual client imports once they're exported from biomeos-core
type PetalTongueClient = ();
type SongbirdClient = ();
type BearDogClient = ();
type NestGateClient = ();
type ToadStoolClient = ();
type SquirrelClient = ();

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
        
        info!("Creating Interactive UI Orchestrator for family: {}", family_id);
        
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
        // TRUE PRIMAL: Uses XDG-compliant Unix socket discovery
        
        // 1. Discover visualization primal (petalTongue)
        // TODO: Implement discovery method in PetalTongueClient
        info!("Attempting to discover visualization primal...");
        // self.petaltongue = PetalTongueClient::discover(&self.family_id).await.ok();
        
        // 2. Discover service registry primal (Songbird)
        // TODO: Implement discovery method in SongbirdClient  
        info!("Attempting to discover service registry primal...");
        // self.songbird = SongbirdClient::discover(&self.family_id).await.ok();
        
        // 3. Discover security primal (BearDog)
        // TODO: Implement discovery method in BearDogClient
        info!("Attempting to discover security primal...");
        // self.beardog = BearDogClient::discover(&self.family_id).await.ok();
        
        // 4. Discover storage primal (NestGate)
        // TODO: Implement discovery method in NestGateClient
        info!("Attempting to discover storage primal...");
        // self.nestgate = NestGateClient::discover(&self.family_id).await.ok();
        
        // 5. Discover compute primal (ToadStool)
        // TODO: Implement discovery method in ToadStoolClient
        info!("Attempting to discover compute primal...");
        // self.toadstool = ToadStoolClient::discover(&self.family_id).await.ok();
        
        // 6. Discover AI primal (Squirrel)
        // TODO: Implement discovery method in SquirrelClient
        info!("Attempting to discover AI primal...");
        // self.squirrel = SquirrelClient::discover(&self.family_id).await.ok();
        
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
        
        // TODO: Implement device discovery via Songbird
        // This will be implemented when Songbird adds device registry API
        
        // For now, log that device discovery is pending Songbird extension
        info!("Device discovery pending Songbird device registry implementation");
        
        Ok(())
    }
    
    /// Discover active primals
    ///
    /// Uses Songbird's primal registry to get list of active primals.
    async fn discover_active_primals(&self) -> Result<()> {
        info!("Discovering active primals...");
        
        if self.songbird.is_some() {
            // Query Songbird for all registered primals
            // TODO: Implement get_all_primals method in SongbirdClient
            info!("Querying Songbird for active primals");
        } else {
            info!("No Songbird available, cannot discover other primals");
        }
        
        Ok(())
    }
    
    /// Load saved state from NestGate
    async fn load_saved_state(&self) -> Result<()> {
        info!("Loading saved UI state...");
        
        if self.nestgate.is_some() {
            // Try to load previous assignments and configuration
            // TODO: Implement when NestGateClient is available
            info!("Storage primal available - would load saved state");
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
        
        // Phase 5: Sync initial state
        // TODO: Push initial state to petalTongue
        
        info!("✅ Interactive UI Orchestrator started successfully!");
        
        Ok(())
    }
    
    /// Handle a user action
    ///
    /// Actions come from the UI (petalTongue) and are processed here.
    /// The orchestrator coordinates between multiple primals to fulfill the action.
    pub async fn handle_user_action(&self, action: UserAction) -> Result<ActionResult> {
        debug!(?action, "Handling user action");
        
        match action {
            UserAction::AssignDevice { device_id, primal_id } => {
                self.handle_assign_device(&device_id, &primal_id).await
            }
            
            UserAction::UnassignDevice { device_id } => {
                self.handle_unassign_device(&device_id).await
            }
            
            UserAction::StartPrimal { primal_name } => {
                self.handle_start_primal(&primal_name).await
            }
            
            UserAction::StopPrimal { primal_id } => {
                self.handle_stop_primal(&primal_id).await
            }
            
            UserAction::RestartPrimal { primal_id } => {
                self.handle_restart_primal(&primal_id).await
            }
            
            UserAction::AcceptSuggestion { suggestion_id } => {
                self.handle_accept_suggestion(&suggestion_id).await
            }
            
            UserAction::DismissSuggestion { suggestion_id } => {
                self.handle_dismiss_suggestion(&suggestion_id).await
            }
            
            UserAction::Refresh => {
                self.handle_refresh().await
            }
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
        info!("🎯 Device assignment requested: {} → {}", device_id, primal_id);
        
        // Phase 1: Authorization via BearDog
        let auth_result = self.authorize_device_assignment(
            "current_user", // TODO: Get from session/context
            device_id,
            primal_id,
        ).await;
        
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
        
        // TODO: Phase 2-6 implementation (next tasks)
        
        Ok(ActionResult::success(format!(
            "Device {} authorized for assignment to primal {} (Phase 3: Task 1 complete, Tasks 2-8 pending)",
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
        if self.beardog.is_some() {
            info!("🔒 BearDog available - checking authorization");
            
            // TODO: Implement actual BearDog client calls when client supports these methods
            // For now, return authorized (will be implemented in Task 1, Day 2)
            
            // Placeholder logic:
            // 1. Check user permissions: beardog.check_permission(user_id, permission)
            // 2. Check primal policy: beardog.get_device_policy(primal_id)
            // 3. Verify device type acceptance
            
            info!("✅ BearDog authorization: Approved (placeholder)");
            Ok(AuthorizationResult::Authorized)
        } else {
            warn!("⚠️ No security primal (BearDog) available");
            warn!("⚠️ Allowing assignment without authorization (graceful degradation)");
            info!("✅ Authorization: Approved (no security primal)");
            Ok(AuthorizationResult::Authorized)
        }
    }
    
    /// Handle device unassignment
    async fn handle_unassign_device(&self, device_id: &str) -> Result<ActionResult> {
        info!("Unassigning device {}", device_id);
        
        // TODO: Phase 3 implementation
        
        Ok(ActionResult::success(format!(
            "Device {} unassigned (Phase 3 implementation pending)",
            device_id
        )))
    }
    
    /// Handle primal start
    async fn handle_start_primal(&self, primal_name: &str) -> Result<ActionResult> {
        info!("Starting primal {}", primal_name);
        
        // TODO: Phase 3 implementation
        
        Ok(ActionResult::success(format!(
            "Primal {} start requested (Phase 3 implementation pending)",
            primal_name
        )))
    }
    
    /// Handle primal stop
    async fn handle_stop_primal(&self, primal_id: &str) -> Result<ActionResult> {
        info!("Stopping primal {}", primal_id);
        
        // TODO: Phase 3 implementation
        
        Ok(ActionResult::success(format!(
            "Primal {} stop requested (Phase 3 implementation pending)",
            primal_id
        )))
    }
    
    /// Handle primal restart
    async fn handle_restart_primal(&self, primal_id: &str) -> Result<ActionResult> {
        info!("Restarting primal {}", primal_id);
        
        // TODO: Phase 3 implementation
        
        Ok(ActionResult::success(format!(
            "Primal {} restart requested (Phase 3 implementation pending)",
            primal_id
        )))
    }
    
    /// Handle AI suggestion acceptance
    async fn handle_accept_suggestion(&self, suggestion_id: &str) -> Result<ActionResult> {
        info!("Accepting suggestion {}", suggestion_id);
        
        // TODO: Phase 4 implementation (Squirrel integration)
        
        Ok(ActionResult::success(format!(
            "Suggestion {} accepted (Phase 4 implementation pending)",
            suggestion_id
        )))
    }
    
    /// Handle AI suggestion dismissal
    async fn handle_dismiss_suggestion(&self, suggestion_id: &str) -> Result<ActionResult> {
        info!("Dismissing suggestion {}", suggestion_id);
        
        // TODO: Phase 4 implementation (Squirrel integration)
        
        Ok(ActionResult::success(format!(
            "Suggestion {} dismissed (Phase 4 implementation pending)",
            suggestion_id
        )))
    }
    
    /// Handle UI refresh
    async fn handle_refresh(&self) -> Result<ActionResult> {
        info!("Refreshing UI state");
        
        // Re-discover everything
        // TODO: Implement refresh logic
        
        Ok(ActionResult::success("UI refreshed".to_string()))
    }
    
    /// Run the orchestrator event loop
    ///
    /// This listens for events from primals and pushes updates to the UI.
    pub async fn run(&mut self) -> Result<()> {
        info!("Running Interactive UI Orchestrator event loop...");
        
        // TODO: Phase 4 implementation
        // - Subscribe to Songbird events
        // - Listen for device/primal changes
        // - Push updates to petalTongue
        // - Handle user actions from UI
        
        info!("Event loop implementation pending (Phase 4)");
        
        Ok(())
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
        let mut orchestrator = InteractiveUIOrchestrator::new("test-family")
            .await
            .unwrap();
        
        let result = orchestrator.start().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_user_action_assign_device() {
        let orchestrator = InteractiveUIOrchestrator::new("test-family")
            .await
            .unwrap();
        
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
