// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use chrono::Utc;
use std::collections::HashMap;

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
fn test_health_issues_empty_for_non_issue_variants() {
    assert!(Health::healthy().issues().is_empty());
    assert!(Health::unknown("x").issues().is_empty());
}
