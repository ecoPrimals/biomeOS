#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::collections::HashMap;
use uuid::Uuid;

use super::super::types::*;

#[test]
fn test_priority_levels() {
    let levels = [
        PriorityLevel::Low,
        PriorityLevel::Normal,
        PriorityLevel::High,
        PriorityLevel::Critical,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let _: PriorityLevel = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_data_sharing_levels() {
    let levels = [
        DataSharingLevel::None,
        DataSharingLevel::Anonymous,
        DataSharingLevel::Aggregated,
        DataSharingLevel::Full,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let _: DataSharingLevel = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_contact_types() {
    let types = [
        ContactType::Email,
        ContactType::SMS,
        ContactType::Slack,
        ContactType::Teams,
        ContactType::Webhook,
    ];

    for contact_type in types {
        let json = serde_json::to_string(&contact_type).unwrap();
        let _: ContactType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_action_results() {
    let results = [
        ActionResult::Success,
        ActionResult::Failure,
        ActionResult::PartialSuccess,
        ActionResult::Skipped,
    ];

    for result in results {
        let json = serde_json::to_string(&result).unwrap();
        let _: ActionResult = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_risk_levels() {
    let levels = [
        RiskLevel::VeryLow,
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::VeryHigh,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let _: RiskLevel = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_suggested_action() {
    let action = SuggestedAction {
        action_type: "restart".to_string(),
        description: "Restart the service".to_string(),
        confidence: 0.9,
        estimated_duration_ms: Some(5000),
        estimated_cost_usd: Some(0.01),
        prerequisites: vec!["backup".to_string()],
        risk_level: RiskLevel::Low,
    };

    assert!((action.confidence - 0.9).abs() < f64::EPSILON);
    assert!(action.estimated_duration_ms.is_some());
}

#[test]
fn test_quiet_hours() {
    let hours = QuietHours {
        start_hour: 22,
        end_hour: 7,
        timezone: "America/New_York".to_string(),
    };

    assert_eq!(hours.start_hour, 22);
    assert_eq!(hours.end_hour, 7);
}

#[test]
fn test_ai_resource_limits() {
    let limits = AIResourceLimits {
        max_cost_per_operation_usd: 1.0,
        max_processing_time_ms: 30000,
        max_memory_usage_mb: 1024,
        priority_level: PriorityLevel::High,
    };

    assert!((limits.max_cost_per_operation_usd - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_risk_tolerance() {
    let tolerance = RiskTolerance {
        financial_risk_tolerance: 0.5,
        operational_risk_tolerance: 0.3,
        data_sharing_tolerance: DataSharingLevel::Aggregated,
        experimental_features_enabled: false,
    };

    assert!(!tolerance.experimental_features_enabled);
}

#[test]
fn test_escalation_contact() {
    let contact = EscalationContact {
        contact_type: ContactType::Slack,
        address: "#alerts".to_string(),
        urgency_level: NotificationUrgency::High,
    };

    assert_eq!(contact.address, "#alerts");
}

#[test]
fn test_session_context() {
    let context = SessionContext {
        session_id: Uuid::new_v4(),
        started_at: chrono::Utc::now(),
        previous_actions: vec![],
        current_goal: "Deploy application".to_string(),
        context_variables: HashMap::new(),
    };

    assert_eq!(context.current_goal, "Deploy application");
}

#[test]
fn test_action_history() {
    let history = ActionHistory {
        action: "create_container".to_string(),
        timestamp: chrono::Utc::now(),
        result: ActionResult::Success,
        confidence_score: 0.95,
    };

    assert_eq!(history.action, "create_container");
}
