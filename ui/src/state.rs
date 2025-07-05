//! Application State Management
//! 
//! This module manages the shared state across all UI views in the biomeOS bootstrap UI.
//! Follows the API-driven, universal, sovereignty-first design principles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// System online status
    pub system_online: bool,
    
    /// System installation state
    pub installation: InstallationState,
    
    /// Primal ecosystem status
    pub primals: PrimalsState,
    
    /// Sovereignty and security status
    pub sovereignty: SovereigntyState,
    
    /// System configuration
    pub config: SystemConfig,
    
    /// Real-time metrics
    pub metrics: SystemMetrics,
    
    /// Real-time system metrics (alias for dashboard compatibility)
    pub system_metrics: SystemMetrics,
    
    /// Sovereignty status (alias for dashboard compatibility)
    pub sovereignty_status: SovereigntyState,
    
    /// Ecosystem status (alias for dashboard compatibility)
    pub ecosystem_status: EcosystemStatus,
    
    /// UI preferences
    pub ui_preferences: UiPreferences,
}

/// Installation and setup state
#[derive(Debug, Clone)]
pub struct InstallationState {
    pub status: InstallationStatus,
    pub current_step: InstallationStep,
    pub progress: f32, // 0.0 to 1.0
    pub steps_completed: Vec<InstallationStep>,
    pub error_log: Vec<String>,
    pub ai_guidance: String,
    pub platform_info: PlatformInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    RequiresInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallationStep {
    PlatformDetection,
    DependencyCheck,
    CoreInstallation,
    PrimalDiscovery,
    SecuritySetup,
    ConfigurationSetup,
    Testing,
    Completion,
}

/// Platform detection information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os_type: String,
    pub architecture: String,
    pub cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u64,
    pub network_interfaces: Vec<String>,
    pub container_runtime: Option<String>,
    pub virtualization: Option<String>,
}

/// Primal ecosystem management state
#[derive(Debug, Clone)]
pub struct PrimalsState {
    pub available_primals: HashMap<String, PrimalInfo>,
    pub active_primals: HashMap<String, PrimalStatus>,
    pub primal_health: HashMap<String, HealthStatus>,
    pub ecosystem_graph: EcosystemGraph,
    pub discovery_status: DiscoveryStatus,
}

#[derive(Debug, Clone)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub api_endpoints: Vec<String>,
    pub installation_status: PrimalInstallationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimalInstallationStatus {
    NotInstalled,
    Installing,
    Installed,
    Running,
    Error,
    Updating,
}

#[derive(Debug, Clone)]
pub struct PrimalStatus {
    pub id: String,
    pub status: PrimalInstallationStatus,
    pub health: HealthStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: PrimalMetrics,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PrimalMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_io: (u64, u64), // (bytes_in, bytes_out)
    pub api_requests: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub url: String,
    pub method: String,
    pub description: String,
    pub status: EndpointStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EndpointStatus {
    Online,
    Offline,
    Error,
    Testing,
    Discovering,
    Unknown,
}

/// Ecosystem graph for visualizing primal relationships
#[derive(Debug, Clone)]
pub struct EcosystemGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub position: (f32, f32),
    pub status: HealthStatus,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Core,
    Primal,
    Service,
    External,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relationship: EdgeType,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Dependency,
    ApiCall,
    DataFlow,
    Control,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryStatus {
    Idle,
    Scanning,
    Complete,
    Error,
}

/// Sovereignty and security monitoring state
#[derive(Debug, Clone)]
pub struct SovereigntyState {
    pub compliance_score: f32,
    pub vendor_locks: Vec<VendorLock>,
    pub crypto_locks: CryptoLockStatus,
    pub genetic_key_info: Option<GeneticKeyInfo>,
    pub ai_cat_door: Option<AiCatDoorStatus>,
    pub ai_providers: Vec<AiProvider>,
    pub dependency_assessment: Vec<DependencyAssessment>,
    pub threat_assessments: Vec<ThreatAssessment>,
}

/// Crypto lock status
#[derive(Debug, Clone)]
pub struct CryptoLockStatus {
    pub active_count: u32,
    pub bypassed_count: u32,
    pub total_count: u32,
}

/// Genetic key information
#[derive(Debug, Clone)]
pub struct GeneticKeyInfo {
    pub access_level: AccessLevel,
    pub lineage: Vec<String>,
    pub cost_multiplier: f64,
}

/// Access levels for genetic key system
#[derive(Debug, Clone, PartialEq)]
pub enum AccessLevel {
    Individual,
    SmallBusiness,
    Enterprise,
    MegaCorp,
}

/// AI Cat Door status
#[derive(Debug, Clone)]
pub struct AiCatDoorStatus {
    pub enabled: bool,
    pub cost_protection: f64,
    pub requests_used: u32,
    pub requests_limit: u32,
}

/// System configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub biome_id: String,
    pub installation_mode: InstallationMode,
    pub deployment_target: DeploymentTarget,
    pub api_endpoints: HashMap<String, String>,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallationMode {
    Basic,
    AiResearch,
    SecureEnterprise,
    ScientificComputing,
    EdgeComputing,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentTarget {
    Native,
    Docker,
    Kubernetes,
    Cloud,
    Hybrid,
}

/// Real-time system metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub uptime: std::time::Duration,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_io: (u64, u64),
    pub api_requests_per_second: f32,
    pub active_connections: u32,
    pub last_updated: DateTime<Utc>,
}

/// UI preferences and customization
#[derive(Debug, Clone)]
pub struct UiPreferences {
    pub theme: UiTheme,
    pub show_advanced_options: bool,
    pub auto_refresh_interval: u64, // milliseconds
    pub preferred_view: String,
    pub dev_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiTheme {
    Sovereign, // Default biomeOS theme
    Light,
    Dark,
    Custom,
}

/// Ecosystem status for dashboard
#[derive(Debug, Clone)]
pub struct EcosystemStatus {
    pub toadstool_status: EndpointStatus,
    pub songbird_status: EndpointStatus,
    pub nestgate_status: EndpointStatus,
    pub squirrel_status: EndpointStatus,
    pub beardog_status: EndpointStatus,
}

#[derive(Debug, Clone)]
pub struct AiProvider {
    pub name: String,
    pub enabled: bool,
    pub cost_per_request: f32,
    pub status: EndpointStatus,
}

#[derive(Debug, Clone)]
pub struct DependencyAssessment {
    pub name: String,
    pub dependency_type: String,
    pub sovereignty_impact: SovereigntyImpact,
    pub alternatives: Vec<String>,
    pub bypass_available: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SovereigntyImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ThreatAssessment {
    pub id: Uuid,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub description: String,
    pub mitigation: Option<String>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    VendorLock,
    DataLeakage,
    SecurityVulnerability,
    DependencyRisk,
    ComplianceViolation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct VendorLock {
    pub name: String,
    pub severity: ThreatSeverity,
    pub description: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            system_online: false,
            installation: InstallationState::new(),
            primals: PrimalsState::new(),
            sovereignty: SovereigntyState::new(),
            config: SystemConfig::new(),
            metrics: SystemMetrics::new(),
            system_metrics: SystemMetrics::new(),
            sovereignty_status: SovereigntyState::new(),
            ecosystem_status: EcosystemStatus {
                toadstool_status: EndpointStatus::Discovering,
                songbird_status: EndpointStatus::Discovering,
                nestgate_status: EndpointStatus::Discovering,
                squirrel_status: EndpointStatus::Discovering,
                beardog_status: EndpointStatus::Discovering,
            },
            ui_preferences: UiPreferences::new(),
        }
    }
}

impl InstallationState {
    fn new() -> Self {
        Self {
            status: InstallationStatus::NotStarted,
            current_step: InstallationStep::PlatformDetection,
            progress: 0.0,
            steps_completed: Vec::new(),
            error_log: Vec::new(),
            ai_guidance: "Welcome to biomeOS! I'll guide you through the installation process.".to_string(),
            platform_info: PlatformInfo::detect(),
        }
    }
}

impl PlatformInfo {
    fn detect() -> Self {
        // Simplified platform detection - in real implementation this would
        // use system APIs to detect actual platform capabilities
        Self {
            os_type: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cores: num_cpus::get() as u32,
            memory_gb: 8, // Placeholder
            storage_gb: 500, // Placeholder
            network_interfaces: vec!["eth0".to_string()],
            container_runtime: None,
            virtualization: None,
        }
    }
}

impl PrimalsState {
    fn new() -> Self {
        Self {
            available_primals: HashMap::new(),
            active_primals: HashMap::new(),
            primal_health: HashMap::new(),
            ecosystem_graph: EcosystemGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            discovery_status: DiscoveryStatus::Idle,
        }
    }
}

impl SovereigntyState {
    fn new() -> Self {
        Self {
            compliance_score: 3.0, // Start optimistic
            vendor_locks: Vec::new(),
            crypto_locks: CryptoLockStatus {
                active_count: 0,
                bypassed_count: 0,
                total_count: 0,
            },
            genetic_key_info: None,
            ai_cat_door: None,
            ai_providers: Vec::new(),
            dependency_assessment: Vec::new(),
            threat_assessments: Vec::new(),
        }
    }
}

impl SystemConfig {
    fn new() -> Self {
        Self {
            biome_id: Uuid::new_v4().to_string(),
            installation_mode: InstallationMode::Basic,
            deployment_target: DeploymentTarget::Native,
            api_endpoints: HashMap::new(),
            feature_flags: HashMap::new(),
        }
    }
}

impl SystemMetrics {
    fn new() -> Self {
        Self {
            uptime: std::time::Duration::from_secs(0),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_io: (0, 0),
            api_requests_per_second: 0.0,
            active_connections: 0,
            last_updated: Utc::now(),
        }
    }
}

impl UiPreferences {
    fn new() -> Self {
        Self {
            theme: UiTheme::Sovereign,
            show_advanced_options: false,
            auto_refresh_interval: 1000,
            preferred_view: "Dashboard".to_string(),
            dev_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(!state.system_online);
        assert!(!state.installation_in_progress);
        assert_eq!(state.current_step, 0);
        assert_eq!(state.total_steps, 5);
    }

    #[test]
    fn test_app_state_refresh() {
        let mut state = AppState::new();
        
        // Initial state should be offline
        assert!(!state.system_online);
        
        // Simulate refresh
        state.refresh();
        
        // After refresh, system should be online (mock data)
        assert!(state.system_online);
    }

    #[test]
    fn test_installation_progress() {
        let mut state = AppState::new();
        
        // Start installation
        state.start_installation();
        assert!(state.installation_in_progress);
        assert_eq!(state.current_step, 0);
        
        // Advance steps
        state.advance_step();
        assert_eq!(state.current_step, 1);
        
        // Complete installation
        for _ in 1..state.total_steps {
            state.advance_step();
        }
        
        assert_eq!(state.current_step, state.total_steps);
        assert!(!state.installation_in_progress); // Should auto-complete
    }

    #[test]
    fn test_primal_ecosystem_status() {
        let state = AppState::new();
        
        // Test initial ecosystem status
        assert_eq!(state.ecosystem_status.toadstool_status, EndpointStatus::Discovering);
        assert_eq!(state.ecosystem_status.songbird_status, EndpointStatus::Discovering);
        assert_eq!(state.ecosystem_status.nestgate_status, EndpointStatus::Discovering);
        assert_eq!(state.ecosystem_status.squirrel_status, EndpointStatus::Discovering);
        assert_eq!(state.ecosystem_status.beardog_status, EndpointStatus::Discovering);
    }

    #[test]
    fn test_sovereignty_status() {
        let state = AppState::new();
        
        // Test initial sovereignty status
        assert_eq!(state.sovereignty_status.compliance_score, 3.0);
        assert_eq!(state.sovereignty_status.crypto_locks.active_count, 0);
        assert_eq!(state.sovereignty_status.crypto_locks.bypassed_count, 0);
        assert_eq!(state.sovereignty_status.crypto_locks.total_count, 0);
    }

    #[test]
    fn test_genetic_key_info() {
        let state = AppState::new();
        
        if let Some(key_info) = &state.sovereignty_status.genetic_key_info {
            assert!(!key_info.lineage.is_empty());
            assert_eq!(key_info.access_level, AccessLevel::Individual);
        }
    }

    #[test]
    fn test_ai_cat_door_status() {
        let state = AppState::new();
        
        if let Some(ai_status) = &state.sovereignty_status.ai_cat_door {
            assert!(ai_status.enabled);
            assert_eq!(ai_status.cost_protection, 20.0);
            assert!(ai_status.requests_used <= ai_status.requests_limit);
        }
    }

    #[test]
    fn test_system_metrics() {
        let state = AppState::new();
        
        // Test metric bounds
        assert!(state.system_metrics.cpu_usage >= 0.0);
        assert!(state.system_metrics.cpu_usage <= 100.0);
        assert!(state.system_metrics.memory_usage >= 0.0);
        assert!(state.system_metrics.memory_usage <= 100.0);
        assert!(state.system_metrics.disk_usage >= 0.0);
        assert!(state.system_metrics.disk_usage <= 100.0);
    }

    #[test]
    fn test_crypto_lock_system() {
        let crypto_locks = CryptoLockSystem::default();
        
        assert_eq!(crypto_locks.active_count, 5);
        assert_eq!(crypto_locks.bypassed_count, 0);
        assert_eq!(crypto_locks.total_count, 5);
        
        // Test lock effectiveness
        let effectiveness = crypto_locks.active_count as f32 / crypto_locks.total_count as f32;
        assert_eq!(effectiveness, 1.0); // 100% effective
    }

    #[test]
    fn test_genetic_key_creation() {
        let key = GeneticKeyInfo {
            access_level: AccessLevel::SmallBusiness,
            lineage: vec!["parent-1".to_string(), "parent-2".to_string()],
            cost_multiplier: 0.1,
        };
        
        assert!(!key.lineage.is_empty());
        assert_eq!(key.access_level, AccessLevel::SmallBusiness);
        assert!(key.cost_multiplier > 0.0);
    }

    #[test]
    fn test_access_level_cost_implications() {
        // Test cost multipliers implied by access levels
        let individual = AccessLevel::Individual; // 1.0x
        let small_business = AccessLevel::SmallBusiness; // 0.1x 
        let enterprise = AccessLevel::Enterprise; // 10x
        let mega_corp = AccessLevel::MegaCorp; // 100x
        
        // Verify enum values are correctly defined
        assert_ne!(individual, small_business);
        assert_ne!(small_business, enterprise);
        assert_ne!(enterprise, mega_corp);
    }

    #[test]
    fn test_ai_provider_status() {
        let provider = AiProvider {
            name: "test-provider".to_string(),
            enabled: true,
            cost_per_request: 0.001,
            status: EndpointStatus::Online,
        };
        
        assert!(provider.enabled);
        assert!(provider.cost_per_request > 0.0);
        assert_eq!(provider.status, EndpointStatus::Online);
    }

    #[test]
    fn test_dependency_assessment() {
        let dependency = DependencyAssessment {
            name: "test-dependency".to_string(),
            dependency_type: "external".to_string(),
            sovereignty_impact: SovereigntyImpact::Low,
            alternatives: vec!["alternative-1".to_string()],
            bypass_available: true,
        };
        
        assert!(!dependency.name.is_empty());
        assert_eq!(dependency.sovereignty_impact, SovereigntyImpact::Low);
        assert!(!dependency.alternatives.is_empty());
        assert!(dependency.bypass_available);
    }

    #[test]
    fn test_threat_assessment() {
        let threat = ThreatAssessment {
            id: uuid::Uuid::new_v4(),
            threat_type: ThreatType::VendorLock,
            severity: ThreatSeverity::Medium,
            description: "Test threat description".to_string(),
            mitigation: Some("Test mitigation".to_string()),
            detected_at: Utc::now(),
        };
        
        assert_eq!(threat.threat_type, ThreatType::VendorLock);
        assert_eq!(threat.severity, ThreatSeverity::Medium);
        assert!(!threat.description.is_empty());
        assert!(threat.mitigation.is_some());
    }

    #[test]
    fn test_system_config() {
        let config = SystemConfig {
            biome_id: "test-biome".to_string(),
            installation_mode: InstallationMode::AiResearch,
            deployment_target: DeploymentTarget::Docker,
            api_endpoints: std::collections::HashMap::new(),
            feature_flags: std::collections::HashMap::new(),
        };
        
        assert!(!config.biome_id.is_empty());
        assert_eq!(config.installation_mode, InstallationMode::AiResearch);
        assert_eq!(config.deployment_target, DeploymentTarget::Docker);
    }

    #[test]
    fn test_system_metrics_realistic_values() {
        let metrics = SystemMetrics {
            uptime: std::time::Duration::from_secs(3600), // 1 hour
            cpu_usage: 25.5,
            memory_usage: 45.2,
            disk_usage: 67.8,
            network_io: (1024 * 1024, 512 * 1024), // 1MB in, 512KB out
            api_requests_per_second: 12.5,
            active_connections: 42,
            last_updated: Utc::now(),
        };
        
        assert_eq!(metrics.uptime.as_secs(), 3600);
        assert!(metrics.cpu_usage > 0.0 && metrics.cpu_usage < 100.0);
        assert!(metrics.memory_usage > 0.0 && metrics.memory_usage < 100.0);
        assert!(metrics.disk_usage > 0.0 && metrics.disk_usage < 100.0);
        assert!(metrics.api_requests_per_second > 0.0);
        assert!(metrics.active_connections > 0);
    }

    #[test]
    fn test_ui_preferences() {
        let preferences = UiPreferences {
            theme: UiTheme::Dark,
            show_advanced_options: true,
            auto_refresh_interval: 5000, // 5 seconds
            preferred_view: "dashboard".to_string(),
            dev_mode: false,
        };
        
        assert_eq!(preferences.theme, UiTheme::Dark);
        assert!(preferences.show_advanced_options);
        assert_eq!(preferences.auto_refresh_interval, 5000);
        assert_eq!(preferences.preferred_view, "dashboard");
        assert!(!preferences.dev_mode);
    }

    #[test]
    fn test_endpoint_status_transitions() {
        // Test valid status transitions
        let statuses = vec![
            EndpointStatus::Unknown,
            EndpointStatus::Discovering,
            EndpointStatus::Online,
            EndpointStatus::Offline,
        ];
        
        // All statuses should be distinct
        for (i, status1) in statuses.iter().enumerate() {
            for (j, status2) in statuses.iter().enumerate() {
                if i != j {
                    assert_ne!(status1, status2);
                }
            }
        }
    }

    #[test]
    fn test_state_serialization() {
        let state = AppState::new();
        
        // Test that key components can be serialized
        let json = serde_json::to_string(&state.system_metrics);
        assert!(json.is_ok());
        
        let json = serde_json::to_string(&state.sovereignty_status);
        assert!(json.is_ok());
        
        let json = serde_json::to_string(&state.ecosystem_status);
        assert!(json.is_ok());
    }

    #[test]
    fn test_state_consistency() {
        let state = AppState::new();
        
        // Test consistency rules
        if state.installation_in_progress {
            assert!(state.current_step <= state.total_steps);
        }
        
        // Crypto locks consistency
        let crypto_locks = &state.sovereignty_status.crypto_locks;
        assert_eq!(crypto_locks.active_count + crypto_locks.bypassed_count, crypto_locks.total_count);
        
        // AI cat door consistency
        if let Some(ai_status) = &state.sovereignty_status.ai_cat_door {
            assert!(ai_status.requests_used <= ai_status.requests_limit);
            assert!(ai_status.cost_protection > 0.0);
        }
    }

    #[test]
    fn test_state_updates() {
        let mut state = AppState::new();
        
        // Test state updates don't break invariants
        state.system_online = true;
        state.refresh();
        
        // System should remain consistent after refresh
        let crypto_locks = &state.sovereignty_status.crypto_locks;
        assert!(crypto_locks.active_count <= crypto_locks.total_count);
        assert!(crypto_locks.bypassed_count <= crypto_locks.total_count);
    }

    #[test]
    fn test_sovereignty_score_calculation() {
        let state = AppState::new();
        let score = state.sovereignty_status.compliance_score;
        
        // Score should be realistic based on current state
        assert!(score >= 0.0);
        assert!(score <= 3.0);
        
        // With 5 active crypto locks and no bypassed locks, score should be high
        if state.sovereignty_status.crypto_locks.bypassed_count == 0 {
            assert!(score >= 2.0); // Should be at least mostly sovereign
        }
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that AppState doesn't consume excessive memory
        let states: Vec<_> = (0..100).map(|_| AppState::new()).collect();
        
        // Should be able to create many states without issues
        assert_eq!(states.len(), 100);
        
        // Each state should have consistent initial values
        for state in &states {
            assert_eq!(state.sovereignty_status.compliance_score, 3.0);
            assert_eq!(state.current_step, 0);
        }
    }
} 