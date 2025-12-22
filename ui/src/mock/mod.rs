//! Unified Mock Data System for BiomeOS UI
//!
//! This module consolidates all mock data that was previously scattered across
//! different UI components into a single, unified system with feature flags
//! for easy switching between mock and live data.

use std::collections::HashSet;
use biomeos_types::PrimalCapability;

pub mod iso_creator;
pub mod byob;
pub mod niche_manager;

/// Feature flag for enabling mock data globally
// Use centralized constants from biomeos-types
pub use biomeos_types::constants::*;

pub const MOCK_DATA_ENABLED: bool = true; // Enable mock data for development

/// Central mock data provider that coordinates all mock data
pub struct UnifiedMockProvider;

impl UnifiedMockProvider {
    /// Check if mock data is enabled
    pub fn is_enabled() -> bool {
        MOCK_DATA_ENABLED || cfg!(debug_assertions) // Always enabled in debug builds
    }

    /// Get the appropriate data provider based on feature flags
    pub fn iso_creator() -> &'static iso_creator::IsoCreatorMockProvider {
        &iso_creator::ISO_CREATOR_MOCK_PROVIDER
    }

    pub fn byob() -> &'static byob::ByobMockProvider {
        &byob::BYOB_MOCK_PROVIDER
    }

    pub fn niche_manager() -> &'static niche_manager::NicheManagerMockProvider {
        &niche_manager::NICHE_MANAGER_MOCK_PROVIDER
    }
}

/// Common data structures used across multiple mock providers
pub mod common {
    use super::*;

    /// Common niche package representation
    #[derive(Debug, Clone)]
    pub struct UnifiedNichePackage {
        pub id: String,
        pub name: String,
        pub description: String,
        pub author: String,
        pub version: String,
        pub category: String,
        pub size_mb: u64,
        pub features: Vec<String>,
        pub dependencies: Vec<String>,
        pub capabilities: HashSet<PrimalCapability>,
        pub manifest_path: String,
        pub icon_path: Option<String>,
    }

    /// Common team information
    #[derive(Debug, Clone)]
    pub struct UnifiedTeamInfo {
        pub name: String,
        pub description: String,
        pub size: TeamSize,
        pub focus_area: String,
        pub experience_level: ExperienceLevel,
        pub required_capabilities: HashSet<PrimalCapability>,
        pub preferred_primals: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub enum TeamSize {
        Individual,
        Small,
        Medium,
        Large,
        Enterprise,
    }

    #[derive(Debug, Clone)]
    pub enum ExperienceLevel {
        Beginner,
        Intermediate,
        Advanced,
        Expert,
    }

    /// Generate common mock teams used across different UI components
    pub fn get_unified_teams() -> Vec<UnifiedTeamInfo> {
        vec![
            UnifiedTeamInfo {
                name: "Frontend Wizards".to_string(),
                description: "Specialized in modern web frontend development with React, Vue, and Next.js".to_string(),
                size: TeamSize::Small,
                focus_area: "Web Development".to_string(),
                experience_level: ExperienceLevel::Intermediate,
                required_capabilities: [
                    PrimalCapability::new("compute", "web-server", "1.0.0"),
                    PrimalCapability::new("web", "development", "1.0.0"),
                    PrimalCapability::new("network", "routing", "1.0.0"),
                ].into_iter().collect(),
                preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
            },
            UnifiedTeamInfo {
                name: "AI Research Lab".to_string(),
                description: "Machine learning and AI research team focusing on deep learning and neural networks".to_string(),
                size: TeamSize::Medium,
                focus_area: "AI Research".to_string(),
                experience_level: ExperienceLevel::Advanced,
                required_capabilities: [
                    PrimalCapability::new("ai", "machine-learning", "1.0.0"),
                    PrimalCapability::new("compute", "gpu-acceleration", "1.0.0"),
                    PrimalCapability::new("data", "analytics", "1.0.0"),
                ].into_iter().collect(),
                preferred_primals: vec!["squirrel".to_string(), "toadstool".to_string()],
            },
            UnifiedTeamInfo {
                name: "Gaming Studio".to_string(),
                description: "Indie game development studio creating immersive gaming experiences".to_string(),
                size: TeamSize::Small,
                focus_area: "Game Development".to_string(),
                experience_level: ExperienceLevel::Intermediate,
                required_capabilities: [
                    PrimalCapability::new("gaming", "engine", "1.0.0"),
                    PrimalCapability::new("compute", "real-time", "1.0.0"),
                    PrimalCapability::new("network", "multiplayer", "1.0.0"),
                ].into_iter().collect(),
                preferred_primals: vec!["toadstool".to_string(), "songbird".to_string()],
            },
            UnifiedTeamInfo {
                name: "DevOps Champions".to_string(),
                description: "Infrastructure and deployment automation specialists".to_string(),
                size: TeamSize::Medium,
                focus_area: "DevOps & Infrastructure".to_string(),
                experience_level: ExperienceLevel::Advanced,
                required_capabilities: [
                    PrimalCapability::new("orchestration", "container-management", "1.0.0"),
                    PrimalCapability::new("security", "access-control", "1.0.0"),
                    PrimalCapability::new("storage", "persistent-volumes", "1.0.0"),
                ].into_iter().collect(),
                preferred_primals: vec!["songbird".to_string(), "nestgate".to_string(), "beardog".to_string()],
            },
        ]
    }

    /// Generate common mock niches used across different UI components
    pub fn get_unified_niches() -> Vec<UnifiedNichePackage> {
        vec![
            UnifiedNichePackage {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management and gaming infrastructure with real-time matchmaking, brackets, and streaming integration".to_string(),
                author: "Tournament Masters Team".to_string(),
                version: "1.5.0".to_string(),
                category: "Gaming".to_string(),
                size_mb: 450,
                features: vec![
                    "Real-time matchmaking".to_string(),
                    "Tournament brackets".to_string(),
                    "Leaderboard system".to_string(),
                    "Anti-cheat integration".to_string(),
                    "Stream overlay support".to_string(),
                ],
                dependencies: vec!["toadstool".to_string(), "songbird".to_string()],
                capabilities: [
                    PrimalCapability::new("gaming", "tournament", "1.0.0"),
                    PrimalCapability::new("compute", "real-time", "1.0.0"), 
                    PrimalCapability::new("network", "low-latency", "1.0.0"),
                ].into_iter().collect(),
                manifest_path: "/niches/gaming-tournament/manifest.yaml".to_string(),
                icon_path: Some("/niches/gaming-tournament/icon.png".to_string()),
            },
            UnifiedNichePackage {
                id: "ai-research".to_string(),
                name: "AI Research Platform".to_string(),
                description: "Machine learning research environment with distributed training, model versioning, and experiment tracking".to_string(),
                author: "Deep Learning Lab".to_string(),
                version: "2.1.0".to_string(),
                category: "Research".to_string(),
                size_mb: 1200,
                features: vec![
                    "Distributed training".to_string(),
                    "Model versioning".to_string(),
                    "Dataset management".to_string(),
                    "Experiment tracking".to_string(),
                    "GPU acceleration".to_string(),
                    "Jupyter notebooks".to_string(),
                ],
                dependencies: vec!["squirrel".to_string(), "toadstool".to_string(), "nestgate".to_string()],
                capabilities: [
                    PrimalCapability::new("ai", "research", "1.0.0"),
                    PrimalCapability::new("compute", "gpu-acceleration", "1.0.0"),
                    PrimalCapability::new("data", "large-datasets", "1.0.0"),
                ].into_iter().collect(),
                manifest_path: "/niches/ai-research/manifest.yaml".to_string(),
                icon_path: Some("/niches/ai-research/icon.png".to_string()),
            },
            UnifiedNichePackage {
                id: "web-development".to_string(),
                name: "Full-Stack Web Platform".to_string(),
                description: "Complete web development environment with frontend frameworks, backend APIs, and database integration".to_string(),
                author: "Web Masters Guild".to_string(),
                version: "3.0.0".to_string(),
                category: "Development".to_string(),
                size_mb: 800,
                features: vec![
                    "React/Vue/Angular support".to_string(),
                    "Node.js backend".to_string(),
                    "Database integration".to_string(),
                    "Hot reload development".to_string(),
                    "Testing frameworks".to_string(),
                    "CI/CD pipeline".to_string(),
                ],
                dependencies: vec!["toadstool".to_string(), "songbird".to_string(), "nestgate".to_string()],
                capabilities: [
                    PrimalCapability::new("web", "full-stack", "1.0.0"),
                    PrimalCapability::new("compute", "containers", "1.0.0"),
                    PrimalCapability::new("data", "databases", "1.0.0"),
                ].into_iter().collect(),
                manifest_path: "/niches/web-development/manifest.yaml".to_string(),
                icon_path: Some("/niches/web-development/icon.png".to_string()),
            },
        ]
    }
} 