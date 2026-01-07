//! Operations Management - Refactored
//!
//! This module has been refactored into focused sub-modules for better maintainability:
//! - `manifest` - Manifest validation and deployment
//! - `service` - Service lifecycle operations  
//! - `runtime` - Runtime operations (logs, exec, monitoring)
//! - `deployment` - Biome deployment orchestration
//!
//! All public APIs are implemented directly on `UniversalBiomeOSManager` in their
//! respective modules. This file exists for backward compatibility and documentation.

// The operations have been moved to:
// - crates/biomeos-core/src/universal_biomeos_manager/manifest.rs
// - crates/biomeos-core/src/universal_biomeos_manager/service.rs
// - crates/biomeos-core/src/universal_biomeos_manager/runtime.rs
// - crates/biomeos-core/src/universal_biomeos_manager/deployment.rs
