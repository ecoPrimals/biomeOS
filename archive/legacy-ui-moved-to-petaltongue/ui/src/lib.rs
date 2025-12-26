//! BiomeOS UI Library - Universal Interface System

pub mod api;
pub mod app;
pub mod backend;
pub mod byob_monitor;
pub mod desktop;
pub mod iso_monitor;
pub mod minimal_app;
pub mod mock;
pub mod niche_monitor;
pub mod state;
pub mod system_monitor;
pub mod types;
pub mod views;

// Re-export commonly used types for convenience
pub use types::*;
