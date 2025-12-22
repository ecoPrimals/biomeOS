//! TUI Widgets for Comprehensive Ecosystem Interface
//!
//! Advanced widgets for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.

use crate::tui::types::{DashboardState, TabId, AiRole, DeploymentPhase, ServiceStatus, InsightSeverity, LogLevel};
use biomeos_types::Health;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Tabs, Wrap,
    },
    Frame,
};

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
            TabId::DeploymentOrchestration => Self::render_deployment_orchestration(f, chunks[2], state),
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
        let health_color = match state.ecosystem_health.overall_status {
            Health::Healthy => Color::Green,
            Health::Degraded { .. } => Color::Yellow,
            Health::Critical { .. } => Color::Red,
            _ => Color::Gray,
        };

        let header_text = format!(
            "🌍 BiomeOS Ecosystem | {} Primals ({} healthy) | {} Services ({} healthy) | {} Deployments",
            state.ecosystem_health.primal_count,
            state.ecosystem_health.healthy_primals,
            state.ecosystem_health.total_services,
            state.ecosystem_health.healthy_services,
            state.ecosystem_health.active_deployments,
        );

        let header = Paragraph::new(header_text)
            .style(Style::default().fg(health_color))
            .block(Block::default().borders(Borders::ALL).title("Ecosystem Status"))
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
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
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
        let primal_items: Vec<ListItem> = state.primal_states
            .iter()
            .map(|(id, primal_state)| {
                let health_icon = match primal_state.health {
                    Health::Healthy => "💚",
                    Health::Degraded { .. } => "💛",
                    Health::Critical { .. } => "❤️",
                    _ => "⚪",
                };
                ListItem::new(format!("{} {} ({})", health_icon, primal_state.metadata.name, id))
            })
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
        let mut map_text = vec![
            Line::from("🌍 BiomeOS Ecosystem Map"),
            Line::from(""),
        ];

        // Add primal representations
        for (id, primal_state) in &state.primal_states {
            let health_icon = match primal_state.health {
                Health::Healthy => "💚",
                Health::Degraded { .. } => "💛", 
                Health::Critical { .. } => "❤️",
                _ => "⚪",
            };
            
            map_text.push(Line::from(vec![
                Span::raw(format!("  {} {} ", health_icon, primal_state.metadata.name)),
                Span::styled(format!("({})", id), Style::default().fg(Color::Gray)),
            ]));
            
            // Show services under each primal
            for service in &primal_state.services {
                let service_icon = match service.status {
                    ServiceStatus::Running => "▶️",
                    ServiceStatus::Starting => "⏳",
                    ServiceStatus::Stopping => "⏹️",
                    ServiceStatus::Failed => "❌",
                    ServiceStatus::Scaling => "⚖️",
                };
                map_text.push(Line::from(format!("    {} {}", service_icon, service.name)));
            }
            map_text.push(Line::from(""));
        }

        if state.primal_states.is_empty() {
            map_text.push(Line::from("🔄 Waiting for primal API connections..."));
        }

        let map = Paragraph::new(map_text)
            .block(Block::default().borders(Borders::ALL).title("🗺️ Ecosystem Map"))
            .wrap(Wrap { trim: true });

        f.render_widget(map, area);
    }

    /// Render key metrics panel
    fn render_key_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let metrics_text = vec![
            Line::from("📊 Key Metrics"),
            Line::from(""),
            Line::from(format!("Primals: {}/{} healthy", 
                state.ecosystem_health.healthy_primals, 
                state.ecosystem_health.primal_count)),
            Line::from(format!("Services: {}/{} running", 
                state.ecosystem_health.healthy_services, 
                state.ecosystem_health.total_services)),
            Line::from(format!("Deployments: {} active", 
                state.ecosystem_health.active_deployments)),
            Line::from(""),
            Line::from("🎯 Top Capabilities:"),
        ];

        let mut all_text = metrics_text;
        
        // Add top capabilities
        let mut sorted_caps: Vec<_> = state.capability_stats.iter().collect();
        sorted_caps.sort_by(|a, b| b.1.cmp(a.1));
        
        for (cap, count) in sorted_caps.iter().take(5) {
            all_text.push(Line::from(format!("  • {}: {}", cap, count)));
        }

        let metrics = Paragraph::new(all_text)
            .block(Block::default().borders(Borders::ALL).title("📈 Metrics"))
            .wrap(Wrap { trim: true });

        f.render_widget(metrics, area);
    }

    /// Render recent events
    fn render_recent_events(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut events = vec![
            Line::from("📋 Recent Events"),
            Line::from(""),
        ];

        // Add recent deployment events
        for event in state.deployment_history.iter().rev().take(5) {
            let icon = match event.event_type {
                crate::tui::types::DeploymentEventType::Started => "🚀",
                crate::tui::types::DeploymentEventType::ServiceDeployed => "✅",
                crate::tui::types::DeploymentEventType::ServiceFailed => "❌",
                crate::tui::types::DeploymentEventType::Completed => "🎉",
                crate::tui::types::DeploymentEventType::Failed => "💥",
                crate::tui::types::DeploymentEventType::RolledBack => "🔄",
            };
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
    fn render_primal_details(f: &mut Frame, area: Rect, id: &str, primal_state: &crate::tui::types::PrimalApiState) {
        let details = vec![
            Line::from(format!("🎯 Primal: {}", primal_state.metadata.name)),
            Line::from(""),
            Line::from(format!("ID: {}", id)),
            Line::from(format!("Version: {}", primal_state.metadata.version)),
            Line::from(format!("Endpoint: {}", primal_state.endpoint)),
            Line::from(format!("Health: {:?}", primal_state.health)),
            Line::from(""),
            Line::from("💻 Resource Usage:"),
            Line::from(format!("  CPU: {:.1}%", primal_state.metadata.resource_usage.cpu_percent)),
            Line::from(format!("  Memory: {:.1} MB", primal_state.metadata.resource_usage.memory_mb)),
            Line::from(format!("  Disk: {:.1} GB", primal_state.metadata.resource_usage.disk_gb)),
            Line::from(""),
            Line::from(format!("📊 Services: {}", primal_state.services.len())),
            Line::from(format!("🎯 Capabilities: {}", primal_state.capabilities.len())),
        ];

        let details_widget = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("📋 Primal Details"))
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
            let empty_msg = Paragraph::new("No active deployments\n\nUse 'D' to deploy a new biome or 'C' to create one")
                .block(Block::default().borders(Borders::ALL).title("🚀 Active Deployments"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(empty_msg, area);
            return;
        }

        let deployment_items: Vec<ListItem> = state.active_deployments
            .iter()
            .map(|deployment| {
                let status_icon = match deployment.status {
                    DeploymentPhase::Validating => "🔍",
                    DeploymentPhase::Deploying => "🚀",
                    DeploymentPhase::Scaling => "⚖️",
                    DeploymentPhase::Configuring => "⚙️",
                    DeploymentPhase::HealthChecking => "💊",
                    DeploymentPhase::Complete => "✅",
                    DeploymentPhase::Failed { .. } => "❌",
                    DeploymentPhase::RollingBack => "🔄",
                };
                
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
            .block(Block::default().borders(Borders::ALL).title("🚀 Active Deployments"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(deployments_list, area, &mut state.deployment_list_state.clone());
    }

    /// Render AI chat interface
    fn render_ai_chat(f: &mut Frame, area: Rect, state: &DashboardState) {
        let chat_messages: Vec<Line> = state.ai_chat_history
            .iter()
            .map(|msg| {
                let prefix = match msg.role {
                    AiRole::Human => "👤",
                    AiRole::Assistant => "🤖",
                    AiRole::System => "⚙️",
                };
                Line::from(format!("{} {}", prefix, msg.content))
            })
            .collect();

        let scroll_offset = chat_messages.len().saturating_sub(area.height as usize - 2) as u16;
        let chat = Paragraph::new(chat_messages)
            .block(Block::default().borders(Borders::ALL).title("🤖 AI Assistant"))
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0));

        f.render_widget(chat, area);
    }

    /// Render AI suggestions
    fn render_ai_suggestions(f: &mut Frame, area: Rect, state: &DashboardState) {
        let suggestions: Vec<Line> = state.ai_suggestions
            .iter()
            .take(3)
            .map(|suggestion| {
                let _confidence_bar = "=".repeat((suggestion.confidence * 10.0) as usize);
                Line::from(format!("💡 {} ({}%)", suggestion.title, (suggestion.confidence * 100.0) as u8))
            })
            .collect();

        let suggestions_widget = Paragraph::new(suggestions)
            .block(Block::default().borders(Borders::ALL).title("💡 AI Suggestions"))
            .wrap(Wrap { trim: true });

        f.render_widget(suggestions_widget, area);
    }

    /// Render AI input area
    fn render_ai_input(f: &mut Frame, area: Rect, state: &DashboardState) {
        let input = Paragraph::new(state.ai_input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title("💬 Ask AI (Enter to send)"))
            .wrap(Wrap { trim: true });

        f.render_widget(input, area);
    }

    /// Render active AI insights
    fn render_active_insights(f: &mut Frame, area: Rect, state: &DashboardState) {
        let insights: Vec<Line> = state.ai_insights
            .iter()
            .map(|insight| {
                let severity_icon = match insight.severity {
                    InsightSeverity::Info => "ℹ️",
                    InsightSeverity::Warning => "⚠️",
                    InsightSeverity::Critical => "🔥",
                    InsightSeverity::Optimization => "⚡",
                };
                Line::from(format!("{} {}", severity_icon, insight.title))
            })
            .collect();

        let insights_widget = Paragraph::new(insights)
            .block(Block::default().borders(Borders::ALL).title("🧠 Active Insights"))
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
        let endpoints: Vec<Line> = state.api_endpoints
            .iter()
            .map(|(endpoint, status)| {
                let status_icon = match status.status {
                    crate::tui::types::ApiStatus::Connected => "🟢",
                    crate::tui::types::ApiStatus::Disconnected => "🔴",
                    crate::tui::types::ApiStatus::Error { .. } => "🟠",
                    crate::tui::types::ApiStatus::Timeout => "🟡",
                };
                Line::from(format!("{} {}", status_icon, endpoint))
            })
            .collect();

        let endpoints_widget = Paragraph::new(endpoints)
            .block(Block::default().borders(Borders::ALL).title("📡 API Endpoints"))
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
            .block(Block::default().borders(Borders::ALL).title("🔄 Synchronization"))
            .wrap(Wrap { trim: true });

        f.render_widget(sync_widget, area);
    }

    /// Render API errors
    fn render_api_errors(f: &mut Frame, area: Rect, state: &DashboardState) {
        let errors: Vec<Line> = state.api_errors
            .iter()
            .take(5)
            .map(|error| Line::from(format!("❌ {}: {}", error.endpoint, error.error)))
            .collect();

        let errors_widget = Paragraph::new(errors)
            .block(Block::default().borders(Borders::ALL).title("⚠️ Recent Errors"))
            .wrap(Wrap { trim: true });

        f.render_widget(errors_widget, area);
    }

    /// Render log filters 
    fn render_log_filters(f: &mut Frame, area: Rect, _state: &DashboardState) {
        let filters = Paragraph::new("🔍 Log Filters: [F] Filter | [L] Level | [S] Source")
            .block(Block::default().borders(Borders::ALL).title("📜 Log Controls"));

        f.render_widget(filters, area);
    }

    /// Render log content
    fn render_log_content(f: &mut Frame, area: Rect, state: &DashboardState) {
        let logs: Vec<Line> = state.log_streams
            .values()
            .flatten()
            .take(50)
            .map(|log| {
                let level_icon = match log.level {
                    LogLevel::Trace => "🔍",
                    LogLevel::Debug => "🐛",
                    LogLevel::Info => "ℹ️",
                    LogLevel::Warn => "⚠️",
                    LogLevel::Error => "❌",
                };
                Line::from(format!("{} [{}] {}", level_icon, log.source, log.message))
            })
            .collect();

        let scroll_offset = logs.len().saturating_sub(area.height as usize - 2) as u16;
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

        let service_items: Vec<ListItem> = state.discovered_services
            .iter()
            .map(|service| {
                ListItem::new(format!("⚙️ {} - {}", service.id, service.endpoint))
            })
            .collect();

        let services_list = List::new(service_items)
            .block(Block::default().borders(Borders::ALL).title("⚙️ Discovered Services"))
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
        let health_color = match state.ecosystem_health.overall_status {
            Health::Healthy => Color::Green,
            Health::Degraded { .. } => Color::Yellow,
            Health::Critical { .. } => Color::Red,
            _ => Color::Gray,
        };

        let overall_health = Paragraph::new(format!(
            "🌍 Ecosystem Health: {:?}\n💊 {} of {} primals healthy\n⚙️ {} of {} services running",
            state.ecosystem_health.overall_status,
            state.ecosystem_health.healthy_primals,
            state.ecosystem_health.primal_count,
            state.ecosystem_health.healthy_services,
            state.ecosystem_health.total_services
        ))
        .style(Style::default().fg(health_color))
        .block(Block::default().borders(Borders::ALL).title("💊 Overall Health"));

        f.render_widget(overall_health, chunks[0]);

        // Individual primal health
        let primal_health: Vec<Line> = state.primal_states
            .iter()
            .map(|(id, primal_state)| {
                let health_icon = match primal_state.health {
                    Health::Healthy => "💚",
                    Health::Degraded { .. } => "💛",
                    Health::Critical { .. } => "❤️",
                    _ => "⚪",
                };
                Line::from(format!("{} {} - {:?}", health_icon, id, primal_state.health))
            })
            .collect();

        let primal_health_widget = Paragraph::new(primal_health)
            .block(Block::default().borders(Borders::ALL).title("🎯 Primal Health"))
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
        let mut resource_lines = vec![
            Line::from("💻 Resource Usage"),
            Line::from(""),
        ];

        for (id, primal_state) in &state.primal_states {
            resource_lines.push(Line::from(format!("🎯 {}", id)));
            resource_lines.push(Line::from(format!("  CPU: {:.1}%", primal_state.metadata.resource_usage.cpu_percent)));
            resource_lines.push(Line::from(format!("  Memory: {:.1} MB", primal_state.metadata.resource_usage.memory_mb)));
            resource_lines.push(Line::from(format!("  Disk: {:.1} GB", primal_state.metadata.resource_usage.disk_gb)));
            resource_lines.push(Line::from(""));
        }

        let resource_widget = Paragraph::new(resource_lines)
            .block(Block::default().borders(Borders::ALL).title("💻 Resources"))
            .wrap(Wrap { trim: true });

        f.render_widget(resource_widget, area);
    }

    /// Render performance metrics
    fn render_performance_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
        let mut perf_lines = vec![
            Line::from("⚡ Performance"),
            Line::from(""),
        ];

        for (id, primal_state) in &state.primal_states {
            perf_lines.push(Line::from(format!("🎯 {}", id)));
            perf_lines.push(Line::from(format!("  RPS: {:.1}", primal_state.metrics.requests_per_second)));
            perf_lines.push(Line::from(format!("  Response: {:?}", primal_state.metrics.average_response_time)));
            perf_lines.push(Line::from(format!("  Error Rate: {:.2}%", primal_state.metrics.error_rate * 100.0)));
            perf_lines.push(Line::from(""));
        }

        let perf_widget = Paragraph::new(perf_lines)
            .block(Block::default().borders(Borders::ALL).title("⚡ Performance"))
            .wrap(Wrap { trim: true });

        f.render_widget(perf_widget, area);
    }
}
