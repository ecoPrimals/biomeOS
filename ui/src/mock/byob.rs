//! Minimal BYOB Mock Data
//!
//! Simplified mock data for BYOB functionality

use biomeos_types::PrimalCapability;
use crate::views::byob::types::*;

/// Minimal team info that matches current struct
pub fn get_mock_teams() -> Vec<TeamInfo> {
    vec![
        TeamInfo {
            id: "team-1".to_string(),
            name: "Frontend Team".to_string(),
            description: "Web development team".to_string(),
            size: TeamSize::Small,
            focus_area: "Web Development".to_string(),
            experience_level: ExperienceLevel::Intermediate,
            required_capabilities: vec![PrimalCapability::compute()].into_iter().collect(),
            preferred_primals: vec!["toadstool".to_string()],
            created_at: "2024-01-01".to_string(),
            members: vec!["alice".to_string(), "bob".to_string()],
            status: TeamStatus::Active,
            workspace_url: Some("https://frontend.local".to_string()),
        },
    ]
}

/// Minimal deployment info
pub fn get_mock_deployments() -> Vec<DeploymentInfo> {
    vec![
        DeploymentInfo {
            id: "deploy-1".to_string(),
            name: "Test App".to_string(),
            status: DeploymentStatus::Running,
            created_at: "2024-01-01".to_string(),
            last_updated: "2024-01-15".to_string(),
            primals: vec!["toadstool".to_string()],
            capabilities: vec![PrimalCapability::compute()],
            resource_usage: ResourceUsage {
                cpu_percent: 25.0,
                memory_percent: 40.0,
                storage_percent: 15.0,
                network_mbps: 12.5,
                cpu_cores: 2.0,
                memory_gb: 4.0,
                storage_gb: 20.0,
            },
            manifest_path: "/mock/app.yaml".to_string(),
            health_status: biomeos_types::Health::Healthy,
            team: "team-1".to_string(),
            updated_at: "2024-01-15".to_string(),
            services: vec!["web-service".to_string()],
            health_score: 0.95,
        },
    ]
}

/// Mock provider for BYOB functionality
pub struct ByobMockProvider;

/// Global mock provider instance
pub static BYOB_MOCK_PROVIDER: ByobMockProvider = ByobMockProvider;

impl ByobMockProvider {
    pub fn get_teams(&self) -> Vec<TeamInfo> {
        get_mock_teams()
    }
    
    pub fn get_deployments(&self) -> Vec<DeploymentInfo> {
        get_mock_deployments()
    }
} 