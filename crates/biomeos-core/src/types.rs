//! BiomeOS Core Types
//!
//! Common types used throughout the biomeOS ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export types from other modules

/// Universal Biome Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalBiomeManifest {
    pub metadata: BiomeMetadata,
    pub primals: Vec<PrimalConfiguration>,
    pub services: Vec<ServiceConfiguration>,
}

/// Biome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub tags: Vec<String>,
}

/// Primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration {
    pub name: String,
    pub primal_type: String,
    pub version: String,
    pub configuration: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    pub name: String,
    pub service_type: String,
    pub configuration: HashMap<String, serde_json::Value>,
    pub resources: ServiceResourceRequirements,
    pub networking: NetworkingConfiguration,
}

/// Service resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_gb: Option<f64>,
    pub storage_gb: Option<f64>,
    pub gpu_required: bool,
}

/// Networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingConfiguration {
    pub ports: Vec<PortConfiguration>,
    pub ingress: Option<IngressConfiguration>,
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfiguration {
    pub name: String,
    pub port: u16,
    pub protocol: String,
    pub expose: bool,
}

/// Ingress configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfiguration {
    pub hostname: String,
    pub path_prefix: String,
    pub tls_enabled: bool,
}
