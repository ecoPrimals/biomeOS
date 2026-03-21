// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    Secret {
        /// Secret name
        name: String,
        /// Key within the secret
        key: String,
    },

    /// Reference to config
    Config {
        /// Config name
        name: String,
        /// Key within the config
        key: String,
    },

    /// Field reference
    FieldRef {
        /// Field path expression
        field_path: String,
    },

    /// Resource field reference
    ResourceFieldRef {
        /// Container name
        container_name: Option<String>,
        /// Resource name
        resource: String,
        /// Divisor for the resource value
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
    /// No mount propagation
    None,
    /// Host-to-container mount propagation
    HostToContainer,
    /// Bidirectional mount propagation
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
    /// Service has started
    ServiceStarted,
    /// Service is healthy
    ServiceHealthy,
    /// Service has completed
    ServiceCompleted,
    /// Custom condition
    Custom(String),
}
