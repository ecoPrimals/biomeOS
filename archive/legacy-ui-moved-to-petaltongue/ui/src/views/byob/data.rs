//! BYOB Data Management
//!
//! This module handles live data management, dynamic primal discovery,
//! and data persistence for the BYOB system.
//!
//! The system is completely universal and agnostic to specific primal names,
//! using capability-based discovery and configuration-driven initialization.

use super::types::*;
use biomeos_types::service::networking::PortProtocol;
use biomeos_types::{Health, PrimalCapability, ResourceRequirements};
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
                    PrimalCapability::compute(),
                    PrimalCapability::data_processing(),
                    PrimalCapability::analytics(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "compute-service".to_string(),
                    port: Some(8080),
                    protocol: PortProtocol::Http,
                    health_check: None,
                    capabilities: [PrimalCapability::compute()].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "songbird".to_string(),
                display_name: "Songbird".to_string(),
                description: "Networking and orchestration services".to_string(),
                capabilities: [
                    PrimalCapability::networking(),
                    PrimalCapability::orchestration(),
                    PrimalCapability::service_discovery(),
                    PrimalCapability::load_balancing(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "orchestration-service".to_string(),
                    port: Some(8081),
                    protocol: PortProtocol::Http,
                    health_check: None,
                    capabilities: [PrimalCapability::orchestration()].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "nestgate".to_string(),
                display_name: "Nestgate".to_string(),
                description: "Storage and data management services".to_string(),
                capabilities: [
                    PrimalCapability::storage(),
                    PrimalCapability::data_processing(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "storage-service".to_string(),
                    port: Some(8082),
                    protocol: PortProtocol::Http,
                    health_check: None,
                    capabilities: [PrimalCapability::storage()].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "beardog".to_string(),
                display_name: "Beardog".to_string(),
                description: "Security and encryption services".to_string(),
                capabilities: [
                    PrimalCapability::security(),
                    PrimalCapability::encryption(),
                    PrimalCapability::authentication(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "security-service".to_string(),
                    port: Some(8083),
                    protocol: PortProtocol::Https,
                    health_check: None,
                    capabilities: [PrimalCapability::security()].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            PrimalDefinition {
                name: "squirrel".to_string(),
                display_name: "Squirrel".to_string(),
                description: "AI and machine learning services".to_string(),
                capabilities: [
                    PrimalCapability::ai(),
                    PrimalCapability::machine_learning(),
                    PrimalCapability::analytics(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "ai-service".to_string(),
                    port: Some(8084),
                    protocol: PortProtocol::Http,
                    health_check: None,
                    capabilities: [PrimalCapability::ai()].into_iter().collect(),
                }],
                metadata: HashMap::new(),
            },
            // Example future primal to show extensibility
            PrimalDefinition {
                name: "future-primal".to_string(),
                display_name: "Future Primal".to_string(),
                description: "Example future primal with custom capabilities".to_string(),
                capabilities: [
                    PrimalCapability::custom("quantum-computing".to_string()),
                    PrimalCapability::custom("time-travel".to_string()),
                    PrimalCapability::compute(),
                ]
                .into_iter()
                .collect(),
                resource_requirements: ResourceRequirements::default(),
                health_endpoints: vec!["/health".to_string()],
                services: vec![PrimalService {
                    name: "quantum-service".to_string(),
                    port: Some(8085),
                    protocol: PortProtocol::Custom("quantum-protocol".to_string()),
                    health_check: None,
                    capabilities: [PrimalCapability::custom("quantum-computing".to_string())]
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
            id: "team-frontend".to_string(),
            name: "Frontend Wizards".to_string(),
            description: "Specialized in modern web frontend development".to_string(),
            size: TeamSize::Small,
            focus_area: "Web Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: [
                PrimalCapability::compute(),
                PrimalCapability::web_development(),
                PrimalCapability::networking(),
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
            created_at: "2024-01-01".to_string(),
            members: vec!["alice".to_string(), "bob".to_string()],
            status: TeamStatus::Active,
            workspace_url: Some("https://frontend.local".to_string()),
        },
        TeamInfo {
            id: "team-ai".to_string(),
            name: "AI Research Lab".to_string(),
            description: "Machine learning and AI research team".to_string(),
            size: TeamSize::Medium,
            focus_area: "AI Research".to_string(),
            experience_level: ExperienceLevel::Advanced,
            required_capabilities: [
                PrimalCapability::ai(),
                PrimalCapability::machine_learning(),
                PrimalCapability::compute(),
                PrimalCapability::analytics(),
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["squirrel".to_string(), "toadstool".to_string()],
            created_at: "2024-01-01".to_string(),
            members: vec!["charlie".to_string(), "diana".to_string()],
            status: TeamStatus::Active,
            workspace_url: Some("https://ai.local".to_string()),
        },
        TeamInfo {
            id: "team-gaming".to_string(),
            name: "Gaming Studio".to_string(),
            description: "Indie game development studio".to_string(),
            size: TeamSize::Small,
            focus_area: "Game Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: [
                PrimalCapability::gaming(),
                PrimalCapability::compute(),
                PrimalCapability::networking(),
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
            created_at: "2024-01-01".to_string(),
            members: vec!["eve".to_string(), "frank".to_string()],
            status: TeamStatus::Active,
            workspace_url: Some("https://gaming.local".to_string()),
        },
        TeamInfo {
            id: "team-security".to_string(),
            name: "Security Experts".to_string(),
            description: "Cybersecurity and encryption specialists".to_string(),
            size: TeamSize::Medium,
            focus_area: "Security".to_string(),
            experience_level: ExperienceLevel::Expert,
            required_capabilities: [
                PrimalCapability::security(),
                PrimalCapability::encryption(),
                PrimalCapability::authentication(),
            ]
            .into_iter()
            .collect(),
            preferred_primals: vec!["beardog".to_string()],
            created_at: "2024-01-01".to_string(),
            members: vec!["grace".to_string(), "henry".to_string()],
            status: TeamStatus::Active,
            workspace_url: Some("https://security.local".to_string()),
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
            health_status: Health::Healthy,
            primals: vec!["toadstool".to_string(), "songbird".to_string()],
            capabilities: [
                PrimalCapability::compute(),
                PrimalCapability::web_development(),
                PrimalCapability::networking(),
            ]
            .into_iter()
            .collect(),
            team: "frontend-team".to_string(),
            updated_at: "2025-01-15T14:22:00Z".to_string(),
            services: vec!["web-service".to_string()],
            health_score: 0.95,
            manifest_path: "/deployments/react-dashboard.yaml".to_string(),
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
            health_status: Health::Degraded {
                issues: vec![],
                impact_score: Some(0.7),
            },
            primals: vec!["squirrel".to_string(), "toadstool".to_string()],
            capabilities: [
                PrimalCapability::ai(),
                PrimalCapability::machine_learning(),
                PrimalCapability::compute(),
            ]
            .into_iter()
            .collect(),
            team: "ai-team".to_string(),
            updated_at: "2025-01-15T14:20:00Z".to_string(),
            services: vec!["ml-service".to_string(), "training-service".to_string()],
            health_score: 0.78,
            manifest_path: "/deployments/ml-pipeline.yaml".to_string(),
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
            health_status: Health::Unknown {
                reason: "System initializing".to_string(),
                last_known: None,
            },
            primals: vec!["beardog".to_string(), "nestgate".to_string()],
            capabilities: [
                PrimalCapability::security(),
                PrimalCapability::storage(),
                PrimalCapability::encryption(),
            ]
            .into_iter()
            .collect(),
            team: "security-team".to_string(),
            updated_at: "2025-01-15T14:25:00Z".to_string(),
            services: vec!["vault-service".to_string()],
            health_score: 0.0,
            manifest_path: "/deployments/data-vault.yaml".to_string(),
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
            health: Health::Healthy,
            uptime: "2h 45m".to_string(),
            primal_name: "toadstool".to_string(),
            capabilities: [PrimalCapability::compute()].into_iter().collect(),
            primal: "toadstool".to_string(),
            endpoints: vec!["http://localhost:8080".to_string()],
            health_check: HealthCheck {
                status: biomeos_types::Health::Healthy,
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 25,
                error_message: None,
            },
        },
        ServiceInfo {
            name: "orchestration-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8081),
            health: Health::Healthy,
            uptime: "3h 12m".to_string(),
            primal_name: "songbird".to_string(),
            capabilities: [
                PrimalCapability::orchestration(),
                PrimalCapability::networking(),
            ]
            .into_iter()
            .collect(),
            primal: "songbird".to_string(),
            endpoints: vec!["http://localhost:8081".to_string()],
            health_check: HealthCheck {
                status: biomeos_types::Health::Healthy,
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 30,
                error_message: None,
            },
        },
        ServiceInfo {
            name: "storage-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8082),
            health: Health::Degraded {
                issues: vec![],
                impact_score: Some(0.6),
            },
            uptime: "1h 30m".to_string(),
            primal_name: "nestgate".to_string(),
            capabilities: [PrimalCapability::storage()].into_iter().collect(),
            primal: "nestgate".to_string(),
            endpoints: vec!["http://localhost:8082".to_string()],
            health_check: HealthCheck {
                status: biomeos_types::Health::Degraded {
                    issues: vec![],
                    impact_score: Some(0.6),
                },
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 100,
                error_message: Some("Slow response".to_string()),
            },
        },
        ServiceInfo {
            name: "security-service".to_string(),
            status: ServiceStatus::Running,
            port: Some(8083),
            health: Health::Healthy,
            uptime: "4h 15m".to_string(),
            primal_name: "beardog".to_string(),
            capabilities: [PrimalCapability::security(), PrimalCapability::encryption()]
                .into_iter()
                .collect(),
            primal: "beardog".to_string(),
            endpoints: vec!["http://localhost:8083".to_string()],
            health_check: HealthCheck {
                status: biomeos_types::Health::Healthy,
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 20,
                error_message: None,
            },
        },
        ServiceInfo {
            name: "ai-service".to_string(),
            status: ServiceStatus::Starting,
            port: Some(8084),
            health: Health::Unknown {
                reason: "Service starting".to_string(),
                last_known: None,
            },
            uptime: "0m".to_string(),
            primal_name: "squirrel".to_string(),
            capabilities: [PrimalCapability::ai(), PrimalCapability::machine_learning()]
                .into_iter()
                .collect(),
            primal: "squirrel".to_string(),
            endpoints: vec!["http://localhost:8084".to_string()],
            health_check: HealthCheck {
                status: biomeos_types::Health::Unknown {
                    reason: "Service starting".to_string(),
                    last_known: None,
                },
                last_check: chrono::Utc::now().to_rfc3339(),
                response_time_ms: 0,
                error_message: Some("Service unavailable".to_string()),
            },
        },
    ]
}

/// Get resource usage for any primal by capability
pub fn get_resource_usage_for_capability(capability: &PrimalCapability) -> ResourceUsage {
    match capability.category.as_str() {
        "compute" => ResourceUsage {
            cpu_percent: 60.0,
            memory_percent: 45.0,
            storage_percent: 30.0,
            network_mbps: 25.0,
            cpu_cores: 4.0,
            memory_gb: 8.0,
            storage_gb: 50.0,
        },
        "ai" | "machine-learning" => ResourceUsage {
            cpu_percent: 85.0,
            memory_percent: 70.0,
            storage_percent: 40.0,
            network_mbps: 50.0,
            cpu_cores: 8.0,
            memory_gb: 16.0,
            storage_gb: 100.0,
        },
        "storage" => ResourceUsage {
            cpu_percent: 20.0,
            memory_percent: 30.0,
            storage_percent: 80.0,
            network_mbps: 15.0,
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 200.0,
        },
        "networking" | "orchestration" => ResourceUsage {
            cpu_percent: 30.0,
            memory_percent: 25.0,
            storage_percent: 10.0,
            network_mbps: 100.0,
            cpu_cores: 2.0,
            memory_gb: 4.0,
            storage_gb: 20.0,
        },
        "security" | "encryption" => ResourceUsage {
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
            memory_percent: 20.0,
            storage_percent: 15.0,
            network_mbps: 10.0,
            cpu_cores: 1.0,
            memory_gb: 2.0,
            storage_gb: 10.0,
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
pub fn get_primal_health_status(primal_name: &str) -> Health {
    let discovery = get_primal_discovery();
    if let Some(_primal) = discovery.get_primal(primal_name) {
        // In a real system, this would check actual health endpoints
        // For now, return mock status based on name hash for consistency
        match primal_name.len() % 4 {
            0 => Health::Healthy,
            1 => Health::Degraded {
                issues: vec![],
                impact_score: Some(0.8),
            },
            2 => Health::Healthy,
            _ => Health::Critical {
                issues: vec![],
                affected_capabilities: vec![],
            },
        }
    } else {
        Health::Unknown {
            reason: "Primal not found".to_string(),
            last_known: None,
        }
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
                health: Health::Healthy,
                uptime: "1h 30m".to_string(),
                primal_name: primal_name.to_string(),
                capabilities: service.capabilities.clone(),
                primal: primal_name.to_string(),
                endpoints: vec![format!("http://localhost:{}", service.port.unwrap_or(8080))],
                health_check: HealthCheck {
                    status: biomeos_types::Health::Healthy,
                    last_check: chrono::Utc::now().to_rfc3339(),
                    response_time_ms: 35,
                    error_message: None,
                },
            })
            .collect()
    } else {
        Vec::new()
    }
}
