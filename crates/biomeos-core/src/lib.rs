// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! `BiomeOS` Core - Universal Ecosystem Management
//!
//! Modular, unified architecture for managing the entire `biomeOS` ecosystem.
//! Now split into focused modules for better maintainability and compliance
//! with the 1000-line file size limit.

// Crate-level lint configuration
#![warn(missing_docs)]
#![allow(clippy::doc_markdown)] // Allow technical terms without backticks
#![forbid(unsafe_code)] // No unsafe code in core

// Core universal manager (now modular)
pub mod universal_biomeos_manager;

// Atomic client - Pure Rust, Tower-based Unix socket communication (ecoBin!)
pub mod atomic_client;
#[cfg(test)]
mod atomic_client_tests;

// Primal adapter pattern (CLI-agnostic integration)
pub mod primal_adapter;

// Core modules
pub mod capabilities; // Capability-based architecture (zero hardcoding)
pub mod capability_registry; // Central capability registry with Unix socket IPC
#[cfg(test)]
mod capability_registry_tests;
/// Wave-based concurrent primal startup
pub mod concurrent_startup;
/// Deployment mode detection (LiveSpore vs development)
pub mod deployment_mode;
pub mod discovery_bootstrap;
pub mod discovery_modern; // Modern trait-based discovery
pub mod family_credentials; // Secure family seed management
pub mod family_discovery; // Dynamic family ID discovery (seed-derived, cryptographic)
/// Auto-discovery of primals from directories
pub mod primal_discovery;
pub mod primal_impls; // Concrete primal implementations
pub mod primal_orchestrator; // Async primal lifecycle orchestration
pub mod retry; // Retry logic and circuit breaker
pub mod socket_discovery; // Capability-based socket discovery (replaces hardcoded /tmp paths)
/// Tower middleware configuration (TOML-based)
pub mod tower_config;

// P2P coordination (BiomeOS's killer feature!)
pub mod p2p_coordination;
pub mod primal_registry;

// Lab integration module (benchScale)
pub mod lab;

// VM Federation (benchScale + libvirt)
pub mod vm_federation;

// Observability (sovereignty-respecting)
pub mod observability;

// Connection Strategy (multi-tier NAT traversal orchestration)
pub mod connection_strategy;

// STUN Extension (optional self-hosted STUN support)
// biomeOS works without this - falls back to public STUN
pub mod stun_extension;

// NUCLEUS model cache (NestGate-integrated, filesystem fallback)
pub mod model_cache;

// Plasmodium - Over-NUCLEUS coordination (slime mold collective)
pub mod plasmodium;

// Legacy support - keep existing modules for compatibility
pub mod ai_first_api;
pub mod byob;
pub mod config;
pub mod config_builder;
/// Integration utilities for live service monitoring and system status
pub mod integration;
pub mod log_session;

// Re-export the main manager and types for easy access
pub use universal_biomeos_manager::{
    GeneticAccessKey, PrimalInfo, PrimalStatistics, UniversalBiomeOSManager,
};

// Re-export atomic client (Universal IPC v3.0 - Multi-Transport)
pub use atomic_client::{
    discover_primal_endpoint, AtomicClient, AtomicPrimalClient, ExecutionResult, JsonRpcRequest,
    JsonRpcResponse,
};

// Re-export core services
pub use universal_biomeos_manager::{HealthMonitor, PrimalDiscoveryService};

// Modern discovery system re-exports
pub use discovery_modern::{
    Capability as DiscoveryCapability, CompositeDiscovery, DiscoveredPrimal, DiscoveryError,
    DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType,
};

// Primal orchestration re-exports (primary Capability enum for orchestration)
pub use capabilities::{Capability, PrimalConfig as CapabilitiesPrimalConfig};
pub use capability_registry::{
    CapabilityRegistry, PrimalInfo as RegistryPrimalInfo, RegisterParams,
};
pub use concurrent_startup::{start_in_waves, DependencyGraph};
pub use family_credentials::FamilyCredentials;
pub use log_session::{LogSessionTracker, PrimalSession};
pub use primal_discovery::{discover_primals, query_primal_metadata, PrimalMetadata};
pub use primal_impls::{
    // Convenience builders
    create_ai_service,
    create_compute_provider,
    create_discovery_orchestrator,
    create_security_provider,
    create_storage_provider,
    // New generic primal system
    GenericManagedPrimal,
    ManagedBearDog,
    ManagedSongbird,
    PrimalBuilder,
};
// Legacy type aliases removed — use PrimalBuilder directly
pub use primal_orchestrator::{
    ManagedPrimal, PrimalHealthMonitor, PrimalOrchestrator, PrimalState,
};
pub use retry::{CircuitBreaker, RetryPolicy};
pub use socket_discovery::{
    discover_endpoint, discover_socket, DiscoveredSocket, DiscoveryMethod, DiscoveryStrategy,
    SocketDiscovery, TransportEndpoint,
};
pub use tower_config::PrimalConfig as TowerPrimalConfig;
pub use tower_config::{DiscoveryConfig, HealthConfig, TowerConfig};

// Connection strategy re-exports
pub use connection_strategy::{
    connect_to_peer, ConnectionResult, ConnectionTier, NatType, PeerConnectionInfo, PortPattern,
    StunResults,
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
