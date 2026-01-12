//! Interactive UI Orchestration for biomeOS
//!
//! This crate implements the orchestration layer for biomeOS's interactive UI,
//! coordinating between multiple primals to create an emergent network effect.
//!
//! ## Network Effect Architecture
//!
//! This is NOT a single primal's feature. It's an **emergent capability** from
//! the cooperation of 7 primals:
//!
//! - **petalTongue**: UI framework and rendering
//! - **biomeOS** (this crate): Orchestration and coordination
//! - **Songbird**: Device/primal discovery and registry
//! - **BearDog**: Authorization and security
//! - **NestGate**: Configuration persistence
//! - **ToadStool**: Resource metrics
//! - **Squirrel**: AI suggestions
//!
//! Value = n² (Metcalfe's Law) = 7² = 49 potential interactions!
//!
//! ## Phases
//!
//! - Phase 1: Foundation (types, events, actions) ✅
//! - Phase 2: Discovery (capability-based) ✅
//! - Phase 3: Interaction (device assignment) ✅
//! - Phase 4: Real-Time (WebSocket/SSE) ✅
//! - Phase 5: Intelligence (AI suggestions) ✅
//! - Phase 6: Polish (error handling, loading) 🚧
//!
//! ## Example
//!
//! ```no_run
//! use biomeos_ui::InteractiveUIOrchestrator;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create the orchestrator (connects to all primals)
//!     let mut orchestrator = InteractiveUIOrchestrator::new().await?;
//!     
//!     // Start the UI (launches petalTongue and syncs state)
//!     orchestrator.start().await?;
//!     
//!     // Handle user actions in a loop
//!     orchestrator.run().await?;
//!     
//!     Ok(())
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod actions;
pub mod events;
pub mod orchestrator;
pub mod state;
pub mod realtime;
pub mod suggestions;
pub mod petaltongue_bridge;

pub use actions::{UserAction, ActionResult};
pub use events::{UIEvent, EventStream};
pub use orchestrator::InteractiveUIOrchestrator;
pub use state::{UIState, Device, PrimalInfo, Assignment};
pub use realtime::{RealTimeEvent, RealTimeEventSubscriber, RealTimeEventHandler};
pub use suggestions::{AISuggestion, AISuggestionManager, SuggestionContext, SuggestionFeedback};
pub use petaltongue_bridge::{PetalTongueRPCBridge, Device as PTDevice, Primal as PTPrimal, NicheTemplate};

