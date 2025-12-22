//! Universal BiomeOS Manager - Modular Architecture
//!
//! Split into focused modules for better maintainability and compliance
//! with the 2000-line file size limit.

// Core functionality and initialization
pub mod core;
pub mod discovery;
pub mod health;
pub mod primals;
pub mod operations;
pub mod ai;

// Re-export from discovery module  
pub use discovery::{PrimalDiscoveryService, DiscoveryResult, ProbeResult};

// Re-export from health module
pub use health::HealthMonitor;

// Re-export from core module
pub use core::{UniversalBiomeOSManager, PrimalInfo};

// Re-export from primals module
pub use primals::PrimalStatistics;

// Re-export from ai module
pub use ai::{AIRecommendation, Priority, GeneticAccessKey};

// Legacy type aliases for backwards compatibility - REMOVED duplicate GeneticAccessKey 