//! Service Manifest Types
//!
//! This module organizes service-related manifest types into focused sub-modules
//! for better maintainability and to keep files under the 2000 line limit.
//!
//! The service types are organized into:
//! - `core`: Core service specifications and metadata
//! - `networking`: Port specifications and load balancing  
//! - `environment`: Environment variables, volumes, and dependencies

pub mod core;
pub mod networking;
pub mod environment;

// Re-export all types for compatibility
pub use core::{
    ServiceSpec, ServiceMetadata, ImageSpec, ImagePullPolicy,
    WasmRuntimeSpec, FunctionCodeSpec, RestartPolicy,
};

pub use networking::{
    PortSpec, PortProtocol, LoadBalancerSpec, LoadBalancingAlgorithm,
    SessionAffinity, AffinityType, AffinityCookieSpec,
    LoadBalancerTimeouts, PortHealthCheckSpec,
};

pub use environment::{
    EnvVarSpec, VolumeMountSpec, MountPropagation,
    ServiceDependency, DependencyCondition,
}; 