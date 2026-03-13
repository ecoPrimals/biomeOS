// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Health Module Tests
//!
//! Comprehensive tests for the health monitoring system.

use super::*;

#[cfg(test)]
mod health_tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        assert_eq!(Health::healthy().score(), 1.0);
        assert_eq!(Health::unhealthy(vec![]).score(), 0.0);
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
        assert_eq!(threshold.value, 80.0);
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
}
