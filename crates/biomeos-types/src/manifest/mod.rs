//! Unified Manifest System
//!
//! This module provides comprehensive manifest management for BiomeOS,
//! supporting service definitions, lifecycle management, and configuration.

pub mod core;
pub mod manifest_core;
pub mod manifest_extensions;
pub mod manifest_security;
pub mod lifecycle;
pub mod networking_core;
pub mod networking_policies;
pub mod networking_services;
pub mod service;
pub mod storage;

// Re-export main manifest types from manifest_core where they're actually defined
pub use manifest_core::{BiomeManifest, ManifestMetadata, BiomeSpec, BiomeType};

// Only re-export what we know exists - ServiceSpec is confirmed to exist
pub use service::ServiceSpec;

// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap; 