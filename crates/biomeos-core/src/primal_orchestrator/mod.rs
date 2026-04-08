// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Orchestrator - Async, concurrent primal lifecycle management
//!
//! Handles complex startup choreography:
//! - `BearDog` → Songbird (crypto provider → discovery)
//! - Songbird → `BearDog` fleet (discovery → crypto cluster)
//! - Songbird → Songbird → Network (cascading discovery)
//! - Concurrent health monitoring
//! - Automatic recovery

mod dependency_resolution;
mod health_monitor;
mod orchestrator;
mod orchestrator_health;
mod orchestrator_lifecycle;
mod state;

#[cfg(test)]
mod orchestrator_tests;

pub use health_monitor::{PrimalHealthMonitor, PrimalHealthMonitorBuilder};
pub use orchestrator::PrimalOrchestrator;
pub use state::{ManagedPrimal, PrimalState};
