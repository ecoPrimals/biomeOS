//! Predictive Health Analytics System
//!
//! This module implements advanced health monitoring and predictive analytics
//! for the biomeOS ecosystem, providing cross-primal health aggregation,
//! trend analysis, and intelligent predictions.

pub mod analytics;
pub mod config;
pub mod monitoring;
pub mod predictions;
pub mod recommendations;
pub mod reports;
pub mod trends;
pub mod types;

// Re-export all public types and structs
pub use analytics::*;
pub use config::*;
pub use monitoring::*;
pub use predictions::*;
pub use recommendations::*;
pub use reports::*;
pub use trends::*;
pub use types::*;
