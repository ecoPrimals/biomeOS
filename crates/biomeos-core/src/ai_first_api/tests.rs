// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AI-First API tests

use std::collections::HashMap;
use uuid::Uuid;

use super::types::*;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod run {
    use super::*;

    #[test]
    fn test_ai_first_response_success() {
        let request_id = Uuid::new_v4();
        let response: AIFirstResponse<String> =
            AIFirstResponse::success(request_id, "test data".to_string(), 100, 0.95);

        assert!(response.success);
        assert!(response.error.is_none());
        assert_eq!(response.data, "test data");
        assert_eq!(response.processing_time_ms, 100);
        assert!((response.confidence_score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ai_first_response_error() {
        let request_id = Uuid::new_v4();
        let error = biomeos_types::BiomeError::internal_error("Test error", Some("ERR001"));
        let response: AIFirstResponse<String> =
            AIFirstResponse::error(request_id, error, 50, "default".to_string());

        assert!(!response.success);
        assert!(response.error.is_some());
        assert!((response.confidence_score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy {
            should_retry: true,
            delay_ms: 1000,
            max_attempts: 3,
            backoff_strategy: BackoffType::Exponential { base: 2.0 },
            retry_conditions: vec!["network_error".to_string()],
            success_probability: 0.8,
        };

        assert!(strategy.should_retry);
        assert_eq!(strategy.max_attempts, 3);
        assert!((strategy.success_probability - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_backoff_types() {
        let types = [
            BackoffType::Linear,
            BackoffType::Exponential { base: 2.0 },
            BackoffType::Fibonacci,
            BackoffType::Custom {
                formula: "x^2".to_string(),
            },
        ];

        for backoff in types {
            let json = serde_json::to_string(&backoff).unwrap();
            let _: BackoffType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert!((metrics.cpu_usage_percent - 0.0).abs() < f64::EPSILON);
        assert!((metrics.memory_usage_mb - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.io_operations, 0);
    }

    #[test]
    fn test_ai_resource_usage_default() {
        let usage = AIResourceUsage::default();
        assert!((usage.compute_units_used - 0.0).abs() < f64::EPSILON);
        assert_eq!(usage.storage_bytes_used, 0);
    }

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert!(metrics.accuracy_score.is_none());
        assert!((metrics.completeness_score - 1.0).abs() < f64::EPSILON);
        assert!((metrics.reliability_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_info_default() {
        let cache = CacheInfo::default();
        assert!(!cache.is_cached);
        assert!(cache.cache_hit_ratio.is_none());
    }

    #[test]
    fn test_rate_limit_status_default() {
        let status = RateLimitStatus::default();
        assert!(status.limit.is_none());
        assert!(status.remaining.is_none());
    }

    #[test]
    fn test_interaction_modes() {
        let modes = [
            InteractionMode::FullyAutonomous,
            InteractionMode::HumanApproval,
            InteractionMode::Collaborative,
            InteractionMode::HumanDirected,
            InteractionMode::HumanSupervised,
            InteractionMode::Emergency,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let _: InteractionMode = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_notification_urgency_levels() {
        let levels = [
            NotificationUrgency::Low,
            NotificationUrgency::Medium,
            NotificationUrgency::High,
            NotificationUrgency::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let _: NotificationUrgency = serde_json::from_str(&json).unwrap();
        }
    }

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
}
