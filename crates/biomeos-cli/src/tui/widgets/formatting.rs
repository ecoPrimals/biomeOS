// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Pure computation and formatting functions for TUI widgets.
//!
//! These functions have no dependency on ratatui rendering types (Frame, Rect, etc.)
//! and are designed for easy unit testing.

use crate::tui::types::{
    AiRole, ApiStatus, DashboardState, DeploymentEventType, DeploymentPhase, InsightSeverity,
    LogLevel, PrimalApiState, ServiceStatus,
};
use biomeos_types::Health;
use ratatui::style::Color;
use std::collections::HashMap;

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
    primal_states: &[(String, &PrimalApiState)],
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
    primal_states: &[(String, &PrimalApiState)],
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
    states: &[(String, &PrimalApiState)],
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
pub fn service_status_icon(status: &ServiceStatus) -> &'static str {
    match status {
        ServiceStatus::Running => "▶️",
        ServiceStatus::Starting => "⏳",
        ServiceStatus::Stopping => "⏹️",
        ServiceStatus::Failed => "❌",
        ServiceStatus::Scaling => "⚖️",
    }
}

/// Get deployment phase icon for display
pub fn deployment_phase_icon(phase: &DeploymentPhase) -> &'static str {
    match phase {
        DeploymentPhase::Validating => "🔍",
        DeploymentPhase::Deploying => "🚀",
        DeploymentPhase::Scaling => "⚖️",
        DeploymentPhase::Configuring => "⚙️",
        DeploymentPhase::HealthChecking => "💊",
        DeploymentPhase::Complete => "✅",
        DeploymentPhase::Failed { .. } => "❌",
        DeploymentPhase::RollingBack => "🔄",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::types::{
        DeploymentPhase, EcosystemHealth, PrimalApiState, PrimalMetadata, PrimalMetrics,
        ResourceUsage, ServiceStatus, TabId,
    };
    use biomeos_types::{
        Health, HealthIssue, HealthIssueCategory, HealthIssueSeverity, PrimalType,
    };
    use ratatui::style::Color;
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
                description: String::new(),
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
        let _ = crate::tui::widgets::WidgetRenderer;
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
