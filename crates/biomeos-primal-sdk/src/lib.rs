//! BiomeOS Primal SDK
//!
//! This SDK provides a clean interface to the unified BiomeOS type system,
//! optimized for primal development with direct access to all capabilities.
//! 
//! All legacy compatibility layers have been removed in favor of the unified
//! UniversalPrimalService architecture from biomeos-types.

// biomeOS unified types - direct re-exports
pub use biomeos_types::{
    // Core service interface
    UniversalPrimalService, PrimalServiceMetadata, UniversalServiceRequest, UniversalServiceResponse,
    UniversalServiceRegistration, ServiceStatus, ConfigValidationResult,
    
    // Error handling and results
    BiomeResult, BiomeError,
    
    // Health system
    Health, HealthReport, CapabilityMetadata,
    
    // Core primal types
    PrimalType, PrimalCapability, PrimalConfiguration,
    
    // Resource monitoring
    ResourceRequirements, ResourceMetrics, NetworkIoMetrics,
};

// Extended types module for primal-specific functionality
pub mod types;

// Re-export extended types for convenience (only what exists)
pub use types::{PrimalRequest, PrimalResponse, RequestPriority};

// No compatibility aliases needed - use unified types directly
