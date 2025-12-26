//! TUI Dashboard Types for Ecosystem Interface
//!
//! Comprehensive types for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.
//! Supports API ingestion from all primals and deployment orchestration.

use crate::health::SystemHealth;
use biomeos_core::universal_biomeos_manager::DiscoveryResult;
use biomeos_types::{Health, PrimalCapability, PrimalType};
use ratatui::widgets::ListState;
// Remove unused serde imports
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Tab information for the comprehensive ecosystem interface
#[derive(Clone)]
pub struct TabInfo {
    pub title: &'static str,
    pub id: TabId,
    pub icon: &'static str,
}

/// Available tabs in the ecosystem dashboard
#[derive(Clone, Copy, PartialEq)]
pub enum TabId {
    // Core ecosystem management
    EcosystemOverview,
    PrimalStatus,
    DeploymentOrchestration,

    // Operational views
    Services,
    Health,

    // AI-first interfaces
    AiAssistant,
    AiInsights,

    // Advanced features
    ApiIngestion,
    Metrics,
    Logs,
}

impl TabId {
    /// Get all available tabs for the ecosystem interface
    pub fn all_tabs() -> Vec<TabInfo> {
        vec![
            TabInfo {
                title: "Ecosystem",
                id: TabId::EcosystemOverview,
                icon: "🌍",
            },
            TabInfo {
                title: "Primals",
                id: TabId::PrimalStatus,
                icon: "🎯",
            },
            TabInfo {
                title: "Deploy",
                id: TabId::DeploymentOrchestration,
                icon: "🚀",
            },
            TabInfo {
                title: "Services",
                id: TabId::Services,
                icon: "⚙️",
            },
            TabInfo {
                title: "Health",
                id: TabId::Health,
                icon: "💊",
            },
            TabInfo {
                title: "AI Assistant",
                id: TabId::AiAssistant,
                icon: "🤖",
            },
            TabInfo {
                title: "AI Insights",
                id: TabId::AiInsights,
                icon: "🧠",
            },
            TabInfo {
                title: "API Data",
                id: TabId::ApiIngestion,
                icon: "📡",
            },
            TabInfo {
                title: "Metrics",
                id: TabId::Metrics,
                icon: "📊",
            },
            TabInfo {
                title: "Logs",
                id: TabId::Logs,
                icon: "📜",
            },
        ]
    }
}

/// Comprehensive dashboard state for ecosystem management
pub struct DashboardState {
    // Navigation state
    pub current_tab: usize,
    pub selected_service: usize,
    pub selected_primal: usize,
    pub selected_deployment: usize,
    pub service_list_state: ListState,
    pub primal_list_state: ListState,
    pub deployment_list_state: ListState,

    // Ecosystem data from primal APIs
    pub primal_states: HashMap<String, PrimalApiState>,
    pub discovered_services: Vec<DiscoveryResult>,
    pub active_deployments: Vec<DeploymentStatus>,
    pub ecosystem_health: EcosystemHealth,

    // Historical data for analytics
    pub system_health_history: VecDeque<(Instant, SystemHealth)>,
    pub primal_health_history: HashMap<String, VecDeque<(Instant, Health)>>,
    pub capability_stats: HashMap<String, usize>,
    pub deployment_history: VecDeque<DeploymentEvent>,

    // AI-first interface state
    pub ai_chat_history: VecDeque<AiChatMessage>,
    pub ai_suggestions: Vec<AiSuggestion>,
    pub ai_insights: Vec<AiInsight>,
    pub ai_input_buffer: String,

    // API ingestion status
    pub api_endpoints: HashMap<String, ApiEndpointStatus>,
    pub last_api_sync: HashMap<String, Instant>,
    pub api_errors: VecDeque<ApiError>,

    // Real-time log streaming
    pub log_streams: HashMap<String, VecDeque<LogEntry>>,
    pub active_log_filters: Vec<LogFilter>,

    // Settings and configuration
    pub update_interval: Duration,
    pub max_history_points: usize,
    pub ai_enabled: bool,
    pub auto_refresh: bool,
}

/// State of a primal obtained from its headless API
#[derive(Debug, Clone)]
pub struct PrimalApiState {
    pub primal_id: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub health: Health,
    pub capabilities: Vec<PrimalCapability>,
    pub metadata: PrimalMetadata,
    pub services: Vec<PrimalServiceInfo>,
    pub metrics: PrimalMetrics,
    pub last_updated: Instant,
    pub api_version: String,
}

/// Metadata from primal API
#[derive(Debug, Clone)]
pub struct PrimalMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub uptime: Duration,
    pub resource_usage: ResourceUsage,
}

/// Service information from primal
#[derive(Debug, Clone)]
pub struct PrimalServiceInfo {
    pub service_id: String,
    pub name: String,
    pub status: ServiceStatus,
    pub replicas: Option<u32>,
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_gb: f64,
    pub network_mbps: f64,
}

/// Service status
#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Running,
    Starting,
    Stopping,
    Failed,
    Scaling,
}

/// Metrics from primal APIs
#[derive(Debug, Clone)]
pub struct PrimalMetrics {
    pub requests_per_second: f64,
    pub average_response_time: Duration,
    pub error_rate: f64,
    pub throughput: f64,
}

/// Overall ecosystem health aggregated from all primals
#[derive(Debug, Clone)]
pub struct EcosystemHealth {
    pub overall_status: Health,
    pub primal_count: usize,
    pub healthy_primals: usize,
    pub total_services: usize,
    pub healthy_services: usize,
    pub active_deployments: usize,
    pub critical_issues: Vec<String>,
}

/// Deployment status for orchestration
#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    pub deployment_id: String,
    pub biome_name: String,
    pub status: DeploymentPhase,
    pub target_environment: String,
    pub progress: u8, // 0-100
    pub started_at: Instant,
    pub estimated_completion: Option<Instant>,
    pub deployed_services: Vec<String>,
    pub failed_services: Vec<String>,
}

/// Deployment phases
#[derive(Debug, Clone)]
pub enum DeploymentPhase {
    Validating,
    Deploying,
    Scaling,
    Configuring,
    HealthChecking,
    Complete,
    Failed { reason: String },
    RollingBack,
}

/// Deployment events for history
#[derive(Debug, Clone)]
pub struct DeploymentEvent {
    pub timestamp: Instant,
    pub deployment_id: String,
    pub event_type: DeploymentEventType,
    pub message: String,
}

/// Types of deployment events
#[derive(Debug, Clone)]
pub enum DeploymentEventType {
    Started,
    ServiceDeployed,
    ServiceFailed,
    Completed,
    Failed,
    RolledBack,
}

/// AI chat message for human/AI interface
#[derive(Debug, Clone)]
pub struct AiChatMessage {
    pub timestamp: Instant,
    pub role: AiRole,
    pub content: String,
    pub context: Option<String>, // Optional context about what system state this relates to
}

/// AI roles in conversation
#[derive(Debug, Clone)]
pub enum AiRole {
    Human,
    Assistant,
    System,
}

/// AI suggestions for operations
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub command: Option<String>,
    pub confidence: f64, // 0.0-1.0
    pub category: AiSuggestionCategory,
    pub can_execute: bool,
}

/// Categories of AI suggestions
#[derive(Debug, Clone)]
pub enum AiSuggestionCategory {
    Scaling,
    Performance,
    Security,
    Deployment,
    Troubleshooting,
    Optimization,
}

/// AI insights from analyzing ecosystem data
#[derive(Debug, Clone)]
pub struct AiInsight {
    pub title: String,
    pub insight: String,
    pub severity: InsightSeverity,
    pub affected_components: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub confidence: f64,
}

/// Severity levels for insights
#[derive(Debug, Clone)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
    Optimization,
}

/// API endpoint status for monitoring ingestion
#[derive(Debug, Clone)]
pub struct ApiEndpointStatus {
    pub endpoint: String,
    pub status: ApiStatus,
    pub last_successful_call: Option<Instant>,
    pub error_count: u32,
    pub average_response_time: Duration,
}

/// API connection status
#[derive(Debug, Clone)]
pub enum ApiStatus {
    Connected,
    Disconnected,
    Error { message: String },
    Timeout,
}

/// API errors for monitoring
#[derive(Debug, Clone)]
pub struct ApiError {
    pub timestamp: Instant,
    pub endpoint: String,
    pub error: String,
    pub retry_count: u32,
}

/// Log entry from streaming
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub source: String,
    pub level: LogLevel,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

/// Log levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log filters for streaming
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub source_pattern: Option<String>,
    pub level_filter: Option<LogLevel>,
    pub message_pattern: Option<String>,
    pub time_range: Option<(Instant, Instant)>,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardState {
    /// Create new comprehensive dashboard state
    pub fn new() -> Self {
        let mut state = Self {
            current_tab: 0,
            selected_service: 0,
            selected_primal: 0,
            selected_deployment: 0,
            service_list_state: ListState::default(),
            primal_list_state: ListState::default(),
            deployment_list_state: ListState::default(),

            primal_states: HashMap::new(),
            discovered_services: Vec::new(),
            active_deployments: Vec::new(),
            ecosystem_health: EcosystemHealth::default(),

            system_health_history: VecDeque::new(),
            primal_health_history: HashMap::new(),
            capability_stats: HashMap::new(),
            deployment_history: VecDeque::new(),

            ai_chat_history: VecDeque::new(),
            ai_suggestions: Vec::new(),
            ai_insights: Vec::new(),
            ai_input_buffer: String::new(),

            api_endpoints: HashMap::new(),
            last_api_sync: HashMap::new(),
            api_errors: VecDeque::new(),

            log_streams: HashMap::new(),
            active_log_filters: Vec::new(),

            update_interval: Duration::from_secs(5),
            max_history_points: 100,
            ai_enabled: true,
            auto_refresh: true,
        };

        // Initialize with first tab selected
        state.service_list_state.select(Some(0));
        state.primal_list_state.select(Some(0));
        state.deployment_list_state.select(Some(0));

        state
    }

    /// Get current tab information
    pub fn current_tab_info(&self) -> TabInfo {
        let tabs = TabId::all_tabs();
        tabs[self.current_tab].clone()
    }

    /// Get currently selected service
    pub fn selected_service(&self) -> Option<&DiscoveryResult> {
        self.discovered_services.get(self.selected_service)
    }

    /// Get currently selected primal
    pub fn selected_primal(&self) -> Option<(&String, &PrimalApiState)> {
        self.primal_states.iter().nth(self.selected_primal)
    }

    /// Get currently selected deployment
    pub fn selected_deployment(&self) -> Option<&DeploymentStatus> {
        self.active_deployments.get(self.selected_deployment)
    }

    /// Update capability statistics from current data
    pub fn update_capability_stats(&mut self) {
        self.capability_stats.clear();

        // Count capabilities from discovered services
        for service in &self.discovered_services {
            for capability in &service.capabilities {
                *self
                    .capability_stats
                    .entry(format!("{:?}", capability))
                    .or_insert(0) += 1;
            }
        }

        // Count capabilities from primal states
        for primal_state in self.primal_states.values() {
            for capability in &primal_state.capabilities {
                *self
                    .capability_stats
                    .entry(format!("{:?}", capability))
                    .or_insert(0) += 1;
            }
        }
    }

    /// Add AI chat message
    pub fn add_ai_message(&mut self, role: AiRole, content: String, context: Option<String>) {
        self.ai_chat_history.push_back(AiChatMessage {
            timestamp: Instant::now(),
            role,
            content,
            context,
        });

        // Keep history manageable
        while self.ai_chat_history.len() > 100 {
            self.ai_chat_history.pop_front();
        }
    }

    /// Update primal state from API data
    pub fn update_primal_state(&mut self, primal_id: String, state: PrimalApiState) {
        // Update health history
        if let Some(history) = self.primal_health_history.get_mut(&primal_id) {
            history.push_back((Instant::now(), state.health.clone()));
            while history.len() > self.max_history_points {
                history.pop_front();
            }
        } else {
            let mut history = VecDeque::new();
            history.push_back((Instant::now(), state.health.clone()));
            self.primal_health_history
                .insert(primal_id.clone(), history);
        }

        // Update primal state
        self.primal_states.insert(primal_id.clone(), state);
        self.last_api_sync.insert(primal_id, Instant::now());

        // Update ecosystem health
        self.update_ecosystem_health();
    }

    /// Update overall ecosystem health
    pub fn update_ecosystem_health(&mut self) {
        let primal_count = self.primal_states.len();
        let healthy_primals = self
            .primal_states
            .values()
            .filter(|p| matches!(p.health, Health::Healthy))
            .count();

        let total_services: usize = self.primal_states.values().map(|p| p.services.len()).sum();

        let healthy_services: usize = self
            .primal_states
            .values()
            .flat_map(|p| &p.services)
            .filter(|s| matches!(s.status, ServiceStatus::Running))
            .count();

        let overall_status = if healthy_primals == primal_count && primal_count > 0 {
            Health::Healthy
        } else if healthy_primals > 0 {
            Health::Degraded {
                issues: vec![],
                impact_score: Some(1.0 - (healthy_primals as f64 / primal_count as f64)),
            }
        } else {
            use biomeos_types::{HealthIssue, HealthIssueCategory, HealthIssueSeverity};
            use chrono::Utc;
            use std::collections::HashMap;
            use uuid::Uuid;

            Health::Critical {
                issues: vec![HealthIssue {
                    id: Uuid::new_v4().to_string(),
                    category: HealthIssueCategory::Software,
                    severity: HealthIssueSeverity::Critical,
                    message: "No healthy primals detected".to_string(),
                    detected_at: Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec!["All".to_string()],
            }
        };

        self.ecosystem_health = EcosystemHealth {
            overall_status,
            primal_count,
            healthy_primals,
            total_services,
            healthy_services,
            active_deployments: self.active_deployments.len(),
            critical_issues: Vec::new(),
        };
    }

    /// Move to next tab
    pub fn next_tab(&mut self) {
        let tabs = TabId::all_tabs();
        self.current_tab = (self.current_tab + 1) % tabs.len();
    }

    /// Move to previous tab
    pub fn previous_tab(&mut self) {
        let tabs = TabId::all_tabs();
        self.current_tab = if self.current_tab == 0 {
            tabs.len() - 1
        } else {
            self.current_tab - 1
        };
    }

    /// Move to next service
    pub fn next_service(&mut self) {
        if !self.discovered_services.is_empty() {
            self.selected_service = (self.selected_service + 1) % self.discovered_services.len();
            self.service_list_state.select(Some(self.selected_service));
        }
    }

    /// Move to previous service
    pub fn previous_service(&mut self) {
        if !self.discovered_services.is_empty() {
            self.selected_service = if self.selected_service == 0 {
                self.discovered_services.len() - 1
            } else {
                self.selected_service - 1
            };
            self.service_list_state.select(Some(self.selected_service));
        }
    }

    /// Add health data point for legacy compatibility
    pub fn add_health_data(&mut self, health: SystemHealth) {
        self.system_health_history
            .push_back((Instant::now(), health));

        // Keep only recent data points
        while self.system_health_history.len() > self.max_history_points {
            self.system_health_history.pop_front();
        }
    }
}

impl Default for EcosystemHealth {
    fn default() -> Self {
        Self {
            overall_status: Health::Unknown {
                reason: "No data available".to_string(),
                last_known: None,
            },
            primal_count: 0,
            healthy_primals: 0,
            total_services: 0,
            healthy_services: 0,
            active_deployments: 0,
            critical_issues: Vec::new(),
        }
    }
}
