// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! TUI Widgets for Comprehensive Ecosystem Interface
//!
//! Advanced widgets for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.
//!
//! Pure computation/formatting is separated from rendering for testability.

use crate::tui::types::{
    AiRole, ApiStatus, DashboardState, DeploymentEventType, InsightSeverity, LogLevel, TabId,
};
use biomeos_types::Health;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};
use std::collections::HashMap;

// =============================================================================
// Pure computation/formatting functions (no Frame, Rect, or rendering types)
// =============================================================================

/// Maps health strings to ratatui colors.
pub(crate) fn health_to_color(health: &str) -> Color {
    match health {
        "Healthy" => Color::Green,
        "Degraded" => Color::Yellow,
        "Critical" => Color::Red,
        _ => Color::Gray,
    }
}

/// Converts Health enum to category string for health_to_color.
pub(crate) fn health_to_category_str(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "Healthy",
        Health::Degraded { .. } => "Degraded",
        Health::Critical { .. } => "Critical",
        _ => "Unknown",
    }
}

/// Maps event type strings to display icons.
pub(crate) fn event_type_to_icon(event_type: &str) -> &'static str {
    match event_type {
        "Started" => "🚀",
        "ServiceDeployed" => "✅",
        "ServiceFailed" => "❌",
        "Completed" => "🎉",
        "Failed" => "💥",
        "RolledBack" => "🔄",
        _ => "•",
    }
}

/// Maps chat role strings to display prefixes.
pub(crate) fn role_to_prefix(role: &str) -> &'static str {
    match role {
        "Human" => "👤",
        "Assistant" => "🤖",
        "System" => "⚙️",
        _ => "•",
    }
}

/// Maps severity strings to display icons.
pub(crate) fn severity_to_icon(severity: &str) -> &'static str {
    match severity {
        "Info" => "ℹ️",
        "Warning" => "⚠️",
        "Critical" => "🔥",
        "Optimization" => "⚡",
        _ => "•",
    }
}

/// Maps API status strings to display icons.
pub(crate) fn api_status_to_icon(status: &str) -> &'static str {
    match status {
        "Connected" => "🟢",
        "Disconnected" => "🔴",
        "Error" => "🟠",
        "Timeout" => "🟡",
        _ => "⚪",
    }
}

/// Maps log level strings to display icons.
pub(crate) fn log_level_to_icon(level: &str) -> &'static str {
    match level {
        "Trace" => "🔍",
        "Debug" => "🐛",
        "Info" => "ℹ️",
        "Warn" => "⚠️",
        "Error" => "❌",
        _ => "•",
    }
}

/// Computes scroll offset for a list of messages given area height.
pub(crate) fn compute_scroll_offset(messages_len: usize, area_height: u16) -> usize {
    messages_len.saturating_sub(area_height as usize - 2)
}

/// Returns top N capabilities by count, sorted descending.
pub(crate) fn compute_top_capabilities(
    stats: &HashMap<String, usize>,
    n: usize,
) -> Vec<(String, usize)> {
    let mut sorted: Vec<_> = stats.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    sorted
        .into_iter()
        .take(n)
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

/// Formats resource usage lines: (name, cpu%, mem_mb, disk_gb).
pub(crate) fn format_resource_usage_lines(
    primal_states: &[(String, &crate::tui::types::PrimalApiState)],
) -> Vec<(String, f64, f64, f64)> {
    primal_states
        .iter()
        .map(|(id, state)| {
            (
                id.clone(),
                state.metadata.resource_usage.cpu_percent,
                state.metadata.resource_usage.memory_mb,
                state.metadata.resource_usage.disk_gb,
            )
        })
        .collect()
}

/// Formats performance lines: (name, error_rate, response_ms, rps).
pub(crate) fn format_performance_lines(
    primal_states: &[(String, &crate::tui::types::PrimalApiState)],
) -> Vec<(String, f64, f64, f64)> {
    primal_states
        .iter()
        .map(|(id, state)| {
            (
                id.clone(),
                state.metrics.error_rate * 100.0,
                state.metrics.average_response_time.as_secs_f64() * 1000.0,
                state.metrics.requests_per_second,
            )
        })
        .collect()
}

/// Formats the ecosystem header text.
pub(crate) fn format_ecosystem_header_text(state: &DashboardState) -> String {
    format!(
        "🌍 BiomeOS Ecosystem | {} Primals ({} healthy) | {} Services ({} healthy) | {} Deployments",
        state.ecosystem_health.primal_count,
        state.ecosystem_health.healthy_primals,
        state.ecosystem_health.total_services,
        state.ecosystem_health.healthy_services,
        state.ecosystem_health.active_deployments,
    )
}

/// Formats primal list items: (display_text, icon, color).
pub(crate) fn format_primal_list_items_data(
    states: &[(String, &crate::tui::types::PrimalApiState)],
) -> Vec<(String, &'static str, Color)> {
    states
        .iter()
        .map(|(id, state)| {
            let icon = health_status_icon(&state.health);
            let color = health_to_color(health_to_category_str(&state.health));
            let display_text = format!("{} ({})", state.metadata.name, id);
            (display_text, icon, color)
        })
        .collect()
}

/// Generates a visual confidence bar string.
pub(crate) fn confidence_bar(confidence: f64) -> String {
    "=".repeat((confidence * 10.0).clamp(0.0, 10.0) as usize)
}

/// Converts DeploymentEventType to string for event_type_to_icon.
pub(crate) fn deployment_event_type_to_str(et: &DeploymentEventType) -> &'static str {
    match et {
        DeploymentEventType::Started => "Started",
        DeploymentEventType::ServiceDeployed => "ServiceDeployed",
        DeploymentEventType::ServiceFailed => "ServiceFailed",
        DeploymentEventType::Completed => "Completed",
        DeploymentEventType::Failed => "Failed",
        DeploymentEventType::RolledBack => "RolledBack",
    }
}

/// Converts AiRole to string for role_to_prefix.
pub(crate) fn role_to_str(role: &AiRole) -> &'static str {
    match role {
        AiRole::Human => "Human",
        AiRole::Assistant => "Assistant",
        AiRole::System => "System",
    }
}

/// Converts InsightSeverity to string for severity_to_icon.
pub(crate) fn severity_to_str(severity: &InsightSeverity) -> &'static str {
    match severity {
        InsightSeverity::Info => "Info",
        InsightSeverity::Warning => "Warning",
        InsightSeverity::Critical => "Critical",
        InsightSeverity::Optimization => "Optimization",
    }
}

/// Converts ApiStatus to string for api_status_to_icon.
pub(crate) fn api_status_to_str(status: &ApiStatus) -> &'static str {
    match status {
        ApiStatus::Connected => "Connected",
        ApiStatus::Disconnected => "Disconnected",
        ApiStatus::Error { .. } => "Error",
        ApiStatus::Timeout => "Timeout",
    }
}

/// Converts LogLevel to string for log_level_to_icon.
pub(crate) fn log_level_to_str(level: &LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "Trace",
        LogLevel::Debug => "Debug",
        LogLevel::Info => "Info",
        LogLevel::Warn => "Warn",
        LogLevel::Error => "Error",
    }
}

// =============================================================================
// Icon helpers (existing, used by pure functions)
// =============================================================================

/// Get health status icon for display (used by multiple widgets)
pub fn health_status_icon(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "💚",
        Health::Degraded { .. } => "💛",
        Health::Critical { .. } => "❤️",
        _ => "⚪",
    }
}

/// Get service status icon for display
pub fn service_status_icon(status: &crate::tui::types::ServiceStatus) -> &'static str {
    match status {
        crate::tui::types::ServiceStatus::Running => "▶️",
        crate::tui::types::ServiceStatus::Starting => "⏳",
        crate::tui::types::ServiceStatus::Stopping => "⏹️",
        crate::tui::types::ServiceStatus::Failed => "❌",
        crate::tui::types::ServiceStatus::Scaling => "⚖️",
    }
}

/// Get deployment phase icon for display
pub fn deployment_phase_icon(phase: &crate::tui::types::DeploymentPhase) -> &'static str {
    match phase {
        crate::tui::types::DeploymentPhase::Validating => "🔍",
        crate::tui::types::DeploymentPhase::Deploying => "🚀",
        crate::tui::types::DeploymentPhase::Scaling => "⚖️",
        crate::tui::types::DeploymentPhase::Configuring => "⚙️",
        crate::tui::types::DeploymentPhase::HealthChecking => "💊",
        crate::tui::types::DeploymentPhase::Complete => "✅",
        crate::tui::types::DeploymentPhase::Failed { .. } => "❌",
        crate::tui::types::DeploymentPhase::RollingBack => "🔄",
    }
}

/// Advanced widget renderer for ecosystem interface
pub struct WidgetRenderer;

impl WidgetRenderer {
    /// Render the comprehensive ecosystem dashboard
    pub fn render_dashboard(f: &mut Frame, state: &DashboardState) {
        let size = f.size();

        // Create main layout with header
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with ecosystem status
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
            .split(size);

        // Render ecosystem header
        Self::render_ecosystem_header(f, chunks[0], state);

        // Render enhanced tabs with icons
        Self::render_enhanced_tabs(f, chunks[1], state);

        // Render content based on current tab
        let current_tab = state.current_tab_info();
        match current_tab.id {
            TabId::EcosystemOverview => Self::render_ecosystem_overview(f, chunks[2], state),
            TabId::PrimalStatus => Self::render_primal_status(f, chunks[2], state),
            TabId::DeploymentOrchestration => {
                Self::render_deployment_orchestration(f, chunks[2], state)
            }
            TabId::Services => Self::render_services(f, chunks[2], state),
            TabId::Health => Self::render_health(f, chunks[2], state),
            TabId::AiAssistant => Self::render_ai_assistant(f, chunks[2], state),
            TabId::AiInsights => Self::render_ai_insights(f, chunks[2], state),
            TabId::ApiIngestion => Self::render_api_ingestion(f, chunks[2], state),
            TabId::Metrics => Self::render_metrics(f, chunks[2], state),
            TabId::Logs => Self::render_logs(f, chunks[2], state),
        }

        // Render enhanced status bar
        Self::render_enhanced_status_bar(f, chunks[3], state);
    }

    /// Render ecosystem health header
    fn render_ecosystem_header(f: &mut Frame, area: Rect, state: &DashboardState) {
        let health_color = health_to_color(health_to_category_str(
            &state.ecosystem_health.overall_status,
        ));
        let header_text = format_ecosystem_header_text(state);

        let header = Paragraph::new(header_text)
            .style(Style::default().fg(health_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Ecosystem Status"),
            )
            .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    /// Render enhanced tabs with icons
    fn render_enhanced_tabs(f: &mut Frame, area: Rect, state: &DashboardState) {
        let tabs = TabId::all_tabs();
        let tab_titles: Vec<Line> = tabs
            .iter()
            .map(|tab| Line::from(format!("{} {}", tab.icon, tab.title)))
            .collect();

        let tabs_widget = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("Dashboard"))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .select(state.current_tab);

        f.render_widget(tabs_widget, area);
    }

    /// Render ecosystem overview tab
    fn render_ecosystem_overview(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left side: Ecosystem map
        Self::render_ecosystem_map(f, chunks[0], state);

        // Right side: Key metrics and recent events
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        Self::render_key_metrics(f, right_chunks[0], state);
        Self::render_recent_events(f, right_chunks[1], state);
    }

    /// Render primal status tab
    fn render_primal_status(f: &mut Frame, area: Rect, state: &DashboardState) {
        if state.primal_states.is_empty() {
            let empty_msg = Paragraph::new("🔄 Discovering primals via API ingestion...\n\nConnect primals to see their status here.")
                .block(Block::default().borders(Borders::ALL).title("🎯 Primal Status"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(empty_msg, area);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // Left: Primal list
        let primal_data: Vec<_> = state
            .primal_states
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        let primal_items: Vec<ListItem> = format_primal_list_items_data(&primal_data)
            .into_iter()
            .map(|(display_text, icon, _color)| ListItem::new(format!("{} {}", icon, display_text)))
            .collect();

        let primal_list = List::new(primal_items)
            .block(Block::default().borders(Borders::ALL).title("🎯 Primals"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(primal_list, chunks[0], &mut state.primal_list_state.clone());

        // Right: Selected primal details
        if let Some((id, primal_state)) = state.selected_primal() {
            Self::render_primal_details(f, chunks[1], id, primal_state);
        }
    }

    /// Render deployment orchestration tab
    fn render_deployment_orchestration(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Deployment controls
                Constraint::Min(0),    // Active deployments
            ])
            .split(area);

        // Top: Deployment controls
        Self::render_deployment_controls(f, chunks[0], state);

        // Bottom: Active deployments
        Self::render_active_deployments(f, chunks[1], state);
    }

    /// Render AI assistant tab
    fn render_ai_assistant(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Chat history
                Constraint::Length(5), // Suggestions
                Constraint::Length(3), // Input
            ])
            .split(area);

        // Chat history
        Self::render_ai_chat(f, chunks[0], state);

        // AI suggestions
        Self::render_ai_suggestions(f, chunks[1], state);

        // Input area
        Self::render_ai_input(f, chunks[2], state);
    }

    /// Render AI insights tab
    fn render_ai_insights(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left: Active insights
        Self::render_active_insights(f, chunks[0], state);

        // Right: Insight analysis
        Self::render_insight_analysis(f, chunks[1], state);
    }

    /// Render API ingestion status tab
    fn render_api_ingestion(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // API endpoints status
                Constraint::Percentage(30), // Sync status
                Constraint::Percentage(30), // Recent errors
            ])
            .split(area);

        Self::render_api_endpoints(f, chunks[0], state);
        Self::render_sync_status(f, chunks[1], state);
        Self::render_api_errors(f, chunks[2], state);
    }

    /// Render logs tab
    fn render_logs(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Log filters
                Constraint::Min(0),    // Log content
            ])
            .split(area);

        Self::render_log_filters(f, chunks[0], state);
        Self::render_log_content(f, chunks[1], state);
    }

    /// Render ecosystem map visualization
    fn render_ecosystem_map(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut map_text = vec![Line::from("🌍 BiomeOS Ecosystem Map"), Line::from("")];

        // Add primal representations
        for (id, primal_state) in &state.primal_states {
            let health_icon = health_status_icon(&primal_state.health);

            map_text.push(Line::from(vec![
                Span::raw(format!("  {} {} ", health_icon, primal_state.metadata.name)),
                Span::styled(format!("({})", id), Style::default().fg(Color::Gray)),
            ]));

            // Show services under each primal
            for service in &primal_state.services {
                let service_icon = service_status_icon(&service.status);
                map_text.push(Line::from(format!("    {} {}", service_icon, service.name)));
            }
            map_text.push(Line::from(""));
        }

        if state.primal_states.is_empty() {
            map_text.push(Line::from("🔄 Waiting for primal API connections..."));
        }

        let map = Paragraph::new(map_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🗺️ Ecosystem Map"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(map, area);
    }

    /// Render key metrics panel
    fn render_key_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let metrics_text = vec![
            Line::from("📊 Key Metrics"),
            Line::from(""),
            Line::from(format!(
                "Primals: {}/{} healthy",
                state.ecosystem_health.healthy_primals, state.ecosystem_health.primal_count
            )),
            Line::from(format!(
                "Services: {}/{} running",
                state.ecosystem_health.healthy_services, state.ecosystem_health.total_services
            )),
            Line::from(format!(
                "Deployments: {} active",
                state.ecosystem_health.active_deployments
            )),
            Line::from(""),
            Line::from("🎯 Top Capabilities:"),
        ];

        let mut all_text = metrics_text;

        // Add top capabilities
        for (cap, count) in compute_top_capabilities(&state.capability_stats, 5) {
            all_text.push(Line::from(format!("  • {}: {}", cap, count)));
        }

        let metrics = Paragraph::new(all_text)
            .block(Block::default().borders(Borders::ALL).title("📈 Metrics"))
            .wrap(Wrap { trim: true });

        f.render_widget(metrics, area);
    }

    /// Render recent events
    fn render_recent_events(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut events = vec![Line::from("📋 Recent Events"), Line::from("")];

        // Add recent deployment events
        for event in state.deployment_history.iter().rev().take(5) {
            let icon = event_type_to_icon(deployment_event_type_to_str(&event.event_type));
            events.push(Line::from(format!("{} {}", icon, event.message)));
        }

        if state.deployment_history.is_empty() {
            events.push(Line::from("No recent events"));
        }

        let events_widget = Paragraph::new(events)
            .block(Block::default().borders(Borders::ALL).title("📰 Events"))
            .wrap(Wrap { trim: true });

        f.render_widget(events_widget, area);
    }

    /// Render primal details
    fn render_primal_details(
        f: &mut Frame,
        area: Rect,
        id: &str,
        primal_state: &crate::tui::types::PrimalApiState,
    ) {
        let details = vec![
            Line::from(format!("🎯 Primal: {}", primal_state.metadata.name)),
            Line::from(""),
            Line::from(format!("ID: {}", id)),
            Line::from(format!("Version: {}", primal_state.metadata.version)),
            Line::from(format!("Endpoint: {}", primal_state.endpoint)),
            Line::from(format!("Health: {:?}", primal_state.health)),
            Line::from(""),
            Line::from("💻 Resource Usage:"),
            Line::from(format!(
                "  CPU: {:.1}%",
                primal_state.metadata.resource_usage.cpu_percent
            )),
            Line::from(format!(
                "  Memory: {:.1} MB",
                primal_state.metadata.resource_usage.memory_mb
            )),
            Line::from(format!(
                "  Disk: {:.1} GB",
                primal_state.metadata.resource_usage.disk_gb
            )),
            Line::from(""),
            Line::from(format!("📊 Services: {}", primal_state.services.len())),
            Line::from(format!(
                "🎯 Capabilities: {}",
                primal_state.capabilities.len()
            )),
        ];

        let details_widget = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Primal Details"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(details_widget, area);
    }

    /// Render deployment controls
    fn render_deployment_controls(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let controls_text = vec![
            Line::from("🚀 Deployment Controls"),
            Line::from(""),
            Line::from("📁 [D] Deploy from manifest"),
            Line::from("🌱 [C] Create new biome"),
            Line::from("⚖️  [S] Scale services"),
            Line::from("🔄 [R] Rollback deployment"),
        ];

        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("🎛️ Controls"))
            .wrap(Wrap { trim: true });

        f.render_widget(controls, area);
    }

    /// Render active deployments
    fn render_active_deployments(f: &mut Frame, area: Rect, state: &DashboardState) {
        if state.active_deployments.is_empty() {
            let empty_msg = Paragraph::new(
                "No active deployments\n\nUse 'D' to deploy a new biome or 'C' to create one",
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🚀 Active Deployments"),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
            f.render_widget(empty_msg, area);
            return;
        }

        let deployment_items: Vec<ListItem> = state
            .active_deployments
            .iter()
            .map(|deployment| {
                let status_icon = deployment_phase_icon(&deployment.status);

                ListItem::new(format!(
                    "{} {} - {} ({}%)",
                    status_icon,
                    deployment.biome_name,
                    deployment.target_environment,
                    deployment.progress
                ))
            })
            .collect();

        let deployments_list = List::new(deployment_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🚀 Active Deployments"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(
            deployments_list,
            area,
            &mut state.deployment_list_state.clone(),
        );
    }

    /// Render AI chat interface
    fn render_ai_chat(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chat_messages: Vec<Line> = state
            .ai_chat_history
            .iter()
            .map(|msg| {
                let prefix = role_to_prefix(role_to_str(&msg.role));
                Line::from(format!("{} {}", prefix, msg.content))
            })
            .collect();

        let scroll_offset = compute_scroll_offset(chat_messages.len(), area.height) as u16;
        let chat = Paragraph::new(chat_messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🤖 AI Assistant"),
            )
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0));

        f.render_widget(chat, area);
    }

    /// Render AI suggestions
    fn render_ai_suggestions(f: &mut Frame, area: Rect, state: &DashboardState) {
        let suggestions: Vec<Line> = state
            .ai_suggestions
            .iter()
            .take(3)
            .map(|suggestion| {
                let _bar = confidence_bar(suggestion.confidence);
                Line::from(format!(
                    "💡 {} ({}%)",
                    suggestion.title,
                    (suggestion.confidence * 100.0) as u8
                ))
            })
            .collect();

        let suggestions_widget = Paragraph::new(suggestions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💡 AI Suggestions"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(suggestions_widget, area);
    }

    /// Render AI input area
    fn render_ai_input(f: &mut Frame, area: Rect, state: &DashboardState) {
        let input = Paragraph::new(state.ai_input_buffer.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💬 Ask AI (Enter to send)"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(input, area);
    }

    /// Render active AI insights
    fn render_active_insights(f: &mut Frame, area: Rect, state: &DashboardState) {
        let insights: Vec<Line> = state
            .ai_insights
            .iter()
            .map(|insight| {
                let severity_icon = severity_to_icon(severity_to_str(&insight.severity));
                Line::from(format!("{} {}", severity_icon, insight.title))
            })
            .collect();

        let insights_widget = Paragraph::new(insights)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🧠 Active Insights"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(insights_widget, area);
    }

    /// Render insight analysis
    fn render_insight_analysis(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let analysis = vec![
            Line::from("🧠 AI Analysis"),
            Line::from(""),
            Line::from("Select an insight to see detailed analysis"),
            Line::from("and recommended actions."),
        ];

        let analysis_widget = Paragraph::new(analysis)
            .block(Block::default().borders(Borders::ALL).title("📊 Analysis"))
            .wrap(Wrap { trim: true });

        f.render_widget(analysis_widget, area);
    }

    /// Render API endpoints status
    fn render_api_endpoints(f: &mut Frame, area: Rect, state: &DashboardState) {
        let endpoints: Vec<Line> = state
            .api_endpoints
            .iter()
            .map(|(endpoint, status)| {
                let status_icon = api_status_to_icon(api_status_to_str(&status.status));
                Line::from(format!("{} {}", status_icon, endpoint))
            })
            .collect();

        let endpoints_widget = Paragraph::new(endpoints)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📡 API Endpoints"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(endpoints_widget, area);
    }

    /// Render sync status
    fn render_sync_status(f: &mut Frame, area: Rect, state: &DashboardState) {
        let sync_info = vec![
            Line::from("🔄 Sync Status"),
            Line::from(""),
            Line::from(format!("Last sync: {} primals", state.last_api_sync.len())),
            Line::from(format!("Errors: {}", state.api_errors.len())),
        ];

        let sync_widget = Paragraph::new(sync_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🔄 Synchronization"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(sync_widget, area);
    }

    /// Render API errors
    fn render_api_errors(f: &mut Frame, area: Rect, state: &DashboardState) {
        let errors: Vec<Line> = state
            .api_errors
            .iter()
            .take(5)
            .map(|error| Line::from(format!("❌ {}: {}", error.endpoint, error.error)))
            .collect();

        let errors_widget = Paragraph::new(errors)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚠️ Recent Errors"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(errors_widget, area);
    }

    /// Render log filters
    fn render_log_filters(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let filters = Paragraph::new("🔍 Log Filters: [F] Filter | [L] Level | [S] Source").block(
            Block::default()
                .borders(Borders::ALL)
                .title("📜 Log Controls"),
        );

        f.render_widget(filters, area);
    }

    /// Render log content
    fn render_log_content(f: &mut Frame, area: Rect, state: &DashboardState) {
        let logs: Vec<Line> = state
            .log_streams
            .values()
            .flatten()
            .take(50)
            .map(|log| {
                let level_icon = log_level_to_icon(log_level_to_str(&log.level));
                Line::from(format!("{} [{}] {}", level_icon, log.source, log.message))
            })
            .collect();

        let scroll_offset = compute_scroll_offset(logs.len(), area.height) as u16;
        let logs_widget = Paragraph::new(logs)
            .block(Block::default().borders(Borders::ALL).title("📜 Live Logs"))
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0));

        f.render_widget(logs_widget, area);
    }

    /// Render enhanced status bar with ecosystem info
    fn render_enhanced_status_bar(f: &mut Frame, area: Rect, state: &DashboardState) {
        let status_text = if state.ai_enabled {
            format!(
                "🤖 AI Enabled | 🌍 {} Primals | ⚙️ {} Services | 🚀 {} Deployments | Press '?' for help",
                state.ecosystem_health.primal_count,
                state.ecosystem_health.total_services,
                state.ecosystem_health.active_deployments
            )
        } else {
            format!(
                "🌍 {} Primals | ⚙️ {} Services | 🚀 {} Deployments | Press '?' for help",
                state.ecosystem_health.primal_count,
                state.ecosystem_health.total_services,
                state.ecosystem_health.active_deployments
            )
        };

        let status_bar = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(status_bar, area);
    }

    /// Render services (enhanced version)
    fn render_services(f: &mut Frame, area: Rect, state: &DashboardState) {
        if state.discovered_services.is_empty() {
            let empty_msg = Paragraph::new("🔄 Discovering services...\n\nServices will appear here as they are discovered via API ingestion from primals.")
                .block(Block::default().borders(Borders::ALL).title("⚙️ Services"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(empty_msg, area);
            return;
        }

        let service_items: Vec<ListItem> = state
            .discovered_services
            .iter()
            .map(|service| ListItem::new(format!("⚙️ {} - {}", service.id, service.endpoint)))
            .collect();

        let services_list = List::new(service_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚙️ Discovered Services"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(services_list, area, &mut state.service_list_state.clone());
    }

    /// Render health (enhanced version)
    fn render_health(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Overall ecosystem health
                Constraint::Min(0),    // Individual primal health
            ])
            .split(area);

        // Overall ecosystem health
        let health_color = health_to_color(health_to_category_str(
            &state.ecosystem_health.overall_status,
        ));

        let overall_health = Paragraph::new(format!(
            "🌍 Ecosystem Health: {:?}\n💊 {} of {} primals healthy\n⚙️ {} of {} services running",
            state.ecosystem_health.overall_status,
            state.ecosystem_health.healthy_primals,
            state.ecosystem_health.primal_count,
            state.ecosystem_health.healthy_services,
            state.ecosystem_health.total_services
        ))
        .style(Style::default().fg(health_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("💊 Overall Health"),
        );

        f.render_widget(overall_health, chunks[0]);

        // Individual primal health
        let primal_health: Vec<Line> = state
            .primal_states
            .iter()
            .map(|(id, primal_state)| {
                let health_icon = health_status_icon(&primal_state.health);
                Line::from(format!(
                    "{} {} - {:?}",
                    health_icon, id, primal_state.health
                ))
            })
            .collect();

        let primal_health_widget = Paragraph::new(primal_health)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🎯 Primal Health"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(primal_health_widget, chunks[1]);
    }

    /// Render metrics (enhanced version)
    fn render_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left: Resource metrics
        Self::render_resource_metrics(f, chunks[0], state);

        // Right: Performance metrics
        Self::render_performance_metrics(f, chunks[1], state);
    }

    /// Render resource metrics
    fn render_resource_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut resource_lines = vec![Line::from("💻 Resource Usage"), Line::from("")];

        let primal_items: Vec<_> = state
            .primal_states
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        for (name, cpu, mem, disk) in format_resource_usage_lines(&primal_items) {
            resource_lines.push(Line::from(format!("🎯 {}", name)));
            resource_lines.push(Line::from(format!("  CPU: {:.1}%", cpu)));
            resource_lines.push(Line::from(format!("  Memory: {:.1} MB", mem)));
            resource_lines.push(Line::from(format!("  Disk: {:.1} GB", disk)));
            resource_lines.push(Line::from(""));
        }

        let resource_widget = Paragraph::new(resource_lines)
            .block(Block::default().borders(Borders::ALL).title("💻 Resources"))
            .wrap(Wrap { trim: true });

        f.render_widget(resource_widget, area);
    }

    /// Render performance metrics
    fn render_performance_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut perf_lines = vec![Line::from("⚡ Performance"), Line::from("")];

        let primal_items: Vec<_> = state
            .primal_states
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        for (name, error_rate, response_ms, rps) in format_performance_lines(&primal_items) {
            perf_lines.push(Line::from(format!("🎯 {}", name)));
            perf_lines.push(Line::from(format!("  RPS: {:.1}", rps)));
            perf_lines.push(Line::from(format!("  Response: {:.0}ms", response_ms)));
            perf_lines.push(Line::from(format!("  Error Rate: {:.2}%", error_rate)));
            perf_lines.push(Line::from(""));
        }

        let perf_widget = Paragraph::new(perf_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚡ Performance"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(perf_widget, area);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::tui::types::{
        DeploymentPhase, EcosystemHealth, PrimalApiState, PrimalMetadata, PrimalMetrics,
        ResourceUsage, ServiceStatus,
    };
    use biomeos_types::{
        Health, HealthIssue, HealthIssueCategory, HealthIssueSeverity, PrimalType,
    };
    use std::collections::HashMap;
    use std::time::Duration;

    // --- health_to_color ---
    #[test]
    fn test_health_to_color_healthy() {
        assert_eq!(health_to_color("Healthy"), Color::Green);
    }

    #[test]
    fn test_health_to_color_degraded() {
        assert_eq!(health_to_color("Degraded"), Color::Yellow);
    }

    #[test]
    fn test_health_to_color_critical() {
        assert_eq!(health_to_color("Critical"), Color::Red);
    }

    #[test]
    fn test_health_to_color_unknown() {
        assert_eq!(health_to_color("Unknown"), Color::Gray);
    }

    #[test]
    fn test_health_to_color_empty_and_unknown_str() {
        assert_eq!(health_to_color(""), Color::Gray);
        assert_eq!(health_to_color("Foo"), Color::Gray);
    }

    // --- health_to_category_str ---
    #[test]
    fn test_health_to_category_str_all_variants() {
        assert_eq!(health_to_category_str(&Health::Healthy), "Healthy");
        assert_eq!(
            health_to_category_str(&Health::Degraded {
                issues: vec![],
                impact_score: Some(0.5),
            }),
            "Degraded"
        );
        use chrono::Utc;
        assert_eq!(
            health_to_category_str(&Health::Critical {
                issues: vec![HealthIssue {
                    id: "test".to_string(),
                    category: HealthIssueCategory::Software,
                    severity: HealthIssueSeverity::Critical,
                    message: "test".to_string(),
                    detected_at: Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec![],
            }),
            "Critical"
        );
        assert_eq!(
            health_to_category_str(&Health::Unknown {
                reason: "x".to_string(),
                last_known: None,
            }),
            "Unknown"
        );
    }

    // --- event_type_to_icon ---
    #[test]
    fn test_event_type_to_icon_all_variants() {
        assert_eq!(event_type_to_icon("Started"), "🚀");
        assert_eq!(event_type_to_icon("ServiceDeployed"), "✅");
        assert_eq!(event_type_to_icon("ServiceFailed"), "❌");
        assert_eq!(event_type_to_icon("Completed"), "🎉");
        assert_eq!(event_type_to_icon("Failed"), "💥");
        assert_eq!(event_type_to_icon("RolledBack"), "🔄");
    }

    #[test]
    fn test_event_type_to_icon_unknown() {
        assert_eq!(event_type_to_icon(""), "•");
        assert_eq!(event_type_to_icon("Foo"), "•");
    }

    // --- role_to_prefix ---
    #[test]
    fn test_role_to_prefix_all_variants() {
        assert_eq!(role_to_prefix("Human"), "👤");
        assert_eq!(role_to_prefix("Assistant"), "🤖");
        assert_eq!(role_to_prefix("System"), "⚙️");
    }

    #[test]
    fn test_role_to_prefix_unknown() {
        assert_eq!(role_to_prefix(""), "•");
        assert_eq!(role_to_prefix("Foo"), "•");
    }

    // --- severity_to_icon ---
    #[test]
    fn test_severity_to_icon_all_variants() {
        assert_eq!(severity_to_icon("Info"), "ℹ️");
        assert_eq!(severity_to_icon("Warning"), "⚠️");
        assert_eq!(severity_to_icon("Critical"), "🔥");
        assert_eq!(severity_to_icon("Optimization"), "⚡");
    }

    #[test]
    fn test_severity_to_icon_unknown() {
        assert_eq!(severity_to_icon(""), "•");
    }

    // --- api_status_to_icon ---
    #[test]
    fn test_api_status_to_icon_all_variants() {
        assert_eq!(api_status_to_icon("Connected"), "🟢");
        assert_eq!(api_status_to_icon("Disconnected"), "🔴");
        assert_eq!(api_status_to_icon("Error"), "🟠");
        assert_eq!(api_status_to_icon("Timeout"), "🟡");
    }

    #[test]
    fn test_api_status_to_icon_unknown() {
        assert_eq!(api_status_to_icon(""), "⚪");
    }

    // --- log_level_to_icon ---
    #[test]
    fn test_log_level_to_icon_all_variants() {
        assert_eq!(log_level_to_icon("Trace"), "🔍");
        assert_eq!(log_level_to_icon("Debug"), "🐛");
        assert_eq!(log_level_to_icon("Info"), "ℹ️");
        assert_eq!(log_level_to_icon("Warn"), "⚠️");
        assert_eq!(log_level_to_icon("Error"), "❌");
    }

    #[test]
    fn test_log_level_to_icon_unknown() {
        assert_eq!(log_level_to_icon(""), "•");
    }

    // --- compute_scroll_offset ---
    #[test]
    fn test_compute_scroll_offset_empty() {
        assert_eq!(compute_scroll_offset(0, 10), 0);
    }

    #[test]
    fn test_compute_scroll_offset_single_item() {
        assert_eq!(compute_scroll_offset(1, 10), 0);
    }

    #[test]
    fn test_compute_scroll_offset_boundary() {
        // area_height 10 -> 10-2=8 visible lines
        let area = 10u16;
        assert_eq!(compute_scroll_offset(8, area), 0);
        assert_eq!(compute_scroll_offset(9, area), 1);
    }

    #[test]
    fn test_compute_scroll_offset_many() {
        assert_eq!(compute_scroll_offset(50, 10), 42);
    }

    #[test]
    fn test_compute_scroll_offset_single_line_area() {
        assert_eq!(compute_scroll_offset(5, 3), 4);
    }

    // --- compute_top_capabilities ---
    #[test]
    fn test_compute_top_capabilities_empty() {
        let stats: HashMap<String, usize> = HashMap::new();
        assert!(compute_top_capabilities(&stats, 5).is_empty());
    }

    #[test]
    fn test_compute_top_capabilities_single() {
        let mut stats = HashMap::new();
        stats.insert("a".to_string(), 10);
        let r = compute_top_capabilities(&stats, 5);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], ("a".to_string(), 10));
    }

    #[test]
    fn test_compute_top_capabilities_n_zero() {
        let mut stats = HashMap::new();
        stats.insert("a".to_string(), 10);
        assert!(compute_top_capabilities(&stats, 0).is_empty());
    }

    #[test]
    fn test_compute_top_capabilities_sorted_desc() {
        let mut stats = HashMap::new();
        stats.insert("a".to_string(), 10);
        stats.insert("b".to_string(), 30);
        stats.insert("c".to_string(), 20);
        let r = compute_top_capabilities(&stats, 3);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0], ("b".to_string(), 30));
        assert_eq!(r[1], ("c".to_string(), 20));
        assert_eq!(r[2], ("a".to_string(), 10));
    }

    #[test]
    fn test_compute_top_capabilities_take_n() {
        let mut stats = HashMap::new();
        stats.insert("a".to_string(), 10);
        stats.insert("b".to_string(), 20);
        stats.insert("c".to_string(), 30);
        let r = compute_top_capabilities(&stats, 2);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], ("c".to_string(), 30));
        assert_eq!(r[1], ("b".to_string(), 20));
    }

    // --- format_resource_usage_lines ---
    fn make_primal(name: &str, cpu: f64, mem: f64, disk: f64) -> PrimalApiState {
        PrimalApiState {
            primal_id: name.to_string(),
            primal_type: PrimalType::new("compute", "test", "1.0.0"),
            endpoint: "http://localhost".to_string(),
            health: Health::Healthy,
            capabilities: vec![],
            metadata: PrimalMetadata {
                name: name.to_string(),
                version: "1.0".to_string(),
                description: "".to_string(),
                uptime: Duration::ZERO,
                resource_usage: ResourceUsage {
                    cpu_percent: cpu,
                    memory_mb: mem,
                    disk_gb: disk,
                    network_mbps: 0.0,
                },
            },
            services: vec![],
            metrics: PrimalMetrics {
                requests_per_second: 0.0,
                average_response_time: Duration::ZERO,
                error_rate: 0.0,
                throughput: 0.0,
            },
            last_updated: std::time::Instant::now(),
            api_version: "1".to_string(),
        }
    }

    #[test]
    fn test_format_resource_usage_lines_empty() {
        let items: Vec<(String, &PrimalApiState)> = vec![];
        assert!(format_resource_usage_lines(&items).is_empty());
    }

    #[test]
    fn test_format_resource_usage_lines_single() {
        let p = make_primal("p1", 25.5, 128.0, 10.0);
        let items = vec![("id1".to_string(), &p)];
        let r = format_resource_usage_lines(&items);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], ("id1".to_string(), 25.5, 128.0, 10.0));
    }

    // --- format_performance_lines ---
    #[test]
    fn test_format_performance_lines_empty() {
        let items: Vec<(String, &PrimalApiState)> = vec![];
        assert!(format_performance_lines(&items).is_empty());
    }

    #[test]
    fn test_format_performance_lines_single() {
        let mut p = make_primal("p1", 0.0, 0.0, 0.0);
        p.metrics = PrimalMetrics {
            requests_per_second: 100.0,
            average_response_time: Duration::from_millis(50),
            error_rate: 0.02,
            throughput: 0.0,
        };
        let items = vec![("id1".to_string(), &p)];
        let r = format_performance_lines(&items);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "id1");
        assert!((r[0].1 - 2.0).abs() < 0.01); // error_rate * 100
        assert!((r[0].2 - 50.0).abs() < 1.0); // response_ms
        assert!((r[0].3 - 100.0).abs() < 0.01); // rps
    }

    // --- format_ecosystem_header_text ---
    #[test]
    fn test_format_ecosystem_header_text() {
        let mut state = DashboardState::new();
        state.ecosystem_health = EcosystemHealth {
            overall_status: Health::Healthy,
            primal_count: 3,
            healthy_primals: 3,
            total_services: 5,
            healthy_services: 5,
            active_deployments: 2,
            critical_issues: vec![],
        };
        let header = format_ecosystem_header_text(&state);
        assert!(header.contains("3 Primals"));
        assert!(header.contains("3 healthy"));
        assert!(header.contains("5 Services"));
        assert!(header.contains("2 Deployments"));
    }

    // --- format_primal_list_items_data ---
    #[test]
    fn test_format_primal_list_items_data_empty() {
        let items: Vec<(String, &PrimalApiState)> = vec![];
        assert!(format_primal_list_items_data(&items).is_empty());
    }

    #[test]
    fn test_format_primal_list_items_data_single() {
        let p = make_primal("MyPrimal", 0.0, 0.0, 0.0);
        let items = vec![("primal-1".to_string(), &p)];
        let r = format_primal_list_items_data(&items);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "MyPrimal (primal-1)");
        assert_eq!(r[0].1, "💚");
        assert_eq!(r[0].2, Color::Green);
    }

    // --- confidence_bar ---
    #[test]
    fn test_confidence_bar_zero() {
        assert_eq!(confidence_bar(0.0), "");
    }

    #[test]
    fn test_confidence_bar_one() {
        assert_eq!(confidence_bar(1.0), "==========");
    }

    #[test]
    fn test_confidence_bar_half() {
        assert_eq!(confidence_bar(0.5), "=====");
    }

    #[test]
    fn test_confidence_bar_boundary() {
        assert_eq!(confidence_bar(0.1), "=");
        assert_eq!(confidence_bar(0.99), "=========");
        assert_eq!(confidence_bar(1.0), "==========");
    }

    #[test]
    fn test_confidence_bar_negative() {
        assert_eq!(confidence_bar(-0.5), "");
    }

    #[test]
    fn test_confidence_bar_over_one() {
        assert_eq!(confidence_bar(2.0), "==========");
    }

    #[test]
    fn test_health_status_icon_healthy() {
        assert_eq!(health_status_icon(&Health::Healthy), "💚");
    }

    #[test]
    fn test_health_status_icon_degraded() {
        assert_eq!(
            health_status_icon(&Health::Degraded {
                issues: vec![],
                impact_score: Some(0.5),
            }),
            "💛"
        );
    }

    #[test]
    fn test_health_status_icon_critical() {
        use chrono::Utc;
        assert_eq!(
            health_status_icon(&Health::Critical {
                issues: vec![HealthIssue {
                    id: "test".to_string(),
                    category: HealthIssueCategory::Software,
                    severity: HealthIssueSeverity::Critical,
                    message: "test".to_string(),
                    detected_at: Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec![],
            }),
            "❤️"
        );
    }

    #[test]
    fn test_health_status_icon_unknown() {
        assert_eq!(
            health_status_icon(&Health::Unknown {
                reason: "test".to_string(),
                last_known: None,
            }),
            "⚪"
        );
    }

    #[test]
    fn test_service_status_icon_all_variants() {
        assert_eq!(service_status_icon(&ServiceStatus::Running), "▶️");
        assert_eq!(service_status_icon(&ServiceStatus::Starting), "⏳");
        assert_eq!(service_status_icon(&ServiceStatus::Stopping), "⏹️");
        assert_eq!(service_status_icon(&ServiceStatus::Failed), "❌");
        assert_eq!(service_status_icon(&ServiceStatus::Scaling), "⚖️");
    }

    #[test]
    fn test_deployment_phase_icon_all_variants() {
        assert_eq!(deployment_phase_icon(&DeploymentPhase::Validating), "🔍");
        assert_eq!(deployment_phase_icon(&DeploymentPhase::Deploying), "🚀");
        assert_eq!(deployment_phase_icon(&DeploymentPhase::Scaling), "⚖️");
        assert_eq!(deployment_phase_icon(&DeploymentPhase::Configuring), "⚙️");
        assert_eq!(
            deployment_phase_icon(&DeploymentPhase::HealthChecking),
            "💊"
        );
        assert_eq!(deployment_phase_icon(&DeploymentPhase::Complete), "✅");
        assert_eq!(
            deployment_phase_icon(&DeploymentPhase::Failed {
                reason: "test".to_string()
            }),
            "❌"
        );
        assert_eq!(deployment_phase_icon(&DeploymentPhase::RollingBack), "🔄");
    }

    #[test]
    fn test_widget_renderer_construction() {
        let _renderer = WidgetRenderer;
    }

    #[test]
    fn test_tab_id_all_tabs() {
        let tabs = TabId::all_tabs();
        assert!(!tabs.is_empty());
        assert_eq!(tabs.len(), 10);
        assert_eq!(tabs[0].icon, "🌍");
        assert_eq!(tabs[0].title, "Ecosystem");
    }

    #[test]
    fn test_dashboard_state_current_tab_info() {
        let state = DashboardState::new();
        let tab_info = state.current_tab_info();
        assert_eq!(tab_info.title, "Ecosystem");
        assert_eq!(tab_info.icon, "🌍");
    }

    #[test]
    fn test_ecosystem_header_text_format() {
        let mut state = DashboardState::new();
        state.ecosystem_health = EcosystemHealth {
            overall_status: Health::Healthy,
            primal_count: 3,
            healthy_primals: 3,
            total_services: 5,
            healthy_services: 5,
            active_deployments: 2,
            critical_issues: vec![],
        };
        let header = format_ecosystem_header_text(&state);
        assert!(header.contains("3 Primals"));
        assert!(header.contains("5 Services"));
        assert!(header.contains("2 Deployments"));
    }
}
