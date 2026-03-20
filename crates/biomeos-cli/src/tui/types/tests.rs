// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use biomeos_core::universal_biomeos_manager::DiscoveryResult;
use biomeos_types::{Health, PrimalCapability, PrimalType};
use std::time::Duration;

#[test]
fn test_tab_id_all_tabs() {
    let tabs = TabId::all_tabs();
    assert!(!tabs.is_empty());
    assert_eq!(tabs.len(), 10);
    assert!(tabs.iter().any(|t| t.id == TabId::EcosystemOverview));
    assert!(tabs.iter().any(|t| t.id == TabId::PrimalStatus));
    assert!(tabs.iter().any(|t| t.id == TabId::Logs));
}

#[test]
fn test_tab_info_clone() {
    let tabs = TabId::all_tabs();
    let first = &tabs[0];
    assert!(!first.title.is_empty());
    assert!(!first.icon.is_empty());
}

#[test]
fn test_dashboard_state_new() {
    let state = DashboardState::new();
    assert_eq!(state.current_tab, 0);
    assert_eq!(state.selected_service, 0);
    assert_eq!(state.selected_primal, 0);
    assert!(state.primal_states.is_empty());
    assert!(state.discovered_services.is_empty());
    assert!(state.ai_chat_history.is_empty());
}

#[test]
fn test_dashboard_state_default() {
    let state = DashboardState::default();
    assert_eq!(state.current_tab, 0);
}

#[test]
fn test_dashboard_state_current_tab_info() {
    let state = DashboardState::new();
    let tab = state.current_tab_info();
    assert!(!tab.title.is_empty());
}

#[test]
fn test_dashboard_state_next_tab() {
    let mut state = DashboardState::new();
    let tabs = TabId::all_tabs();
    state.next_tab();
    assert_eq!(state.current_tab, 1);
    state.current_tab = tabs.len() - 1;
    state.next_tab();
    assert_eq!(state.current_tab, 0);
}

#[test]
fn test_dashboard_state_previous_tab() {
    let mut state = DashboardState::new();
    state.previous_tab();
    assert_eq!(state.current_tab, TabId::all_tabs().len() - 1);
    state.current_tab = 1;
    state.previous_tab();
    assert_eq!(state.current_tab, 0);
}

#[test]
fn test_dashboard_state_add_ai_message() {
    let mut state = DashboardState::new();
    state.add_ai_message(AiRole::Human, "hello".to_string(), None);
    state.add_ai_message(AiRole::Assistant, "hi".to_string(), None);
    assert_eq!(state.ai_chat_history.len(), 2);
}

#[test]
fn test_dashboard_state_update_capability_stats() {
    let mut state = DashboardState::new();
    state.discovered_services.push(DiscoveryResult {
        id: "s1".into(),
        endpoint: "http://a".into(),
        primal_type: PrimalType::new("orchestration", "tower", "1.0.0"),
        capabilities: vec![PrimalCapability::new("storage", "file", "1.0")],
        health: Health::Healthy,
        discovered_at: chrono::Utc::now(),
    });
    state.update_capability_stats();
    assert!(!state.capability_stats.is_empty());
}

#[test]
fn test_ecosystem_health_default() {
    let h = EcosystemHealth::default();
    assert_eq!(h.primal_count, 0);
    assert_eq!(h.healthy_primals, 0);
}

#[test]
fn test_service_status_variants() {
    let _ = format!("{:?}", ServiceStatus::Running);
    let _ = format!("{:?}", ServiceStatus::Starting);
    let _ = format!("{:?}", ServiceStatus::Stopping);
    let _ = format!("{:?}", ServiceStatus::Failed);
    let _ = format!("{:?}", ServiceStatus::Scaling);
}

#[test]
fn test_deployment_phase_variants() {
    let _ = format!("{:?}", DeploymentPhase::Validating);
    let _ = format!("{:?}", DeploymentPhase::Complete);
    let _ = format!(
        "{:?}",
        DeploymentPhase::Failed {
            reason: "err".into(),
        }
    );
}

#[test]
fn test_ai_role_variants() {
    let _ = format!("{:?}", AiRole::Human);
    let _ = format!("{:?}", AiRole::Assistant);
    let _ = format!("{:?}", AiRole::System);
}

#[test]
fn test_ai_suggestion_category_variants() {
    let _ = format!("{:?}", AiSuggestionCategory::Scaling);
    let _ = format!("{:?}", AiSuggestionCategory::Security);
}

#[test]
fn test_insight_severity_variants() {
    let _ = format!("{:?}", InsightSeverity::Info);
    let _ = format!("{:?}", InsightSeverity::Critical);
}

#[test]
fn test_api_status_variants() {
    let _ = format!("{:?}", ApiStatus::Connected);
    let _ = format!("{:?}", ApiStatus::Disconnected);
    let _ = format!(
        "{:?}",
        ApiStatus::Error {
            message: "err".into(),
        }
    );
    let _ = format!("{:?}", ApiStatus::Timeout);
}

#[test]
fn test_log_level_variants() {
    let _ = format!("{:?}", LogLevel::Trace);
    let _ = format!("{:?}", LogLevel::Error);
}

#[test]
fn test_primal_metadata_debug() {
    let m = PrimalMetadata {
        name: "test".into(),
        version: "1.0".into(),
        description: "desc".into(),
        uptime: Duration::from_secs(100),
        resource_usage: ResourceUsage {
            cpu_percent: 50.0,
            memory_mb: 256.0,
            disk_gb: 10.0,
            network_mbps: 1.0,
        },
    };
    let _ = format!("{m:?}");
}

#[test]
fn test_log_filter_debug() {
    let f = LogFilter {
        source_pattern: Some("*.log".into()),
        level_filter: None,
        message_pattern: None,
        time_range: None,
    };
    let _ = format!("{f:?}");
}

#[test]
fn test_dashboard_state_next_previous_service_with_items() {
    use biomeos_core::universal_biomeos_manager::DiscoveryResult;
    use biomeos_types::{Health, PrimalType};

    let mut state = DashboardState::new();
    state.discovered_services.push(DiscoveryResult {
        id: "a".into(),
        endpoint: "unix:///a".into(),
        primal_type: PrimalType::new("t", "n", "1.0"),
        capabilities: vec![],
        health: Health::Healthy,
        discovered_at: chrono::Utc::now(),
    });
    state.discovered_services.push(DiscoveryResult {
        id: "b".into(),
        endpoint: "unix:///b".into(),
        primal_type: PrimalType::new("t", "n", "1.0"),
        capabilities: vec![],
        health: Health::Healthy,
        discovered_at: chrono::Utc::now(),
    });
    state.next_service();
    assert_eq!(state.selected_service, 1);
    state.previous_service();
    assert_eq!(state.selected_service, 0);
}

#[test]
fn test_dashboard_state_selected_primal_and_deployment() {
    use biomeos_types::{Health, PrimalType};
    use std::time::{Duration, Instant};

    let mut state = DashboardState::new();
    assert!(state.selected_primal().is_none());
    assert!(state.selected_deployment().is_none());

    state.primal_states.insert(
        "p1".into(),
        PrimalApiState {
            primal_id: "p1".into(),
            primal_type: PrimalType::new("a", "b", "1"),
            endpoint: String::new(),
            health: Health::Healthy,
            capabilities: vec![],
            metadata: PrimalMetadata {
                name: "n".into(),
                version: "1".into(),
                description: String::new(),
                uptime: Duration::ZERO,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.,
                    memory_mb: 0.,
                    disk_gb: 0.,
                    network_mbps: 0.,
                },
            },
            services: vec![],
            metrics: PrimalMetrics {
                requests_per_second: 0.,
                average_response_time: Duration::ZERO,
                error_rate: 0.,
                throughput: 0.,
            },
            last_updated: Instant::now(),
            api_version: "1".into(),
        },
    );
    state.active_deployments.push(DeploymentStatus {
        deployment_id: "d1".into(),
        biome_name: "b".into(),
        status: DeploymentPhase::Complete,
        target_environment: "local".into(),
        progress: 100,
        started_at: Instant::now(),
        estimated_completion: None,
        deployed_services: vec![],
        failed_services: vec![],
    });
    assert!(state.selected_primal().is_some());
    assert!(state.selected_deployment().is_some());
}

#[test]
fn test_update_ecosystem_health_degraded_mixed() {
    use biomeos_types::{Health, PrimalType};
    use std::time::{Duration, Instant};

    let mut state = DashboardState::new();
    assert!(matches!(
        state.ecosystem_health.overall_status,
        Health::Unknown { .. }
    ));

    state.primal_states.insert(
        "good".into(),
        PrimalApiState {
            primal_id: "good".into(),
            primal_type: PrimalType::new("a", "b", "1"),
            endpoint: String::new(),
            health: Health::Healthy,
            capabilities: vec![],
            metadata: PrimalMetadata {
                name: "n".into(),
                version: "1".into(),
                description: String::new(),
                uptime: Duration::ZERO,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.,
                    memory_mb: 0.,
                    disk_gb: 0.,
                    network_mbps: 0.,
                },
            },
            services: vec![],
            metrics: PrimalMetrics {
                requests_per_second: 0.,
                average_response_time: Duration::ZERO,
                error_rate: 0.,
                throughput: 0.,
            },
            last_updated: Instant::now(),
            api_version: "1".into(),
        },
    );
    state.primal_states.insert(
        "bad".into(),
        PrimalApiState {
            primal_id: "bad".into(),
            primal_type: PrimalType::new("a", "b", "1"),
            endpoint: String::new(),
            health: Health::Healthy,
            capabilities: vec![],
            metadata: PrimalMetadata {
                name: "n".into(),
                version: "1".into(),
                description: String::new(),
                uptime: Duration::ZERO,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.,
                    memory_mb: 0.,
                    disk_gb: 0.,
                    network_mbps: 0.,
                },
            },
            services: vec![],
            metrics: PrimalMetrics {
                requests_per_second: 0.,
                average_response_time: Duration::ZERO,
                error_rate: 0.,
                throughput: 0.,
            },
            last_updated: Instant::now(),
            api_version: "1".into(),
        },
    );
    state.primal_states.get_mut("bad").unwrap().health = Health::Degraded {
        issues: vec![],
        impact_score: None,
    };
    state.update_ecosystem_health();
    assert!(matches!(
        state.ecosystem_health.overall_status,
        Health::Degraded { .. }
    ));
}

#[test]
fn test_add_health_data_trims_history() {
    use biomeos_types::Health;

    let mut state = DashboardState::new();
    state.max_history_points = 3;
    for i in 0..10 {
        state.add_health_data(crate::health::SystemHealth {
            overall_status: Health::Healthy,
            cpu_usage: i as f64,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_status: "ok".into(),
        });
    }
    assert_eq!(state.system_health_history.len(), 3);
}

#[test]
fn test_ai_chat_history_cap_trims() {
    let mut state = DashboardState::new();
    for i in 0..120 {
        state.add_ai_message(AiRole::Human, format!("m{i}"), None);
    }
    assert!(state.ai_chat_history.len() <= 100);
}

#[test]
fn test_update_primal_state_inserts_history() {
    use biomeos_types::{Health, PrimalType};
    use std::time::{Duration, Instant};

    let mut state = DashboardState::new();
    let ps = PrimalApiState {
        primal_id: "x".into(),
        primal_type: PrimalType::new("a", "b", "1"),
        endpoint: String::new(),
        health: Health::Healthy,
        capabilities: vec![],
        metadata: PrimalMetadata {
            name: "n".into(),
            version: "1".into(),
            description: String::new(),
            uptime: Duration::ZERO,
            resource_usage: ResourceUsage {
                cpu_percent: 0.,
                memory_mb: 0.,
                disk_gb: 0.,
                network_mbps: 0.,
            },
        },
        services: vec![],
        metrics: PrimalMetrics {
            requests_per_second: 0.,
            average_response_time: Duration::ZERO,
            error_rate: 0.,
            throughput: 0.,
        },
        last_updated: Instant::now(),
        api_version: "1".into(),
    };
    state.update_primal_state("x".into(), ps);
    assert!(state.primal_states.contains_key("x"));
    assert!(state.primal_health_history.contains_key("x"));
}
