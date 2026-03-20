// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#[cfg(all(test, feature = "deprecated-tui"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod rendering_tests {
    use crate::tui::primal_ecosystem::{
        DeploymentEvent, DeploymentEventType, DeploymentPhase, DeploymentStatus, EcosystemHealth,
        PrimalApiState, PrimalMetadata, PrimalMetrics, PrimalServiceInfo, ResourceUsage,
        ServiceStatus,
    };
    use crate::tui::types::{
        AiChatMessage, AiInsight, AiRole, AiSuggestion, AiSuggestionCategory, ApiEndpointStatus,
        ApiError, ApiStatus, DashboardState, InsightSeverity, LogEntry, LogLevel,
    };
    use crate::tui::widgets::rendering::WidgetRenderer;
    use biomeos_core::universal_biomeos_manager::DiscoveryResult;
    use biomeos_types::{Health, PrimalCapability, PrimalType};
    use chrono::Utc;
    use ratatui::{Terminal, backend::TestBackend};
    use std::collections::{HashMap, VecDeque};
    use std::time::{Duration, Instant};

    fn draw(state: &DashboardState) {
        let backend = TestBackend::new(100, 45);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|f| {
                WidgetRenderer::render_dashboard(f, state);
            })
            .expect("draw");
    }

    fn minimal_primal(id: &str) -> PrimalApiState {
        PrimalApiState {
            primal_id: id.to_string(),
            primal_type: PrimalType::new("compute", "test", "1.0.0"),
            endpoint: "http://127.0.0.1:1".to_string(),
            health: Health::Healthy,
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            metadata: PrimalMetadata {
                name: format!("Primal-{id}"),
                version: "1.0.0".to_string(),
                description: String::new(),
                uptime: Duration::from_secs(60),
                resource_usage: ResourceUsage {
                    cpu_percent: 12.5,
                    memory_mb: 256.0,
                    disk_gb: 1.5,
                    network_mbps: 0.0,
                },
            },
            services: vec![PrimalServiceInfo {
                service_id: "s1".to_string(),
                name: "svc".to_string(),
                status: ServiceStatus::Running,
                replicas: Some(1),
                resource_usage: ResourceUsage {
                    cpu_percent: 1.0,
                    memory_mb: 64.0,
                    disk_gb: 0.0,
                    network_mbps: 0.0,
                },
            }],
            metrics: PrimalMetrics {
                requests_per_second: 10.0,
                average_response_time: Duration::from_millis(12),
                error_rate: 0.01,
                throughput: 1.0,
            },
            last_updated: Instant::now(),
            api_version: "1".to_string(),
        }
    }

    #[test]
    fn render_dashboard_all_tabs_empty_state() {
        for tab in 0..10 {
            let mut state = DashboardState::new();
            state.current_tab = tab;
            state.ai_enabled = tab % 2 == 0;
            draw(&state);
        }
    }

    #[test]
    fn render_dashboard_all_tabs_rich_state() {
        let mut state = DashboardState::new();
        state.ecosystem_health = EcosystemHealth {
            overall_status: Health::Healthy,
            primal_count: 1,
            healthy_primals: 1,
            total_services: 1,
            healthy_services: 1,
            active_deployments: 1,
            critical_issues: vec![],
        };
        state
            .primal_states
            .insert("p1".to_string(), minimal_primal("p1"));
        state.capability_stats.insert("cap_a".to_string(), 5);
        state.capability_stats.insert("cap_b".to_string(), 2);
        state.deployment_history.push_back(DeploymentEvent {
            timestamp: Instant::now(),
            deployment_id: "d1".to_string(),
            event_type: DeploymentEventType::Started,
            message: "started".to_string(),
        });
        state.active_deployments.push(DeploymentStatus {
            deployment_id: "dep1".to_string(),
            biome_name: "biome-a".to_string(),
            status: DeploymentPhase::Deploying,
            target_environment: "dev".to_string(),
            progress: 50,
            started_at: Instant::now(),
            estimated_completion: None,
            deployed_services: vec![],
            failed_services: vec![],
        });
        state.discovered_services.push(DiscoveryResult {
            id: "disc1".to_string(),
            endpoint: "unix:///tmp/x".to_string(),
            primal_type: PrimalType::new("compute", "x", "1.0"),
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            health: Health::Healthy,
            discovered_at: Utc::now(),
        });
        state.ai_chat_history.push_back(AiChatMessage {
            timestamp: Instant::now(),
            role: AiRole::Assistant,
            content: "hello".to_string(),
            context: None,
        });
        state.ai_suggestions.push(AiSuggestion {
            id: "s1".to_string(),
            title: "scale".to_string(),
            description: "desc".to_string(),
            command: None,
            confidence: 0.85,
            category: AiSuggestionCategory::Scaling,
            can_execute: false,
        });
        state.ai_insights.push(AiInsight {
            title: "insight".to_string(),
            insight: "details".to_string(),
            severity: InsightSeverity::Warning,
            affected_components: vec![],
            recommended_actions: vec![],
            confidence: 0.9,
        });
        state.ai_input_buffer = "type here".to_string();
        state.api_endpoints.insert(
            "e1".to_string(),
            ApiEndpointStatus {
                endpoint: "http://a".to_string(),
                status: ApiStatus::Connected,
                last_successful_call: Some(Instant::now()),
                error_count: 0,
                average_response_time: Duration::from_millis(5),
            },
        );
        state.last_api_sync.insert("e1".to_string(), Instant::now());
        state.api_errors.push_back(ApiError {
            timestamp: Instant::now(),
            endpoint: "bad".to_string(),
            error: "oops".to_string(),
            retry_count: 1,
        });
        let mut logs = VecDeque::new();
        logs.push_back(LogEntry {
            timestamp: Instant::now(),
            source: "src1".to_string(),
            level: LogLevel::Info,
            message: "hello log".to_string(),
            metadata: HashMap::new(),
        });
        state.log_streams.insert("src1".to_string(), logs);

        for tab in 0..10 {
            state.current_tab = tab;
            draw(&state);
        }
    }

    #[test]
    fn render_dashboard_primal_status_empty_map() {
        let mut state = DashboardState::new();
        state.current_tab = 1;
        state.primal_states.clear();
        draw(&state);
    }

    #[test]
    fn render_dashboard_active_deployments_empty() {
        let mut state = DashboardState::new();
        state.current_tab = 2;
        state.active_deployments.clear();
        draw(&state);
    }

    #[test]
    fn render_dashboard_services_empty() {
        let mut state = DashboardState::new();
        state.current_tab = 3;
        state.discovered_services.clear();
        draw(&state);
    }

    #[test]
    fn render_dashboard_ai_enabled_false_status_bar_branch() {
        let mut state = DashboardState::new();
        state.ai_enabled = false;
        state.ecosystem_health.primal_count = 2;
        state.ecosystem_health.total_services = 3;
        state.ecosystem_health.active_deployments = 1;
        draw(&state);
    }

    #[test]
    fn render_dashboard_deployment_history_empty_message() {
        let mut state = DashboardState::new();
        state.current_tab = 0;
        state.deployment_history.clear();
        draw(&state);
    }

    #[test]
    fn render_dashboard_metrics_and_health_degraded_ecosystem() {
        let mut state = DashboardState::new();
        state.ecosystem_health.overall_status = Health::Degraded {
            issues: vec![],
            impact_score: Some(0.3),
        };
        state
            .primal_states
            .insert("a".to_string(), minimal_primal("a"));
        state.current_tab = 4;
        draw(&state);
        state.current_tab = 8;
        draw(&state);
    }

    #[test]
    fn render_dashboard_logs_tab_with_filters() {
        let mut state = DashboardState::new();
        state.current_tab = 9;
        let mut logs = VecDeque::new();
        for i in 0..60 {
            logs.push_back(LogEntry {
                timestamp: Instant::now(),
                source: "app".to_string(),
                level: LogLevel::Warn,
                message: format!("msg {i}"),
                metadata: HashMap::new(),
            });
        }
        state.log_streams.insert("app".to_string(), logs);
        draw(&state);
    }
}
