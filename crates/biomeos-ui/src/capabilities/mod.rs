//! Capabilities Module
//!
//! Generic, capability-based services provided by biomeOS.
//! NO primal-specific code - pure TRUE PRIMAL architecture!
//!
//! ## Available Capabilities
//!
//! - `device_management` - Device and primal management for UI primals

pub mod device_management;

// Re-export for convenience
pub use device_management::DeviceManagementProvider;

