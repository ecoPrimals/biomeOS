// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

//! Health Module Tests
//!
//! Comprehensive tests for the health monitoring system.

use super::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod health_tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        assert!((Health::healthy().score() - 1.0).abs() < f64::EPSILON);
        assert!((Health::unhealthy(vec![]).score() - 0.0).abs() < f64::EPSILON);
        assert!(Health::degraded(vec![]).score() > 0.5);
        assert!(Health::critical(vec![], vec![]).score() < 0.5);
    }

    #[test]
    fn test_health_status_checks() {
        let healthy = Health::healthy();
        assert!(healthy.is_healthy());
        assert!(healthy.is_operational());
        assert!(!healthy.is_terminal());

        let unhealthy = Health::unhealthy(vec![]);
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_operational());
        assert!(unhealthy.is_terminal());
    }

    #[test]
    fn test_health_degraded() {
        let health = Health::degraded(vec![]);
        match health {
            Health::Degraded {
                issues,
                impact_score,
            } => {
                assert!(issues.is_empty());
                assert!(impact_score.is_some());
            }
            _ => panic!("Expected Degraded health status"),
        }
    }

    #[test]
    fn test_health_critical() {
        let affected = vec!["compute".to_string(), "storage".to_string()];
        let health = Health::critical(vec![], affected);

        match health {
            Health::Critical {
                affected_capabilities,
                ..
            } => {
                assert_eq!(affected_capabilities.len(), 2);
            }
            _ => panic!("Expected Critical health status"),
        }
    }

    #[test]
    fn test_health_unknown() {
        let health = Health::unknown("Service unreachable");
        match health {
            Health::Unknown { reason, last_known } => {
                assert_eq!(reason, "Service unreachable");
                assert!(last_known.is_none());
            }
            _ => panic!("Expected Unknown health status"),
        }
    }

    #[test]
    fn test_health_starting() {
        let health = Health::Starting {
            phase: StartupPhase::Initializing,
            progress: 25,
        };
        assert!(health.is_transitioning());
        assert!(!health.is_terminal());
    }

    #[test]
    fn test_health_stopping() {
        let health = Health::Stopping {
            phase: ShutdownPhase::DroppingConnections,
            progress: 75,
        };
        assert!(health.is_transitioning());
        assert!(!health.is_terminal());
    }

    #[test]
    fn test_health_maintenance() {
        let health = Health::Maintenance {
            maintenance_type: MaintenanceType::Planned,
            estimated_completion: None,
        };
        assert!(!health.is_terminal());
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_health_issues() {
        let issue = HealthIssue {
            id: "issue-1".to_string(),
            category: HealthIssueCategory::Resource,
            severity: HealthIssueSeverity::High,
            message: "High CPU usage".to_string(),
            detected_at: Utc::now(),
            details: HashMap::new(),
            remediation: vec![],
        };

        let health = Health::degraded(vec![issue]);
        assert_eq!(health.issues().len(), 1);
    }

    #[test]
    fn test_health_issue_severity_impact_score() {
        assert!(
            HealthIssueSeverity::Low.impact_score() < HealthIssueSeverity::Medium.impact_score()
        );
        assert!(
            HealthIssueSeverity::Medium.impact_score() < HealthIssueSeverity::High.impact_score()
        );
        assert!(
            HealthIssueSeverity::High.impact_score() < HealthIssueSeverity::Critical.impact_score()
        );
        assert!(
            HealthIssueSeverity::Critical.impact_score()
                < HealthIssueSeverity::Emergency.impact_score()
        );
    }

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert!(config.interval_secs > 0);
        assert!(config.timeout_secs > 0);
        assert!(config.failure_threshold > 0);
        assert!(config.success_threshold > 0);
    }

    #[test]
    fn test_health_check_target_http() {
        let target = HealthCheckTarget::Http {
            url: "/health".to_string(),
            method: "GET".to_string(),
        };
        match target {
            HealthCheckTarget::Http { url, method } => {
                assert_eq!(url, "/health");
                assert_eq!(method, "GET");
            }
            _ => panic!("Expected Http target"),
        }
    }

    #[test]
    fn test_health_check_target_tcp() {
        let target = HealthCheckTarget::Tcp {
            host: "localhost".to_string(),
            port: 8080,
        };
        match target {
            HealthCheckTarget::Tcp { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected Tcp target"),
        }
    }

    #[test]
    fn test_metric_threshold() {
        let threshold = MetricThreshold {
            value: 80.0,
            operator: ThresholdOperator::GreaterThan,
            action: ThresholdAction::MarkDegraded,
        };
        assert!((threshold.value - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_threshold_actions() {
        let alert = ThresholdAction::TriggerAlert {
            alert_type: "pager".to_string(),
        };
        match alert {
            ThresholdAction::TriggerAlert { alert_type } => {
                assert_eq!(alert_type, "pager");
            }
            _ => panic!("Expected TriggerAlert"),
        }

        let remediate = ThresholdAction::ExecuteRemediation {
            action_id: "restart".to_string(),
        };
        match remediate {
            ThresholdAction::ExecuteRemediation { action_id } => {
                assert_eq!(action_id, "restart");
            }
            _ => panic!("Expected ExecuteRemediation"),
        }
    }

    #[test]
    fn test_startup_phases() {
        let phases = [
            StartupPhase::Initializing,
            StartupPhase::LoadingConfiguration,
            StartupPhase::ConnectingDependencies,
            StartupPhase::StartingServices,
            StartupPhase::RunningHealthChecks,
            StartupPhase::Ready,
        ];
        for phase in phases {
            let health = Health::Starting {
                phase,
                progress: 50,
            };
            assert!(!health.is_terminal());
        }
    }

    #[test]
    fn test_shutdown_phases() {
        let phases = [
            ShutdownPhase::Initiated,
            ShutdownPhase::DroppingConnections,
            ShutdownPhase::FlushingData,
            ShutdownPhase::StoppingServices,
            ShutdownPhase::Cleanup,
            ShutdownPhase::Stopped,
        ];
        for phase in phases {
            let health = Health::Stopping {
                phase,
                progress: 50,
            };
            assert!(!health.is_terminal());
        }
    }

    #[test]
    fn test_maintenance_types() {
        let types = [
            MaintenanceType::Planned,
            MaintenanceType::Emergency,
            MaintenanceType::Security,
            MaintenanceType::Performance,
            MaintenanceType::Configuration,
        ];
        for mtype in types {
            let health = Health::Maintenance {
                maintenance_type: mtype,
                estimated_completion: None,
            };
            assert!(!health.is_terminal());
        }
    }

    #[test]
    fn test_health_issue_category() {
        let categories = [
            HealthIssueCategory::Resource,
            HealthIssueCategory::Performance,
            HealthIssueCategory::Configuration,
            HealthIssueCategory::Dependency,
            HealthIssueCategory::Security,
            HealthIssueCategory::Network,
            HealthIssueCategory::Authentication,
            HealthIssueCategory::Data,
            HealthIssueCategory::Hardware,
            HealthIssueCategory::Software,
        ];

        for category in categories {
            let issue = HealthIssue {
                id: "test".to_string(),
                category,
                severity: HealthIssueSeverity::Low,
                message: "Test issue".to_string(),
                detected_at: Utc::now(),
                details: HashMap::new(),
                remediation: vec![],
            };
            assert!(!issue.message.is_empty());
        }
    }

    #[test]
    fn test_remediation_action() {
        let action = RemediationAction {
            id: "restart-service".to_string(),
            action_type: RemediationActionType::Restart,
            description: "Restart the service".to_string(),
            automated: true,
            command: Some("systemctl restart service".to_string()),
            estimated_duration_secs: Some(30),
        };

        assert!(action.automated);
        assert!(action.command.is_some());
    }

    #[test]
    fn test_remediation_action_types() {
        let types = [
            RemediationActionType::Restart,
            RemediationActionType::Reconfigure,
            RemediationActionType::Scale,
            RemediationActionType::Clear,
            RemediationActionType::Update,
            RemediationActionType::Replace,
            RemediationActionType::Manual,
        ];

        for action_type in types {
            let action = RemediationAction {
                id: "test".to_string(),
                action_type,
                description: "Test".to_string(),
                automated: false,
                command: None,
                estimated_duration_secs: None,
            };
            assert!(!action.description.is_empty());
        }
    }

    #[test]
    fn test_remediation_action_type_custom() {
        let action = RemediationAction {
            id: "custom".to_string(),
            action_type: RemediationActionType::Custom {
                action_type: "custom_restart".to_string(),
            },
            description: "Custom action".to_string(),
            automated: false,
            command: None,
            estimated_duration_secs: None,
        };
        assert!(!action.description.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HealthEventTrigger Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_health_event_trigger_variants() {
        let _ = HealthEventTrigger::ScheduledCheck;
        let _ = HealthEventTrigger::ManualCheck;
        let _ = HealthEventTrigger::MetricThreshold {
            metric: "cpu".to_string(),
            threshold: 0.9,
        };
        let _ = HealthEventTrigger::ExternalEvent {
            source: "monitor".to_string(),
        };
        let _ = HealthEventTrigger::Startup;
        let _ = HealthEventTrigger::Shutdown;
        let _ = HealthEventTrigger::ConfigurationChange;
        let _ = HealthEventTrigger::Custom {
            trigger: "custom".to_string(),
        };
    }

    #[test]
    fn test_health_event_trigger_serialization() {
        let trigger = HealthEventTrigger::MetricThreshold {
            metric: "memory_usage".to_string(),
            threshold: 0.85,
        };
        let json = serde_json::to_string(&trigger).expect("serialize");
        assert!(json.contains("memory_usage"));
        let parsed: HealthEventTrigger = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, HealthEventTrigger::MetricThreshold { .. }));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HealthReport, HealthEvent, HealthSubject Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_health_report_creation() {
        let subject = HealthSubject {
            id: "primal-1".to_string(),
            subject_type: HealthSubjectType::Primal,
            name: "Test Primal".to_string(),
            version: "1.0.0".to_string(),
        };

        let report = HealthReport {
            id: Uuid::new_v4(),
            subject,
            health: Health::healthy(),
            components: HashMap::new(),
            metrics: HealthMetrics {
                response_time: None,
                resources: None,
                errors: None,
                availability: None,
                custom: HashMap::new(),
            },
            history: vec![],
            generated_at: Utc::now(),
            next_check_at: None,
        };

        assert!(report.health.is_healthy());
        assert!(report.components.is_empty());
    }

    #[test]
    fn test_health_subject_type_variants() {
        let _ = HealthSubjectType::Primal;
        let _ = HealthSubjectType::Service;
        let _ = HealthSubjectType::System;
        let _ = HealthSubjectType::Component;
        let _ = HealthSubjectType::Custom {
            subject_type: "custom".to_string(),
        };
    }

    #[test]
    fn test_health_subject_type_serialization() {
        let subject_type = HealthSubjectType::Custom {
            subject_type: "custom_primal".to_string(),
        };
        let json = serde_json::to_string(&subject_type).expect("serialize");
        let parsed: HealthSubjectType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, HealthSubjectType::Custom { .. }));
    }

    #[test]
    fn test_component_health_creation() {
        let comp = ComponentHealth {
            name: "api".to_string(),
            health: Health::healthy(),
            metrics: HashMap::new(),
            last_check: Utc::now(),
        };
        assert_eq!(comp.name, "api");
        assert!(comp.health.is_healthy());
    }

    #[test]
    fn test_health_event_creation() {
        let event = HealthEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            previous_health: Some(Health::healthy()),
            new_health: Health::degraded(vec![]),
            trigger: HealthEventTrigger::MetricThreshold {
                metric: "cpu".to_string(),
                threshold: 0.9,
            },
            context: HashMap::new(),
        };
        assert!(event.previous_health.is_some());
        assert!(event.new_health.is_operational());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HealthMetrics, ResponseTimeMetrics, ResourceMetrics Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_response_time_metrics() {
        let metrics = ResponseTimeMetrics {
            average_ms: 50.0,
            p50_ms: 45.0,
            p95_ms: 120.0,
            p99_ms: 200.0,
            max_ms: 350.0,
        };
        assert!((metrics.average_ms - 50.0).abs() < f64::EPSILON);
        assert!((metrics.p95_ms - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_resource_metrics() {
        let metrics = ResourceMetrics {
            cpu_usage: Some(0.65),
            memory_usage: Some(0.42),
            disk_usage: Some(0.30),
            network_io: Some(NetworkIoMetrics {
                bytes_in_per_sec: 1000.0,
                bytes_out_per_sec: 500.0,
                packets_in_per_sec: 10.0,
                packets_out_per_sec: 5.0,
            }),
        };
        assert!(
            metrics
                .cpu_usage
                .is_some_and(|v| (v - 0.65).abs() < f64::EPSILON)
        );
        assert!(metrics.network_io.is_some());
    }

    #[test]
    fn test_network_io_metrics() {
        let metrics = NetworkIoMetrics {
            bytes_in_per_sec: 1024.0,
            bytes_out_per_sec: 512.0,
            packets_in_per_sec: 100.0,
            packets_out_per_sec: 50.0,
        };
        assert!((metrics.bytes_in_per_sec - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_error_metrics() {
        let mut by_cat = HashMap::new();
        by_cat.insert("timeout".to_string(), 0.1);
        by_cat.insert("connection".to_string(), 0.05);

        let metrics = ErrorMetrics {
            error_rate: 0.02,
            errors_by_category: by_cat,
            recent_errors: 5,
        };
        assert!((metrics.error_rate - 0.02).abs() < f64::EPSILON);
        assert_eq!(metrics.recent_errors, 5);
    }

    #[test]
    fn test_availability_metrics() {
        let metrics = AvailabilityMetrics {
            uptime_percentage: 0.999,
            uptime_seconds: 86400,
            downtime_seconds: 86,
            outage_count: 2,
            mttr_seconds: Some(43.0),
        };
        assert!((metrics.uptime_percentage - 0.999).abs() < f64::EPSILON);
        assert_eq!(metrics.outage_count, 2);
    }

    #[test]
    fn test_health_metrics_serialization() {
        let metrics = HealthMetrics {
            response_time: Some(ResponseTimeMetrics {
                average_ms: 25.0,
                p50_ms: 20.0,
                p95_ms: 80.0,
                p99_ms: 150.0,
                max_ms: 200.0,
            }),
            resources: None,
            errors: None,
            availability: None,
            custom: HashMap::new(),
        };
        let json = serde_json::to_string(&metrics).expect("serialize");
        let parsed: HealthMetrics = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.response_time.is_some());
    }

    #[test]
    fn test_health_report_serialization() {
        let report = HealthReport {
            id: Uuid::new_v4(),
            subject: HealthSubject {
                id: "sys-1".to_string(),
                subject_type: HealthSubjectType::System,
                name: "System".to_string(),
                version: "1.0".to_string(),
            },
            health: Health::healthy(),
            components: HashMap::new(),
            metrics: HealthMetrics {
                response_time: None,
                resources: None,
                errors: None,
                availability: None,
                custom: HashMap::new(),
            },
            history: vec![],
            generated_at: Utc::now(),
            next_check_at: None,
        };
        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: HealthReport = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.health.is_healthy());
    }

    #[test]
    fn test_health_score_degraded_with_issues() {
        let issue = HealthIssue {
            id: "cpu-1".to_string(),
            category: HealthIssueCategory::Resource,
            severity: HealthIssueSeverity::Medium,
            message: "High CPU".to_string(),
            detected_at: Utc::now(),
            details: HashMap::new(),
            remediation: vec![],
        };
        let health = Health::degraded(vec![issue]);
        let score = health.score();
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_health_score_starting_progress() {
        let health = Health::Starting {
            phase: StartupPhase::Ready,
            progress: 100,
        };
        assert!((health.score() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_health_score_stopping_progress() {
        let health = Health::Stopping {
            phase: ShutdownPhase::Stopped,
            progress: 100,
        };
        assert!(health.score() < 0.5);
    }

    #[test]
    fn test_health_issue_category_custom() {
        let issue = HealthIssue {
            id: "custom-1".to_string(),
            category: HealthIssueCategory::Custom {
                category: "custom_cat".to_string(),
            },
            severity: HealthIssueSeverity::Low,
            message: "Custom".to_string(),
            detected_at: Utc::now(),
            details: HashMap::new(),
            remediation: vec![],
        };
        assert!(!issue.message.is_empty());
    }

    #[test]
    fn test_maintenance_type_custom() {
        let health = Health::Maintenance {
            maintenance_type: MaintenanceType::Custom {
                maintenance_type: "custom_maint".to_string(),
            },
            estimated_completion: Some(Utc::now()),
        };
        assert!(!health.is_terminal());
    }

    #[test]
    fn test_health_serde_roundtrip_healthy_and_degraded() {
        let h = Health::healthy();
        let json = serde_json::to_string(&h).expect("serialize");
        let back: Health = serde_json::from_str(&json).expect("deserialize");
        assert!(back.is_healthy());

        let degraded = Health::degraded(vec![HealthIssue {
            id: "i1".to_string(),
            category: HealthIssueCategory::Resource,
            severity: HealthIssueSeverity::Low,
            message: "m".to_string(),
            detected_at: Utc::now(),
            details: HashMap::new(),
            remediation: vec![],
        }]);
        let json = serde_json::to_string(&degraded).expect("serialize");
        let back: Health = serde_json::from_str(&json).expect("deserialize");
        assert!(back.is_operational());
    }

    #[test]
    fn test_health_serde_roundtrip_unknown_with_nested_last_known() {
        let inner = Box::new(Health::healthy());
        let health = Health::Unknown {
            reason: "probe".to_string(),
            last_known: Some(inner),
        };
        let json = serde_json::to_string(&health).expect("serialize");
        let back: Health = serde_json::from_str(&json).expect("deserialize");
        match back {
            Health::Unknown { last_known, .. } => {
                assert!(last_known.is_some());
                assert!(last_known.unwrap().is_healthy());
            }
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn test_health_check_target_command_function_custom_serde() {
        for target in [
            HealthCheckTarget::Command {
                command: "/bin/true".to_string(),
                args: vec!["-v".to_string()],
            },
            HealthCheckTarget::Function {
                function: "check".to_string(),
            },
            HealthCheckTarget::Custom {
                target: "custom:probe".to_string(),
            },
        ] {
            let json = serde_json::to_string(&target).expect("serialize");
            let _: HealthCheckTarget = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_metric_threshold_full_serde_roundtrip() {
        let mt = MetricThreshold {
            value: 0.92,
            operator: ThresholdOperator::GreaterThan,
            action: ThresholdAction::MarkCritical,
        };
        let json = serde_json::to_string(&mt).expect("serialize");
        let back: MetricThreshold = serde_json::from_str(&json).expect("deserialize");
        assert!((back.value - 0.92).abs() < f64::EPSILON);
        assert!(matches!(back.operator, ThresholdOperator::GreaterThan));
    }

    #[test]
    fn test_health_issues_empty_for_non_issue_variants() {
        assert!(Health::healthy().issues().is_empty());
        assert!(Health::unknown("x").issues().is_empty());
    }

    #[test]
    fn test_health_check_config_serde_roundtrip() {
        let cfg = HealthCheckConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let back: HealthCheckConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg.interval_secs, back.interval_secs);
        assert_eq!(cfg.failure_threshold, back.failure_threshold);
    }
}
