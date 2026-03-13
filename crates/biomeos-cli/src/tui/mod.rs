// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS TUI Dashboard — **DEPRECATED**
//!
//! This module is superseded by petalTongue, the universal UI primal for the ecoPrimals
//! ecosystem. It remains behind the `deprecated-tui` feature flag for backward compatibility
//! only. New UI work should target petalTongue.
//!
//! Enable with: `cargo build -p biomeos-cli --features deprecated-tui`

pub mod dashboard;
pub mod events;
pub mod types;
pub mod widgets;

pub use dashboard::BiomeOSDashboard;
pub use types::{TabId, TabInfo};

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;

/// Initialize and run the BiomeOS TUI dashboard
pub async fn run_dashboard(manager: UniversalBiomeOSManager) -> Result<()> {
    let mut dashboard = BiomeOSDashboard::new(manager);
    dashboard.run().await
}
