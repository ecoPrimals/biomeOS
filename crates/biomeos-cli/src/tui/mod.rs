//! BiomeOS TUI Dashboard - Modular Structure
//!
//! This module provides a comprehensive terminal user interface for the BiomeOS ecosystem,
//! organized into focused sub-modules for better maintainability.

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
