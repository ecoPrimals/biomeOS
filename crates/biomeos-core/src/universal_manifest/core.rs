//! Core Universal Biome Manifest Types
//!
//! This module defines the core manifest structure and metadata types
//! for universal biome manifests.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::requirements::BiomeRequirements;
use super::services::ServiceDefinition;
use super::resources::GlobalResourceSpec;
use super::security::SecurityRequirements;
use super::networking::NetworkingSpec;
use super::storage::StorageSpec;
use super::monitoring::MonitoringSpec;
use super::deployment::DeploymentPreferences;
use super::validation::ValidationSpec;

/// Universal biome manifest - agnostic to specific Primal implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalBiomeManifest {
    /// API version for manifest compatibility
    pub api_version: String,
    /// Manifest type (always "Biome")
    pub kind: String,
    /// Biome metadata
    pub metadata: BiomeMetadata,
    /// Capability requirements for this biome
    pub requirements: BiomeRequirements,
    /// Service definitions
    pub services: Vec<ServiceDefinition>,
    /// Resource specifications
    pub resources: GlobalResourceSpec,
    /// Security requirements
    pub security: SecurityRequirements,
    /// Networking configuration
    pub networking: NetworkingSpec,
    /// Storage requirements
    pub storage: StorageSpec,
    /// Monitoring and observability
    pub monitoring: MonitoringSpec,
    /// Deployment preferences
    pub deployment: DeploymentPreferences,
    /// Validation rules
    pub validation: ValidationSpec,
}

/// Biome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    /// Biome name
    pub name: String,
    /// Biome description
    pub description: String,
    /// Version
    pub version: String,
    /// Authors
    pub authors: Vec<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Labels for categorization
    pub labels: HashMap<String, String>,
    /// Annotations for additional metadata
    pub annotations: HashMap<String, String>,
    /// Biome type classification
    pub biome_type: BiomeType,
    /// Maturity level
    pub maturity: MaturityLevel,
    /// License information
    pub license: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Source code repository
    pub repository: Option<String>,
    /// Support contact information
    pub support: Option<String>,
}

/// Biome type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeType {
    /// Application biome
    Application,
    /// Infrastructure biome
    Infrastructure,
    /// Data processing biome
    DataProcessing,
    /// AI/ML biome
    AiMl,
    /// Development environment biome
    Development,
    /// Research biome
    Research,
    /// Custom biome type
    Custom(String),
}

/// Maturity level of the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaturityLevel {
    /// Experimental - not suitable for production
    Experimental,
    /// Alpha - early testing phase
    Alpha,
    /// Beta - feature complete but may have issues
    Beta,
    /// Stable - production ready
    Stable,
    /// Deprecated - being phased out
    Deprecated,
}

/// Resource summary for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    /// Total CPU requirement
    pub total_cpu: f64,
    /// Total memory requirement in MB
    pub total_memory_mb: u64,
    /// Total storage requirement in MB
    pub total_storage_mb: u64,
    /// Network bandwidth requirement in Mbps
    pub network_bandwidth_mbps: u64,
    /// GPU requirement
    pub gpu_required: bool,
    /// Specialized hardware requirements
    pub specialized_hardware: Vec<String>,
} 