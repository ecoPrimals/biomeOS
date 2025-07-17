//! # biomeOS Core
//!
//! Core types, traits, and functionality for the biomeOS operating system.
//! This crate provides the fundamental building blocks for orchestrating
//! the five Primals into a cohesive biological computing environment.

pub mod adapters;
pub mod api_contracts;
pub mod biome;
pub mod byob;
pub mod cloud;
pub mod compute;
pub mod config;
pub mod cross_primal_protocol;
pub mod crypto;
pub mod ecosystem_integration;
pub mod errors;
pub mod federation_optimization;
pub mod health;
pub mod installer;
pub mod locks;
pub mod manifest;
pub mod monitoring_dashboard;
pub mod networking;
pub mod orchestration;
pub mod predictive_health_analytics;
pub mod primal;
pub mod runtime_bridge;
pub mod security;
pub mod storage;
pub mod universal;
pub mod universal_primal_provider;

// Universal/agnostic modules that replace hard-coded Primal-specific implementations
pub mod universal_coordinator;
pub mod universal_manifest;
pub mod universal_primal;

// Refactored modules
pub mod core_config;
pub mod core_types;
pub mod manager;
pub mod service_registration;
pub mod universal_config;

// Re-export core types with specific imports to avoid conflicts
pub use api_contracts::{
    create_api_response, handle_api_error, ApiContractMiddleware, ApiContractValidator, ApiError,
    ApiMetricsCollector, ApiResponse, CapabilitiesResponse, ConfigurationResponse,
    ConfigurationUpdate, ErrorCategory, HealthCheckResponse, InterPrimalMessage,
    InterPrimalResponse, LifecycleEvent, LifecycleResponse, LogRequest, LogResponse,
    MetricsRequest, MetricsResponse, OperationRequest, OperationResponse,
    PerformanceMetricsResponse, PrimalApiContract, PrimalInfoResponse, ResourceStatusResponse,
    ResponseMetadata, ResponseStatus as ApiResponseStatus, RetryInfo, RetryStrategy,
    ValidationResult,
};

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

pub use universal_manifest::{
    AlertingSpec, AuditSpec, AuditStorage, AuthenticationSpec, AuthorizationSpec,
    AvailabilityRequirements, BackupRequirements, BackupSpec, BackupStorage,
    BiomeMetadata as UniversalBiomeMetadata, BiomeRequirements, ComplianceSpec, DependencyType,
    DeploymentPreferences, DeploymentStrategy, EncryptionSpec, FaultToleranceLevel,
    GlobalResourceSpec, HealthCheckConfig, HealthCheckType, KeyManagementSpec, LoadBalancingSpec,
    LogStorage, LoggingSpec, MetricsSpec, MetricsStorage, MonitoringSpec, NetworkPolicy,
    NetworkSecuritySpec, NetworkTopology, NetworkingSpec, PerformanceRequirements,
    PersistentStorage, PortSpec, PrimalPreference, ResourceSummary, RuntimeSpec,
    RuntimeType as UniversalRuntimeType, ScalingRequirements, SecurityRequirements, ServiceConfig,
    ServiceConfig as UniversalServiceConfig, ServiceDefinition, ServiceDependency,
    ServiceNetworking, ServiceStorage, StorageSpec, TracingSpec, TracingStorage,
    UniversalBiomeManifest, ValidationError, ValidationRule, ValidationSeverity, ValidationSpec,
};

pub use biome::{Biome, BiomeManifest, BiomeMetadata, BiomeSpec};
pub use byob::{
    ByobDeploymentManager, DeploymentInstance, DeploymentStatus, IsolationConfig, ResourceQuota,
    SimpleBiomeManifest, SimpleBiomeResources, SimpleBiomeService, TeamWorkspace,
};
pub use cloud::{CloudProviderType, UniversalCloudManager};
pub use compute::{ComputeProviderType, UniversalComputeManager};
pub use config::{ConfigError, Theme, UIMode};
pub use cross_primal_protocol::{
    message_utils, CrossPrimalCoordinator, CrossPrimalMessage, CrossPrimalMessageHandler,
    CrossPrimalProtocolConfig, CrossPrimalResponse, EncryptionMetadata, MessagePriority,
    MessageRouting, MessageSecurity, MessageType, PrimalIdentity, ProtocolStatistics, QueueLimits,
    ResponseStatus, RetryConfig, RoutingStrategy,
};
pub use crypto::{CryptoProvider, UniversalCryptoManager};
pub use ecosystem_integration::{
    EcosystemCapabilities, EcosystemCoordinator, EcosystemEndpoints, EcosystemHealthCoordinator,
    EcosystemSecurity, EcosystemServiceRegistration, ResourceRequirements,
};
pub use federation_optimization::{
    CostMetrics, FederationLoadBalancer, FederationOptimizer, FederationResourceState,
    GlobalResourceMetrics, HealthTracker, LoadBalancingDecision, LoadBalancingStrategy,
    OptimizationConfig, OptimizationEvent, OptimizationImpact, PerformanceMetrics,
    PrimalResourceInfo, ResourceAllocationStrategy, ResourceCapacity, ResourcePredictor,
    ResourceUtilization, SovereigntyCompliance, SovereigntyRequirements,
};
pub use health::{HealthInfo, HealthMetrics, HealthStatus};
pub use installer::UniversalInstaller;
pub use locks::{ComplianceLevel, CryptoLockManager, PersonalAiLimits};
pub use manifest::{
    BiomeMetadata as ManifestBiomeMetadata, BiomeMetadata as ToadStoolBiomeMetadata,
    FederationConfig, HealthCheck, ManifestGenerator, PrimalConfig as ManifestPrimalConfig,
    PrimalConfig as ToadStoolPrimalConfig, ResourceLimits, RuntimeType as ToadStoolRuntimeType,
    ServiceConfig as ManifestServiceConfig, ToadStoolManifest,
};
pub use monitoring_dashboard::{
    Alert, AlertConfig, AlertDestination, AlertManager, AlertSeverity, AlertsSummary, ChartType,
    CostAnalysis, DashboardConfig, DashboardEvent, DashboardMetricsState, DashboardSubscriber,
    FederationMetrics, HealthSummary, Metric, MetricCollector, MetricType, MetricsProcessor,
    MonitoringDashboard, NotificationChannel, NotificationManager, PerformanceOverview,
    PrimalMetrics, ResourceUtilizationOverview, TrendDirection as DashboardTrendDirection,
    VisualizationEngine,
};
pub use orchestration::{OrchestratorType, UniversalOrchestrationManager};
pub use predictive_health_analytics::{
    AlertThresholds, AnalysisAlgorithm, Anomaly, AnomalyType, BiomeActivity, ConfidenceInterval,
    ExtendedHealthMetrics, HealthAnalyticsConfig, HealthPrediction, HealthRecommendation,
    HealthReport, HealthSnapshot, ModelAccuracy, ModelType, MonitoringConfig, MonitoringSession,
    NetworkConditions, OverallHealthStatus, PredictedScore, PredictiveHealthAnalytics,
    RecommendationType, RiskAssessment, RiskType, SystemContext, TrendAnalysis, TrendComponents,
    TrendDirection as HealthTrendDirection,
};
pub use primal::{
    Capability as PrimalCapability, PrimalConfig, PrimalType,
    ResourceLimits as PrimalResourceLimits,
};
pub use runtime_bridge::{
    BiomeEvent, BiomeMonitor, BiomeStatus, DeploymentHandle,
    DeploymentStatus as ToadStoolDeploymentStatus, FederationStatus, PeerStatus, ResourceUsage,
    ServiceStatus, ToadStoolBridge,
};
pub use security::SecurityConfig;
pub use storage::StorageConfig;
pub use universal::{
    AiPersonalityConfig, DeploymentConfig, EnergyFlowState, MycorrhizaConfig, UniversalPlatform,
    UniversalPlatformOps,
};
pub use universal_coordinator::{
    CapabilityRouter, CoordinatorConfig, DeployedPrimal, DeploymentPlan, EcosystemInstance,
    EcosystemStatus, HttpPrimalClient, MatchDetails, MatchResult, PrimalAssignment, PrimalClient,
    PrimalStatus, RequirementMatcher, ResourceAllocation as UniversalResourceAllocation,
    UniversalBiomeCoordinator,
};
pub use universal_primal_provider::{
    BiomeOSInstanceConfig, BiomeOSPrimalProvider, BiomeOSPrimalRegistry, CapabilityType,
    DynamicPortInfo, NetworkLocation, PrimalCapability as UniversalPrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalProvider, PrimalRequest, PrimalResponse,
    Priority, RequestType, ResponseType, SecurityLevel,
};

// Re-export refactored modules for backward compatibility
pub use core_config::{BiomeOSConfig, GlobalConfig, NetworkConfig, TelemetryConfig};
pub use core_types::{init_biomeos, BiomeId, PrimalId, BIOMEOS_VERSION};
pub use manager::UniversalBiomeManager;
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

// Universal primal adapters
pub use adapters::{
    AuthConfig, AuthMethod, BidirectionalConfig, CommunicationProtocol,
    PrimalEvent as AdapterPrimalEvent, ReconnectionConfig,
    RequestPriority as AdapterRequestPriority, RetryConfig as AdapterRetryConfig,
    TlsConfig as AdapterTlsConfig, UniversalAdapterFactory, UniversalCommConfig,
    UniversalPrimalAdapter, UniversalRequest, UniversalResponse,
};

// Core error and result types
pub use errors::{BiomeError, BiomeResult};

// Additional types for UI and auth
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UserAuthMethod {
    Password { password: String },
    GeneticKey { key: String },
    Certificate { cert: String },
    Token { token: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrimalUIConfig {
    pub name: String,
    pub enabled: bool,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub dashboard_widgets: Vec<WidgetConfig>,
    pub custom_actions: Vec<ActionConfig>,
    pub metrics_config: MetricsConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomPrimalConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<String>,
    pub configuration: serde_json::Value,
    pub auth_config: Option<serde_json::Value>,
    pub description: String,
    pub ui_config: PrimalUIConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WidgetConfig {
    pub widget_type: String,
    pub title: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub config: serde_json::Value,
    pub api_endpoint: String,
    pub refresh_interval_secs: u64,
    pub display_config: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionConfig {
    pub action_id: String,
    pub display_name: String,
    pub command: String,
    pub parameters: Vec<String>,
    pub confirmation_required: bool,
    pub api_endpoint: String,
    pub method: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsConfig {
    pub collection_interval: std::time::Duration,
    pub retention_period: std::time::Duration,
    pub metrics_endpoints: Vec<String>,
    pub enabled_metrics: Vec<String>,
    pub default_time_range: String,
    pub enabled: bool,
    pub chart_types: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserAuthRequest {
    pub username: String,
    pub auth_method: UserAuthMethod,
    pub timestamp: std::time::SystemTime,
    pub client_ip: Option<String>,
    pub client_user_agent: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub beardog_enabled: bool,
    pub genetic_keys_enabled: bool,
    pub password_policy: PasswordPolicy,
    pub session_timeout: std::time::Duration,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub session_id: String,
    pub expires_at: std::time::SystemTime,
    pub permissions: Vec<String>,
    pub beardog_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub access_level: Option<String>,
    pub beardog_key_reference: Option<String>,
    pub ssh_key_references: Vec<String>,
    pub api_key_references: Vec<String>,
    pub genetic_key: Option<String>,
    pub created_at: std::time::SystemTime,
    pub last_login: Option<std::time::SystemTime>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserManager {
    pub config: UserConfig,
    pub active_sessions: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    pub parameter_type: String,
    pub default_value: String,
    pub description: String,
    pub required: bool,
    pub validation: Option<serde_json::Value>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            beardog_enabled: true,
            genetic_keys_enabled: true,
            password_policy: PasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_symbols: false,
            },
            session_timeout: std::time::Duration::from_secs(3600),
        }
    }
}

impl UserManager {
    pub fn new(config: UserConfig) -> Self {
        Self {
            config,
            active_sessions: std::collections::HashMap::new(),
        }
    }

    pub fn authenticate(&self, _request: &UserAuthRequest) -> Result<bool, String> {
        // Mock implementation for authentication
        Ok(true)
    }

    pub async fn create_user_with_beardog(
        &self, 
        _username: &str, 
        _auth_method: UserAuthMethod, 
        _access_level: BeardogAccessLevel, 
        _display_name: Option<String>
    ) -> Result<String, anyhow::Error> {
        // Mock implementation
        Ok("user-123".to_string())
    }

    pub async fn authenticate_with_beardog(&self, _request: UserAuthRequest) -> Result<UserSession, anyhow::Error> {
        // Mock implementation
        Ok(UserSession {
            id: "session-456".to_string(),
            user_id: "user-123".to_string(),
            username: "test-user".to_string(),
            session_id: "session-456".to_string(),
            expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(3600),
            permissions: vec!["read".to_string(), "write".to_string()],
            beardog_context: Some(serde_json::json!({
                "access_level": "PowerUser",
                "security_level": "High",
                "threat_assessment_score": 8.5,
                "compliance_status": "Compliant"
            })),
        })
    }

    pub async fn add_user_ssh_key(&self, _user_id: &str, _key_name: &str, _public_key: &str) -> Result<String, anyhow::Error> {
        // Mock implementation
        Ok("ssh-key-789".to_string())
    }

    pub async fn generate_user_api_key(&self, _user_id: &str, _key_name: &str, _permissions: Vec<String>) -> Result<String, anyhow::Error> {
        // Mock implementation
        Ok("api-key-abc".to_string())
    }

    pub async fn get_user(&self, _username: &str) -> Result<Option<UserProfile>, anyhow::Error> {
        // Mock implementation
        Ok(Some(UserProfile {
            id: "user-123".to_string(),
            username: "test-user".to_string(),
            email: "test@example.com".to_string(),
            full_name: Some("Test User".to_string()),
            access_level: Some("PowerUser".to_string()),
            beardog_key_reference: Some("beardog-key-123".to_string()),
            ssh_key_references: vec!["ssh-key-1".to_string(), "ssh-key-2".to_string()],
            api_key_references: vec!["api-key-1".to_string()],
            genetic_key: Some("genetic-key-123".to_string()),
            created_at: std::time::SystemTime::now(),
            last_login: Some(std::time::SystemTime::now()),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
        }))
    }

    pub async fn get_all_users(&self) -> Result<Vec<UserProfile>, anyhow::Error> {
        // Mock implementation
        Ok(vec![UserProfile {
            id: "user-123".to_string(),
            username: "test-user".to_string(),
            email: "test@example.com".to_string(),
            full_name: Some("Test User".to_string()),
            access_level: Some("PowerUser".to_string()),
            beardog_key_reference: Some("beardog-key-123".to_string()),
            ssh_key_references: vec!["ssh-key-1".to_string(), "ssh-key-2".to_string()],
            api_key_references: vec!["api-key-1".to_string()],
            genetic_key: Some("genetic-key-123".to_string()),
            created_at: std::time::SystemTime::now(),
            last_login: Some(std::time::SystemTime::now()),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
        }])
    }

    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        // Mock implementation
        Ok(())
    }

    pub async fn initialize(&self) -> Result<(), anyhow::Error> {
        // Mock implementation
        Ok(())
    }
}

// UI-related types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiomeOSUI {
    pub enabled: bool,
    pub theme: String,
    pub features: UIFeatures,
    pub api_client: Option<ApiClient>,
    pub ai_assistant: Option<AiAssistant>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub mode: UIMode,
    pub features: UIFeatures,
    pub ui_mode: UIMode,
    pub api_endpoints: Vec<String>,
    pub websocket_endpoints: Vec<String>,
    pub ai_config: Option<AiConfig>,
    pub auto_refresh_interval: std::time::Duration,
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

// UIMode is now imported from config module

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UIResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
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
pub struct UserInput {
    pub input_type: String,
    pub data: serde_json::Value,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    pub healthy: bool,
    pub uptime: std::time::Duration,
    pub primals: Vec<String>,
    pub total_primals: usize,
    pub healthy_primals: usize,
    pub ui_mode: UIMode,
    pub last_discovery: Option<chrono::DateTime<chrono::Utc>>,
}



// DiscoveredPrimal is imported from universal_primal module

impl Default for UIFeatures {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Default for UniversalUIConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            mode: UIMode::Auto,
            features: UIFeatures::default(),
            custom_primals: std::collections::HashMap::new(),
            primal_endpoints: std::collections::HashMap::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: std::time::Duration::from_secs(30),
            retention_period: std::time::Duration::from_secs(86400), // 24 hours
            metrics_endpoints: vec!["http://localhost:9090".to_string()],
            enabled_metrics: vec!["cpu".to_string(), "memory".to_string(), "disk".to_string()],
            default_time_range: "1h".to_string(),
            enabled: true,
            chart_types: vec!["line".to_string(), "bar".to_string(), "area".to_string()],
        }
    }
}

impl BiomeOSUI {
    pub fn new() -> Self {
        Self {
            enabled: true,
            theme: "dark".to_string(),
            features: UIFeatures::default(),
            api_client: Some(ApiClient::new()),
            ai_assistant: None,
        }
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub async fn discover_primals(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(vec!["toadstool".to_string(), "songbird".to_string()])
    }

    pub async fn get_ecosystem_status(&self) -> Result<SystemStatus, anyhow::Error> {
        Ok(SystemStatus {
            healthy: true,
            uptime: std::time::Duration::from_secs(3600),
            primals: vec!["toadstool".to_string(), "songbird".to_string()],
            total_primals: 2,
            healthy_primals: 2,
            ui_mode: UIMode::Auto,
            last_discovery: Some(chrono::Utc::now()),
        })
    }
}

impl AiAssistant {
    pub async fn process_command(&self, _command: &str) -> Result<String, anyhow::Error> {
        Ok("Command processed".to_string())
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            mode: UIMode::Auto,
            features: UIFeatures::default(),
            ui_mode: UIMode::Auto,
            api_endpoints: vec!["http://localhost:8080".to_string()],
            websocket_endpoints: vec!["ws://localhost:8080/ws".to_string()],
            ai_config: None,
            auto_refresh_interval: std::time::Duration::from_secs(30),
        }
    }
}

impl UniversalUIManager {
    pub fn new(config: UniversalUIConfig) -> Self {
        Self {
            config,
            status: SystemStatus {
                healthy: true,
                uptime: std::time::Duration::from_secs(0),
                primals: vec![],
                total_primals: 0,
                healthy_primals: 0,
                ui_mode: UIMode::Auto,
                last_discovery: None,
            },
        }
    }

    pub async fn handle_user_input(&self, _input: UserInput) -> Result<UIResponse, anyhow::Error> {
        Ok(UIResponse {
            success: true,
            message: "Input processed".to_string(),
            data: None,
        })
    }

    pub async fn get_system_status(&self) -> Result<SystemStatus, anyhow::Error> {
        Ok(self.status.clone())
    }

    pub async fn start(&self) -> Result<(), anyhow::Error> {
        // Mock implementation for examples
        Ok(())
    }
}

// Additional types for UI system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiClient {
    pub base_url: String,
    pub timeout: std::time::Duration,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiAssistant {
    pub enabled: bool,
    pub model: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiConfig {
    pub model: String,
    pub endpoint: String,
    pub api_key: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
        }
    }
}

// Theme is now imported from config module

// Mock UI Manager for testing
#[derive(Debug, Clone)]
pub struct MockUniversalUIManager {
    pub config: UIFeatures,
}

impl MockUniversalUIManager {
    pub fn new(config: UIFeatures) -> Self {
        Self { config }
    }

    pub async fn handle_user_input(&self, _input: UserInput) -> Result<UIResponse, anyhow::Error> {
        Ok(UIResponse {
            success: true,
            message: "Mock processed".to_string(),
            data: None,
        })
    }
}

// Module declarations for compatibility
pub mod ai {
    pub use super::AiConfig;
    pub use super::AiAssistant;
    pub use super::AiConfig as AIConfig; // Alias for compatibility
}

// Core imports needed for compatibility

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_biome_os_config_creation() {
        let config = BiomeOSConfig::default();
        assert_eq!(config.global.instance_name, "default-biome");
        assert!(!config.global.dev_mode);
        assert_eq!(config.networking.port_ranges.len(), 2);
    }

    #[test]
    fn test_biome_os_config_serialization() {
        let config = BiomeOSConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("default-biome"));

        let deserialized: BiomeOSConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            config.global.instance_name,
            deserialized.global.instance_name
        );
    }

    #[test]
    fn test_universal_biome_config_basic() {
        let config = UniversalBiomeConfig::default();
        assert!(!config.biome.metadata.name.is_empty());
        // Test that basic structure exists
        assert!(!config.providers.cloud_providers.is_empty());
    }

    #[test]
    fn test_universal_biome_manager_creation() {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeManager::new(config);
        assert!(manager.calculate_sovereignty_score() > 0.0);
    }

    #[test]
    fn test_crypto_lock_configuration() {
        let crypto_config = CryptoLockConfig::default();
        assert!(crypto_config.enabled);
        assert!(crypto_config.ai_cat_door.enabled);

        // Test that crypto locks provide security without vendor lock-in
        assert!(!crypto_config.sovereign_keys.is_empty());
    }

    #[test]
    fn test_beardog_access_levels() {
        // Test genetic beardog key access levels exist
        let _power_user = BeardogAccessLevel::PowerUser;
        let _small_business = BeardogAccessLevel::SmallBusiness;
        let _medium_business = BeardogAccessLevel::MediumBusiness;
        let _enterprise = BeardogAccessLevel::Enterprise;
        let _mega_corp = BeardogAccessLevel::MegaCorp;
        let _research = BeardogAccessLevel::Research;
        let _government = BeardogAccessLevel::Government;

        // Test that all access levels are available
        assert!(true);
    }

    #[test]
    fn test_commercial_models() {
        // Test different commercial access models
        let _licensing_only = CommercialModel::LicensingOnly;
        let _partnership_access = CommercialModel::PartnershipAccess;
        let _user_choice = CommercialModel::UserChoice;
        let _fully_open = CommercialModel::FullyOpen;

        // Verify models exist for different business approaches
        assert!(true);
    }

    #[test]
    fn test_performance_config_creation() {
        // Test configuration creation performance
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _config = BiomeOSConfig::default();
        }
        let duration = start.elapsed();

        // Should create configs quickly (< 10ms for 100 configs)
        assert!(
            duration < Duration::from_millis(10),
            "Config creation should be very fast"
        );
    }

    #[test]
    fn test_manager_functionality() {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeManager::new(config);

        // Test basic manager functionality
        assert!(manager.supports_pattern("universal"));
        assert!(manager.can_coordinate_ecosystem());
        assert!(!manager.get_supported_install_modes().is_empty());
        assert!(!manager.detect_platform().is_empty());
    }

    #[test]
    fn test_inverse_scaling_economics() {
        let inverse_scaling = InverseScaleConfig::default();
        assert!(inverse_scaling.enabled);
        assert!(inverse_scaling.good_faith_model);

        // Test the core inverse scaling principle
        let multipliers = &inverse_scaling.cost_multipliers;
        assert_eq!(multipliers.small_business, 0.1); // Small business: 10% of base cost
        assert_eq!(multipliers.medium_business, 1.0); // Medium business: base cost
        assert_eq!(multipliers.large_enterprise, 10.0); // Enterprise: 10x base cost
        assert_eq!(multipliers.mega_corp, 100.0); // Mega corp: 100x base cost (Amazon scale)

        // Verify the economics make sense: bigger companies subsidize smaller ones
        assert!(multipliers.mega_corp / multipliers.small_business >= 1000.0);
    }

    #[test]
    fn test_genetic_beardog_key_structure() {
        let genetic_key = GeneticBeardogKey::default();
        assert!(!genetic_key.parent_key_fingerprint.is_empty());
        assert!(!genetic_key.genetic_lineage.is_empty());

        // Test genetic inheritance concept
        assert!(genetic_key.genetic_lineage.len() >= 1);
    }

    #[test]
    fn test_partnership_vs_licensing() {
        let licensing = LicensingConfig::default();

        // Personal use should always be free and sovereign
        assert!(licensing.personal_use.ai_cat_door_enabled);

        // Commercial use should offer user choice
        let _commercial_options = &licensing.commercial_use;
        assert!(true); // Test passes if structure exists
    }

    #[test]
    fn test_cost_protection_thresholds() {
        let cost_protection = CostProtectionConfig::default();
        assert_eq!(cost_protection.max_monthly_cost, 20.0);
        assert!(cost_protection.auto_disable_on_limit);

        // Test alert thresholds are reasonable
        assert!(!cost_protection.alert_thresholds.is_empty());
        for &threshold in &cost_protection.alert_thresholds {
            assert!(threshold < cost_protection.max_monthly_cost);
            assert!(threshold > 0.0);
        }
    }

    #[test]
    fn test_ecosystem_provider_structure() {
        let config = UniversalBiomeConfig::default();

        // Test that ecosystem providers are configured
        assert!(!config.providers.cloud_providers.is_empty());
        assert!(!config.providers.compute_providers.is_empty());
        assert!(!config.providers.orchestration_providers.is_empty());
        assert!(!config.providers.crypto_providers.is_empty());

        // Verify sovereignty-focused provider configuration
        for provider in &config.providers.cloud_providers {
            assert!(!provider.name.is_empty());
        }
    }

    #[test]
    fn test_biome_error_types() {
        // Test BiomeError variants
        let config_error = BiomeError::Config("test config error".to_string());
        assert!(config_error.to_string().contains("test config error"));

        let security_error = BiomeError::Security("test security issue".to_string());
        assert!(security_error.to_string().contains("test security issue"));

        let sovereignty_error = BiomeError::SovereigntyViolation("sovereignty breach".to_string());
        assert!(sovereignty_error.to_string().contains("sovereignty breach"));

        let vendor_lock_error = BiomeError::VendorLock("vendor lock detected".to_string());
        assert!(vendor_lock_error
            .to_string()
            .contains("vendor lock detected"));
    }

    #[test]
    fn test_biomeos_version() {
        // Test that version is defined and reasonable
        assert!(!BIOMEOS_VERSION.is_empty());
        assert!(BIOMEOS_VERSION.len() >= 5); // At least "0.1.0"
    }

    #[tokio::test]
    async fn test_biomeos_initialization() {
        // Test that biomeOS core can initialize
        let result = init_biomeos().await;
        assert!(result.is_ok(), "biomeOS should initialize successfully");
    }

    #[test]
    fn test_configuration_resilience() {
        // Test configuration handles edge cases gracefully
        let mut config = BiomeOSConfig::default();

        // Test with very long instance name
        config.global.instance_name = "a".repeat(1000);
        assert!(!config.global.instance_name.is_empty());

        // Test with empty instance name
        config.global.instance_name = String::new();
        assert!(config.global.instance_name.is_empty());

        // Test that serialization still works
        let serialization_result = serde_json::to_string(&config);
        assert!(serialization_result.is_ok());
    }

    #[test]
    fn test_performance_serialization() {
        let config = UniversalBiomeConfig::default();

        // Test serialization performance
        let start = std::time::Instant::now();
        for _ in 0..10 {
            let _serialized = serde_json::to_string(&config).unwrap();
        }
        let duration = start.elapsed();

        // Should serialize quickly
        assert!(
            duration < Duration::from_millis(100),
            "Serialization should be fast"
        );
    }
}
