//! BiomeOS Primal SDK
//!
//! This SDK provides a clean interface to the unified BiomeOS type system,
//! optimized for primal development with direct access to all capabilities.
//!
//! All legacy compatibility layers have been removed in favor of the unified
//! UniversalPrimalService architecture from biomeos-types.

// biomeOS unified types - direct re-exports
pub use biomeos_types::{
    BiomeError,

    // Error handling and results
    BiomeResult,
    CapabilityMetadata,

    ConfigValidationResult,

    // Health system
    Health,
    HealthReport,
    NetworkIoMetrics,
    PrimalCapability,
    PrimalConfiguration,

    PrimalServiceMetadata,
    // Core primal types
    PrimalType,
    ResourceMetrics,
    // Resource monitoring
    ResourceRequirements,
    ServiceStatus,
    // Core service interface
    UniversalPrimalService,
    UniversalServiceRegistration,
    UniversalServiceRequest,
    UniversalServiceResponse,
};

// Extended types module for primal-specific functionality
pub mod types;

// Re-export extended types for convenience (only what exists)
pub use types::{PrimalRequest, PrimalResponse, RequestPriority};

// No compatibility aliases needed - use unified types directly
