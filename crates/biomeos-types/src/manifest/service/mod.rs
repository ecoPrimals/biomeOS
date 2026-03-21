// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
pub mod environment;
pub mod networking;

// Re-export all types for compatibility
pub use core::{
    FunctionCodeSpec, ImagePullPolicy, ImageSpec, RestartPolicy, ServiceMetadata, ServiceSpec,
    WasmRuntimeSpec,
};

pub use networking::{
    AffinityCookieSpec, AffinityType, LoadBalancerSpec, LoadBalancerTimeouts,
    LoadBalancingAlgorithm, PortHealthCheckSpec, PortProtocol, PortSpec, SessionAffinity,
};

pub use environment::{
    DependencyCondition, EnvVarSpec, MountPropagation, ServiceDependency, VolumeMountSpec,
};
