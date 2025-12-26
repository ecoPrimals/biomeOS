//! BYOB Types and Data Structures
//!
//! This module contains all the data structures, enums, and types used
//! throughout the BYOB (Build Your Own Biome) system.
//!
//! The system is designed to be completely universal and agnostic,
//! supporting any primal names and capabilities without hardcoding.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Use unified types from biomeos-types instead of duplicating
use biomeos_types::{
    service::networking::PortProtocol,
    // Removed unused imports
    Health,
    HealthCheckConfig,
    PrimalCapability,
    ResourceRequirements,
};

/// Workflow state for the BYOB process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowState {
    TeamSelection,
    NicheSelection,
    NicheCustomization,
    ManifestGeneration,
    YamlEditing,
    Deployment,
    Completed,
}

/// Universal primal definition - completely agnostic to specific names
/// This uses unified types from biomeos-types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDefinition {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub capabilities: HashSet<PrimalCapability>,
    pub resource_requirements: ResourceRequirements,
    pub health_endpoints: Vec<String>,
    pub services: Vec<PrimalService>,
    pub metadata: HashMap<String, String>,
}

/// Service provided by a primal - using unified protocol type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalService {
    pub name: String,
    pub port: Option<u16>,
    pub protocol: PortProtocol,                  // Now using unified type
    pub health_check: Option<HealthCheckConfig>, // Now using unified type
    pub capabilities: HashSet<PrimalCapability>,
}

/// Niche template definition - now completely universal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub features: Vec<String>,
    pub required_capabilities: HashSet<PrimalCapability>,
    pub preferred_primals: Vec<String>, // Optional hints, not requirements
    pub manifest_template: String,
    pub customization_options: Vec<CustomizationOption>,
    pub metadata: HashMap<String, String>,
}

/// Niche category classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NicheCategory {
    WebDevelopment,
    AIResearch,
    Gaming,
    DataScience,
    DevOps,
    Security,
    Custom(String),
}

impl NicheCategory {
    pub fn display_name(&self) -> &str {
        match self {
            NicheCategory::WebDevelopment => "Web Development",
            NicheCategory::AIResearch => "AI Research",
            NicheCategory::Gaming => "Gaming",
            NicheCategory::DataScience => "Data Science",
            NicheCategory::DevOps => "DevOps",
            NicheCategory::Security => "Security",
            NicheCategory::Custom(name) => name,
        }
    }
}

/// Niche difficulty level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NicheDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Customization option for niche templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub option_type: CustomizationType,
    pub default_value: String,
    pub required: bool,
    pub validation_regex: Option<String>,
    pub depends_on_capability: Option<PrimalCapability>,
}

/// Types of customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomizationType {
    Text,
    Select(Vec<String>),
    Number { min: Option<i32>, max: Option<i32> },
    Boolean,
    MultiSelect(Vec<String>),
    Capabilities(HashSet<PrimalCapability>),
}

/// Team information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
    pub created_at: String,
    pub status: TeamStatus,
    pub workspace_url: Option<String>,
    pub size: TeamSize,
    pub focus_area: String,
    pub experience_level: ExperienceLevel,
    pub required_capabilities: HashSet<PrimalCapability>,
    pub preferred_primals: Vec<String>,
}

/// Team status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TeamStatus {
    Active,
    Inactive,
    Configuring,
    Deploying,
    Archived,
}

/// Team size classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TeamSize {
    Individual,
    Solo,
    Small,  // 2-5 people
    Medium, // 6-15 people
    Large,  // 16+ people
    Enterprise,
}

/// Experience level classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub status: DeploymentStatus,
    pub created_at: String,
    pub last_updated: String,
    pub primals: Vec<String>,
    pub capabilities: Vec<PrimalCapability>,
    pub resource_usage: ResourceUsage,
    pub manifest_path: String,
    // Additional fields needed by data.rs
    pub health_status: Health,
    pub team: String,
    pub updated_at: String,
    pub services: Vec<String>,
    pub health_score: f64,
}

/// Deployment status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Preparing,
    Deploying,
    Running,
    Stopped,
    Failed,
    Error,
    Updating,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub storage_percent: f64,
    pub network_mbps: f64,
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub storage_gb: f64,
}

// Health status - now directly uses the unified Health type from biomeos-types

/// Health check result for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: Health,
    pub last_check: String,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub primal: String,
    pub status: ServiceStatus,
    pub endpoints: Vec<String>,
    pub health_check: HealthCheck,
    pub port: Option<u16>,
    pub health: Health,
    pub uptime: String,
    pub primal_name: String,
    pub capabilities: HashSet<PrimalCapability>,
}

/// Service status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Failed,
    Error,
}

/// Team data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamData {
    pub team_info: TeamInfo,
    pub selected_primals: Vec<String>,
    pub available_primals: Vec<PrimalDefinition>,
    pub resource_requirements: ResourceRequirements,
}

/// Resource quota for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bandwidth_mbps: f64,
    pub used_cpu_cores: f64,
    pub used_memory_gb: f64,
    pub used_storage_gb: f64,
    pub used_deployments: usize,
}

/// Deployment data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentData {
    pub deployments: Vec<DeploymentInfo>,
    pub services: Vec<ServiceInfo>,
    pub total_resource_usage: ResourceUsage,
    pub active_deployments: usize,
    pub available_primals: Vec<PrimalDefinition>,
}

/// Primal registry - dynamic discovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRegistry {
    pub primals: HashMap<String, PrimalDefinition>,
    pub capabilities: HashMap<PrimalCapability, Vec<String>>,
    pub last_updated: String,
}

impl TeamData {
    pub fn new() -> Self {
        Self {
            team_info: TeamInfo {
                id: String::new(),
                name: String::new(),
                description: String::new(),
                members: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                status: TeamStatus::Active,
                workspace_url: None,
                size: TeamSize::Solo,
                focus_area: String::new(),
                experience_level: ExperienceLevel::Beginner,
                required_capabilities: HashSet::new(),
                preferred_primals: Vec::new(),
            },
            selected_primals: Vec::new(),
            available_primals: Vec::new(),
            resource_requirements: ResourceRequirements::default(),
        }
    }
}

impl Default for TeamData {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentData {
    pub fn new() -> Self {
        Self {
            deployments: Vec::new(),
            services: Vec::new(),
            total_resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_percent: 0.0,
                storage_percent: 0.0,
                network_mbps: 0.0,
                cpu_cores: 0.0,
                memory_gb: 0.0,
                storage_gb: 0.0,
            },
            active_deployments: 0,
            available_primals: Vec::new(),
        }
    }
}

impl Default for DeploymentData {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimalRegistry {
    pub fn new() -> Self {
        Self {
            primals: HashMap::new(),
            capabilities: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn find_primals_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Vec<&PrimalDefinition> {
        if let Some(primal_names) = self.capabilities.get(capability) {
            primal_names
                .iter()
                .filter_map(|name| self.primals.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn find_primals_by_capabilities(
        &self,
        capabilities: &HashSet<PrimalCapability>,
    ) -> Vec<&PrimalDefinition> {
        self.primals
            .values()
            .filter(|primal| {
                capabilities
                    .iter()
                    .any(|cap| primal.capabilities.contains(cap))
            })
            .collect()
    }

    pub fn get_all_capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.keys().cloned().collect()
    }

    pub fn register_primal(&mut self, primal: PrimalDefinition) {
        // Update capabilities index
        for capability in &primal.capabilities {
            self.capabilities
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(primal.name.clone());
        }

        // Register the primal
        self.primals.insert(primal.name.clone(), primal);
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

impl Default for PrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamInfo {
    /// Create a basic TeamInfo with minimal required fields
    pub fn basic(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            members: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: TeamStatus::Active,
            workspace_url: None,
            size: TeamSize::Small,
            focus_area: "Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: std::collections::HashSet::new(),
            preferred_primals: Vec::new(),
        }
    }
}

impl ServiceInfo {
    /// Create a basic ServiceInfo with minimal required fields
    pub fn basic(name: String, primal: String) -> Self {
        Self {
            name: name.clone(),
            primal: primal.clone(),
            primal_name: primal,
            status: ServiceStatus::Running,
            port: Some(8080),
            endpoints: vec!["http://localhost:8080".to_string()],
            health: biomeos_types::Health::Healthy,
            uptime: "Unknown".to_string(),
            capabilities: std::collections::HashSet::new(),
            health_check: HealthCheck {
                status: biomeos_types::Health::Healthy,
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 50,
                error_message: None,
            },
        }
    }
}

impl DeploymentInfo {
    /// Create a basic DeploymentInfo with minimal required fields
    pub fn basic(id: String, name: String, team: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            name,
            status: DeploymentStatus::Running,
            created_at: now.clone(),
            last_updated: now.clone(),
            primals: Vec::new(),
            capabilities: Vec::new(),
            resource_usage: ResourceUsage::default(),
            manifest_path: "/unknown".to_string(),
            health_status: biomeos_types::Health::Healthy,
            team,
            updated_at: now,
            services: Vec::new(),
            health_score: 0.9,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            storage_percent: 0.0,
            network_mbps: 0.0,
            cpu_cores: 1.0,
            memory_gb: 2.0,
            storage_gb: 10.0,
        }
    }
}
