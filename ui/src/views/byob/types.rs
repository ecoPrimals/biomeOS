//! BYOB Types and Data Structures
//!
//! This module contains all the data structures, enums, and types used
//! throughout the BYOB (Build Your Own Biome) system.
//!
//! The system is designed to be completely universal and agnostic,
//! supporting any primal names and capabilities without hardcoding.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

/// Universal capability system - extensible for any functionality
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Core capabilities
    Compute,
    Storage,
    Networking,
    Security,
    AI,
    Orchestration,

    // Advanced capabilities
    Encryption,
    Authentication,
    LoadBalancing,
    ServiceDiscovery,
    Monitoring,
    Analytics,

    // Specialized capabilities
    Gaming,
    WebDevelopment,
    MachineLearning,
    DataProcessing,
    Federation,

    // Custom capability for extensibility
    Custom(String),
}

/// Service provided by a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalService {
    pub name: String,
    pub port: Option<u16>,
    pub protocol: ServiceProtocol,
    pub health_check: Option<HealthCheck>,
    pub capabilities: HashSet<PrimalCapability>,
}

/// Service protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceProtocol {
    HTTP,
    HTTPS,
    TCP,
    UDP,
    WebSocket,
    gRPC,
    Custom(String),
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
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
    pub name: String,
    pub description: String,
    pub size: TeamSize,
    pub focus_area: String,
    pub experience_level: ExperienceLevel,
    pub required_capabilities: HashSet<PrimalCapability>,
    pub preferred_primals: Vec<String>,
}

/// Team size classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TeamSize {
    Solo,
    Small,  // 2-5 people
    Medium, // 6-15 people
    Large,  // 16+ people
}

/// Experience level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Deployment status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    NotStarted,
    Preparing,
    Deploying,
    Running,
    Failed,
    Completed,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub status: DeploymentStatus,
    pub created_at: String,
    pub last_updated: String,
    pub resource_usage: ResourceUsage,
    pub health_status: HealthStatus,
    pub primals: Vec<String>,
    pub capabilities: HashSet<PrimalCapability>,
    // Compatibility fields
    pub team: String,
    pub updated_at: String,
    pub services: Vec<String>,
    pub health_score: f64,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub storage_percent: f64,
    pub network_mbps: f64,
    // Compatibility fields
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub storage_gb: f64,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub port: Option<u16>,
    pub health: HealthStatus,
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

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub storage_gb: f64,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: f64,
    pub required_capabilities: HashSet<PrimalCapability>,
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
                name: String::new(),
                description: String::new(),
                size: TeamSize::Solo,
                focus_area: String::new(),
                experience_level: ExperienceLevel::Beginner,
                required_capabilities: HashSet::new(),
                preferred_primals: Vec::new(),
            },
            selected_primals: Vec::new(),
            available_primals: Vec::new(),
            resource_requirements: ResourceRequirements {
                cpu_cores: 1.0,
                memory_gb: 2.0,
                storage_gb: 10.0,
                gpu_required: false,
                network_bandwidth_mbps: 10.0,
                required_capabilities: HashSet::new(),
            },
        }
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

impl PrimalRegistry {
    pub fn new() -> Self {
        Self {
            primals: HashMap::new(),
            capabilities: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn register_primal(&mut self, primal: PrimalDefinition) {
        // Update capability index
        for capability in &primal.capabilities {
            self.capabilities
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(primal.name.clone());
        }

        self.primals.insert(primal.name.clone(), primal);
        self.last_updated = chrono::Utc::now().to_rfc3339();
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

    pub fn get_all_capabilities(&self) -> HashSet<PrimalCapability> {
        self.capabilities.keys().cloned().collect()
    }
}

// Helper implementations for UI display
impl PrimalCapability {
    pub fn display_name(&self) -> String {
        match self {
            PrimalCapability::Compute => "Compute".to_string(),
            PrimalCapability::Storage => "Storage".to_string(),
            PrimalCapability::Networking => "Networking".to_string(),
            PrimalCapability::Security => "Security".to_string(),
            PrimalCapability::AI => "AI/ML".to_string(),
            PrimalCapability::Orchestration => "Orchestration".to_string(),
            PrimalCapability::Encryption => "Encryption".to_string(),
            PrimalCapability::Authentication => "Authentication".to_string(),
            PrimalCapability::LoadBalancing => "Load Balancing".to_string(),
            PrimalCapability::ServiceDiscovery => "Service Discovery".to_string(),
            PrimalCapability::Monitoring => "Monitoring".to_string(),
            PrimalCapability::Analytics => "Analytics".to_string(),
            PrimalCapability::Gaming => "Gaming".to_string(),
            PrimalCapability::WebDevelopment => "Web Development".to_string(),
            PrimalCapability::MachineLearning => "Machine Learning".to_string(),
            PrimalCapability::DataProcessing => "Data Processing".to_string(),
            PrimalCapability::Federation => "Federation".to_string(),
            PrimalCapability::Custom(name) => name.clone(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            PrimalCapability::Compute => "Computational processing and task execution".to_string(),
            PrimalCapability::Storage => "Data storage and retrieval services".to_string(),
            PrimalCapability::Networking => "Network communication and routing".to_string(),
            PrimalCapability::Security => "Security services and protection".to_string(),
            PrimalCapability::AI => "Artificial intelligence and machine learning".to_string(),
            PrimalCapability::Orchestration => "Service orchestration and management".to_string(),
            PrimalCapability::Encryption => {
                "Data encryption and cryptographic services".to_string()
            }
            PrimalCapability::Authentication => "User authentication and authorization".to_string(),
            PrimalCapability::LoadBalancing => {
                "Load balancing and traffic distribution".to_string()
            }
            PrimalCapability::ServiceDiscovery => "Service discovery and registration".to_string(),
            PrimalCapability::Monitoring => "System monitoring and observability".to_string(),
            PrimalCapability::Analytics => "Data analytics and insights".to_string(),
            PrimalCapability::Gaming => "Gaming-specific services and features".to_string(),
            PrimalCapability::WebDevelopment => "Web development tools and services".to_string(),
            PrimalCapability::MachineLearning => {
                "Machine learning training and inference".to_string()
            }
            PrimalCapability::DataProcessing => "Data processing and transformation".to_string(),
            PrimalCapability::Federation => {
                "Cross-system federation and interoperability".to_string()
            }
            PrimalCapability::Custom(name) => format!("Custom capability: {}", name),
        }
    }
}

impl NicheCategory {
    pub fn display_name(&self) -> String {
        match self {
            NicheCategory::WebDevelopment => "Web Development".to_string(),
            NicheCategory::AIResearch => "AI Research".to_string(),
            NicheCategory::Gaming => "Gaming".to_string(),
            NicheCategory::DataScience => "Data Science".to_string(),
            NicheCategory::DevOps => "DevOps".to_string(),
            NicheCategory::Security => "Security".to_string(),
            NicheCategory::Custom(name) => name.clone(),
        }
    }
}

impl Default for PrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TeamData {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeploymentData {
    fn default() -> Self {
        Self::new()
    }
}
