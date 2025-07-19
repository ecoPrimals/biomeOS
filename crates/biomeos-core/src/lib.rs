//! # biomeOS Core
//!
//! Core types, traits, and functionality for the biomeOS operating system.
//! This crate provides the fundamental building blocks for orchestrating
//! any primals into a cohesive biological computing environment using
//! capability-based discovery instead of hardcoded implementations.

pub mod adapters;
pub mod biome;
pub mod byob;
pub mod cloud;
pub mod ecosystem_integration;
pub mod compute;
pub mod config;
pub mod crypto;
pub mod errors;
pub mod health;
pub mod installer;
pub mod locks;
pub mod manifest;
pub mod monitoring_dashboard;
pub mod networking;
pub mod primal;
pub mod runtime_bridge;
pub mod security;
pub mod storage;
pub mod universal;
pub mod orchestration;

// Universal primal client system - capability-based, not hardcoded
pub mod primal_clients;

// Universal biomeOS manager using capability-based discovery
pub mod universal_biomeos_manager;

// Universal/agnostic modules that replace hard-coded Primal-specific implementations
pub mod universal_primal;
pub mod universal_primal_provider;

// Refactored modules
pub mod core_config;
pub mod core_types;
pub mod service_registration;
pub mod universal_config;

// Re-export core types with specific imports to avoid conflicts
pub use errors::{BiomeError, BiomeResult};

// Universal/agnostic types that replace hard-coded Primal-specific implementations
pub use universal_primal::{
    Capability, Capability as UniversalCapability, CapabilityCategory, CapabilityRequest,
    CapabilityRequirement, CapabilityResponse, Constraint, CoordinationRequest,
    CoordinationResponse, CoordinationType, DefaultDiscoveryService, DiscoveredPrimal,
    EcosystemEvent, EcosystemEventType, ParameterSpec, PerformanceSpec, PrimalEndpoint,
    PrimalMetadata, PrimalPeer, RequestContext, RequestPriority as UniversalRequestPriority,
    ResourceRequirements as UniversalResourceRequirements, ScalingSpec,
    TlsConfig as UniversalTlsConfig, UniversalDiscoveryService, UniversalPrimalProvider,
};

// Universal primal client system exports
pub use primal_clients::{
    UniversalPrimalManager, UniversalPrimalClient, NetworkPrimalDiscovery,
    CapabilityCategory as PrimalCapabilityCategory,
    CapabilityRequirement as PrimalCapabilityRequirement, CapabilityResponse as PrimalCapabilityResponse,
    ServiceInfo, StorageSpec, Credentials, AuthToken, PrimalHealth, HealthStatus as PrimalHealthStatus,
};

// Universal biomeOS manager exports
pub use universal_biomeos_manager::{
    UniversalBiomeOSManager, BiomeOSConfig, EcosystemHealth,
    PrimalInfo, PrimalHealth as BiomeOSPrimalHealth, create_biomeos_manager,
    create_biomeos_manager_with_config,
};

pub use biome::{Biome, BiomeManifest, BiomeMetadata, BiomeSpec};
pub use byob::{
    ByobDeploymentManager, DeploymentInstance, DeploymentStatus, IsolationConfig, ResourceQuota,
    SimpleBiomeManifest, SimpleBiomeResources, SimpleBiomeService, TeamWorkspace,
};

pub use cloud::{
    CloudProviderType,
};
pub use compute::{
    ComputeProviderType,
};
pub use config::{
    Config, ConfigError, Theme, UIMode,
    
    
};
pub use crypto::{
    CryptoProvider,
};
pub use health::{HealthMetrics, HealthStatus};
pub use installer::UniversalInstaller;
pub use locks::CryptoLockManager;
pub use monitoring_dashboard::{
    DashboardConfig, UniversalMonitoringDashboard,
};
pub use networking::NetworkConfig;
pub use primal::PrimalType;
pub use runtime_bridge::ToadStoolBridge;
pub use security::SecurityConfig;
pub use storage::{
    StorageConfig,
};
pub use universal::{
    AiAssistant, AiPersonalityConfig as UniversalAiPersonalityConfig,
    DeploymentConfig as UniversalDeploymentConfig, MycorrhizaConfig as UniversalMycorrhizaConfig,
    PlatformDiagnostics, PlatformInfo as UniversalPlatformInfo, PlatformResources,
    PerformanceMetrics, SecurityStatus, ServiceStatus, UniversalPlatform,
};

// Re-export refactored modules for backward compatibility
pub use core_config::{BiomeOSConfig as CoreBiomeOSConfig, GlobalConfig, TelemetryConfig as CoreTelemetryConfig, NetworkConfig as CoreNetworkConfig, TelemetryConfig};
pub use core_types::{init_biomeos, BiomeId, PrimalId, BIOMEOS_VERSION};
pub use service_registration::{
    BiomeResourceRequirements, ComplianceStatus, PrimalEvent, PrimalEventListener, RuntimeType,
    ServiceCapabilities, ServiceRegistration,
};
pub use universal_config::{
    AccessPriority, AiCatDoorConfig, BeardogAccessLevel, BusinessCostMultipliers,
    BusinessThreshold, CloudProviderConfig, CommercialLicenseConfig, CommercialModel,
    ComputeProviderConfig, ContainerProviderConfig, CostProtectionConfig, CryptoLockConfig,
    CryptoProviderConfig, GeneticBeardogKey, InverseScaleConfig, LicensingConfig,
    OrchestrationProviderConfig, PartnershipConfig, PersonalLicenseConfig, SovereignKeyConfig,
    SovereigntyLevel, UniversalBiomeConfig, UniversalPlatformConfig, UniversalProviderConfig,
};

// Universal primal adapters - these are now capability-based
pub use adapters::{
    UniversalCommConfig,
    UniversalPrimalAdapter,
};

// Core functionality types

// Common result and error types
pub type Result<T> = std::result::Result<T, BiomeError>;

// Common traits
pub trait BiomeOSComponent {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn health(&self) -> HealthStatus;
}

// Version information

// Convenience type aliases
pub type BiomeOSResult<T> = Result<T>;
pub type BiomeOSError = BiomeError;

// Additional re-exports for convenience
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use uuid::Uuid;

// UI-related types
/// API client for biomeOS UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiClient {
    pub endpoint: String,
    pub timeout: u64,
}

pub struct BiomeOSUI {
    pub enabled: bool,
    pub theme: String,
    pub features: UIFeatures,
    pub api_client: Option<ApiClient>,
    pub ai_assistant: Option<AiAssistant>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIFeatures {
    pub dashboard: bool,
    pub metrics: bool,
    pub logs: bool,
    pub settings: bool,
    pub ai_assistant: bool,
    pub real_time_monitoring: bool,
    pub deployment_wizard: bool,
    pub service_management: bool,
    pub log_viewer: bool,
    pub metrics_dashboard: bool,
    pub custom_dashboards: bool,
    pub multi_primal_coordination: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Custom primal configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomPrimalConfig {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniversalUIConfig {
    pub theme: String,
    pub mode: UIMode,
    pub features: UIFeatures,
    pub custom_primals: std::collections::HashMap<String, CustomPrimalConfig>,
    pub primal_endpoints: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniversalUIManager {
    pub config: UniversalUIConfig,
    pub status: SystemStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    pub overall_health: String,
    pub active_primals: Vec<String>,
    pub resource_usage: ResourceUsage,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub network_bytes_per_sec: u64,
}

impl Default for BiomeOSUI {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "dark".to_string(),
            features: UIFeatures {
                dashboard: true,
                metrics: true,
                logs: true,
                settings: true,
                ai_assistant: true,
                real_time_monitoring: true,
                deployment_wizard: true,
                service_management: true,
                log_viewer: true,
                metrics_dashboard: true,
                custom_dashboards: true,
                multi_primal_coordination: true,
            },
            api_client: None,
            ai_assistant: None,
        }
    }
}

impl UniversalUIManager {
    pub fn new(config: UniversalUIConfig) -> Self {
        Self {
            config,
            status: SystemStatus {
                overall_health: "unknown".to_string(),
                active_primals: Vec::new(),
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_percent: 0.0,
                    disk_percent: 0.0,
                    network_bytes_per_sec: 0,
                },
                last_updated: Utc::now(),
            },
        }
    }

    pub async fn initialize(&self) -> std::result::Result<(), anyhow::Error> {
        // Mock implementation
        Ok(())
    }
}
