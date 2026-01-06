//! BiomeOS Core - Universal Ecosystem Management
//!
//! Modular, unified architecture for managing the entire biomeOS ecosystem.
//! Now split into focused modules for better maintainability and compliance
//! with the 2000-line file size limit.

// Core universal manager (now modular)
pub mod universal_biomeos_manager;

// Primal adapter pattern (CLI-agnostic integration)
pub mod primal_adapter;

// API adapter pattern (API-agnostic integration)
pub mod api_adapter;

// Primal client infrastructure
// Legacy primal-specific clients (deprecated - use Universal Primal Client instead)
// pub mod clients;
pub mod adaptive_client; // Adaptive HTTP client with version tolerance
pub mod capabilities; // Capability-based architecture (zero hardcoding)
pub mod concurrent_startup; // Wave-based concurrent primal startup
pub mod discovery_bootstrap;
pub mod discovery_http; // HTTP-based discovery implementation
pub mod discovery_modern; // Modern trait-based discovery
pub mod family_credentials; // Secure family seed management
pub mod primal_client;
pub mod primal_discovery; // Auto-discovery of primals from directories
pub mod primal_health; // Primal health monitoring
pub mod primal_impls; // Concrete primal implementations
pub mod primal_orchestrator; // Async primal lifecycle orchestration
pub mod retry; // Retry logic and circuit breaker
pub mod tower_config; // Tower configuration (TOML-based)
pub mod capability_registry; // Central capability registry with Unix socket IPC

// P2P coordination (BiomeOS's killer feature!)
pub mod p2p_coordination;
pub mod primal_registry;

// Lab integration module (benchScale)
pub mod lab;

// VM Federation (benchScale + libvirt)
pub mod vm_federation;

// Observability (sovereignty-respecting)
pub mod observability;

// Legacy support - keep existing modules for compatibility
pub mod ai_first_api;
pub mod byob;
pub mod config;
pub mod config_builder;
pub mod integration;

// Re-export the main manager and types for easy access
pub use universal_biomeos_manager::{
    GeneticAccessKey, PrimalInfo, PrimalStatistics, UniversalBiomeOSManager,
};

// Re-export core services
pub use universal_biomeos_manager::{HealthMonitor, PrimalDiscoveryService};

// Modern discovery system re-exports
pub use discovery_modern::{
    Capability as DiscoveryCapability, CompositeDiscovery, DiscoveredPrimal, DiscoveryError, DiscoveryResult,
    HealthStatus as DiscoveryHealthStatus, PrimalType, PrimalDiscovery,
};
// Convenience alias for the most commonly used HealthStatus
pub use discovery_modern::HealthStatus;
pub use discovery_http::{create_local_discovery, HttpDiscovery, HttpDiscoveryBuilder};

// Adaptive client infrastructure re-exports
pub use adaptive_client::{
    AdaptiveHttpClient, AdaptiveResponse, ApiVersion, BearDogResponse, BirdSongClient,
    BirdSongDecryptRequest, BirdSongDecryptResponse, BirdSongEncryptRequest,
    BirdSongEncryptResponse,
};

// Primal orchestration re-exports (primary Capability enum for orchestration)
pub use concurrent_startup::{start_in_waves, DependencyGraph};
pub use family_credentials::FamilyCredentials;
pub use primal_discovery::{discover_primals, query_primal_metadata, PrimalMetadata};
pub use tower_config::{TowerConfig, DiscoveryConfig, HealthConfig};
pub use tower_config::PrimalConfig as TowerPrimalConfig;
pub use capabilities::{Capability, PrimalConfig as CapabilitiesPrimalConfig};
pub use primal_orchestrator::{ManagedPrimal, PrimalOrchestrator, PrimalState};
pub use primal_health::{PrimalHealthMonitor, HealthStatus as PrimalHealthStatus, RecoveryStrategy};
pub use retry::{RetryPolicy, CircuitBreaker};
pub use capability_registry::{CapabilityRegistry, PrimalInfo as RegistryPrimalInfo, RegisterParams};
pub use primal_impls::{
    // New generic primal system
    GenericManagedPrimal,
    PrimalBuilder,
    // Convenience builders
    create_ai_service,
    create_compute_provider,
    create_discovery_orchestrator,
    create_security_provider,
    create_storage_provider,
    // Legacy type aliases (deprecated)
    BearDogConfig,
    ManagedBearDog,
    ManagedSongbird,
    SongbirdConfig,
    TowerBuilder,
};

// Legacy re-exports for backwards compatibility
pub use universal_biomeos_manager as manager;

// Re-export key types directly instead of aliases
pub use UniversalBiomeOSManager as Manager;

// AI-first API exports
pub use ai_first_api::{
    AIFirstResponse, AIResponseMetadata, HumanInteractionContext, SuggestedAction,
};

// Configuration builder exports
pub use config_builder::BiomeOSConfigBuilder;

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
    use biomeos_types::BiomeOSConfig;

    #[tokio::test]
    async fn test_config_creation() {
        let config = BiomeOSConfig::default();
        assert!(!config.discovery.methods.is_empty());
        // Default method is Static
        assert!(matches!(
            config.discovery.default_method,
            biomeos_types::config::resources::DiscoveryMethod::Static
        ));
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
