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
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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
        let mut update_interval = interval(self.state.update_interval);

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
                DashboardAction::ShowHelp => {
                    // TODO: Implement help modal
                }
                DashboardAction::None => {}
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
        self.cleanup_terminal(&mut terminal)?;
        Ok(())
    }

    /// Refresh all dashboard data
    async fn refresh_data(&mut self) -> Result<()> {
        // Refresh system health
        let system_health = self.manager.get_system_health().await;
        self.state.add_health_data(system_health);

        // Refresh discovered services
        self.state.discovered_services = self
            .manager
            .discover_network_scan()
            .await
            .unwrap_or_else(|_| Vec::new());

        // Update service list state if needed
        if !self.state.discovered_services.is_empty() {
            if self.state.selected_service >= self.state.discovered_services.len() {
                self.state.selected_service = 0;
                self.state.service_list_state.select(Some(0));
            }
        }

        // Update capability statistics
        self.state.update_capability_stats();

        Ok(())
    }

    /// Handle item selection
    async fn handle_selection(&mut self) -> Result<()> {
        if let Some(_selected_service) = self.state.selected_service() {
            // TODO: Implement service selection actions
            // For now, just refresh the selected service's data
        }
        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
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
mod tests {
    use super::*;
    use biomeos_core::config::BiomeOSConfig;

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
}
