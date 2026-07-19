#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::collections::HashMap;
use uuid::Uuid;

use super::super::types::*;

#[test]
fn test_ai_response_metadata_default() {
    let metadata = AIResponseMetadata::default();
    assert!(metadata.dependencies.is_empty());
}

#[test]
fn test_ai_first_response_serialization_roundtrip() {
    let request_id = Uuid::new_v4();
    let response: AIFirstResponse<serde_json::Value> =
        AIFirstResponse::success(request_id, serde_json::json!({"key": "value"}), 50, 0.9);

    let json = serde_json::to_string(&response).unwrap();
    let parsed: AIFirstResponse<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.success, response.success);
    assert_eq!(parsed.data, response.data);
    assert!((parsed.confidence_score - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_human_interaction_context_serialization() {
    let context = HumanInteractionContext {
        user_id: Some("user-123".to_string()),
        interaction_mode: InteractionMode::HumanApproval,
        preferences: AIUserPreferences {
            model_preferences: HashMap::new(),
            auto_approval_thresholds: HashMap::new(),
            notifications: NotificationPreferences {
                email_enabled: false,
                push_enabled: false,
                urgency_threshold: NotificationUrgency::Medium,
                quiet_hours: None,
            },
            resource_limits: AIResourceLimits {
                max_cost_per_operation_usd: 0.0,
                max_processing_time_ms: 0,
                max_memory_usage_mb: 0,
                priority_level: PriorityLevel::Normal,
            },
            risk_tolerance: RiskTolerance {
                financial_risk_tolerance: 0.0,
                operational_risk_tolerance: 0.0,
                data_sharing_tolerance: DataSharingLevel::None,
                experimental_features_enabled: false,
            },
            learning_enabled: false,
        },
        approval_required: true,
        confidence_threshold: 0.8,
        escalation_config: EscalationConfig {
            escalation_contacts: vec![],
            escalation_thresholds: HashMap::new(),
            auto_escalation_enabled: false,
            escalation_delay_minutes: 15,
        },
        session_context: None,
    };

    let json = serde_json::to_string(&context).unwrap();
    let parsed: HumanInteractionContext = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.user_id, Some("user-123".to_string()));
    assert!(matches!(
        parsed.interaction_mode,
        InteractionMode::HumanApproval
    ));
}

#[test]
fn test_ai_user_preferences_serialization() {
    let prefs = AIUserPreferences {
        model_preferences: HashMap::from([("inference".to_string(), "llama-3".to_string())]),
        auto_approval_thresholds: HashMap::new(),
        notifications: NotificationPreferences {
            email_enabled: false,
            push_enabled: false,
            urgency_threshold: NotificationUrgency::Medium,
            quiet_hours: None,
        },
        resource_limits: AIResourceLimits {
            max_cost_per_operation_usd: 0.0,
            max_processing_time_ms: 0,
            max_memory_usage_mb: 0,
            priority_level: PriorityLevel::Normal,
        },
        risk_tolerance: RiskTolerance {
            financial_risk_tolerance: 0.0,
            operational_risk_tolerance: 0.0,
            data_sharing_tolerance: DataSharingLevel::None,
            experimental_features_enabled: false,
        },
        learning_enabled: true,
    };

    let json = serde_json::to_string(&prefs).unwrap();
    let parsed: AIUserPreferences = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed.model_preferences.get("inference"),
        Some(&"llama-3".to_string())
    );
}

#[test]
fn test_notification_preferences_serialization() {
    let prefs = NotificationPreferences {
        email_enabled: true,
        push_enabled: false,
        urgency_threshold: NotificationUrgency::High,
        quiet_hours: Some(QuietHours {
            start_hour: 22,
            end_hour: 7,
            timezone: "UTC".to_string(),
        }),
    };

    let json = serde_json::to_string(&prefs).unwrap();
    let parsed: NotificationPreferences = serde_json::from_str(&json).unwrap();
    assert!(parsed.email_enabled);
    assert!(matches!(
        parsed.urgency_threshold,
        NotificationUrgency::High
    ));
    assert!(parsed.quiet_hours.is_some());
}

#[test]
fn test_escalation_config_serialization() {
    let config = EscalationConfig {
        escalation_contacts: vec![EscalationContact {
            contact_type: ContactType::Slack,
            address: "#ops".to_string(),
            urgency_level: NotificationUrgency::Critical,
        }],
        escalation_thresholds: HashMap::from([("error_rate".to_string(), 0.9)]),
        auto_escalation_enabled: true,
        escalation_delay_minutes: 5,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: EscalationConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.escalation_contacts.len(), 1);
    assert!(parsed.auto_escalation_enabled);
}

#[test]
fn test_retry_strategy_serialization_roundtrip() {
    let strategy = RetryStrategy {
        should_retry: true,
        delay_ms: 500,
        max_attempts: 5,
        backoff_strategy: BackoffType::Fibonacci,
        retry_conditions: vec!["timeout".to_string()],
        success_probability: 0.7,
    };

    let json = serde_json::to_string(&strategy).unwrap();
    let parsed: RetryStrategy = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.max_attempts, 5);
    assert!(matches!(parsed.backoff_strategy, BackoffType::Fibonacci));
}

#[test]
fn test_session_context_with_action_history() {
    let context = SessionContext {
        session_id: Uuid::new_v4(),
        started_at: chrono::Utc::now(),
        previous_actions: vec![ActionHistory {
            action: "deploy".to_string(),
            timestamp: chrono::Utc::now(),
            result: ActionResult::Success,
            confidence_score: 0.95,
        }],
        current_goal: "Deploy to staging".to_string(),
        context_variables: HashMap::from([("step".to_string(), serde_json::json!(1))]),
    };

    assert_eq!(context.previous_actions.len(), 1);
    assert_eq!(context.current_goal, "Deploy to staging");
}

#[test]
fn test_ai_first_response_error_serialization() {
    let request_id = Uuid::new_v4();
    let error = biomeos_types::BiomeError::internal_error("Test", Some("E001"));
    let response: AIFirstResponse<()> = AIFirstResponse::error(request_id, error, 10, ());

    let json = serde_json::to_string(&response).unwrap();
    let parsed: AIFirstResponse<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert!(!parsed.success);
    assert!(parsed.error.is_some());
}

#[test]
fn test_risk_tolerance_serialization() {
    let tolerance = RiskTolerance {
        financial_risk_tolerance: 0.2,
        operational_risk_tolerance: 0.5,
        data_sharing_tolerance: DataSharingLevel::Anonymous,
        experimental_features_enabled: true,
    };

    let json = serde_json::to_string(&tolerance).unwrap();
    let parsed: RiskTolerance = serde_json::from_str(&json).unwrap();
    assert!((parsed.financial_risk_tolerance - 0.2).abs() < 0.01);
    assert!(matches!(
        parsed.data_sharing_tolerance,
        DataSharingLevel::Anonymous
    ));
}
