// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Common test infrastructure for NUCLEUS atomic testing
// Provides fixtures, helpers, chaos engine, and fault injection

pub mod fixtures;
pub mod helpers;
pub mod chaos_engine;
pub mod fault_injector;

// Re-export commonly used items
pub use fixtures::*;
pub use helpers::*;
pub use chaos_engine::{ChaosEngine, ChaosScenario};
pub use fault_injector::{FaultInjector, Fault, FaultHandle};
