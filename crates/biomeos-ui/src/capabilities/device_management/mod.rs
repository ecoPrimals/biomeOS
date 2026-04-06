// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Device Management Capability
//!
//! Provides the `device.management` capability for biomeOS.
//! ANY primal can discover and use this capability at runtime.
//!
//! ## TRUE PRIMAL Principles
//!
//! - **Capability-based**: Advertises "device.management" via Songbird
//! - **Discovery-driven**: Primals discover this at runtime
//! - **No hardcoding**: No specific primal names in code
//! - **Self-knowledge only**: biomeOS knows its own devices/primals
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ANY UI Primal (petalTongue, CLI, Web, etc.)                  │
//! │  └─ discover_by_capability("device.management") via Songbird │
//! └──────────────────────────────────────────────────────────────┘
//!                            ↓ Runtime Discovery
//! ┌──────────────────────────────────────────────────────────────┐
//! │ biomeOS                                                       │
//! │  └─ DeviceManagementCapability (this module)                 │
//! │      ├─ get_devices() → discover via Songbird                │
//! │      ├─ get_primals() → query registry                       │
//! │      ├─ get_templates() → load from NestGate                 │
//! │      ├─ assign_device(device, primal) → orchestrate          │
//! │      └─ deploy_niche(config) → create niche                  │
//! └──────────────────────────────────────────────────────────────┘
//! ```

pub mod discovery;
pub mod provider;
pub mod templates;
pub mod types;

#[cfg(test)]
mod provider_tests;

pub use provider::DeviceManagementProvider;
pub use templates::{builtin_templates, node_template, tower_template};
pub use types::*;

// Re-export for compatibility during transition
// These types match the old petaltongue_bridge types
pub use types::{
    Device, DeviceStatus, DeviceType, ManagedPrimal as Primal, NicheTemplate, PrimalRole,
    PrimalStatus, ResourceRequirements, ValidationResult,
};
