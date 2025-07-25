//! TUI Dashboard Types
//!
//! Common types and data structures used throughout the TUI dashboard.

use biomeos_core::universal_biomeos_manager::DiscoveryResult;
use biomeos_core::SystemHealth;
use ratatui::widgets::ListState;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Tab information for the TUI interface
#[derive(Clone)]
pub struct TabInfo {
    pub title: &'static str,
    pub id: TabId,
}

/// Available tabs in the dashboard
#[derive(Clone, Copy, PartialEq)]
pub enum TabId {
    Overview,
    Services,
    Health,
    Discovery,
    Metrics,
}

impl TabId {
    /// Get all available tabs
    pub fn all_tabs() -> Vec<TabInfo> {
        vec![
            TabInfo {
                title: "Overview",
                id: TabId::Overview,
            },
            TabInfo {
                title: "Services",
                id: TabId::Services,
            },
            TabInfo {
                title: "Health",
                id: TabId::Health,
            },
            TabInfo {
                title: "Discovery",
                id: TabId::Discovery,
            },
            TabInfo {
                title: "Metrics",
                id: TabId::Metrics,
            },
        ]
    }
}

/// Dashboard state containing all UI state and data
pub struct DashboardState {
    // Navigation state
    pub current_tab: usize,
    pub selected_service: usize,
    pub service_list_state: ListState,

    // Data storage
    pub discovered_services: Vec<DiscoveryResult>,
    pub system_health_history: VecDeque<(Instant, SystemHealth)>,
    pub capability_stats: HashMap<String, usize>,

    // Settings
    pub update_interval: Duration,
    pub max_history_points: usize,
}

impl DashboardState {
    /// Create new dashboard state with default values
    pub fn new() -> Self {
        let mut service_list_state = ListState::default();
        service_list_state.select(Some(0));

        Self {
            current_tab: 0,
            selected_service: 0,
            service_list_state,
            discovered_services: Vec::new(),
            system_health_history: VecDeque::new(),
            capability_stats: HashMap::new(),
            update_interval: Duration::from_secs(5),
            max_history_points: 60, // 5 minutes at 5-second intervals
        }
    }

    /// Move to next service in the list
    pub fn next_service(&mut self) {
        if !self.discovered_services.is_empty() {
            self.selected_service = (self.selected_service + 1) % self.discovered_services.len();
            self.service_list_state.select(Some(self.selected_service));
        }
    }

    /// Move to previous service in the list
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

    /// Get the currently selected service
    pub fn selected_service(&self) -> Option<&DiscoveryResult> {
        self.discovered_services.get(self.selected_service)
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        let tabs = TabId::all_tabs();
        self.current_tab = (self.current_tab + 1) % tabs.len();
    }

    /// Switch to previous tab
    pub fn previous_tab(&mut self) {
        let tabs = TabId::all_tabs();
        self.current_tab = if self.current_tab == 0 {
            tabs.len() - 1
        } else {
            self.current_tab - 1
        };
    }

    /// Get current tab info
    pub fn current_tab_info(&self) -> TabInfo {
        let tabs = TabId::all_tabs();
        tabs[self.current_tab].clone()
    }

    /// Add health data point to history
    pub fn add_health_data(&mut self, health: SystemHealth) {
        self.system_health_history
            .push_back((Instant::now(), health));

        // Keep only recent data points
        while self.system_health_history.len() > self.max_history_points {
            self.system_health_history.pop_front();
        }
    }

    /// Update capability statistics
    pub fn update_capability_stats(&mut self) {
        self.capability_stats.clear();

        for service in &self.discovered_services {
            for capability in &service.capabilities {
                let cap_name = format!("{:?}", capability);
                *self.capability_stats.entry(cap_name).or_insert(0) += 1;
            }
        }
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}
