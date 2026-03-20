// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Main TUI Dashboard
//!
//! The primary dashboard implementation that coordinates all TUI components.

use crate::tui::events::{DashboardAction, EventHandler};
use crate::tui::types::DashboardState;
use crate::tui::widgets::WidgetRenderer;
use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Instant;
use tokio::time::interval;

/// Main TUI Dashboard for BiomeOS ecosystem monitoring
pub struct BiomeOSDashboard {
    manager: UniversalBiomeOSManager,
    state: DashboardState,
    event_handler: EventHandler,
}

impl BiomeOSDashboard {
    /// Create a new BiomeOS dashboard
    pub fn new(manager: UniversalBiomeOSManager) -> Self {
        Self {
            manager,
            state: DashboardState::new(),
            event_handler: EventHandler::new(),
        }
    }

    /// Run the TUI dashboard
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initial data load
        self.refresh_data().await?;

        let mut last_update = Instant::now();
        let _update_interval = interval(self.state.update_interval);

        // Main event loop
        loop {
            // Handle events
            match self.event_handler.poll_event()? {
                DashboardAction::Quit => break,
                DashboardAction::NextTab => self.state.next_tab(),
                DashboardAction::PreviousTab => self.state.previous_tab(),
                DashboardAction::MoveUp => self.state.previous_service(),
                DashboardAction::MoveDown => self.state.next_service(),
                DashboardAction::Select => self.handle_selection().await?,
                DashboardAction::Refresh => {
                    self.refresh_data().await?;
                    last_update = Instant::now();
                }
                DashboardAction::ShowHelp | DashboardAction::None => {
                    // Help functionality not implemented yet - skip for now
                }
            }

            // Auto-refresh data
            if last_update.elapsed() >= self.state.update_interval {
                self.refresh_data().await?;
                last_update = Instant::now();
            }

            // Draw interface
            terminal.draw(|f| WidgetRenderer::render_dashboard(f, &self.state))?;

            tokio::task::yield_now().await;
        }

        // Cleanup terminal
        Self::cleanup_terminal(&mut terminal)?;
        Ok(())
    }

    /// Refresh all dashboard data
    async fn refresh_data(&mut self) -> Result<()> {
        // Refresh system health
        let system_health = self.manager.get_system_health().await;
        // Convert HealthReport to SystemHealth for TUI
        let cli_health = crate::health::SystemHealth {
            overall_status: system_health.health.clone(),
            cpu_usage: system_health
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.cpu_usage)
                .map_or(0.0, |u| u * 100.0),
            memory_usage: system_health
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.memory_usage)
                .map_or(0.0, |u| u * 100.0),
            disk_usage: system_health
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.disk_usage)
                .map_or(0.0, |u| u * 100.0),
            network_status: "OK".to_string(),
        };
        self.state.add_health_data(cli_health);

        // Refresh discovered services
        // Convert String endpoints to DiscoveryResult for TUI compatibility
        let endpoints = self
            .manager
            .discover_network_scan()
            .await
            .unwrap_or_else(|_| Vec::new());

        // For now, create minimal DiscoveryResult from endpoints
        self.state.discovered_services = endpoints
            .into_iter()
            .map(|endpoint| {
                use biomeos_core::universal_biomeos_manager::DiscoveryResult;
                use biomeos_primal_sdk::{Health, PrimalType};
                use uuid::Uuid;

                DiscoveryResult {
                    id: Uuid::new_v4().to_string(),
                    primal_type: PrimalType::new("unknown", "Unknown Service", "1.0.0"),
                    endpoint,
                    capabilities: vec![],
                    health: Health::Unknown {
                        reason: "Not probed yet".to_string(),
                        last_known: None,
                    },
                    discovered_at: chrono::Utc::now(),
                }
            })
            .collect();

        // Update service list state if needed
        if !self.state.discovered_services.is_empty()
            && self.state.selected_service >= self.state.discovered_services.len()
        {
            self.state.selected_service = 0;
            self.state.service_list_state.select(Some(0));
        }

        // Update capability statistics
        self.state.update_capability_stats();

        Ok(())
    }

    /// Handle item selection
    async fn handle_selection(&self) -> Result<()> {
        if let Some(_selected_service) = self.state.selected_service() {
            // Service selection handling - simplified for now
            tracing::info!("Service selected for action");
        }
        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    /// Get current dashboard state (for testing)
    #[cfg(test)]
    pub fn state(&self) -> &DashboardState {
        &self.state
    }

    /// Set update interval (for testing)
    #[cfg(test)]
    pub fn set_update_interval(&mut self, interval: std::time::Duration) {
        self.state.update_interval = interval;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_types::BiomeOSConfig;

    async fn create_test_dashboard() -> BiomeOSDashboard {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config).await.unwrap();
        BiomeOSDashboard::new(manager)
    }

    #[tokio::test]
    async fn test_dashboard_creation() {
        let dashboard = create_test_dashboard().await;
        assert_eq!(dashboard.state().current_tab, 0);
        assert_eq!(dashboard.state().selected_service, 0);
    }

    #[tokio::test]
    async fn test_tab_navigation() {
        let mut dashboard = create_test_dashboard().await;

        // Test next tab
        dashboard.state.next_tab();
        assert_eq!(dashboard.state().current_tab, 1);

        // Test previous tab
        dashboard.state.previous_tab();
        assert_eq!(dashboard.state().current_tab, 0);
    }

    #[tokio::test]
    async fn test_data_refresh() {
        let mut dashboard = create_test_dashboard().await;
        let result = dashboard.refresh_data().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_navigation_empty_list_no_panic() {
        let mut dashboard = create_test_dashboard().await;
        assert!(dashboard.state().discovered_services.is_empty());
        dashboard.state.next_service();
        dashboard.state.previous_service();
    }

    #[tokio::test]
    async fn test_set_update_interval() {
        let mut dashboard = create_test_dashboard().await;
        dashboard.set_update_interval(std::time::Duration::from_millis(250));
        assert_eq!(dashboard.state().update_interval.as_millis(), 250);
    }

    #[tokio::test]
    async fn test_handle_selection_smoke() {
        let dashboard = create_test_dashboard().await;
        assert!(dashboard.handle_selection().await.is_ok());
    }
}
