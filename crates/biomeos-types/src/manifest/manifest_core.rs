// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Core Manifest Structures
//!
//! This module contains the fundamental manifest types including BiomeManifest,
//! ManifestMetadata, BiomeSpec, and BiomeType definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::Environment;
use crate::primal::ResourceRequirements;

// Re-export from other modules for use in core structures
pub use super::manifest_extensions::{
    BiomeDependency, BiomeNetworkingSpec, ConfigSpec, HealthMonitoringSpec, LifecycleSpec,
    ScalingSpec, SecretSpec,
};
pub use super::manifest_security::BiomeSecuritySpec;
pub use super::networking_core::NetworkSpec;
pub use super::service::ServiceSpec;
pub use super::storage::VolumeSpec;

/// Universal Biome Manifest
///
/// This is the unified structure for all biome manifests, consolidating
/// the various manifest formats across the ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    /// Manifest metadata
    pub metadata: ManifestMetadata,

    /// Biome specification
    pub spec: BiomeSpec,

    /// Services defined in this biome
    pub services: HashMap<String, ServiceSpec>,

    /// Networks defined in this biome
    pub networks: HashMap<String, NetworkSpec>,

    /// Volumes defined in this biome
    pub volumes: HashMap<String, VolumeSpec>,

    /// Secrets defined in this biome
    pub secrets: HashMap<String, SecretSpec>,

    /// Configuration defined in this biome
    pub configs: HashMap<String, ConfigSpec>,

    /// Dependencies on other biomes
    pub dependencies: Vec<BiomeDependency>,
}

/// Manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    /// Manifest name
    pub name: String,

    /// Manifest version
    pub version: String,

    /// API version for compatibility
    pub api_version: String,

    /// Manifest kind/type
    pub kind: String,

    /// Manifest description
    pub description: Option<String>,

    /// Manifest author
    pub author: Option<String>,

    /// License information
    pub license: Option<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// Documentation URL
    pub documentation: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Labels for selection and grouping
    pub labels: HashMap<String, String>,

    /// Annotations for additional metadata
    pub annotations: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,

    /// Manifest namespace
    pub namespace: Option<String>,
}

/// Biome specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSpec {
    /// Biome type and categorization
    pub biome_type: BiomeType,

    /// Target environment
    pub environment: Environment,

    /// Resource requirements for the entire biome
    pub resources: Option<ResourceRequirements>,

    /// Security policies
    pub security: Option<BiomeSecuritySpec>,

    /// Health monitoring configuration
    pub health: Option<HealthMonitoringSpec>,

    /// Networking configuration
    pub networking: Option<BiomeNetworkingSpec>,

    /// Scaling configuration
    pub scaling: Option<ScalingSpec>,

    /// Lifecycle hooks
    pub lifecycle: Option<LifecycleSpec>,

    /// Biome-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Biome types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeType {
    /// Application biome
    Application {
        /// Application type
        app_type: String,
        /// Framework used
        framework: Option<String>,
    },

    /// Service biome
    Service {
        /// Service type
        service_type: String,
        /// Protocol used
        protocol: Option<String>,
    },

    /// Infrastructure biome
    Infrastructure {
        /// Infrastructure component
        component: String,
        /// Provider
        provider: Option<String>,
    },

    /// Development biome
    Development {
        /// Development environment
        dev_env: String,
        /// Tools included
        tools: Vec<String>,
    },

    /// Research biome
    Research {
        /// Research domain
        domain: String,
        /// Methodologies
        methodologies: Vec<String>,
    },

    /// Custom biome type
    Custom {
        /// Custom type name
        type_name: String,
        /// Custom attributes
        attributes: HashMap<String, String>,
    },
}

/// Default implementations
impl Default for BiomeManifest {
    fn default() -> Self {
        Self {
            metadata: ManifestMetadata::default(),
            spec: BiomeSpec::default(),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            secrets: HashMap::new(),
            configs: HashMap::new(),
            dependencies: Vec::new(),
        }
    }
}

impl Default for ManifestMetadata {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            api_version: "v1".to_string(),
            kind: "BiomeManifest".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            documentation: None,
            tags: Vec::new(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            namespace: None,
        }
    }
}

impl Default for BiomeSpec {
    fn default() -> Self {
        Self {
            biome_type: BiomeType::Application {
                app_type: "generic".to_string(),
                framework: None,
            },
            environment: Environment::Development,
            resources: None,
            security: None,
            health: None,
            networking: None,
            scaling: None,
            lifecycle: None,
            config: HashMap::new(),
        }
    }
}
