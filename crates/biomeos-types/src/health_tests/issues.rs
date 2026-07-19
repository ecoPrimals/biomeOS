// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use chrono::Utc;
use std::collections::HashMap;

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
    assert!(HealthIssueSeverity::Low.impact_score() < HealthIssueSeverity::Medium.impact_score());
    assert!(HealthIssueSeverity::Medium.impact_score() < HealthIssueSeverity::High.impact_score());
    assert!(
        HealthIssueSeverity::High.impact_score() < HealthIssueSeverity::Critical.impact_score()
    );
    assert!(
        HealthIssueSeverity::Critical.impact_score()
            < HealthIssueSeverity::Emergency.impact_score()
    );
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
