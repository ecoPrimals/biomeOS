// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unified `BiomeOS` Type System
//!
//! This crate provides a comprehensive, unified type system for the `BiomeOS` ecosystem,
//! consolidating all types that were previously scattered across multiple crates.

// Allow doc_markdown warnings for internal documentation - the important thing is
// that the documentation exists, not that every technical term has backticks.
// This significantly reduces noise while maintaining documentation coverage.
#![warn(missing_docs)]
#![allow(clippy::doc_markdown)]
// Deny unsafe code in type definitions
#![forbid(unsafe_code)]

use std::collections::HashMap;

// Core unified modules
pub mod api_schema;
/// NUCLEUS atomic type definitions (Tower, Node, Nest, Full)
pub mod atomic;
/// Well-known capability taxonomy for discovery
pub mod capability_taxonomy;
pub mod config;
pub mod constants;
pub mod defaults; // Runtime defaults (socket paths, etc.) with env var overrides
pub mod env_config; // Centralized environment variable constants and typed accessors
pub mod error;
pub mod health;
pub mod identifiers;
pub mod jsonrpc;
pub mod manifest;
pub mod network_config; // Capability-based network configuration
/// XDG-compliant system paths
pub mod paths;
pub mod primal;
pub mod primal_names;
pub mod service;
pub mod surgical; // Surgical simulation and medical domain types
pub mod tarpc_types; // tarpc service definitions for high-performance RPC
pub mod time_series; // Cross-spring time series exchange (ecoPrimals/time-series/v1)
pub mod xr; // Extended Reality (VR/AR/XR) types

// Re-export key types from each module
pub use api_schema::{
    ApiSchemaResponse, ApiSchemaType, OperationMetadata, ParameterMetadata, PrimalInfo,
    SchemaDiscoveryConfig,
};
pub use atomic::{
    AtomicCapability, AtomicTier, FullNucleus, NestAtomic, NodeAtomic, PrimalHealth,
    ProviderHealthMap, TowerAtomic,
};
pub use jsonrpc::{JsonRpcError, JsonRpcInput, JsonRpcRequest, JsonRpcResponse, JSONRPC_VERSION};

// tarpc types for high-performance RPC
pub use capability_taxonomy::CapabilityTaxonomy; // Well-known capability taxonomy (enum)
pub use constants::*;
pub use defaults::{socket_path, RuntimeConfig}; // Runtime configuration with env var overrides
pub use error::{BiomeError, BiomeResult, IpcError};
pub use network_config::{NetworkConfig, PortConfig}; // Capability-based network configuration
pub use paths::SystemPaths;
pub use tarpc_types::{
    protocol_from_env, HealthMetrics, HealthStatus, JwtSecretResult, LineageResult, ProtocolInfo,
    ProtocolPreference, RegistrationResult, ServiceInfo, ServiceRegistration, SignatureResult,
    VersionInfo, PROTOCOL_ENV_VAR,
}; // XDG-compliant paths

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

// Surgical domain type exports
pub use surgical::{
    BiosignalSample, BiosignalStreamConfig, BiosignalType, CompartmentModel, DamageType,
    PkModelParams, PkModelResult, SurgicalProcedure, SurgicalSessionMetrics, SurgicalSessionState,
    ToolTissueInteraction,
};

// Cross-spring time series exports
pub use time_series::{CrossSpringTimeSeries, TimeSeriesError, TimeSeriesSource, SCHEMA_V1};

// XR (Extended Reality) type exports
pub use xr::{
    AnatomyLayer, AnatomyModel, HapticCommand, HapticDeviceCapabilities, HapticDeviceType,
    MotionCaptureConfig, Pose6DoF, StereoConfig, SurgicalInstrument, TissueMaterial,
    TrackedDeviceType, TrackingFrame, VisualOutputCapability,
};

// Additional commonly used types
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

/// Convenience type alias for HashMap used throughout BiomeOS
pub type BiomeMap<K, V> = HashMap<K, V>;
/// Convenience type alias for UTC DateTime used throughout BiomeOS
pub type BiomeDateTime = DateTime<Utc>;

/// Unified build information for the entire BiomeOS ecosystem
pub const BUILD_INFO: &str = concat!("BiomeOS Types v", env!("CARGO_PKG_VERSION"));

// Re-export version constants from constants module
pub use constants::version::{API_VERSION, MCP_PROTOCOL_VERSION, TYPES_VERSION, VERSION};
