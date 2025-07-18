//! Universal Biome Manifest System
//!
//! This module provides a universal, primal-agnostic manifest system that replaces
//! the ToadStool-specific manifest. It describes biomes in terms of capabilities
//! and requirements rather than specific Primal implementations.
//!
//! The module is organized into the following sub-modules:
//! - `core`: Core manifest structure and metadata
//! - `requirements`: Biome requirements and constraints
//! - `services`: Service definitions and specifications
//! - `resources`: Resource specifications and management
//! - `security`: Security requirements and policies
//! - `networking`: Network configuration and policies
//! - `storage`: Storage specifications and classes
//! - `monitoring`: Monitoring and observability configuration
//! - `deployment`: Deployment preferences and strategies
//! - `validation`: Validation rules and error handling
//! - `implementation`: Implementation of manifest methods

pub mod core;
pub mod requirements;
pub mod services;
pub mod resources;
pub mod security;
pub mod networking;
pub mod storage;
pub mod monitoring;
pub mod deployment;
pub mod validation;
pub mod implementation;

// Re-export the main types
pub use core::*;
pub use requirements::*;
pub use resources::*;
pub use storage::*;
pub use deployment::*;
pub use validation::*;

// Re-export security types
pub use security::{
    AuditSpec, AuditStorage, AuthenticationSpec, AuthorizationSpec,
    BackupSpec, BackupStorage, ComplianceSpec, EncryptionSpec,
    KeyManagementSpec, NetworkSecuritySpec, SecurityPolicy,
    SecurityRequirements, SecurityRule
};

// Re-export monitoring types
pub use monitoring::{
    AlertingSpec, LoggingSpec, LogStorage, MetricsSpec, MetricsStorage,
    MonitoringSpec, TracingSpec, TracingStorage
};

// Re-export services types (resolving conflicts)
pub use services::{
    ConfigMount, DependencyType, FaultToleranceLevel, 
    HealthCheckConfig, HealthCheckType, LoadBalancingConfig,
    LoadBalancingSpec, NetworkPolicy, PortSpec, ResourceLimits,
    RestartPolicy, RuntimeSpec, RuntimeType, SecurityContext,
    ServiceConfig, ServiceDefinition, ServiceDependency,
    ServiceMounts, ServiceNetworking, ServiceSpec, ServiceStorage,
    TemporaryStorage, ValidationStrategy, VolumeMount
};

// Re-export networking types (resolving conflicts)
pub use networking::{
    NetworkTopology, NetworkingSpec
};

// Re-export persistent storage from services
pub use services::PersistentStorage; 