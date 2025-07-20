//! # biomeOS - Universal Biological Computing Platform
//!
//! biomeOS provides a universal orchestration platform that leverages the mature
//! capabilities of existing primals rather than reimplementing core functionality:
//! - Toadstool: Universal parser, validator, and executor
//! - Songbird: Universal discovery, coordination, and routing
//! - BiomeOS: Thin coordination layer providing universal adapter patterns

pub use biomeos_core::*;
pub mod universal_adapter;

/// Universal adapter for coordinating between Toadstool and Songbird
pub use universal_adapter::{
    BiomeDeployment, BiomeOSUniversalAdapter, CapabilityRegistry, DeployedService,
    DiscoveredPrimal, ManifestMetadata, ParsedManifest, PrimalHealth, PrimalSpec, ResolvedPrimal,
    ServiceSpec, ServiceStatus, SongbirdClient, SystemHealth, ToadstoolClient,
    UniversalHealthMonitor,
};

/// Universal UI types for examples - simplified for the new architecture
pub mod universal_ui {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UIFeatures {
        pub ai_assistant: bool,
        pub real_time_monitoring: bool,
        pub deployment_wizard: bool,
        pub service_management: bool,
        pub log_viewer: bool,
        pub metrics_dashboard: bool,
        pub custom_dashboards: bool,
        pub primal_coordination: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DiscoveredPrimal {
        pub id: String,
        pub primal_type: String,
        pub endpoint: String,
        pub capabilities: Vec<String>,
        pub health: PrimalHealth,
        pub metadata: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PrimalHealth {
        pub status: String,
        pub last_seen: chrono::DateTime<chrono::Utc>,
        pub response_time_ms: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UniversalUIConfig {
        pub theme: String,
        pub mode: crate::UIMode,
        pub features: UIFeatures,
        pub discovered_primals: Vec<DiscoveredPrimal>,
        pub toadstool_endpoint: Option<String>,
        pub songbird_endpoint: Option<String>,
    }

    /// Universal UI Manager - Note: doesn't implement Clone/Serialize due to adapter
    #[derive(Debug)]
    pub struct UniversalUIManager {
        pub config: UniversalUIConfig,
        pub system_health: crate::SystemHealth,
        pub adapter: Option<crate::BiomeOSUniversalAdapter>,
    }

    impl UniversalUIManager {
        pub fn new(config: UniversalUIConfig) -> Self {
            Self {
                config,
                system_health: crate::SystemHealth {
                    toadstool_status: crate::ServiceStatus {
                        available: false,
                        response_time_ms: 0,
                        last_error: None,
                    },
                    songbird_status: crate::ServiceStatus {
                        available: false,
                        response_time_ms: 0,
                        last_error: None,
                    },
                    discovered_primals: vec![],
                    last_updated: chrono::Utc::now(),
                },
                adapter: None,
            }
        }

        pub async fn initialize_adapter(&mut self) -> Result<(), anyhow::Error> {
            let adapter = crate::BiomeOSUniversalAdapter::new()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize adapter: {}", e))?;
            self.adapter = Some(adapter);
            Ok(())
        }

        pub async fn discover_primals(&self) -> Result<Vec<DiscoveredPrimal>, anyhow::Error> {
            if let Some(adapter) = &self.adapter {
                let primals = adapter
                    .discover_primals()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to discover primals: {}", e))?;
                Ok(primals
                    .into_iter()
                    .map(|p| DiscoveredPrimal {
                        id: p.id,
                        primal_type: p.primal_type,
                        endpoint: p.endpoint,
                        capabilities: p.capabilities,
                        health: PrimalHealth {
                            status: p.health.status,
                            last_seen: p.health.last_seen,
                            response_time_ms: p.health.response_time_ms,
                        },
                        metadata: p.metadata,
                    })
                    .collect())
            } else {
                Ok(vec![])
            }
        }

        pub async fn get_system_health(&self) -> Result<crate::SystemHealth, anyhow::Error> {
            if let Some(adapter) = &self.adapter {
                adapter
                    .get_system_health()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get system health: {}", e))
            } else {
                Ok(self.system_health.clone())
            }
        }

        pub async fn deploy_biome(
            &self,
            manifest_path: &str,
        ) -> Result<crate::BiomeDeployment, anyhow::Error> {
            if let Some(adapter) = &self.adapter {
                adapter
                    .process_biome_manifest(manifest_path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to deploy biome: {}", e))
            } else {
                Err(anyhow::anyhow!("Universal adapter not initialized"))
            }
        }

        pub async fn start(&self) -> Result<(), anyhow::Error> {
            // Universal UI manager now focuses on coordinating between Toadstool and Songbird
            tracing::info!("Starting Universal UI Manager with delegation architecture");
            tracing::info!("Toadstool endpoint: {:?}", self.config.toadstool_endpoint);
            tracing::info!("Songbird endpoint: {:?}", self.config.songbird_endpoint);
            Ok(())
        }
    }

    impl Default for UIFeatures {
        fn default() -> Self {
            Self {
                ai_assistant: true,
                real_time_monitoring: true,
                deployment_wizard: true,
                service_management: true,
                log_viewer: true,
                metrics_dashboard: true,
                custom_dashboards: true,
                primal_coordination: true,
            }
        }
    }

    impl Default for UniversalUIConfig {
        fn default() -> Self {
            Self {
                theme: "default".to_string(),
                mode: crate::UIMode::Auto,
                features: UIFeatures::default(),
                discovered_primals: vec![],
                toadstool_endpoint: Some("http://localhost:8084".to_string()),
                songbird_endpoint: Some("http://localhost:8080".to_string()),
            }
        }
    }
}

/// Re-export ecosystem integration for external use
pub mod ecosystem {
    pub use biomeos_core::ecosystem_integration::*;
}

/// Re-export universal adapter for primal coordination
pub mod coordination {
    pub use crate::universal_adapter::*;
}

/// Universal adapter pattern types
pub mod universal {
    pub use crate::universal_adapter::{
        BiomeOSUniversalAdapter, CapabilityRegistry, SongbirdClient, ToadstoolClient,
        UniversalHealthMonitor,
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "biomeOS v",
    env!("CARGO_PKG_VERSION"),
    " - Universal Adapter Architecture"
);

/// Architecture description
pub const ARCHITECTURE: &str =
    "Universal Adapter: Delegates to Toadstool (parsing) + Songbird (discovery)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.')); // Version should contain dots
    }

    #[test]
    fn test_build_info() {
        assert!(BUILD_INFO.contains("biomeOS"));
        assert!(BUILD_INFO.contains("Universal Adapter"));
    }

    #[test]
    fn test_architecture() {
        assert!(ARCHITECTURE.contains("Toadstool"));
        assert!(ARCHITECTURE.contains("Songbird"));
        assert!(ARCHITECTURE.contains("Delegates"));
    }

    #[tokio::test]
    async fn test_universal_ui_manager() {
        let config = universal_ui::UniversalUIConfig::default();
        let mut manager = universal_ui::UniversalUIManager::new(config);

        // Test initialization
        assert!(manager.adapter.is_none());

        // Test discovery without adapter
        let primals = manager.discover_primals().await.unwrap();
        assert!(primals.is_empty());

        // Test health check without adapter
        let health = manager.get_system_health().await.unwrap();
        assert!(!health.toadstool_status.available);
        assert!(!health.songbird_status.available);
    }
}
