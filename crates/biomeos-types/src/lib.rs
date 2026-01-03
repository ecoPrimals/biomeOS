//! Unified BiomeOS Type System
//!
//! This crate provides a comprehensive, unified type system for the BiomeOS ecosystem,
//! consolidating all types that were previously scattered across multiple crates.

use std::collections::HashMap;

// Core unified modules
pub mod api_schema;
pub mod config;
pub mod constants;
pub mod error;
pub mod health;
pub mod identifiers;
pub mod manifest;
pub mod primal;
pub mod service;

// Re-export key types from each module
pub use api_schema::{
    ApiSchemaResponse, ApiSchemaType, OperationMetadata, ParameterMetadata, PrimalInfo,
    SchemaDiscoveryConfig,
};
pub use constants::*;
pub use error::{BiomeError, BiomeResult};

// Health system exports
pub use health::{
    AvailabilityMetrics, ComponentHealth, ErrorMetrics, Health, HealthCheckConfig, HealthIssue,
    HealthIssueCategory, HealthIssueSeverity, HealthReport, HealthSubject, HealthSubjectType,
    MaintenanceType, NetworkIoMetrics, RemediationAction, RemediationActionType, ResourceMetrics,
    ResponseTimeMetrics, ShutdownPhase, StartupPhase,
};

// Configuration exports - basic re-exports
pub use config::{
    BiomeOSConfig, ConfigMetadata, Environment, FeatureFlags, OrganizationScale, ResourceConfig,
    SecurityConfig, SystemConfig,
};

// Primal system exports
pub use primal::{
    CapabilityMetadata, ConfigValidationResult, PrimalCapability, PrimalConfiguration,
    PrimalServiceMetadata, PrimalType, ResourceRequirements, ServiceStatus, UniversalPrimalService,
    UniversalServiceRegistration, UniversalServiceRequest, UniversalServiceResponse,
};

// Additional service types
pub use service::core::ServiceSpec;

// Service system exports - using module paths to avoid conflicts
pub use service::core::{ServiceDependency, ServiceEndpoint, ServiceMetadata, UniversalService};

// Manifest system exports - using module paths to avoid conflicts
pub use manifest::{BiomeManifest, BiomeSpec, BiomeType, ManifestMetadata};

// Identifier exports - strong-typed domain IDs
pub use identifiers::{Endpoint, FamilyId, IdError, PrimalId, SessionId, TowerId};

// Additional commonly used types
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

// Convenience type aliases for common patterns
pub type BiomeMap<K, V> = HashMap<K, V>;
pub type BiomeDateTime = DateTime<Utc>;

/// Unified build information for the entire BiomeOS ecosystem
pub const BUILD_INFO: &str = concat!("BiomeOS Types v", env!("CARGO_PKG_VERSION"));

// Re-export version constants from constants module
pub use constants::version::{API_VERSION, MCP_PROTOCOL_VERSION, TYPES_VERSION, VERSION};
