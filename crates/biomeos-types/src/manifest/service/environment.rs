//! Service Environment Types
//!
//! This module contains environment-related service types including environment
//! variables, volume mounts, and service dependencies.

use serde::{Deserialize, Serialize};

/// Environment variable specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvVarSpec {
    /// Direct value
    Value(String),

    /// Reference to secret
    Secret { name: String, key: String },

    /// Reference to config
    Config { name: String, key: String },

    /// Field reference
    FieldRef { field_path: String },

    /// Resource field reference
    ResourceFieldRef {
        container_name: Option<String>,
        resource: String,
        divisor: Option<String>,
    },
}

/// Volume mount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMountSpec {
    /// Volume name
    pub name: String,

    /// Mount path
    pub mount_path: String,

    /// Sub path
    pub sub_path: Option<String>,

    /// Read only
    pub read_only: bool,

    /// Mount propagation
    pub mount_propagation: Option<MountPropagation>,
}

/// Mount propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountPropagation {
    None,
    HostToContainer,
    Bidirectional,
}

/// Service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Service name
    pub service: String,

    /// Dependency condition
    pub condition: DependencyCondition,

    /// Restart dependency
    pub restart: bool,
}

/// Dependency conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyCondition {
    ServiceStarted,
    ServiceHealthy,
    ServiceCompleted,
    Custom(String),
}
