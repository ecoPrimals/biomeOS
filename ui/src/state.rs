//! Application State Management
//! 
//! This module manages the shared state across all UI views in the biomeOS bootstrap UI.
//! Follows the API-driven, universal, sovereignty-first design principles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Installation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstallationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    RequiresInput,
}

/// Installation steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Primal installation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub enum DiscoveryStatus {
    Idle,
    Scanning,
    Complete,
    Error,
}

/// Sovereignty and security state
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoLockStatus {
    pub active_count: u32,
    pub bypassed_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticKeyInfo {
    pub access_level: AccessLevel,
    pub lineage: Vec<String>,
    pub cost_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Individual,
    SmallBusiness,
    Enterprise,
    MegaCorp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorStatus {
    pub enabled: bool,
    pub cost_protection: f64,
    pub requests_used: u32,
    pub requests_limit: u32,
}

#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub biome_id: String,
    pub installation_mode: InstallationMode,
    pub deployment_target: DeploymentTarget,
    pub api_endpoints: HashMap<String, String>,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub enum InstallationMode {
    Basic,
    AiResearch,
    SecureEnterprise,
    ScientificComputing,
    EdgeComputing,
    Custom,
}

#[derive(Debug, Clone)]
pub enum DeploymentTarget {
    Native,
    Docker,
    Kubernetes,
    Cloud,
    Hybrid,
}

/// System metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct UiPreferences {
    pub theme: UiTheme,
    pub show_advanced_options: bool,
    pub auto_refresh_interval: u64, // milliseconds
    pub preferred_view: String,
    pub dev_mode: bool,
}

#[derive(Debug, Clone)]
pub enum UiTheme {
    Sovereign, // Default biomeOS theme
    Light,
    Dark,
    Custom,
}

/// Ecosystem status for all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub toadstool_status: EndpointStatus,
    pub songbird_status: EndpointStatus,
    pub nestgate_status: EndpointStatus,
    pub squirrel_status: EndpointStatus,
    pub beardog_status: EndpointStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub name: String,
    pub enabled: bool,
    pub cost_per_request: f32,
    pub status: EndpointStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAssessment {
    pub name: String,
    pub dependency_type: String,
    pub sovereignty_impact: SovereigntyImpact,
    pub alternatives: Vec<String>,
    pub bypass_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub id: Uuid,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub description: String,
    pub mitigation: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Threat types for security assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatType {
    VendorLock,
    DataLeakage,
    SecurityVulnerability,
    DependencyRisk,
    ComplianceViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            ecosystem_status: EcosystemStatus::new(),
            ui_preferences: UiPreferences::new(),
        }
    }
    
    /// Refresh application state
    pub fn refresh(&mut self) {
        self.system_metrics = SystemMetrics::new();
        self.sovereignty_status = self.sovereignty.clone();
        self.ecosystem_status = EcosystemStatus::new();
    }
    
    /// Start installation process
    pub fn start_installation(&mut self) {
        self.installation.status = InstallationStatus::InProgress;
        self.installation.current_step = InstallationStep::PlatformDetection;
        self.installation.progress = 0.0;
    }
    
    /// Advance to next installation step
    pub fn advance_step(&mut self) {
        use InstallationStep::*;
        
        self.installation.current_step = match self.installation.current_step {
            PlatformDetection => DependencyCheck,
            DependencyCheck => CoreInstallation,
            CoreInstallation => PrimalDiscovery,
            PrimalDiscovery => SecuritySetup,
            SecuritySetup => ConfigurationSetup,
            ConfigurationSetup => Testing,
            Testing => Completion,
            Completion => Completion, // Stay at completion
        };
        
        let total_steps = 8.0;
        let current_step_num = match self.installation.current_step {
            PlatformDetection => 1.0,
            DependencyCheck => 2.0,
            CoreInstallation => 3.0,
            PrimalDiscovery => 4.0,
            SecuritySetup => 5.0,
            ConfigurationSetup => 6.0,
            Testing => 7.0,
            Completion => 8.0,
        };
        
        self.installation.progress = current_step_num / total_steps;
        
        if self.installation.current_step == Completion {
            self.installation.status = InstallationStatus::Completed;
        }
    }
    
    /// Get installation progress as 0-based step number
    pub fn current_step(&self) -> usize {
        match self.installation.current_step {
            InstallationStep::PlatformDetection => 0,
            InstallationStep::DependencyCheck => 1,
            InstallationStep::CoreInstallation => 2,
            InstallationStep::PrimalDiscovery => 3,
            InstallationStep::SecuritySetup => 4,
            InstallationStep::ConfigurationSetup => 5,
            InstallationStep::Testing => 6,
            InstallationStep::Completion => 7,
        }
    }
    
    /// Get total number of installation steps
    pub fn total_steps(&self) -> usize {
        8
    }
    
    /// Check if installation is in progress
    pub fn installation_in_progress(&self) -> bool {
        self.installation.status == InstallationStatus::InProgress
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
            ai_guidance: "Ready to begin installation".to_string(),
            platform_info: PlatformInfo::detect(),
        }
    }
}

impl PlatformInfo {
    fn detect() -> Self {
        Self {
            os_type: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            cores: 8,
            memory_gb: 16,
            storage_gb: 500,
            network_interfaces: vec!["eth0".to_string()],
            container_runtime: Some("docker".to_string()),
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
            ecosystem_graph: EcosystemGraph { nodes: Vec::new(), edges: Vec::new() },
            discovery_status: DiscoveryStatus::Idle,
        }
    }
}

impl SovereigntyState {
    fn new() -> Self {
        Self {
            compliance_score: 0.85,
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
            biome_id: "default-biome".to_string(),
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
            auto_refresh_interval: 5000,
            preferred_view: "dashboard".to_string(),
            dev_mode: false,
        }
    }
}

impl EcosystemStatus {
    pub fn new() -> Self {
        Self {
            toadstool_status: EndpointStatus::Unknown,
            songbird_status: EndpointStatus::Unknown,
            nestgate_status: EndpointStatus::Unknown,
            squirrel_status: EndpointStatus::Unknown,
            beardog_status: EndpointStatus::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(!state.system_online);
        assert!(!state.installation_in_progress());
        assert_eq!(state.current_step(), 0);
        assert_eq!(state.total_steps(), 8);
    }

    #[test]
    fn test_app_state_refresh() {
        let mut state = AppState::new();
        
        // Test refresh functionality
        state.refresh();
        
        // Should have updated metrics
        assert!(state.system_metrics.last_updated <= Utc::now());
    }

    #[test]
    fn test_installation_progress() {
        let mut state = AppState::new();
        
        // Start installation
        state.start_installation();
        assert!(state.installation_in_progress());
        assert_eq!(state.current_step(), 0);
        
        // Advance step
        state.advance_step();
        assert_eq!(state.current_step(), 1);
        
        // Complete all steps
        for _ in 1..state.total_steps() {
            state.advance_step();
        }
        
        assert_eq!(state.current_step(), state.total_steps() - 1);
        assert!(!state.installation_in_progress()); // Should auto-complete
    }

    #[test]
    fn test_threat_assessment() {
        let threat = ThreatAssessment {
            id: Uuid::new_v4(),
            threat_type: ThreatType::VendorLock,
            severity: ThreatSeverity::High,
            description: "AWS dependency detected".to_string(),
            mitigation: Some("Use crypto locks".to_string()),
            detected_at: Utc::now(),
        };
        
        assert_eq!(threat.threat_type, ThreatType::VendorLock);
    }

    #[test]
    fn test_state_serialization() {
        let state = AppState::new();
        
        // Test serialization of key components
        let json = serde_json::to_string(&state.system_metrics);
        assert!(json.is_ok());
        
        let json = serde_json::to_string(&state.sovereignty_status);
        assert!(json.is_ok());
        
        let json = serde_json::to_string(&state.ecosystem_status);
        assert!(json.is_ok());
    }

    #[test]
    fn test_state_consistency() {
        let mut state = AppState::new();
        
        // Test that state remains consistent during operations
        if state.installation_in_progress() {
            assert!(state.current_step() <= state.total_steps());
        }
        
        // Test refresh maintains consistency
        state.refresh();
        
        // Test various state transitions
        for _i in 0..10 {
            state.refresh();
            
            // State should remain valid
            assert!(state.sovereignty.compliance_score >= 0.0);
            assert!(state.sovereignty.compliance_score <= 1.0);
            
            // Test specific field access without moving
            let current_step_index = state.current_step();
            let installation_step_index = match state.installation.current_step {
                InstallationStep::PlatformDetection => 0,
                InstallationStep::DependencyCheck => 1,
                InstallationStep::CoreInstallation => 2,
                InstallationStep::PrimalDiscovery => 3,
                InstallationStep::SecuritySetup => 4,
                InstallationStep::ConfigurationSetup => 5,
                InstallationStep::Testing => 6,
                InstallationStep::Completion => 7,
            };
            assert_eq!(current_step_index, installation_step_index);
        }
    }
}
