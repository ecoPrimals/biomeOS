// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Primal Orchestrator - Async, concurrent primal lifecycle management
//!
//! Handles complex startup choreography:
//! - `BearDog` → Songbird (crypto provider → discovery)
//! - Songbird → `BearDog` fleet (discovery → crypto cluster)
//! - Songbird → Songbird → Network (cascading discovery)
//! - Concurrent health monitoring
//! - Automatic recovery

mod health_monitor;
mod orchestrator;
mod state;

pub use health_monitor::{PrimalHealthMonitor, PrimalHealthMonitorBuilder};
pub use orchestrator::PrimalOrchestrator;
pub use state::{ManagedPrimal, PrimalState};
