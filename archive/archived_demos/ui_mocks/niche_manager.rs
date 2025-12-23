//! Niche Manager Mock Data Provider
//!
//! This module provides mock data specifically for the Niche Manager system,
//! including marketplace niches, local packages, and publishing data.

use crate::views::niche_manager::types::*;
use crate::state::InstallationStatus;

// Mock-specific types for niche manager
#[derive(Debug, Clone)]
pub struct PublishingStats {
    pub total_published: u32,
    pub downloads: u32,
    pub rating: f32,
    pub reviews: u32,
}

#[derive(Debug, Clone)]
pub struct SystemRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u64,
    pub architecture: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NichePrice {
    Free,
    Paid { amount: f64, currency: String },
}
use super::common::UnifiedNichePackage;

/// Niche Manager-specific mock data provider
pub struct NicheManagerMockProvider;

/// Static instance of the niche manager mock provider
pub static NICHE_MANAGER_MOCK_PROVIDER: NicheManagerMockProvider = NicheManagerMockProvider;

impl NicheManagerMockProvider {
    /// Get mock local niches converted from unified format
    pub fn get_local_niches(&self) -> Vec<NichePackage> {
        super::common::get_unified_niches()
            .into_iter()
            .map(|unified| self.convert_unified_niche(unified))
            .collect()
    }

    /// Get mock marketplace niches
    pub fn get_marketplace_niches(&self) -> Vec<MarketplaceNiche> {
        vec![
            MarketplaceNiche {
                id: "blockchain-node".to_string(),
                name: "Blockchain Node".to_string(),
                description: "Full blockchain node with support for Bitcoin, Ethereum, and other cryptocurrencies".to_string(),
                author: "CryptoDevs Collective".to_string(),
                publisher: "Verified Publisher".to_string(),
                version: "3.2.1".to_string(),
                category: NicheCategory::Blockchain,
                difficulty: NicheDifficulty::Advanced,
                rating: 4.7,
                download_count: 12_450,
                size_mb: 2048,
                price: NichePrice::Free,
                tags: vec![
                    "blockchain".to_string(),
                    "cryptocurrency".to_string(),
                    "node".to_string(),
                    "mining".to_string(),
                ],
                features: vec![
                    "Multi-currency support".to_string(),
                    "Built-in wallet".to_string(),
                    "Mining pool integration".to_string(),
                    "Real-time market data".to_string(),
                    "Secure cold storage".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 8,
                    min_memory_gb: 32,
                    min_storage_gb: 1000,
                    required_features: vec!["persistent_storage".to_string(), "network_access".to_string()],
                    supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
                },
                screenshots: vec![
                    "/marketplace/blockchain-node/screenshot1.png".to_string(),
                    "/marketplace/blockchain-node/screenshot2.png".to_string(),
                ],
                verified: true,
                last_updated: chrono::Utc::now() - chrono::Duration::days(3),
                compatibility_score: 95.2,
            },
            MarketplaceNiche {
                id: "cloud-native-dev".to_string(),
                name: "Cloud-Native Development".to_string(),
                description: "Complete cloud-native development environment with Kubernetes, Docker, and microservices tools".to_string(),
                author: "CloudMasters Inc.".to_string(),
                publisher: "Enterprise Publisher".to_string(),
                version: "2.5.0".to_string(),
                category: NicheCategory::Development,
                difficulty: NicheDifficulty::Intermediate,
                rating: 4.9,
                download_count: 8_920,
                size_mb: 1536,
                price: NichePrice::Paid { amount: 49.99, currency: "USD".to_string() },
                tags: vec![
                    "kubernetes".to_string(),
                    "docker".to_string(),
                    "microservices".to_string(),
                    "devops".to_string(),
                ],
                features: vec![
                    "Kubernetes cluster management".to_string(),
                    "Docker container orchestration".to_string(),
                    "CI/CD pipeline tools".to_string(),
                    "Service mesh integration".to_string(),
                    "Monitoring and logging".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 6,
                    min_memory_gb: 16,
                    min_storage_gb: 100,
                    required_features: vec!["containerization".to_string(), "orchestration".to_string()],
                    supported_architectures: vec!["x86_64".to_string()],
                },
                screenshots: vec![
                    "/marketplace/cloud-native-dev/dashboard.png".to_string(),
                    "/marketplace/cloud-native-dev/pipeline.png".to_string(),
                    "/marketplace/cloud-native-dev/monitoring.png".to_string(),
                ],
                verified: true,
                last_updated: chrono::Utc::now() - chrono::Duration::days(1),
                compatibility_score: 98.7,
            },
            MarketplaceNiche {
                id: "iot-gateway".to_string(),
                name: "IoT Gateway Platform".to_string(),
                description: "Industrial IoT gateway with protocol adapters, edge computing, and real-time analytics".to_string(),
                author: "Industrial Automation Team".to_string(),
                publisher: "Community Publisher".to_string(),
                version: "1.8.2".to_string(),
                category: NicheCategory::IoT,
                difficulty: NicheDifficulty::Expert,
                rating: 4.3,
                download_count: 3_210,
                size_mb: 892,
                price: NichePrice::Free,
                tags: vec![
                    "iot".to_string(),
                    "industrial".to_string(),
                    "gateway".to_string(),
                    "protocols".to_string(),
                    "edge".to_string(),
                ],
                features: vec![
                    "Multi-protocol support (MQTT, OPC-UA, Modbus)".to_string(),
                    "Edge computing capabilities".to_string(),
                    "Real-time data processing".to_string(),
                    "Device management".to_string(),
                    "Secure communication".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 2,
                    min_memory_gb: 4,
                    min_storage_gb: 50,
                    required_features: vec!["serial_ports".to_string(), "network_protocols".to_string()],
                    supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string(), "armv7".to_string()],
                },
                screenshots: vec![
                    "/marketplace/iot-gateway/devices.png".to_string(),
                    "/marketplace/iot-gateway/protocols.png".to_string(),
                ],
                verified: false,
                last_updated: chrono::Utc::now() - chrono::Duration::days(14),
                compatibility_score: 87.4,
            },
        ]
    }

    /// Get mock featured niches (subset of marketplace)
    pub fn get_featured_niches(&self) -> Vec<MarketplaceNiche> {
        let marketplace = self.get_marketplace_niches();
        marketplace.into_iter().take(2).collect() // Just the first 2 as featured
    }

    /// Get mock publishing statistics
    pub fn get_publishing_stats(&self) -> PublishingStats {
        PublishingStats {
            total_published: 7,
            total_downloads: 45_280,
            average_rating: 4.6,
            active_niches: 5,
            pending_reviews: 2,
            revenue_usd: 1_247.50,
        }
    }

    /// Convert unified niche to niche manager format
    fn convert_unified_niche(&self, unified: UnifiedNichePackage) -> NichePackage {
        NichePackage {
            id: unified.id,
            name: unified.name,
            description: unified.description,
            author: unified.author,
            version: unified.version,
            category: self.map_category(&unified.category),
            difficulty: NicheDifficulty::Intermediate, // Default difficulty
            tags: vec![unified.category.to_lowercase()],
            features: unified.features,
            requirements: SystemRequirements {
                min_cpu_cores: 4,
                min_memory_gb: 8,
                min_storage_gb: (unified.size_mb / 1024) as u32 + 10, // Convert MB to GB + overhead
                required_features: unified.capabilities
                    .iter()
                    .map(|cap| format!("{}_{}", cap.category, cap.name))
                    .collect(),
                supported_architectures: vec!["x86_64".to_string()],
            },
            manifest_path: unified.manifest_path,
            icon_path: unified.icon_path,
            installation_status: InstallationStatus::NotInstalled,
            local_version: None,
            size_mb: unified.size_mb,
        }
    }

    /// Map category string to NicheCategory enum
    fn map_category(&self, category: &str) -> NicheCategory {
        match category.to_lowercase().as_str() {
            "gaming" => NicheCategory::Gaming,
            "research" | "ai" => NicheCategory::Research,
            "development" | "web" => NicheCategory::Development,
            "security" => NicheCategory::Security,
            "media" => NicheCategory::Media,
            "iot" => NicheCategory::IoT,
            "blockchain" => NicheCategory::Blockchain,
            _ => NicheCategory::Other(category.to_string()),
        }
    }
} 