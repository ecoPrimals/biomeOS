// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability-based primal handlers
//!
//! This module contains the handlers for capability-based primal operations,
//! extracted from neural_executor for better organization and maintainability.
//!
//! ## Deep Debt Principles
//!
//! - Capability-based discovery (runtime resolution)
//! - No hardcoded primal names where possible
//! - Socket nucleation for deterministic paths
//! - Environment-driven configuration
//!
//! ## Module structure
//!
//! - `discovery` - Capability resolution and binary discovery
//! - `primal_start` - Capability-based primal start handler
//! - `health` - Capability-based health check handler
//!
//! ## Dispatch
//!
//! The neural executor dispatches to these handlers by node type:
//! - `primal_start` | `start` → [`primal_start_capability`]
//! - `health.check_all` (capability-based) → [`health_check_capability`]

mod discovery;
mod health;
mod primal_start;

// Re-export public API for backwards compatibility
pub use discovery::discover_primal_binary;
pub use health::health_check_capability;
pub use primal_start::primal_start_capability;
