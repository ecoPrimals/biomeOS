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

pub mod core;
pub mod capabilities;
pub mod configuration;
pub mod service;

// Re-export core types
pub use core::{
    PrimalType, ResourceRequirements, PrimalMetadata,
};

// Re-export capability types
pub use capabilities::{
    CapabilityMetadata, CapabilityParameter, PrimalCapability,
    CapabilityPerformance, LatencyCharacteristics,
};

// Re-export configuration types  
pub use configuration::{
    PrimalConfiguration, ConfigurationParameters, PrimalDependency,
    NetworkConfiguration, PortConfiguration, IngressConfiguration,
    TlsConfiguration, CertificateSource, NetworkPolicy, NetworkPolicyType,
    NetworkRule, NetworkTarget, LoadBalancingConfig, LoadBalancingAlgorithm,
    HealthCheckConfig, SecurityConfiguration, AuthenticationConfig,
    AuthenticationMethod, TokenConfig, AuthorizationConfig, AuthorizationModel,
    AuthorizationPolicy, EncryptionConfig, KeyManagementConfig, KeySource,
    AuditConfig, AuditDestination, AuditEvent,
};

// Re-export service types
pub use service::{
    UniversalPrimalService, PrimalServiceMetadata, UniversalServiceRequest,
    ServiceRequestContext, SecurityContext, AccessLevel, RequestPriority,
    UniversalServiceResponse, ServiceResponseMetadata, ResponseStatus,
    UniversalServiceRegistration, ServiceEndpoint, ServiceConstraints,
    NetworkRequirements, NetworkSecurity, SecurityRequirements,
    EncryptionRequirements, ServiceStatus, ConfigValidationResult,
};

// All legacy types have been unified - use UniversalServiceRequest/Response instead 