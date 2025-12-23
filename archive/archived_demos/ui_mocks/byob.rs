//! BYOB (Build Your Own Biome) Mock Data Provider
//!
//! This module provides mock data specifically for the BYOB workflow system,
//! including teams, deployments, services, and primal discovery data.

use std::collections::HashMap;
use biomeos_types::PrimalCapability;
use crate::views::byob::types::*;
use crate::state::{PrimalStatus, PrimalInfo};

// Mock-specific types for BYOB
#[derive(Debug, Clone)]
pub struct PrimalServiceInstance {
    pub name: String,
    pub endpoint: String,
    pub status: String,
    pub uptime: String,
}

#[derive(Debug, Clone)]
pub struct DeploymentMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_io: f32,
}

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub url: String,
    pub port: u16,
    pub protocol: String,
    pub health_status: String,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_gb: f32,
    pub disk_gb: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceInstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Healthy,
    Scaling,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EndpointHealth {
    Healthy,
    Warning,
    Critical,
    Unknown,
}
use super::common::{UnifiedTeamInfo, TeamSize, ExperienceLevel};

/// BYOB-specific mock data provider
pub struct ByobMockProvider;

/// Static instance of the BYOB mock provider
pub static BYOB_MOCK_PROVIDER: ByobMockProvider = ByobMockProvider;

impl ByobMockProvider {
    /// Get mock teams converted to BYOB-specific format
    pub fn get_teams(&self) -> Vec<TeamInfo> {
        super::common::get_unified_teams()
            .into_iter()
            .map(|unified_team| self.convert_unified_team(unified_team))
            .collect()
    }

    /// Get mock deployments for demonstration
    pub fn get_deployments(&self) -> Vec<DeploymentInfo> {
        vec![
            DeploymentInfo {
                id: "deploy-web-frontend".to_string(),
                name: "Frontend Web App".to_string(),
                description: "Modern React-based web application with real-time features".to_string(),
                status: DeploymentStatus::Running,
                team: "Frontend Wizards".to_string(),
                environment: "production".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::days(7),
                last_updated: chrono::Utc::now() - chrono::Duration::hours(2),
                primal_services: vec![
                    PrimalServiceInstance {
                        primal_name: "toadstool".to_string(),
                        service_type: "web-server".to_string(),
                        status: ServiceInstanceStatus::Healthy,
                        endpoint: Some("https://app.example.com".to_string()),
                        resource_usage: ResourceUsage {
                            cpu_percent: 45.2,
                            memory_percent: 62.1,
                            disk_percent: 23.4,
                            network_mbps: 12.3,
                        },
                    },
                    PrimalServiceInstance {
                        primal_name: "songbird".to_string(),
                        service_type: "load-balancer".to_string(),
                        status: ServiceInstanceStatus::Healthy,
                        endpoint: Some("internal://lb.example.com".to_string()),
                        resource_usage: ResourceUsage {
                            cpu_percent: 12.1,
                            memory_percent: 18.9,
                            disk_percent: 5.2,
                            network_mbps: 45.7,
                        },
                    },
                ],
                metrics: DeploymentMetrics {
                    uptime_percent: 99.9,
                    response_time_ms: 89.5,
                    error_rate_percent: 0.1,
                    total_requests: 1_234_567,
                },
            },
            DeploymentInfo {
                id: "deploy-ai-research".to_string(),
                name: "AI Research Platform".to_string(),
                description: "Machine learning research environment with distributed training".to_string(),
                status: DeploymentStatus::Scaling,
                team: "AI Research Lab".to_string(),
                environment: "research".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::days(14),
                last_updated: chrono::Utc::now() - chrono::Duration::minutes(15),
                primal_services: vec![
                    PrimalServiceInstance {
                        primal_name: "squirrel".to_string(),
                        service_type: "ml-training".to_string(),
                        status: ServiceInstanceStatus::Scaling,
                        endpoint: Some("internal://ml.research.com:8080".to_string()),
                        resource_usage: ResourceUsage {
                            cpu_percent: 89.4,
                            memory_percent: 91.2,
                            disk_percent: 67.8,
                            network_mbps: 156.7,
                        },
                    },
                    PrimalServiceInstance {
                        primal_name: "nestgate".to_string(),
                        service_type: "data-storage".to_string(),
                        status: ServiceInstanceStatus::Healthy,
                        endpoint: Some("internal://storage.research.com".to_string()),
                        resource_usage: ResourceUsage {
                            cpu_percent: 23.1,
                            memory_percent: 34.5,
                            disk_percent: 78.9,
                            network_mbps: 89.2,
                        },
                    },
                ],
                metrics: DeploymentMetrics {
                    uptime_percent: 98.7,
                    response_time_ms: 245.2,
                    error_rate_percent: 0.3,
                    total_requests: 45_678,
                },
            },
        ]
    }

    /// Get mock service instances for demonstration
    pub fn get_services(&self) -> Vec<ServiceInfo> {
        vec![
            ServiceInfo {
                id: "service-web-frontend".to_string(),
                name: "React Frontend".to_string(),
                description: "Main web application frontend".to_string(),
                service_type: "web-application".to_string(),
                version: "3.2.1".to_string(),
                status: ServiceStatus::Running,
                primal_provider: "toadstool".to_string(),
                endpoints: vec![
                    ServiceEndpoint {
                        name: "web".to_string(),
                        url: "https://app.example.com".to_string(),
                        port: 443,
                        protocol: "https".to_string(),
                        health_status: EndpointHealth::Healthy,
                    },
                    ServiceEndpoint {
                        name: "api".to_string(),
                        url: "https://api.example.com".to_string(),
                        port: 443,
                        protocol: "https".to_string(),
                        health_status: EndpointHealth::Healthy,
                    },
                ],
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2.0,
                    memory_gb: 4.0,
                    storage_gb: 20.0,
                    network_mbps: 100.0,
                },
                capabilities: [
                    PrimalCapability::new("web", "server", "1.0.0"),
                    PrimalCapability::new("compute", "containers", "1.0.0"),
                ].into_iter().collect(),
            },
            ServiceInfo {
                id: "service-ml-training".to_string(),
                name: "ML Training Service".to_string(),
                description: "Distributed machine learning training orchestrator".to_string(),
                service_type: "ml-training".to_string(),
                version: "2.1.0".to_string(),
                status: ServiceStatus::Training,
                primal_provider: "squirrel".to_string(),
                endpoints: vec![
                    ServiceEndpoint {
                        name: "training-api".to_string(),
                        url: "internal://ml-api.research.com:8080".to_string(),
                        port: 8080,
                        protocol: "http".to_string(),
                        health_status: EndpointHealth::Healthy,
                    },
                    ServiceEndpoint {
                        name: "monitoring".to_string(),
                        url: "internal://ml-monitor.research.com:9090".to_string(),
                        port: 9090,
                        protocol: "http".to_string(),
                        health_status: EndpointHealth::Warning,
                    },
                ],
                resource_requirements: ResourceRequirements {
                    cpu_cores: 16.0,
                    memory_gb: 64.0,
                    storage_gb: 500.0,
                    network_mbps: 1000.0,
                },
                capabilities: [
                    PrimalCapability::new("ai", "training", "1.0.0"),
                    PrimalCapability::new("compute", "gpu-acceleration", "1.0.0"),
                ].into_iter().collect(),
            },
        ]
    }

    /// Get mock primal discovery data
    pub fn get_primal_discovery(&self) -> HashMap<String, PrimalInfo> {
        let mut primals = HashMap::new();

        primals.insert("toadstool".to_string(), PrimalInfo {
            name: "toadstool".to_string(),
            display_name: "Toadstool Compute".to_string(),
            description: "Universal compute orchestration and container management".to_string(),
            version: "1.5.2".to_string(),
            status: PrimalStatus::Online,
            capabilities: [
                PrimalCapability::new("compute", "containers", "1.0.0"),
                PrimalCapability::new("compute", "virtual-machines", "1.0.0"),
                PrimalCapability::new("web", "server", "1.0.0"),
            ].into_iter().collect(),
            endpoints: vec![
                "http://toadstool.local:8080".to_string(),
                "https://toadstool.example.com".to_string(),
            ],
            resource_usage: ResourceUsage {
                cpu_percent: 34.2,
                memory_percent: 51.8,
                disk_percent: 23.1,
                network_mbps: 67.4,
            },
        });

        primals.insert("songbird".to_string(), PrimalInfo {
            name: "songbird".to_string(),
            display_name: "Songbird Orchestration".to_string(),
            description: "Service mesh and network orchestration platform".to_string(),
            version: "2.0.1".to_string(),
            status: PrimalStatus::Online,
            capabilities: [
                PrimalCapability::new("orchestration", "service-mesh", "1.0.0"),
                PrimalCapability::new("network", "load-balancing", "1.0.0"),
                PrimalCapability::new("network", "service-discovery", "1.0.0"),
            ].into_iter().collect(),
            endpoints: vec!["http://songbird.local:8081".to_string()],
            resource_usage: ResourceUsage {
                cpu_percent: 15.7,
                memory_percent: 28.3,
                disk_percent: 12.4,
                network_mbps: 234.1,
            },
        });

        primals.insert("squirrel".to_string(), PrimalInfo {
            name: "squirrel".to_string(),
            display_name: "Squirrel AI Platform".to_string(),
            description: "Machine learning and AI orchestration platform".to_string(),
            version: "1.8.0".to_string(),
            status: PrimalStatus::Training,
            capabilities: [
                PrimalCapability::new("ai", "training", "1.0.0"),
                PrimalCapability::new("ai", "inference", "1.0.0"),
                PrimalCapability::new("compute", "gpu-acceleration", "1.0.0"),
            ].into_iter().collect(),
            endpoints: vec!["http://squirrel.local:8082".to_string()],
            resource_usage: ResourceUsage {
                cpu_percent: 78.9,
                memory_percent: 86.2,
                disk_percent: 45.7,
                network_mbps: 145.6,
            },
        });

        primals
    }

    /// Convert unified team info to BYOB-specific format
    fn convert_unified_team(&self, unified: UnifiedTeamInfo) -> TeamInfo {
        TeamInfo {
            name: unified.name,
            description: unified.description,
            size: match unified.size {
                TeamSize::Individual => crate::views::byob::types::TeamSize::Individual,
                TeamSize::Small => crate::views::byob::types::TeamSize::Small,
                TeamSize::Medium => crate::views::byob::types::TeamSize::Medium,
                TeamSize::Large => crate::views::byob::types::TeamSize::Large,
                TeamSize::Enterprise => crate::views::byob::types::TeamSize::Enterprise,
            },
            focus_area: unified.focus_area,
            experience_level: match unified.experience_level {
                ExperienceLevel::Beginner => crate::views::byob::types::ExperienceLevel::Beginner,
                ExperienceLevel::Intermediate => crate::views::byob::types::ExperienceLevel::Intermediate,
                ExperienceLevel::Advanced => crate::views::byob::types::ExperienceLevel::Advanced,  
                ExperienceLevel::Expert => crate::views::byob::types::ExperienceLevel::Expert,
            },
            required_capabilities: unified.required_capabilities,
            preferred_primals: unified.preferred_primals,
        }
    }
} 