//! Minimal Niche Manager Mock Data
//!
//! Simplified mock data that aligns with current unified types

use crate::views::niche_manager::types::*;

/// Get minimal mock marketplace niches
pub fn get_mock_marketplace_niches() -> Vec<MarketplaceNiche> {
    vec![
        MarketplaceNiche {
            package: NichePackage {
                id: "gaming-niche".to_string(),
                name: "Gaming Niche".to_string(),
                description: "A niche for gaming applications".to_string(),
                author: "biomeOS Team".to_string(),
                version: "1.0.0".to_string(),
                category: NicheCategory::Gaming,
                difficulty: NicheDifficulty::Intermediate,
                tags: vec!["gaming".to_string()],
                features: vec!["GPU acceleration".to_string()],
                requirements: get_minimal_system_requirements(),
                manifest_path: "/niches/gaming.yaml".to_string(),
                icon_path: None,
                size_mb: 150,
                downloads: 1000,
                rating: 4.5,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-15".to_string(),
                status: NicheStatus::Published,
            },
            featured: true,
            security_score: 95.0,
            community_rating: 4.8,
            verified: true,
            last_updated: "2024-01-15".to_string(),
        },
        MarketplaceNiche {
            package: NichePackage {
                id: "ai-niche".to_string(),
                name: "AI Research Niche".to_string(),
                description: "A niche for AI research and development".to_string(),
                author: "biomeOS Team".to_string(),
                version: "1.1.0".to_string(),
                category: NicheCategory::Research,
                difficulty: NicheDifficulty::Advanced,
                tags: vec!["ai".to_string(), "research".to_string()],
                features: vec!["ML libraries".to_string()],
                requirements: get_minimal_system_requirements(),
                manifest_path: "/niches/ai.yaml".to_string(),
                icon_path: None,
                size_mb: 200,
                downloads: 500,
                rating: 4.2,
                created_at: "2024-01-10".to_string(),
                updated_at: "2024-01-20".to_string(),
                status: NicheStatus::Published,
            },
            featured: false,
            security_score: 87.0,
            community_rating: 4.1,
            verified: true,
            last_updated: "2024-01-20".to_string(),
        },
    ]
}

/// Get minimal publishing stats
pub fn get_mock_publishing_stats() -> crate::views::niche_manager::types::PublishingStats {
    crate::views::niche_manager::types::PublishingStats {
        downloads: 1500,
        rating: 4.3,
        reviews: 25,
    }
}

/// Mock provider for niche manager functionality
pub struct NicheManagerMockProvider;

/// Global mock provider instance
pub static NICHE_MANAGER_MOCK_PROVIDER: NicheManagerMockProvider = NicheManagerMockProvider;

impl NicheManagerMockProvider {
    pub fn get_marketplace_niches(&self) -> Vec<MarketplaceNiche> {
        get_mock_marketplace_niches()
    }

    pub fn get_publishing_stats(&self) -> PublishingStats {
        get_mock_publishing_stats()
    }

    pub fn get_system_requirements(&self) -> SystemRequirements {
        get_minimal_system_requirements()
    }
}

/// Minimal system requirements for compatibility
pub fn get_minimal_system_requirements() -> crate::views::niche_manager::types::SystemRequirements {
    crate::views::niche_manager::types::SystemRequirements {
        min_cpu_cores: 4,
        min_memory_gb: 8,
        min_storage_gb: 50,
        required_features: vec!["docker".to_string()],
        supported_architectures: vec!["x86_64".to_string()],
    }
}
