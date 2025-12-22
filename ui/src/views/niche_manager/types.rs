//! Types and data structures for the Niche Manager
//!
//! This module contains all the data structures, enums, and type definitions
//! used throughout the niche management system.

use std::collections::HashMap;

/// Main tabs available in the Niche Manager interface
#[derive(Debug, Clone, PartialEq)]
pub enum NicheManagerTab {
    Browse,
    Create,
    Edit,
    Test,
    Marketplace,
}

/// Complete niche package definition
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NichePackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub tags: Vec<String>,
    pub features: Vec<String>,
    pub requirements: SystemRequirements,
    pub manifest_path: String,
    pub icon_path: Option<String>,
    pub size_mb: u64,
    pub downloads: u64,
    pub rating: f32,
    pub created_at: String,
    pub updated_at: String,
    pub status: NicheStatus,
}

/// Categories for organizing niche packages
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum NicheCategory {
    Gaming,
    Research,
    Development,
    Enterprise,
    IoT,
    Education,
    Healthcare,
    Finance,
    Media,
    Custom,
}

/// Difficulty levels for niche packages
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum NicheDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Status of a niche package
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum NicheStatus {
    Draft,
    Testing,
    Published,
    Deprecated,
    Private,
}

/// System requirements for running a niche
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SystemRequirements {
    pub min_cpu_cores: u32,
    pub min_memory_gb: u32,
    pub min_storage_gb: u64,
    pub required_features: Vec<String>,
    pub supported_architectures: Vec<String>,
}

/// Template for creating new niches
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub template_yaml: String,
    pub parameters: Vec<TemplateParameter>,
    pub examples: Vec<TemplateExample>,
}

/// Parameter definition for niche templates
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation: Option<String>,
}

/// Types of parameters supported in templates
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Choice(Vec<String>),
    Array,
}

/// Example configuration for a template
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TemplateExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

/// Sorting options for niche lists
#[derive(Debug, Clone, PartialEq)]
pub enum NicheSortBy {
    Name,
    Category,
    Rating,
    Downloads,
    Recent,
    Size,
}

/// Editor modes for niche editing
#[derive(Debug, Clone, PartialEq)]
pub enum NicheEditorMode {
    Visual,
    YAML,
    Preview,
}

/// Complete niche manifest structure
#[derive(Debug, Clone)]
pub struct NicheManifest {
    pub metadata: NicheMetadata,
    pub services: Vec<ServiceDefinition>,
    pub resources: ResourceRequirements,
    pub networking: NetworkingConfig,
    pub security: SecurityConfig,
    pub dependencies: Vec<String>,
}

/// Metadata for a niche package
#[derive(Debug, Clone)]
pub struct NicheMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

/// Service definition within a niche
#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    pub name: String,
    pub primal: String,
    pub runtime: String,
    pub image: Option<String>,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub resources: ServiceResources,
}

/// Resource requirements for a service
#[derive(Debug, Clone)]
pub struct ServiceResources {
    pub cpu: f32,
    pub memory_gb: f32,
    pub storage_gb: f32,
    pub gpu: Option<u32>,
}

/// Total resource requirements for a niche
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub total_cpu: f32,
    pub total_memory_gb: f32,
    pub total_storage_gb: f32,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: Option<u32>,
}

/// Networking configuration for a niche
#[derive(Debug, Clone)]
pub struct NetworkingConfig {
    pub load_balancing: bool,
    pub service_discovery: bool,
    pub ingress_enabled: bool,
    pub mesh_enabled: bool,
}

/// Security configuration for a niche
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub network_policies: bool,
    pub resource_quotas: bool,
    pub encryption_at_rest: bool,
    pub encryption_in_transit: bool,
    pub access_control: String,
}

/// Result of a niche test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub message: String,
    pub duration_ms: u64,
}

/// Status of a test execution
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Running,
}

/// Status of the publishing process
#[derive(Debug, Clone, PartialEq)]
pub enum PublishingStatus {
    Idle,
    Validating,
    Uploading,
    Published,
    Failed,
}

/// Publishing statistics for niches
#[derive(Debug, Clone)]
pub struct PublishingStats {
    pub downloads: u64,
    pub rating: f32,
    pub reviews: u32,
}

/// Marketplace niche with additional metadata
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketplaceNiche {
    pub package: NichePackage,
    pub verified: bool,
    pub featured: bool,
    pub security_score: f32,
    pub community_rating: f32,
    pub last_updated: String,
}

impl Default for NicheManifest {
    fn default() -> Self {
        Self {
            metadata: NicheMetadata {
                name: "New Niche".to_string(),
                version: "1.0.0".to_string(),
                description: "A new niche package".to_string(),
                author: "Unknown".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
            },
            services: Vec::new(),
            resources: ResourceRequirements {
                total_cpu: 1.0,
                total_memory_gb: 1.0,
                total_storage_gb: 10.0,
                gpu_required: false,
                network_bandwidth_mbps: None,
            },
            networking: NetworkingConfig {
                load_balancing: false,
                service_discovery: true,
                ingress_enabled: false,
                mesh_enabled: false,
            },
            security: SecurityConfig {
                network_policies: true,
                resource_quotas: true,
                encryption_at_rest: false,
                encryption_in_transit: false,
                access_control: "basic".to_string(),
            },
            dependencies: Vec::new(),
        }
    }
}

impl Default for SystemRequirements {
    fn default() -> Self {
        Self {
            min_cpu_cores: 1,
            min_memory_gb: 1,
            min_storage_gb: 10,
            required_features: Vec::new(),
            supported_architectures: vec!["x86_64".to_string()],
        }
    }
}
