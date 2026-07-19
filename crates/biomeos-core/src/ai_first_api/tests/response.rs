#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::collections::HashMap;
use uuid::Uuid;

use super::super::types::*;

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
