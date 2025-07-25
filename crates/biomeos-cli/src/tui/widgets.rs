//! TUI Widgets and Rendering Components
//!
//! Contains reusable widgets and rendering functions for the TUI dashboard.

use crate::tui::types::{DashboardState, TabId, TabInfo};
use biomeos_core::SystemHealth;
use biomeos_primal_sdk::PrimalHealth;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, ListState, Paragraph,
        Sparkline, Tabs, Wrap,
    },
    Frame,
};
use std::collections::HashMap;

/// Widget renderer for the TUI dashboard
pub struct WidgetRenderer;

impl WidgetRenderer {
    /// Render the main dashboard layout
    pub fn render_dashboard(f: &mut Frame, state: &DashboardState) {
        let size = f.size();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
            .split(size);

        // Render tabs
        Self::render_tabs(f, chunks[0], state);

        // Render content based on current tab
        let current_tab = state.current_tab_info();
        match current_tab.id {
            TabId::Overview => Self::render_overview(f, chunks[1], state),
            TabId::Services => Self::render_services(f, chunks[1], state),
            TabId::Health => Self::render_health(f, chunks[1], state),
            TabId::Discovery => Self::render_discovery(f, chunks[1], state),
            TabId::Metrics => Self::render_metrics(f, chunks[1], state),
        }

        // Render status bar
        Self::render_status_bar(f, chunks[2]);
    }

    /// Render the tabs bar
    fn render_tabs(f: &mut Frame, area: Rect, state: &DashboardState) {
        let tabs = TabInfo::all_tabs();
        let tab_titles: Vec<Line> = tabs.iter().map(|tab| Line::from(tab.title)).collect();

        let tabs_widget = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("BiomeOS Dashboard"),
            )
            .select(state.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs_widget, area);
    }

    /// Render the overview tab
    fn render_overview(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // System status
                Constraint::Min(0),    // Services and metrics
            ])
            .split(area);

        // System status
        Self::render_system_status(f, chunks[0], state);

        // Services and metrics
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        Self::render_services_overview(f, bottom_chunks[0], state);
        Self::render_capabilities_overview(f, bottom_chunks[1], state);
    }

    /// Render system status overview
    fn render_system_status(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // Get latest system health if available
        let latest_health = state.system_health_history.back().map(|(_, health)| health);

        if let Some(health) = latest_health {
            // CPU usage
            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU").borders(Borders::ALL))
                .gauge_style(Self::get_usage_style(
                    health.resource_usage.cpu_usage_percent,
                ))
                .percent(health.resource_usage.cpu_usage_percent as u16);
            f.render_widget(cpu_gauge, chunks[0]);

            // Memory usage
            let memory_gauge = Gauge::default()
                .block(Block::default().title("Memory").borders(Borders::ALL))
                .gauge_style(Self::get_usage_style(
                    health.resource_usage.memory_usage_percent,
                ))
                .percent(health.resource_usage.memory_usage_percent as u16);
            f.render_widget(memory_gauge, chunks[1]);

            // Disk usage
            let disk_gauge = Gauge::default()
                .block(Block::default().title("Disk").borders(Borders::ALL))
                .gauge_style(Self::get_usage_style(
                    health.resource_usage.disk_usage_percent,
                ))
                .percent(health.resource_usage.disk_usage_percent as u16);
            f.render_widget(disk_gauge, chunks[2]);

            // Overall status
            let status_text = format!("{:?}", health.overall_status);
            let status_color = Self::get_health_status_color(&status_text);
            let status_paragraph = Paragraph::new(status_text)
                .block(Block::default().title("Status").borders(Borders::ALL))
                .style(status_color)
                .alignment(Alignment::Center);
            f.render_widget(status_paragraph, chunks[3]);
        } else {
            // Show placeholder when no data is available
            for chunk in chunks {
                let placeholder = Paragraph::new("No Data")
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(placeholder, chunk);
            }
        }
    }

    /// Render services overview
    fn render_services_overview(f: &mut Frame, area: Rect, state: &DashboardState) {
        let services: Vec<ListItem> = state
            .discovered_services
            .iter()
            .take(10) // Show top 10 services
            .map(|service| {
                let health_style = Self::get_primal_health_style(&service.health);
                let health_icon = Self::get_health_icon(&service.health);

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", health_icon), health_style),
                    Span::styled(&service.id, Style::default().fg(Color::White)),
                    Span::styled(
                        format!(" ({})", service.endpoint),
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let services_list = List::new(services)
            .block(
                Block::default()
                    .title(format!("Services ({})", state.discovered_services.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(services_list, area);
    }

    /// Render capabilities overview
    fn render_capabilities_overview(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut sorted_caps: Vec<_> = state.capability_stats.iter().collect();
        sorted_caps.sort_by(|a, b| b.1.cmp(a.1));

        let cap_items: Vec<ListItem> = sorted_caps
            .iter()
            .take(10)
            .map(|(cap, count)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}x ", count), Style::default().fg(Color::Cyan)),
                    Span::styled(cap.as_str(), Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let capabilities_list = List::new(cap_items).block(
            Block::default()
                .title("Top Capabilities")
                .borders(Borders::ALL),
        );

        f.render_widget(capabilities_list, area);
    }

    /// Render the services tab
    fn render_services(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Service list
        Self::render_service_list(f, chunks[0], state);

        // Service details
        Self::render_service_details(f, chunks[1], state);
    }

    /// Render the service list
    fn render_service_list(f: &mut Frame, area: Rect, state: &DashboardState) {
        let services: Vec<ListItem> = state
            .discovered_services
            .iter()
            .map(|service| {
                let health_style = Self::get_primal_health_style(&service.health);
                let health_icon = Self::get_health_icon(&service.health);

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", health_icon), health_style),
                    Span::styled(&service.id, Style::default().fg(Color::White)),
                    Span::styled(
                        format!(" [{}]", service.primal_type.category),
                        Style::default().fg(Color::Yellow),
                    ),
                ]))
            })
            .collect();

        let services_list = List::new(services)
            .block(
                Block::default()
                    .title(format!(
                        "Discovered Services ({})",
                        state.discovered_services.len()
                    ))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Gray));

        f.render_stateful_widget(services_list, area, &mut state.service_list_state.clone());
    }

    /// Render service details
    fn render_service_details(f: &mut Frame, area: Rect, state: &DashboardState) {
        if let Some(service) = state.selected_service() {
            let details = vec![
                format!("ID: {}", service.id),
                format!("Type: {}", service.primal_type.category),
                format!("Endpoint: {}", service.endpoint),
                format!("Health: {:?}", service.health),
                format!("Capabilities: {}", service.capabilities.len()),
                format!(
                    "Discovered: {}",
                    service.discovered_at.format("%Y-%m-%d %H:%M:%S")
                ),
            ];

            let details_text = details.join("\n");
            let details_paragraph = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title("Service Details")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(details_paragraph, area);
        } else {
            let placeholder = Paragraph::new("Select a service to view details")
                .block(
                    Block::default()
                        .title("Service Details")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Center);
            f.render_widget(placeholder, area);
        }
    }

    /// Render the health tab
    fn render_health(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Health metrics chart
        Self::render_health_chart(f, chunks[0], state);

        // Health breakdown
        Self::render_health_breakdown(f, chunks[1], state);
    }

    /// Render health metrics chart
    fn render_health_chart(f: &mut Frame, area: Rect, state: &DashboardState) {
        if state.system_health_history.is_empty() {
            let placeholder =
                Paragraph::new("No health data available yet\n\nWait for data collection...")
                    .block(
                        Block::default()
                            .title("System Health History")
                            .borders(Borders::ALL),
                    )
                    .alignment(Alignment::Center);
            f.render_widget(placeholder, area);
            return;
        }

        // Prepare data for chart
        let data: Vec<(f64, f64)> = state
            .system_health_history
            .iter()
            .enumerate()
            .map(|(i, (_, health))| (i as f64, health.resource_usage.cpu_usage_percent))
            .collect();

        let datasets = vec![Dataset::default()
            .name("CPU Usage")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&data)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("System Health History")
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, state.max_history_points as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("Usage %")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 100.0]),
            );

        f.render_widget(chart, area);
    }

    /// Render health breakdown
    fn render_health_breakdown(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Service health breakdown
        let mut service_health_stats = HashMap::new();
        for service in &state.discovered_services {
            let health_str = format!("{:?}", service.health);
            *service_health_stats.entry(health_str).or_insert(0) += 1;
        }

        let health_items: Vec<ListItem> = service_health_stats
            .iter()
            .map(|(health, count)| {
                let (color, icon) = match health.as_str() {
                    "Healthy" => (Color::Green, "●"),
                    "Degraded" => (Color::Yellow, "◐"),
                    "Unhealthy" => (Color::Red, "●"),
                    _ => (Color::Gray, "?"),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(format!("{}: {}", health, count), Style::default()),
                ]))
            })
            .collect();

        let health_list = List::new(health_items).block(
            Block::default()
                .title("Service Health")
                .borders(Borders::ALL),
        );

        f.render_widget(health_list, chunks[0]);

        // System resources
        if let Some((_, health)) = state.system_health_history.back() {
            let resource_info = vec![
                format!("CPU: {:.1}%", health.resource_usage.cpu_usage_percent),
                format!("Memory: {:.1}%", health.resource_usage.memory_usage_percent),
                format!("Disk: {:.1}%", health.resource_usage.disk_usage_percent),
                format!(
                    "Network I/O: {:.1} MB/s",
                    health.resource_usage.network_io_mbps
                ),
                format!(
                    "Active Connections: {}",
                    health.resource_usage.active_connections
                ),
            ];

            let resource_text = resource_info.join("\n");
            let resource_paragraph = Paragraph::new(resource_text).block(
                Block::default()
                    .title("System Resources")
                    .borders(Borders::ALL),
            );

            f.render_widget(resource_paragraph, chunks[1]);
        }
    }

    /// Render the discovery tab
    fn render_discovery(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let placeholder = Paragraph::new("Interactive Discovery Interface\n\n[Coming Soon]\n\nFeatures:\n• Live service scanning\n• Capability filtering\n• Discovery method selection\n• Network topology visualization")
            .block(Block::default().title("Service Discovery").borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(placeholder, area);
    }

    /// Render the metrics tab
    fn render_metrics(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let placeholder = Paragraph::new("Advanced Metrics & Analytics\n\n[Coming Soon]\n\nFeatures:\n• Historical trends\n• Performance analytics\n• Capability usage stats\n• Ecosystem health scoring")
            .block(Block::default().title("Advanced Metrics").borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(placeholder, area);
    }

    /// Render the status bar
    fn render_status_bar(f: &mut Frame, area: Rect) {
        let status_text = Line::from(vec![
            Span::styled(
                "Tab/Shift+Tab: Switch tabs",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("↑/↓: Navigate", Style::default().fg(Color::Gray)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Enter: Select", Style::default().fg(Color::Gray)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("R: Refresh", Style::default().fg(Color::Gray)),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled("Q: Quit", Style::default().fg(Color::Gray)),
        ]);

        let status_bar = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(status_bar, area);
    }

    /// Get style for health status
    fn get_health_status_color(status: &str) -> Style {
        match status {
            "Healthy" => Style::default().fg(Color::Green),
            "Degraded" => Style::default().fg(Color::Yellow),
            "Unhealthy" | "Critical" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Gray),
        }
    }

    /// Get style for primal health
    fn get_primal_health_style(health: &PrimalHealth) -> Style {
        match health {
            PrimalHealth::Healthy => Style::default().fg(Color::Green),
            PrimalHealth::Degraded => Style::default().fg(Color::Yellow),
            PrimalHealth::Unhealthy => Style::default().fg(Color::Red),
            PrimalHealth::Unknown => Style::default().fg(Color::Gray),
        }
    }

    /// Get style for resource usage percentage
    fn get_usage_style(percentage: f64) -> Style {
        let color = if percentage > 90.0 {
            Color::Red
        } else if percentage > 70.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        Style::default().fg(color)
    }

    /// Get health icon for display
    fn get_health_icon(health: &PrimalHealth) -> &'static str {
        match health {
            PrimalHealth::Healthy => "●",
            PrimalHealth::Degraded => "◐",
            PrimalHealth::Unhealthy => "●",
            PrimalHealth::Unknown => "?",
        }
    }
}
