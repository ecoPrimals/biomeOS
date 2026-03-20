// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primary dashboard state and behavior.

use super::ai::{AiChatMessage, AiInsight, AiRole, AiSuggestion};
use super::api_logs::{ApiEndpointStatus, ApiError, LogEntry, LogFilter};
use super::tabs::{TabId, TabInfo};
use crate::health::SystemHealth;
use crate::tui::primal_ecosystem::{
    DeploymentEvent, DeploymentStatus, EcosystemHealth, PrimalApiState, ServiceStatus,
};
use biomeos_core::universal_biomeos_manager::DiscoveryResult;
use biomeos_types::Health;
use ratatui::widgets::ListState;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

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
