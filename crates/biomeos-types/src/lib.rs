//! Unified BiomeOS Type System
//!
//! This crate provides a comprehensive, unified type system for the BiomeOS ecosystem,
//! consolidating all types that were previously scattered across multiple crates.

use std::collections::HashMap;

// Core unified modules
pub mod constants;
pub mod error;
pub mod health;
pub mod primal;
pub mod service;
pub mod manifest;
pub mod config;

// Re-export key types from each module
pub use constants::*;
pub use error::{BiomeError, BiomeResult};

// Health system exports
pub use health::{
    Health, HealthReport, HealthSubject, HealthSubjectType, ComponentHealth,
    HealthIssue, HealthIssueCategory, HealthIssueSeverity, StartupPhase, ShutdownPhase, MaintenanceType,
    HealthCheckConfig, ResourceMetrics, NetworkIoMetrics, AvailabilityMetrics, ResponseTimeMetrics, ErrorMetrics,
    RemediationAction, RemediationActionType,
};

// Configuration exports - basic re-exports
pub use config::{
    BiomeOSConfig, SystemConfig, Environment, OrganizationScale, 
    ResourceConfig, SecurityConfig, FeatureFlags, ConfigMetadata,
};

// Primal system exports
pub use primal::{
    PrimalType, PrimalCapability, PrimalConfiguration, ResourceRequirements,
    UniversalPrimalService, PrimalServiceMetadata,
    UniversalServiceRequest, UniversalServiceResponse, UniversalServiceRegistration,
    ServiceStatus, ConfigValidationResult,

    CapabilityMetadata,
};

// Additional service types
pub use service::core::ServiceSpec;

// Service system exports - using module paths to avoid conflicts
pub use service::core::{UniversalService, ServiceMetadata, ServiceEndpoint, ServiceDependency};

// Manifest system exports - using module paths to avoid conflicts
pub use manifest::{BiomeManifest, ManifestMetadata, BiomeSpec, BiomeType};

// Additional commonly used types
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};

// Convenience type aliases for common patterns
pub type BiomeMap<K, V> = HashMap<K, V>;
pub type BiomeDateTime = DateTime<Utc>;

/// Unified build information for the entire BiomeOS ecosystem
pub const BUILD_INFO: &str = concat!(
    "BiomeOS Types v",
    env!("CARGO_PKG_VERSION")
);

// Re-export version constants from constants module
pub use constants::version::{VERSION, TYPES_VERSION, API_VERSION, MCP_PROTOCOL_VERSION}; 