// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Interactive UI Orchestration for `biomeOS`
//!
//! This crate implements the orchestration layer for `biomeOS`'s interactive UI,
//! coordinating between multiple primals to create an emergent network effect.

#![expect(
    clippy::doc_markdown,
    reason = "primal names and biomeOS terms are domain vocabulary, not code references"
)]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

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
//!     let mut orchestrator = InteractiveUIOrchestrator::new("1894e909e454").await?;
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

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod actions;
pub mod capabilities; // ✅ TRUE PRIMAL capability-based architecture!
/// JSON-RPC device.management Unix server (shared with `device_management_server` binary).
pub mod device_management_server;
pub mod events;
pub mod orchestrator;
pub mod primal_client; // EVOLVED (Jan 27, 2026): Extracted from orchestrator
pub mod realtime;
pub mod state;
pub mod suggestions;

pub use actions::{ActionResult, UserAction};
pub use events::{EventStream, UIEvent};
pub use orchestrator::InteractiveUIOrchestrator;
// Re-export capability-based types for compatibility
pub use capabilities::device_management::{
    Device as PTDevice, DeviceManagementProvider, NicheTemplate, Primal as PTPrimal,
};
pub use capabilities::{HapticPipeline, MotionCaptureAdapter, StereoRenderAdapter};
pub use realtime::{RealTimeEvent, RealTimeEventHandler, RealTimeEventSubscriber};
pub use state::{Assignment, Device, PrimalInfo, UIState};
pub use suggestions::{AISuggestion, AISuggestionManager, SuggestionContext, SuggestionFeedback};
