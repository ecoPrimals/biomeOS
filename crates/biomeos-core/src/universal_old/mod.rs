//! Universal Platform Capabilities
//!
//! This module implements the core "next era" platform capabilities that make
//! biomeOS truly universal, grandma-safe, and AI-first.
//!
//! The functionality is organized into specialized modules:
//! - `mycorrhiza`: MYCORRHIZA Energy Flow Management
//! - `platform`: Universal Platform Detection and Management
//! - `implementation`: UniversalPlatform implementation details

pub mod mycorrhiza;
pub mod platform;
pub mod implementation;

// Re-export all public items from the modules
pub use mycorrhiza::*;
pub use platform::*;
pub use implementation::*; 