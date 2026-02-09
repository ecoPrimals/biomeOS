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
//!
//! ## Architecture (Refactored Jan 30, 2026)
//!
//! Smart domain-driven refactoring into 7 focused modules:
//! - `action_handler`: User action coordination
//! - `authorization`: BearDog authorization checks
//! - `validation`: Songbird validation logic
//! - `capacity`: ToadStool capacity checks
//! - `discovery`: Runtime primal/device discovery
//! - `persistence`: NestGate data persistence
//! - `ui_sync`: petalTongue UI updates

use crate::{
    actions::{ActionResult, UserAction},
    events::EventBroadcaster,
    state::UIState,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Domain modules (Jan 30, 2026 refactor)
pub mod action_handler;
pub mod authorization;
pub mod capacity;
pub mod discovery;
pub mod persistence;
pub mod ui_sync;
pub mod validation;

use action_handler::ActionHandler;
use discovery::Discovery;
use ui_sync::UISync;

// ═══════════════════════════════════════════════════════════════════════════
// PRIMAL CLIENTS - Using AtomicClient for Pure Rust JSON-RPC
// ═══════════════════════════════════════════════════════════════════════════
//
// ✅ EVOLVED (Jan 27, 2026): Extracted to primal_client module for reuse
// ✅ EVOLVED (Jan 30, 2026): Refactored into domain-driven modules
//
// All primal communication uses:
// - Pure Rust Unix socket JSON-RPC (no C dependencies)
// - Capability-based discovery via SystemPaths
// - Runtime primal discovery (no hardcoded paths)
// ═══════════════════════════════════════════════════════════════════════════

use crate::primal_client::PrimalConnections;

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

    /// Dynamic primal connections (DEEP DEBT EVOLUTION: replaced 6 fixed fields)
    connections: PrimalConnections,

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
            connections: PrimalConnections::default(),
            family_id,
        })
    }

    /// Discover and connect to all primals
    ///
    /// Uses capability-based discovery to find primals. No hardcoded assumptions!
    async fn discover_primals(&mut self) -> Result<()> {
        let result = Discovery::discover_primals().await?;
        self.connections = result.connections;
        Ok(())
    }

    /// Discover devices from available primals
    async fn discover_devices(&self) -> Result<()> {
        Discovery::discover_devices(&self.connections).await
    }

    /// Discover active primals
    async fn discover_active_primals(&self) -> Result<()> {
        Discovery::discover_active_primals(&self.connections).await
    }

    /// Load saved state from NestGate
    async fn load_saved_state(&self) -> Result<()> {
        Discovery::load_saved_state(&self.connections, &self.family_id).await
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
        if self.connections.petaltongue().is_some() {
            info!("✅ petalTongue available - UI will be rendered");
        } else {
            warn!("⚠️  No petalTongue available - running headless");
        }

        // Phase 5: Sync initial state to petalTongue
        let initial_state = self.build_initial_ui_state().await;
        let petaltongue = self.connections.petaltongue().cloned();
        let _ = UISync::initialize_ui(&petaltongue, initial_state).await;

        info!("✅ Interactive UI Orchestrator started successfully!");

        Ok(())
    }

    /// Build the initial UI state from discovered primals
    async fn build_initial_ui_state(&self) -> serde_json::Value {
        Discovery::build_initial_ui_state(
            &self.family_id,
            &self.connections,
        )
        .await
    }

    /// Handle a user action
    ///
    /// Actions come from the UI (petalTongue) and are processed here.
    /// The orchestrator coordinates between multiple primals to fulfill the action.
    pub async fn handle_user_action(&self, action: UserAction) -> Result<ActionResult> {
        debug!(?action, "Handling user action");

        ActionHandler::handle_user_action(
            action,
            &self.family_id,
            &self.connections,
        )
        .await
    }

    /// Run the orchestrator event loop
    ///
    /// This listens for events from primals and pushes updates to the UI.
    pub async fn run(&mut self) -> Result<()> {
        info!("Running Interactive UI Orchestrator event loop...");

        // Subscribe to registry provider events if available
        let registry_name = std::env::var("BIOMEOS_REGISTRY_PROVIDER")
            .unwrap_or_else(|_| "songbird".to_string());
        if let Some(registry) = self.connections.get(&registry_name) {
            match registry
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

            // Check for pending events from registry provider
            if let Some(registry) = self.connections.get(&registry_name) {
                if let Ok(events) = registry.call("events.poll", serde_json::json!({})).await {
                    if let Some(event_list) = events.as_array() {
                        for event in event_list {
                            self.handle_primal_event(event);
                        }
                    }
                }
            }

            // Push any state updates to petalTongue
            let pt = self.connections.petaltongue().cloned();
            let _ = UISync::send_heartbeat(&pt).await;
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
