//! Universal BiomeOS Manager - Modular Architecture
//!
//! Split into focused modules for better maintainability and compliance
//! with the 2000-line file size limit.

// Core functionality and initialization
pub mod ai;
// pub mod client_registry;  // Depends on legacy clients module
pub mod core;
pub mod discovery;
pub mod health;
pub mod operations;
pub mod primals;

#[cfg(test)]
mod tests;

// Re-export from discovery module
pub use discovery::{DiscoveryResult, PrimalDiscoveryService, ProbeResult};

// Re-export from health module
pub use health::HealthMonitor;

// Re-export from core module
pub use core::{PrimalInfo, UniversalBiomeOSManager};

// Re-export from primals module
pub use primals::PrimalStatistics;

// Re-export from ai module
pub use ai::{AIRecommendation, GeneticAccessKey, Priority};

// Re-export from client_registry module (commented out - depends on legacy clients)
// pub use client_registry::ClientRegistry;

// Legacy type aliases for backwards compatibility - REMOVED duplicate GeneticAccessKey
