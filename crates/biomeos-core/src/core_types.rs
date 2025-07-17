//! Core biomeOS Types and Constants
//!
//! This module contains the fundamental types, constants, and type aliases
//! used throughout the biomeOS ecosystem.

use crate::BiomeResult;
use uuid::Uuid;

/// biomeOS version information
pub const BIOMEOS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Unique identifier for a biome instance
pub type BiomeId = Uuid;

/// Unique identifier for a Primal instance  
pub type PrimalId = String;

/// Initialize biomeOS core systems
pub async fn init_biomeos() -> BiomeResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    tracing::info!("biomeOS v{} initializing...", BIOMEOS_VERSION);

    // Core initialization will be expanded as we add more functionality
    Ok(())
}
