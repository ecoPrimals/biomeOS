//! Unified Primal Type System
//!
//! This module provides a comprehensive primal type system for the BiomeOS ecosystem,
//! organizing all primal-related types into logical sub-modules for better maintainability.
//!
//! The module is split into:
//! - `core`: Basic primal types and metadata
//! - `capabilities`: Capability system and performance metrics
//! - `configuration`: Configuration, network, and security settings
//! - `service`: Service interfaces and request/response handling

pub mod capabilities;
pub mod configuration;
pub mod core;
pub mod service;

#[cfg(test)]
mod core_tests;

// Re-export core types
pub use core::{PrimalMetadata, PrimalType, ResourceRequirements};

// Re-export capability types
pub use capabilities::{
    CapabilityMetadata, CapabilityParameter, CapabilityPerformance, LatencyCharacteristics,
    PrimalCapability,
};

// Re-export configuration types
pub use configuration::{
    AuditConfig, AuditDestination, AuditEvent, AuthenticationConfig, AuthenticationMethod,
    AuthorizationConfig, AuthorizationModel, AuthorizationPolicy, CertificateSource,
    ConfigurationParameters, EncryptionConfig, HealthCheckConfig, IngressConfiguration,
    KeyManagementConfig, KeySource, LoadBalancingAlgorithm, LoadBalancingConfig,
    NetworkConfiguration, NetworkPolicy, NetworkPolicyType, NetworkRule, NetworkTarget,
    PortConfiguration, PrimalConfiguration, PrimalDependency, SecurityConfiguration,
    TlsConfiguration, TokenConfig,
};

// Re-export service types
pub use service::{
    AccessLevel, ConfigValidationResult, EncryptionRequirements, NetworkRequirements,
    NetworkSecurity, PrimalServiceMetadata, RequestPriority, ResponseStatus, SecurityContext,
    SecurityRequirements, ServiceConstraints, ServiceEndpoint, ServiceRequestContext,
    ServiceResponseMetadata, ServiceStatus, UniversalPrimalService, UniversalServiceRegistration,
    UniversalServiceRequest, UniversalServiceResponse,
};

// All legacy types have been unified - use UniversalServiceRequest/Response instead
