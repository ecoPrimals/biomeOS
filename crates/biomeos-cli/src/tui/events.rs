// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! TUI Event Handling
//!
//! Handles keyboard input and other events for the TUI dashboard.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

/// Available dashboard actions that can be triggered by user input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardAction {
    /// Quit the application
    Quit,
    /// Switch to next tab
    NextTab,
    /// Switch to previous tab
    PreviousTab,
    /// Move selection up
    MoveUp,
    /// Move selection down
    MoveDown,
    /// Select current item
    Select,
    /// Refresh dashboard data
    Refresh,
    /// Show help modal
    ShowHelp,
    /// No action
    None,
}

/// Event handler for the TUI dashboard
pub struct EventHandler {
    /// Polling timeout for events
    poll_timeout: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        Self {
            poll_timeout: Duration::from_millis(100),
        }
    }

    /// Poll for events and return the corresponding dashboard action
    pub fn poll_event(&self) -> Result<DashboardAction> {
        if event::poll(self.poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                return Ok(Self::handle_key_event(key.code));
            }
        }
        Ok(DashboardAction::None)
    }

    /// Handle keyboard input and map to dashboard actions
    fn handle_key_event(key_code: KeyCode) -> DashboardAction {
        match key_code {
            // Navigation
            KeyCode::Char('q') | KeyCode::Esc => DashboardAction::Quit,
            KeyCode::Tab | KeyCode::Char('l') => DashboardAction::NextTab,
            KeyCode::BackTab | KeyCode::Char('p') => DashboardAction::PreviousTab,
            KeyCode::Up | KeyCode::Char('k') => DashboardAction::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => DashboardAction::MoveDown,
            KeyCode::Enter => DashboardAction::Select,

            // Actions
            KeyCode::Char('r') => DashboardAction::Refresh,
            KeyCode::Char('h') => DashboardAction::ShowHelp,

            _ => DashboardAction::None,
        }
    }

    /// Set polling timeout
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new();
        assert_eq!(handler.poll_timeout, Duration::from_millis(100));
    }

    #[test]
    fn test_key_mappings() {
        // Test quit keys
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('q')),
            DashboardAction::Quit
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Esc),
            DashboardAction::Quit
        );

        // Test navigation
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Tab),
            DashboardAction::NextTab
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::BackTab),
            DashboardAction::PreviousTab
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Up),
            DashboardAction::MoveUp
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Down),
            DashboardAction::MoveDown
        );

        // Test actions
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Enter),
            DashboardAction::Select
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('r')),
            DashboardAction::Refresh
        );
        // F5 key not available in current crossterm version
        // Using 'r' key for refresh instead

        // Test vim-style navigation
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('k')),
            DashboardAction::MoveUp
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('j')),
            DashboardAction::MoveDown
        );
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('l')),
            DashboardAction::NextTab
        );

        // Test unknown key
        assert_eq!(
            EventHandler::handle_key_event(KeyCode::Char('z')),
            DashboardAction::None
        );
    }
}
