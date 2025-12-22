//! BiomeOS Core - Universal Ecosystem Management
//!
//! Modular, unified architecture for managing the entire biomeOS ecosystem.
//! Now split into focused modules for better maintainability and compliance
//! with the 2000-line file size limit.

// Core universal manager (now modular)
pub mod universal_biomeos_manager;

// Legacy support - keep existing modules for compatibility
pub mod ai_first_api;
pub mod byob;
pub mod config;
pub mod config_builder;
pub mod integration;

// Re-export the main manager and types for easy access
pub use universal_biomeos_manager::{
    UniversalBiomeOSManager, 
    PrimalInfo,
    PrimalStatistics,
    GeneticAccessKey,
};

// Re-export core services
pub use universal_biomeos_manager::{
    PrimalDiscoveryService,
    HealthMonitor,
};

// Legacy re-exports for backwards compatibility
pub use universal_biomeos_manager as manager;

// Re-export key types directly instead of aliases
pub use UniversalBiomeOSManager as Manager;

// AI-first API exports
pub use ai_first_api::{
    AIFirstResponse, AIResponseMetadata, HumanInteractionContext,
    SuggestedAction,
};

// ✅ MIGRATION NOTE: The following types are now imported from biomeos-types:
// - BiomeMetadata -> ManifestMetadata
// - PrimalConfiguration (unified version)
// - ServiceConfiguration -> ServiceSpec
// - UniversalBiomeManifest -> BiomeManifest
// - HealthStatus -> Health
// - SystemHealth -> HealthReport

// Build information now centralized in biomeos-types constants module
pub use biomeos_types::BUILD_INFO;

// ✅ MIGRATION COMPLETE: BiomeError now imported from biomeos-types
// The old local BiomeError definition has been replaced with the unified version
// from biomeos-types which includes AI-first features, retry strategies, and
// comprehensive error context for both human and AI interaction.

// ✅ MIGRATION NOTE: From implementations moved to biomeos-types
// The unified BiomeError in biomeos-types already provides From implementations

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_creation() {
        let config = BiomeOSConfig::default();
        assert!(!config.discovery.methods.is_empty());
        assert!(config.discovery.default_method == biomeos_types::config::resources::DiscoveryMethod::Static);
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();

        // Test that manager can be created without panicking
        // Manager is initialized during construction
        let _registered_primals = manager.get_registered_primals().await;
    }
}
