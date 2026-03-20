// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! TUI Dashboard Types for Ecosystem Interface
//!
//! Comprehensive types for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.
//! Supports API ingestion from all primals and deployment orchestration.

pub use super::primal_ecosystem::{
    DeploymentEvent, DeploymentEventType, DeploymentPhase, DeploymentStatus, EcosystemHealth,
    PrimalApiState, PrimalMetadata, PrimalMetrics, PrimalServiceInfo, ResourceUsage, ServiceStatus,
};
use crate::health::SystemHealth;
use biomeos_core::universal_biomeos_manager::DiscoveryResult;
use biomeos_types::Health;
use ratatui::widgets::ListState;
// Remove unused serde imports
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Tab information for the comprehensive ecosystem interface
#[derive(Clone)]
pub struct TabInfo {
    /// Tab display title
    pub title: &'static str,
    /// Tab identifier
    pub id: TabId,
    /// Tab icon (emoji)
    pub icon: &'static str,
}

/// Available tabs in the ecosystem dashboard
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    /// Overview of the entire ecosystem
    EcosystemOverview,
    /// Status of individual primals
    PrimalStatus,
    /// Deployment orchestration view
    DeploymentOrchestration,

    /// Running services view
    Services,
    /// Health monitoring view
    Health,

    /// AI assistant chat interface
    AiAssistant,
    /// AI-generated insights view
    AiInsights,

    /// API data ingestion status
    ApiIngestion,
    /// Metrics and performance view
    Metrics,
    /// Log streaming view
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
    /// Index of the currently active tab
    pub current_tab: usize,
    /// Index of the currently selected service
    pub selected_service: usize,
    /// Index of the currently selected primal
    pub selected_primal: usize,
    /// Index of the currently selected deployment
    pub selected_deployment: usize,
    /// List widget state for services
    pub service_list_state: ListState,
    /// List widget state for primals
    pub primal_list_state: ListState,
    /// List widget state for deployments
    pub deployment_list_state: ListState,

    /// State of each primal obtained via API
    pub primal_states: HashMap<String, PrimalApiState>,
    /// Services discovered in the ecosystem
    pub discovered_services: Vec<DiscoveryResult>,
    /// Currently active deployments
    pub active_deployments: Vec<DeploymentStatus>,
    /// Aggregated ecosystem health status
    pub ecosystem_health: EcosystemHealth,

    /// Historical system health data points
    pub system_health_history: VecDeque<(Instant, SystemHealth)>,
    /// Historical health data per primal
    pub primal_health_history: HashMap<String, VecDeque<(Instant, Health)>>,
    /// Capability usage statistics
    pub capability_stats: HashMap<String, usize>,
    /// Historical deployment events
    pub deployment_history: VecDeque<DeploymentEvent>,

    /// AI chat message history
    pub ai_chat_history: VecDeque<AiChatMessage>,
    /// AI-generated suggestions
    pub ai_suggestions: Vec<AiSuggestion>,
    /// AI-generated insights
    pub ai_insights: Vec<AiInsight>,
    /// Current AI input text buffer
    pub ai_input_buffer: String,

    /// API endpoint connection status
    pub api_endpoints: HashMap<String, ApiEndpointStatus>,
    /// Timestamp of last successful API sync per endpoint
    pub last_api_sync: HashMap<String, Instant>,
    /// Recent API errors
    pub api_errors: VecDeque<ApiError>,

    /// Log streams per source
    pub log_streams: HashMap<String, VecDeque<LogEntry>>,
    /// Active log filters
    pub active_log_filters: Vec<LogFilter>,

    /// Dashboard data refresh interval
    pub update_interval: Duration,
    /// Maximum number of history data points to retain
    pub max_history_points: usize,
    /// Whether AI features are enabled
    pub ai_enabled: bool,
    /// Whether automatic data refresh is enabled
    pub auto_refresh: bool,
}

/// AI chat message for human/AI interface
#[derive(Debug, Clone)]
pub struct AiChatMessage {
    /// When the message was sent
    pub timestamp: Instant,
    /// Who sent the message
    pub role: AiRole,
    /// Message content
    pub content: String,
    /// Optional context about what system state this relates to
    pub context: Option<String>,
}

/// AI roles in conversation
#[derive(Debug, Clone)]
pub enum AiRole {
    /// Human user
    Human,
    /// AI assistant
    Assistant,
    /// System-generated message
    System,
}

/// AI suggestions for operations
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    /// Unique suggestion identifier
    pub id: String,
    /// Short title for the suggestion
    pub title: String,
    /// Detailed description of the suggestion
    pub description: String,
    /// CLI command to execute (if applicable)
    pub command: Option<String>,
    /// Confidence score (0.0–1.0)
    pub confidence: f64,
    /// Category of the suggestion
    pub category: AiSuggestionCategory,
    /// Whether the suggestion can be executed automatically
    pub can_execute: bool,
}

/// Categories of AI suggestions
#[derive(Debug, Clone)]
pub enum AiSuggestionCategory {
    /// Scaling-related suggestion
    Scaling,
    /// Performance optimization suggestion
    Performance,
    /// Security-related suggestion
    Security,
    /// Deployment-related suggestion
    Deployment,
    /// Troubleshooting suggestion
    Troubleshooting,
    /// Resource optimization suggestion
    Optimization,
}

/// AI insights from analyzing ecosystem data
#[derive(Debug, Clone)]
pub struct AiInsight {
    /// Insight title
    pub title: String,
    /// Detailed insight description
    pub insight: String,
    /// Severity level of the insight
    pub severity: InsightSeverity,
    /// Components affected by this insight
    pub affected_components: Vec<String>,
    /// Recommended actions to address the insight
    pub recommended_actions: Vec<String>,
    /// Confidence in the insight accuracy (0.0–1.0)
    pub confidence: f64,
}

/// Severity levels for insights
#[derive(Debug, Clone)]
pub enum InsightSeverity {
    /// Informational insight
    Info,
    /// Warning-level insight
    Warning,
    /// Critical issue requiring immediate attention
    Critical,
    /// Optimization opportunity
    Optimization,
}

/// API endpoint status for monitoring ingestion
#[derive(Debug, Clone)]
pub struct ApiEndpointStatus {
    /// Endpoint URL
    pub endpoint: String,
    /// Current connection status
    pub status: ApiStatus,
    /// Timestamp of last successful API call
    pub last_successful_call: Option<Instant>,
    /// Number of consecutive errors
    pub error_count: u32,
    /// Average response time across recent calls
    pub average_response_time: Duration,
}

/// API connection status
#[derive(Debug, Clone)]
pub enum ApiStatus {
    /// Successfully connected
    Connected,
    /// Not connected
    Disconnected,
    /// Connection error
    Error {
        /// Error description
        message: String,
    },
    /// Connection timed out
    Timeout,
}

/// API errors for monitoring
#[derive(Debug, Clone)]
pub struct ApiError {
    /// When the error occurred
    pub timestamp: Instant,
    /// Endpoint that produced the error
    pub endpoint: String,
    /// Error description
    pub error: String,
    /// Number of retry attempts made
    pub retry_count: u32,
}

/// Log entry from streaming
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// When the log entry was created
    pub timestamp: Instant,
    /// Source component that produced the log
    pub source: String,
    /// Log severity level
    pub level: LogLevel,
    /// Log message content
    pub message: String,
    /// Additional structured metadata
    pub metadata: HashMap<String, String>,
}

/// Log levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    /// Trace-level detail
    Trace,
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warn,
    /// Error message
    Error,
}

/// Log filters for streaming
#[derive(Debug, Clone)]
pub struct LogFilter {
    /// Filter by source pattern (glob)
    pub source_pattern: Option<String>,
    /// Filter by minimum log level
    pub level_filter: Option<LogLevel>,
    /// Filter by message content pattern
    pub message_pattern: Option<String>,
    /// Filter by time range
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
                    .entry(format!("{capability:?}"))
                    .or_insert(0) += 1;
            }
        }

        // Count capabilities from primal states
        for primal_state in self.primal_states.values() {
            for capability in &primal_state.capabilities {
                *self
                    .capability_stats
                    .entry(format!("{capability:?}"))
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_core::universal_biomeos_manager::DiscoveryResult;
    use biomeos_types::{Health, PrimalCapability, PrimalType};
    use std::time::Duration;

    #[test]
    fn test_tab_id_all_tabs() {
        let tabs = TabId::all_tabs();
        assert!(!tabs.is_empty());
        assert_eq!(tabs.len(), 10);
        assert!(tabs.iter().any(|t| t.id == TabId::EcosystemOverview));
        assert!(tabs.iter().any(|t| t.id == TabId::PrimalStatus));
        assert!(tabs.iter().any(|t| t.id == TabId::Logs));
    }

    #[test]
    fn test_tab_info_clone() {
        let tabs = TabId::all_tabs();
        let first = &tabs[0];
        assert!(!first.title.is_empty());
        assert!(!first.icon.is_empty());
    }

    #[test]
    fn test_dashboard_state_new() {
        let state = DashboardState::new();
        assert_eq!(state.current_tab, 0);
        assert_eq!(state.selected_service, 0);
        assert_eq!(state.selected_primal, 0);
        assert!(state.primal_states.is_empty());
        assert!(state.discovered_services.is_empty());
        assert!(state.ai_chat_history.is_empty());
    }

    #[test]
    fn test_dashboard_state_default() {
        let state = DashboardState::default();
        assert_eq!(state.current_tab, 0);
    }

    #[test]
    fn test_dashboard_state_current_tab_info() {
        let state = DashboardState::new();
        let tab = state.current_tab_info();
        assert!(!tab.title.is_empty());
    }

    #[test]
    fn test_dashboard_state_next_tab() {
        let mut state = DashboardState::new();
        let tabs = TabId::all_tabs();
        state.next_tab();
        assert_eq!(state.current_tab, 1);
        state.current_tab = tabs.len() - 1;
        state.next_tab();
        assert_eq!(state.current_tab, 0);
    }

    #[test]
    fn test_dashboard_state_previous_tab() {
        let mut state = DashboardState::new();
        state.previous_tab();
        assert_eq!(state.current_tab, TabId::all_tabs().len() - 1);
        state.current_tab = 1;
        state.previous_tab();
        assert_eq!(state.current_tab, 0);
    }

    #[test]
    fn test_dashboard_state_add_ai_message() {
        let mut state = DashboardState::new();
        state.add_ai_message(AiRole::Human, "hello".to_string(), None);
        state.add_ai_message(AiRole::Assistant, "hi".to_string(), None);
        assert_eq!(state.ai_chat_history.len(), 2);
    }

    #[test]
    fn test_dashboard_state_update_capability_stats() {
        let mut state = DashboardState::new();
        state.discovered_services.push(DiscoveryResult {
            id: "s1".into(),
            endpoint: "http://a".into(),
            primal_type: PrimalType::new("orchestration", "tower", "1.0.0"),
            capabilities: vec![PrimalCapability::new("storage", "file", "1.0")],
            health: Health::Healthy,
            discovered_at: chrono::Utc::now(),
        });
        state.update_capability_stats();
        assert!(!state.capability_stats.is_empty());
    }

    #[test]
    fn test_ecosystem_health_default() {
        let h = EcosystemHealth::default();
        assert_eq!(h.primal_count, 0);
        assert_eq!(h.healthy_primals, 0);
    }

    #[test]
    fn test_service_status_variants() {
        let _ = format!("{:?}", ServiceStatus::Running);
        let _ = format!("{:?}", ServiceStatus::Starting);
        let _ = format!("{:?}", ServiceStatus::Stopping);
        let _ = format!("{:?}", ServiceStatus::Failed);
        let _ = format!("{:?}", ServiceStatus::Scaling);
    }

    #[test]
    fn test_deployment_phase_variants() {
        let _ = format!("{:?}", DeploymentPhase::Validating);
        let _ = format!("{:?}", DeploymentPhase::Complete);
        let _ = format!(
            "{:?}",
            DeploymentPhase::Failed {
                reason: "err".into(),
            }
        );
    }

    #[test]
    fn test_ai_role_variants() {
        let _ = format!("{:?}", AiRole::Human);
        let _ = format!("{:?}", AiRole::Assistant);
        let _ = format!("{:?}", AiRole::System);
    }

    #[test]
    fn test_ai_suggestion_category_variants() {
        let _ = format!("{:?}", AiSuggestionCategory::Scaling);
        let _ = format!("{:?}", AiSuggestionCategory::Security);
    }

    #[test]
    fn test_insight_severity_variants() {
        let _ = format!("{:?}", InsightSeverity::Info);
        let _ = format!("{:?}", InsightSeverity::Critical);
    }

    #[test]
    fn test_api_status_variants() {
        let _ = format!("{:?}", ApiStatus::Connected);
        let _ = format!("{:?}", ApiStatus::Disconnected);
        let _ = format!(
            "{:?}",
            ApiStatus::Error {
                message: "err".into(),
            }
        );
        let _ = format!("{:?}", ApiStatus::Timeout);
    }

    #[test]
    fn test_log_level_variants() {
        let _ = format!("{:?}", LogLevel::Trace);
        let _ = format!("{:?}", LogLevel::Error);
    }

    #[test]
    fn test_primal_metadata_debug() {
        let m = PrimalMetadata {
            name: "test".into(),
            version: "1.0".into(),
            description: "desc".into(),
            uptime: Duration::from_secs(100),
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 256.0,
                disk_gb: 10.0,
                network_mbps: 1.0,
            },
        };
        let _ = format!("{m:?}");
    }

    #[test]
    fn test_log_filter_debug() {
        let f = LogFilter {
            source_pattern: Some("*.log".into()),
            level_filter: None,
            message_pattern: None,
            time_range: None,
        };
        let _ = format!("{f:?}");
    }

    #[test]
    fn test_dashboard_state_next_previous_service_with_items() {
        use biomeos_core::universal_biomeos_manager::DiscoveryResult;
        use biomeos_types::{Health, PrimalType};

        let mut state = DashboardState::new();
        state.discovered_services.push(DiscoveryResult {
            id: "a".into(),
            endpoint: "unix:///a".into(),
            primal_type: PrimalType::new("t", "n", "1.0"),
            capabilities: vec![],
            health: Health::Healthy,
            discovered_at: chrono::Utc::now(),
        });
        state.discovered_services.push(DiscoveryResult {
            id: "b".into(),
            endpoint: "unix:///b".into(),
            primal_type: PrimalType::new("t", "n", "1.0"),
            capabilities: vec![],
            health: Health::Healthy,
            discovered_at: chrono::Utc::now(),
        });
        state.next_service();
        assert_eq!(state.selected_service, 1);
        state.previous_service();
        assert_eq!(state.selected_service, 0);
    }

    #[test]
    fn test_dashboard_state_selected_primal_and_deployment() {
        use biomeos_types::{Health, PrimalType};
        use std::time::{Duration, Instant};

        let mut state = DashboardState::new();
        assert!(state.selected_primal().is_none());
        assert!(state.selected_deployment().is_none());

        state.primal_states.insert(
            "p1".into(),
            PrimalApiState {
                primal_id: "p1".into(),
                primal_type: PrimalType::new("a", "b", "1"),
                endpoint: String::new(),
                health: Health::Healthy,
                capabilities: vec![],
                metadata: PrimalMetadata {
                    name: "n".into(),
                    version: "1".into(),
                    description: String::new(),
                    uptime: Duration::ZERO,
                    resource_usage: ResourceUsage {
                        cpu_percent: 0.,
                        memory_mb: 0.,
                        disk_gb: 0.,
                        network_mbps: 0.,
                    },
                },
                services: vec![],
                metrics: PrimalMetrics {
                    requests_per_second: 0.,
                    average_response_time: Duration::ZERO,
                    error_rate: 0.,
                    throughput: 0.,
                },
                last_updated: Instant::now(),
                api_version: "1".into(),
            },
        );
        state.active_deployments.push(DeploymentStatus {
            deployment_id: "d1".into(),
            biome_name: "b".into(),
            status: DeploymentPhase::Complete,
            target_environment: "local".into(),
            progress: 100,
            started_at: Instant::now(),
            estimated_completion: None,
            deployed_services: vec![],
            failed_services: vec![],
        });
        assert!(state.selected_primal().is_some());
        assert!(state.selected_deployment().is_some());
    }

    #[test]
    fn test_update_ecosystem_health_degraded_mixed() {
        use biomeos_types::{Health, PrimalType};
        use std::time::{Duration, Instant};

        let mut state = DashboardState::new();
        assert!(matches!(
            state.ecosystem_health.overall_status,
            Health::Unknown { .. }
        ));

        state.primal_states.insert(
            "good".into(),
            PrimalApiState {
                primal_id: "good".into(),
                primal_type: PrimalType::new("a", "b", "1"),
                endpoint: String::new(),
                health: Health::Healthy,
                capabilities: vec![],
                metadata: PrimalMetadata {
                    name: "n".into(),
                    version: "1".into(),
                    description: String::new(),
                    uptime: Duration::ZERO,
                    resource_usage: ResourceUsage {
                        cpu_percent: 0.,
                        memory_mb: 0.,
                        disk_gb: 0.,
                        network_mbps: 0.,
                    },
                },
                services: vec![],
                metrics: PrimalMetrics {
                    requests_per_second: 0.,
                    average_response_time: Duration::ZERO,
                    error_rate: 0.,
                    throughput: 0.,
                },
                last_updated: Instant::now(),
                api_version: "1".into(),
            },
        );
        state.primal_states.insert(
            "bad".into(),
            PrimalApiState {
                primal_id: "bad".into(),
                primal_type: PrimalType::new("a", "b", "1"),
                endpoint: String::new(),
                health: Health::Healthy,
                capabilities: vec![],
                metadata: PrimalMetadata {
                    name: "n".into(),
                    version: "1".into(),
                    description: String::new(),
                    uptime: Duration::ZERO,
                    resource_usage: ResourceUsage {
                        cpu_percent: 0.,
                        memory_mb: 0.,
                        disk_gb: 0.,
                        network_mbps: 0.,
                    },
                },
                services: vec![],
                metrics: PrimalMetrics {
                    requests_per_second: 0.,
                    average_response_time: Duration::ZERO,
                    error_rate: 0.,
                    throughput: 0.,
                },
                last_updated: Instant::now(),
                api_version: "1".into(),
            },
        );
        state.primal_states.get_mut("bad").unwrap().health = Health::Degraded {
            issues: vec![],
            impact_score: None,
        };
        state.update_ecosystem_health();
        assert!(matches!(
            state.ecosystem_health.overall_status,
            Health::Degraded { .. }
        ));
    }

    #[test]
    fn test_add_health_data_trims_history() {
        use biomeos_types::Health;

        let mut state = DashboardState::new();
        state.max_history_points = 3;
        for i in 0..10 {
            state.add_health_data(crate::health::SystemHealth {
                overall_status: Health::Healthy,
                cpu_usage: i as f64,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_status: "ok".into(),
            });
        }
        assert_eq!(state.system_health_history.len(), 3);
    }

    #[test]
    fn test_ai_chat_history_cap_trims() {
        let mut state = DashboardState::new();
        for i in 0..120 {
            state.add_ai_message(AiRole::Human, format!("m{i}"), None);
        }
        assert!(state.ai_chat_history.len() <= 100);
    }

    #[test]
    fn test_update_primal_state_inserts_history() {
        use biomeos_types::{Health, PrimalType};
        use std::time::{Duration, Instant};

        let mut state = DashboardState::new();
        let ps = PrimalApiState {
            primal_id: "x".into(),
            primal_type: PrimalType::new("a", "b", "1"),
            endpoint: String::new(),
            health: Health::Healthy,
            capabilities: vec![],
            metadata: PrimalMetadata {
                name: "n".into(),
                version: "1".into(),
                description: String::new(),
                uptime: Duration::ZERO,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.,
                    memory_mb: 0.,
                    disk_gb: 0.,
                    network_mbps: 0.,
                },
            },
            services: vec![],
            metrics: PrimalMetrics {
                requests_per_second: 0.,
                average_response_time: Duration::ZERO,
                error_rate: 0.,
                throughput: 0.,
            },
            last_updated: Instant::now(),
            api_version: "1".into(),
        };
        state.update_primal_state("x".into(), ps);
        assert!(state.primal_states.contains_key("x"));
        assert!(state.primal_health_history.contains_key("x"));
    }
}
