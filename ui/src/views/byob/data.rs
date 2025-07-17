//! BYOB Data Management
//!
//! This module handles live data management, dynamic primal discovery,
//! and data persistence for the BYOB system.
//!
//! The system is completely universal and agnostic to specific primal names,
//! using capability-based discovery and configuration-driven initialization.

use super::types::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Dynamic primal discovery system
pub struct PrimalDiscovery {
    registry: PrimalRegistry,
    config_path: String,
}

impl PrimalDiscovery {
    pub fn new(config_path: &str) -> Self {
        let mut discovery = Self {
            registry: PrimalRegistry::new(),
            config_path: config_path.to_string(),
        };

        // Load primals from configuration
        discovery.load_primals_from_config();

        // If no primals found, use fallback system
        if discovery.registry.primals.is_empty() {
            discovery.load_fallback_primals();
        }

        discovery
    }

    fn load_primals_from_config(&mut self) {
        // Try to load from various configuration sources
        let config_paths = vec![
            format!("{}/primals.yaml", self.config_path),
            format!("{}/primals.json", self.config_path),
            format!("{}/primals/", self.config_path),
            "config/primals.yaml".to_string(),
            "primals.yaml".to_string(),
        ];

        for path in config_paths {
            if let Ok(primals) = self.load_primals_from_path(&path) {
                for primal in primals {
                    self.registry.register_primal(primal);
                }
                return;
            }
        }
    }

    fn load_primals_from_path(
        &self,
        path: &str,
    ) -> Result<Vec<PrimalDefinition>, Box<dyn std::error::Error>> {
        if Path::new(path).is_dir() {
            // Load from directory of primal definition files
            let mut primals = Vec::new();
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();
                if let Some(extension) = file_path.extension() {
                    if extension == "yaml" || extension == "yml" || extension == "json" {
                        if let Ok(primal) = self.load_primal_from_file(&file_path) {
                            primals.push(primal);
                        }
                    }
                }
            }
            Ok(primals)
        } else {
            // Load from single file
            let primals = self.load_primals_from_file(path)?;
            Ok(primals)
        }
    }

    fn load_primal_from_file(
        &self,
        file_path: &Path,
    ) -> Result<PrimalDefinition, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        if file_path.extension().unwrap_or_default() == "json" {
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(serde_yaml::from_str(&content)?)
        }
    }

    fn load_primals_from_file(
        &self,
        file_path: &str,
    ) -> Result<Vec<PrimalDefinition>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        if file_path.ends_with(".json") {
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(serde_yaml::from_str(&content)?)
        }
    }

    fn load_fallback_primals(&mut self) {
        // Universal fallback system with current ecosystem primals
        let fallback_primals = vec![
            // Current ecosystem primals with their capabilities
            PrimalDefinition {
                name: "toadstool".to_string(),
                display_name: "Toadstool".to_string(),
                description: "Compute and processing services".to_string(),
                capabilities: [
                    PrimalCapability::Compute,
                    PrimalCapability::DataProcessing,
                    PrimalCapability::Analytics,
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2.0,
                    memory_gb: 4.0,
                    storage_gb: 20.0,
                    gpu_required: false,
                    network_bandwidth_mbps: 50.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "compute-service".to_string(),
                    port: Some(8080),
                    protocol: ServiceProtocol::HTTP,
                    health_check: None,
                    capabilities: [PrimalCapability::Compute].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "songbird".to_string(),
                display_name: "Songbird".to_string(),
                description: "Networking and orchestration services".to_string(),
                capabilities: [
                    PrimalCapability::Networking,
                    PrimalCapability::Orchestration,
                    PrimalCapability::ServiceDiscovery,
                    PrimalCapability::LoadBalancing,
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_gb: 2.0,
                    storage_gb: 10.0,
                    gpu_required: false,
                    network_bandwidth_mbps: 100.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "orchestration-service".to_string(),
                    port: Some(8081),
                    protocol: ServiceProtocol::HTTP,
                    health_check: None,
                    capabilities: [PrimalCapability::Orchestration].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "nestgate".to_string(),
                display_name: "Nestgate".to_string(),
                description: "Storage and data management services".to_string(),
                capabilities: [PrimalCapability::Storage, PrimalCapability::DataProcessing]
                    .into_iter()
                    .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_gb: 2.0,
                    storage_gb: 100.0,
                    gpu_required: false,
                    network_bandwidth_mbps: 25.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "storage-service".to_string(),
                    port: Some(8082),
                    protocol: ServiceProtocol::HTTP,
                    health_check: None,
                    capabilities: [PrimalCapability::Storage].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "beardog".to_string(),
                display_name: "Beardog".to_string(),
                description: "Security and encryption services".to_string(),
                capabilities: [
                    PrimalCapability::Security,
                    PrimalCapability::Encryption,
                    PrimalCapability::Authentication,
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.5,
                    memory_gb: 3.0,
                    storage_gb: 15.0,
                    gpu_required: false,
                    network_bandwidth_mbps: 30.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "security-service".to_string(),
                    port: Some(8083),
                    protocol: ServiceProtocol::HTTPS,
                    health_check: None,
                    capabilities: [PrimalCapability::Security].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "squirrel".to_string(),
                display_name: "Squirrel".to_string(),
                description: "AI and machine learning services".to_string(),
                capabilities: [
                    PrimalCapability::AI,
                    PrimalCapability::MachineLearning,
                    PrimalCapability::Analytics,
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 4.0,
                    memory_gb: 8.0,
                    storage_gb: 50.0,
                    gpu_required: true,
                    network_bandwidth_mbps: 75.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "ai-service".to_string(),
                    port: Some(8084),
                    protocol: ServiceProtocol::HTTP,
                    health_check: None,
                    capabilities: [PrimalCapability::AI].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            // Example future primal to show extensibility
            PrimalDefinition {
                name: "future-primal".to_string(),
                display_name: "Future Primal".to_string(),
                description: "Example future primal with custom capabilities".to_string(),
                capabilities: [
                    PrimalCapability::Custom("quantum-computing".to_string()),
                    PrimalCapability::Custom("time-travel".to_string()),
                    PrimalCapability::Compute,
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 8.0,
                    memory_gb: 16.0,
                    storage_gb: 100.0,
                    gpu_required: true,
                    network_bandwidth_mbps: 200.0,
                    required_capabilities: HashSet::new(),
                },
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "quantum-service".to_string(),
                    port: Some(8085),
                    protocol: ServiceProtocol::Custom("quantum-protocol".to_string()),
                    health_check: None,
                    capabilities: [PrimalCapability::Custom("quantum-computing".to_string())]
                        .into_iter()
                        .collect(),
                }],
                metadata: [
                    ("version".to_string(), "2.0".to_string()),
                    ("experimental".to_string(), "true".to_string()),
                ]
                .into_iter()
                .collect(),
            },
        ];

        for primal in fallback_primals {
            self.registry.register_primal(primal);
        }
    }

    pub fn get_registry(&self) -> &PrimalRegistry {
        &self.registry
    }

    pub fn find_primals_for_capabilities(
        &self,
        capabilities: &HashSet<PrimalCapability>,
    ) -> Vec<&PrimalDefinition> {
        self.registry.find_primals_by_capabilities(capabilities)
    }

    pub fn get_all_primals(&self) -> Vec<&PrimalDefinition> {
        self.registry.primals.values().collect()
    }

    pub fn get_primal(&self, name: &str) -> Option<&PrimalDefinition> {
        self.registry.primals.get(name)
    }
}

/// Get universal primal discovery instance
pub fn get_primal_discovery() -> PrimalDiscovery {
    PrimalDiscovery::new("config")
}

/// Get mock team data for demonstration
pub fn get_mock_teams() -> Vec<TeamInfo> {
    vec![
        TeamInfo {
            name: "Frontend Wizards".to_string(),
            description: "Specialized in modern web frontend development".to_string(),
            size: TeamSize::Small,
            focus_area: "Web Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: [
                PrimalCapability::Compute,
                PrimalCapability::WebDevelopment,
                PrimalCapability::Networking,
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
        },
        TeamInfo {
            name: "AI Research Lab".to_string(),
            description: "Machine learning and AI research team".to_string(),
            size: TeamSize::Medium,
            focus_area: "AI Research".to_string(),
            experience_level: ExperienceLevel::Advanced,
            required_capabilities: [
                PrimalCapability::AI,
                PrimalCapability::MachineLearning,
                PrimalCapability::Compute,
                PrimalCapability::Analytics,
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["squirrel".to_string(), "toadstool".to_string()],
        },
        TeamInfo {
            name: "Gaming Studio".to_string(),
            description: "Indie game development studio".to_string(),
            size: TeamSize::Small,
            focus_area: "Game Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: [
                PrimalCapability::Gaming,
                PrimalCapability::Compute,
                PrimalCapability::Networking,
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
        },
        TeamInfo {
            name: "Security Experts".to_string(),
            description: "Cybersecurity and encryption specialists".to_string(),
            size: TeamSize::Medium,
            focus_area: "Security".to_string(),
            experience_level: ExperienceLevel::Expert,
            required_capabilities: [
                PrimalCapability::Security,
                PrimalCapability::Encryption,
                PrimalCapability::Authentication,
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["beardog".to_string()],
        },
    ]
}

/// Get mock deployment data with universal primal support
pub fn get_mock_deployments() -> Vec<DeploymentInfo> {
    vec![
        DeploymentInfo {
            id: "deploy-001".to_string(),
            name: "React Dashboard".to_string(),
            status: DeploymentStatus::Running,
            created_at: "2025-01-15T10:30:00Z".to_string(),
            last_updated: "2025-01-15T14:22:00Z".to_string(),
            resource_usage: ResourceUsage {
                cpu_percent: 25.0,
                memory_percent: 40.0,
                storage_percent: 15.0,
                network_mbps: 12.5,
                cpu_cores: 2.0,
                memory_gb: 4.0,
                storage_gb: 20.0,
            },
            health_status: HealthStatus::Healthy,
            primals: vec!["toadstool".to_string(), "songbird".to_string()],
            capabilities: [
                PrimalCapability::Compute,
                PrimalCapability::WebDevelopment,
                PrimalCapability::Networking,
            ]
            .into_iter()
            .collect(),
            team: "frontend-team".to_string(),
            updated_at: "2025-01-15T14:22:00Z".to_string(),
            services: vec!["web-service".to_string()],
            health_score: 0.95,
        },
        DeploymentInfo {
            id: "deploy-002".to_string(),
            name: "ML Training Pipeline".to_string(),
            status: DeploymentStatus::Running,
            created_at: "2025-01-15T09:15:00Z".to_string(),
            last_updated: "2025-01-15T14:20:00Z".to_string(),
            resource_usage: ResourceUsage {
                cpu_percent: 85.0,
                memory_percent: 70.0,
                storage_percent: 60.0,
                network_mbps: 45.0,
                cpu_cores: 8.0,
                memory_gb: 16.0,
                storage_gb: 100.0,
            },
            health_status: HealthStatus::Warning,
            primals: vec!["squirrel".to_string(), "toadstool".to_string()],
            capabilities: [
                PrimalCapability::AI,
                PrimalCapability::MachineLearning,
                PrimalCapability::Compute,
            ]
            .into_iter()
            .collect(),
            team: "ai-team".to_string(),
            updated_at: "2025-01-15T14:20:00Z".to_string(),
            services: vec!["ml-service".to_string(), "training-service".to_string()],
            health_score: 0.78,
        },
        DeploymentInfo {
            id: "deploy-003".to_string(),
            name: "Secure Data Vault".to_string(),
            status: DeploymentStatus::Preparing,
            created_at: "2025-01-15T14:00:00Z".to_string(),
            last_updated: "2025-01-15T14:25:00Z".to_string(),
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_percent: 0.0,
                storage_percent: 0.0,
                network_mbps: 0.0,
                cpu_cores: 0.0,
                memory_gb: 0.0,
                storage_gb: 0.0,
            },
            health_status: HealthStatus::Unknown,
            primals: vec!["beardog".to_string(), "nestgate".to_string()],
            capabilities: [
                PrimalCapability::Security,
                PrimalCapability::Storage,
                PrimalCapability::Encryption,
            ]
            .into_iter()
            .collect(),
            team: "security-team".to_string(),
            updated_at: "2025-01-15T14:25:00Z".to_string(),
            services: vec!["vault-service".to_string()],
            health_score: 0.0,
        },
    ]
}

/// Get mock service data with universal primal support
pub fn get_mock_services() -> Vec<ServiceInfo> {
    vec![
        ServiceInfo {
            name: "compute-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8080),
            health: HealthStatus::Healthy,
            uptime: "2h 45m".to_string(),
            primal_name: "toadstool".to_string(),
            capabilities: [PrimalCapability::Compute].into_iter().collect(),
        },
        ServiceInfo {
            name: "orchestration-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8081),
            health: HealthStatus::Healthy,
            uptime: "3h 12m".to_string(),
            primal_name: "songbird".to_string(),
            capabilities: [
                PrimalCapability::Orchestration,
                PrimalCapability::Networking,
            ]
            .into_iter()
            .collect(),
        },
        ServiceInfo {
            name: "storage-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8082),
            health: HealthStatus::Warning,
            uptime: "1h 30m".to_string(),
            primal_name: "nestgate".to_string(),
            capabilities: [PrimalCapability::Storage].into_iter().collect(),
        },
        ServiceInfo {
            name: "security-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8083),
            health: HealthStatus::Healthy,
            uptime: "4h 15m".to_string(),
            primal_name: "beardog".to_string(),
            capabilities: [PrimalCapability::Security, PrimalCapability::Encryption]
                .into_iter()
                .collect(),
        },
        ServiceInfo {
            name: "ai-service".to_string(),
            status: ServiceStatus::Starting,
            port: Some(8084),
            health: HealthStatus::Unknown,
            uptime: "0m".to_string(),
            primal_name: "squirrel".to_string(),
            capabilities: [PrimalCapability::AI, PrimalCapability::MachineLearning]
                .into_iter()
                .collect(),
        },
    ]
}

/// Get resource usage for any primal by capability
pub fn get_resource_usage_for_capability(capability: &PrimalCapability) -> ResourceUsage {
    match capability {
        PrimalCapability::Compute => ResourceUsage {
            cpu_percent: 60.0,
            memory_percent: 45.0,
            storage_percent: 30.0,
            network_mbps: 25.0,
            cpu_cores: 4.0,
            memory_gb: 8.0,
            storage_gb: 50.0,
        },
        PrimalCapability::AI | PrimalCapability::MachineLearning => ResourceUsage {
            cpu_percent: 85.0,
            memory_percent: 70.0,
            storage_percent: 40.0,
            network_mbps: 50.0,
            cpu_cores: 8.0,
            memory_gb: 16.0,
            storage_gb: 100.0,
        },
        PrimalCapability::Storage => ResourceUsage {
            cpu_percent: 20.0,
            memory_percent: 30.0,
            storage_percent: 80.0,
            network_mbps: 15.0,
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 200.0,
        },
        PrimalCapability::Networking | PrimalCapability::Orchestration => ResourceUsage {
            cpu_percent: 30.0,
            memory_percent: 25.0,
            storage_percent: 10.0,
            network_mbps: 100.0,
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 20.0,
        },
        PrimalCapability::Security | PrimalCapability::Encryption => ResourceUsage {
            cpu_percent: 40.0,
            memory_percent: 35.0,
            storage_percent: 20.0,
            network_mbps: 30.0,
            cpu_cores: 3.0,
            memory_gb: 6.0,
            storage_gb: 30.0,
        },
        _ => ResourceUsage {
            cpu_percent: 25.0,
            memory_percent: 30.0,
            storage_percent: 15.0,
            network_mbps: 20.0,
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 25.0,
        },
    }
}

/// Check if a primal supports a given capability
pub fn primal_supports_capability(primal_name: &str, capability: &PrimalCapability) -> bool {
    let discovery = get_primal_discovery();
    if let Some(primal) = discovery.get_primal(primal_name) {
        primal.capabilities.contains(capability)
    } else {
        false
    }
}

/// Get health status for any primal
pub fn get_primal_health_status(primal_name: &str) -> HealthStatus {
    let discovery = get_primal_discovery();
    if let Some(_primal) = discovery.get_primal(primal_name) {
        // In a real system, this would check actual health endpoints
        // For now, return mock status based on name hash for consistency
        match primal_name.len() % 4 {
            0 => HealthStatus::Healthy,
            1 => HealthStatus::Warning,
            2 => HealthStatus::Healthy,
            _ => HealthStatus::Critical,
        }
    } else {
        HealthStatus::Unknown
    }
}

/// Get services for any primal
pub fn get_primal_services(primal_name: &str) -> Vec<ServiceInfo> {
    let discovery = get_primal_discovery();
    if let Some(primal) = discovery.get_primal(primal_name) {
        primal
            .services
            .iter()
            .map(|service| ServiceInfo {
                name: service.name.clone(),
                status: ServiceStatus::Running,
                port: service.port,
                health: HealthStatus::Healthy,
                uptime: "1h 30m".to_string(),
                primal_name: primal_name.to_string(),
                capabilities: service.capabilities.clone(),
            })
            .collect()
    } else {
        Vec::new()
    }
}
