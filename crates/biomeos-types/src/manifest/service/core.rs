//! Core Service Types
//!
//! This module contains the core service specification types including ServiceSpec,
//! ServiceMetadata, ImageSpec, and related fundamental types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::health::HealthCheckConfig;
use crate::primal::{PrimalCapability, PrimalType, ResourceRequirements};

// Forward declarations for types defined in other modules
use super::{
    EnvVarSpec,
    PortSpec,
    ServiceDependency,
    // Note: Scaling, security, and deployment modules will be implemented
    // as part of the Universal Adapter architecture integration
    VolumeMountSpec,
};

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Service image or binary
    pub image: ImageSpec,

    /// Service ports
    pub ports: Vec<PortSpec>,

    /// Environment variables
    pub environment: HashMap<String, EnvVarSpec>,

    /// Volume mounts
    pub volumes: Vec<VolumeMountSpec>,

    /// Resource requirements
    pub resources: Option<ResourceRequirements>,

    /// Health checks
    pub health_checks: Vec<HealthCheckConfig>,

    /// Service dependencies
    pub depends_on: Vec<ServiceDependency>,

    /// Service configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Scaling configuration (delegated to Toadstool orchestration)
    pub scaling: Option<serde_json::Value>,

    /// Security context (delegated to BearDog security primal)
    pub security: Option<serde_json::Value>,

    /// Restart policy
    pub restart_policy: RestartPolicy,

    /// Deployment strategy (delegated to Toadstool execution)
    pub deployment: Option<serde_json::Value>,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service name
    pub name: String,

    /// Service description
    pub description: Option<String>,

    /// Service version
    pub version: String,

    /// Service labels
    pub labels: HashMap<String, String>,

    /// Service annotations
    pub annotations: HashMap<String, String>,

    /// Primal type this service implements
    pub primal_type: Option<PrimalType>,

    /// Capabilities this service provides
    pub capabilities: Vec<PrimalCapability>,
}

/// Image specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSpec {
    /// Container image
    Container {
        /// Image name
        name: String,
        /// Image tag
        tag: String,
        /// Registry URL
        registry: Option<String>,
        /// Image pull policy
        pull_policy: ImagePullPolicy,
        /// Image pull secrets
        pull_secrets: Vec<String>,
    },

    /// Binary executable
    Binary {
        /// Binary path
        path: String,
        /// Binary arguments
        args: Vec<String>,
        /// Working directory
        working_dir: Option<String>,
    },

    /// WASM module
    Wasm {
        /// Module path
        module: String,
        /// Runtime configuration
        runtime: WasmRuntimeSpec,
    },

    /// Function/lambda
    Function {
        /// Function handler
        handler: String,
        /// Runtime environment
        runtime: String,
        /// Function code
        code: FunctionCodeSpec,
    },
}

/// Image pull policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImagePullPolicy {
    Always,
    IfNotPresent,
    Never,
}

/// WASM runtime specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmRuntimeSpec {
    /// WASM runtime (wasmtime, wasmer, etc.)
    pub runtime: String,

    /// Runtime version
    pub version: Option<String>,

    /// Runtime configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Memory limits
    pub memory_limit: Option<usize>,

    /// CPU limits
    pub cpu_limit: Option<f64>,
}

/// Function code specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionCodeSpec {
    /// Inline code
    Inline(String),

    /// Code from file
    File(String),

    /// Code from URL
    Url(String),

    /// Code from S3/object storage
    S3 {
        bucket: String,
        key: String,
        region: Option<String>,
    },
}

/// Restart policy for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
    UnlessStopped,
}
