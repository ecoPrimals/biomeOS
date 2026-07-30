
use super::*;

#[test]
fn test_ai_error_context_default() {
    let ctx = AIErrorContext::default();
    assert_eq!(ctx.category, AIErrorCategory::UserError);
    assert_eq!(ctx.severity, ErrorSeverity::Error);
    assert!(!ctx.requires_human_intervention);
    assert!((ctx.automation_confidence - 0.5).abs() < f64::EPSILON);
    assert!(ctx.automation_hints.is_empty());
    assert!(ctx.suggested_actions.is_empty());
}

#[test]
fn test_ai_error_context_new() {
    let ctx = AIErrorContext::new(AIErrorCategory::NetworkFailure);
    assert_eq!(ctx.category, AIErrorCategory::NetworkFailure);
    assert_eq!(ctx.severity, ErrorSeverity::Error);
}

#[test]
fn test_ai_error_context_with_retry() {
    let strategy = RetryStrategy::exponential_backoff(3, 100, 5000);
    let ctx = AIErrorContext::with_retry(AIErrorCategory::RateLimiting, strategy);
    assert_eq!(ctx.category, AIErrorCategory::RateLimiting);
    assert!(ctx.retry_strategy.should_retry);
    assert_eq!(ctx.retry_strategy.max_attempts, 3);
}

#[test]
fn test_ai_error_context_with_hint() {
    let ctx = AIErrorContext::new(AIErrorCategory::ConfigurationIssue)
        .with_hint("Check BIND_ADDRESS env var");
    assert_eq!(ctx.automation_hints.len(), 1);
    assert_eq!(ctx.automation_hints[0], "Check BIND_ADDRESS env var");
}

#[test]
fn test_ai_error_context_with_severity() {
    let ctx =
        AIErrorContext::new(AIErrorCategory::SystemError).with_severity(ErrorSeverity::Critical);
    assert_eq!(ctx.severity, ErrorSeverity::Critical);
}

#[test]
fn test_ai_error_context_requires_human() {
    let ctx = AIErrorContext::new(AIErrorCategory::HumanInterventionRequired).requires_human();
    assert!(ctx.requires_human_intervention);
    assert!((ctx.automation_confidence - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_retry_strategy_no_retry() {
    let s = RetryStrategy::no_retry();
    assert!(!s.should_retry);
    assert_eq!(s.max_attempts, 0);
    assert_eq!(s.delay_ms, 0);
}

#[test]
fn test_retry_strategy_exponential_backoff() {
    let s = RetryStrategy::exponential_backoff(5, 200, 10000);
    assert!(s.should_retry);
    assert_eq!(s.max_attempts, 5);
    assert_eq!(s.delay_ms, 200);
    assert!((s.success_probability - 0.7).abs() < f64::EPSILON);
    assert!(s.max_retry_time_ms.is_some());
    if let BackoffType::Exponential { base, max_delay_ms } = s.backoff_strategy {
        assert!((base - 2.0).abs() < f64::EPSILON);
        assert_eq!(max_delay_ms, 10000);
    } else {
        panic!("Expected Exponential backoff");
    }
}

#[test]
fn test_retry_strategy_linear_backoff() {
    let s = RetryStrategy::linear_backoff(4, 100, 50);
    assert!(s.should_retry);
    assert_eq!(s.max_attempts, 4);
    assert_eq!(s.delay_ms, 100);
    assert!((s.success_probability - 0.6).abs() < f64::EPSILON);
    if let BackoffType::Linear { increment_ms } = s.backoff_strategy {
        assert_eq!(increment_ms, 50);
    } else {
        panic!("Expected Linear backoff");
    }
}

#[test]
fn test_error_severity_ordering() {
    assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
    assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
    assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    assert!(ErrorSeverity::Critical < ErrorSeverity::Emergency);
}

#[test]
fn test_action_risk_level_ordering() {
    assert!(ActionRiskLevel::None < ActionRiskLevel::Low);
    assert!(ActionRiskLevel::Low < ActionRiskLevel::Medium);
    assert!(ActionRiskLevel::Medium < ActionRiskLevel::High);
    assert!(ActionRiskLevel::High < ActionRiskLevel::Critical);
}

#[test]
fn test_ai_error_category_serialization() {
    let categories = [
        AIErrorCategory::ResourceLimitation,
        AIErrorCategory::ConfigurationIssue,
        AIErrorCategory::NetworkFailure,
        AIErrorCategory::RateLimiting,
    ];
    for cat in categories {
        let json = serde_json::to_string(&cat).expect("serialize");
        let parsed: AIErrorCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cat, parsed);
    }
}

#[test]
fn test_retry_strategy_serialization() {
    let s = RetryStrategy::exponential_backoff(3, 100, 5000);
    let json = serde_json::to_string(&s).expect("serialize");
    let parsed: RetryStrategy = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(s.should_retry, parsed.should_retry);
    assert_eq!(s.max_attempts, parsed.max_attempts);
}

#[test]
fn test_backoff_type_serialization() {
    let linear = BackoffType::Linear { increment_ms: 500 };
    let json = serde_json::to_string(&linear).expect("serialize");
    let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, BackoffType::Linear { increment_ms: 500 }));

    let exp = BackoffType::Exponential {
        base: 2.0,
        max_delay_ms: 10000,
    };
    let json = serde_json::to_string(&exp).expect("serialize");
    let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        parsed,
        BackoffType::Exponential {
            base: 2.0,
            max_delay_ms: 10000
        }
    ));
}

#[test]
fn test_action_type_custom_serialization() {
    let custom = ActionType::Custom {
        action_type: "custom.restart".to_string(),
    };
    let json = serde_json::to_string(&custom).expect("serialize");
    let parsed: ActionType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        parsed,
        ActionType::Custom { action_type } if action_type == "custom.restart"
    ));
}

#[test]
fn test_all_ai_error_category_variants_serialization() {
    let categories = [
        AIErrorCategory::ResourceLimitation,
        AIErrorCategory::ConfigurationIssue,
        AIErrorCategory::SecurityViolation,
        AIErrorCategory::NetworkFailure,
        AIErrorCategory::RuntimeError,
        AIErrorCategory::HumanInterventionRequired,
        AIErrorCategory::DependencyFailure,
        AIErrorCategory::RateLimiting,
        AIErrorCategory::DataIssue,
        AIErrorCategory::ServiceUnavailable,
        AIErrorCategory::SystemError,
        AIErrorCategory::UserError,
    ];
    for cat in categories {
        let json = serde_json::to_string(&cat).expect("serialize");
        let parsed: AIErrorCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cat, parsed);
    }
}

#[test]
fn test_backoff_type_fibonacci_serialization() {
    let fib = BackoffType::Fibonacci { max_delay_ms: 5000 };
    let json = serde_json::to_string(&fib).expect("serialize");
    let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        parsed,
        BackoffType::Fibonacci { max_delay_ms: 5000 }
    ));
}

#[test]
fn test_backoff_type_custom_serialization() {
    let custom = BackoffType::Custom {
        delays_ms: vec![100, 200, 400, 800],
    };
    let json = serde_json::to_string(&custom).expect("serialize");
    let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        parsed,
        BackoffType::Custom { delays_ms } if delays_ms == vec![100, 200, 400, 800]
    ));
}

#[test]
fn test_action_type_display_debug() {
    let variants = [
        ActionType::Retry,
        ActionType::Restart,
        ActionType::UpdateConfig,
        ActionType::Scale,
        ActionType::Fallback,
        ActionType::RequestHuman,
        ActionType::LogMore,
        ActionType::HealthCheck,
        ActionType::Custom {
            action_type: "custom.test".to_string(),
        },
    ];
    for v in variants {
        let debug_str = format!("{v:?}");
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_action_type_all_variants_serialization() {
    let variants = [
        ActionType::Retry,
        ActionType::Restart,
        ActionType::UpdateConfig,
        ActionType::Scale,
        ActionType::Fallback,
        ActionType::RequestHuman,
        ActionType::LogMore,
        ActionType::HealthCheck,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).expect("serialize");
        let parsed: ActionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(v, parsed, "round-trip failed for {v:?}");
    }
}

#[test]
fn test_error_severity_comparisons() {
    assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
    assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
    assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    assert!(ErrorSeverity::Critical < ErrorSeverity::Emergency);
    assert!(ErrorSeverity::Emergency > ErrorSeverity::Info);
    assert_eq!(
        ErrorSeverity::Error.cmp(&ErrorSeverity::Error),
        std::cmp::Ordering::Equal
    );
}

#[test]
fn test_error_severity_serialization() {
    let severities = [
        ErrorSeverity::Info,
        ErrorSeverity::Warning,
        ErrorSeverity::Error,
        ErrorSeverity::Critical,
        ErrorSeverity::Emergency,
    ];
    for sev in severities {
        let json = serde_json::to_string(&sev).expect("serialize");
        let parsed: ErrorSeverity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sev, parsed);
    }
}

#[test]
fn test_suggested_action_serialization() {
    let action = SuggestedAction {
        id: "retry-1".to_string(),
        action_type: ActionType::Retry,
        description: "Retry the operation".to_string(),
        automated: true,
        confidence: 0.8,
        estimated_duration_secs: Some(5),
        prerequisites: vec!["network".to_string()],
        expected_outcome: "Success".to_string(),
        risk_level: ActionRiskLevel::Low,
    };
    let json = serde_json::to_string(&action).expect("serialize");
    let parsed: SuggestedAction = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.id, action.id);
    assert!((parsed.confidence - action.confidence).abs() < f64::EPSILON);
}
